#!/usr/bin/env python3
"""G7-PATENT gate: verify OST design-around claims against the cgap corpus.

Parses docs/jira/v26.11.2/patent/claims.txt (pipe-format, 2-space indent =
continuation), runs each row's DISTINCT-MECHANISM grep family src-scoped
against crates/, classifies src vs tests hits (tests-only != practicing),
emits ost_report.json.

Exit contract (docs/jira/v26.11.2/01-release-gates.md G7-PATENT):
  0  all rows DISTINCT-MECHANISM with landed evidence
  1  any row PENDING / PARTIAL (split) / NOT-DISTINCT (mechanism present)
  3  loud [SKIP]: claims.txt or corpus absent
"""
import argparse
import hashlib
import json
import os
import re
import subprocess
import sys

REPO = "/Users/sac/cgap"
DEFAULT_CLAIMS = "docs/jira/v26.11.2/patent/claims.txt"
DEFAULT_REPORT = "docs/jira/v26.11.2/patent/ost_report.json"

# Per-row patented-mechanism grep families (negative-surface evidence):
# zero src-scoped hits for the PATENTED mechanism = design-around surface.
# Sources: docs/jira/v26.11.2/waves/w2-07-ost-greps.md
ROW_GREPS = {
    "US10,817,467": {
        "family": "CAS/compare_exchange/AtomicU64/shared-memory (src-scoped)",
        "pattern": r"CAS|compare_exchange|AtomicU64|shared-memory",
        "paths": ["crates/cgap-storage/src", "crates/cgap-parquet/src"],
    },
    "US11,216,456": {
        "family": "runtime tuple-binding propagation (src-scoped)",
        "pattern": r"tuple[_ ]?binding|TupleBinding|propagate_binding",
        "paths": ["crates"],
    },
    "US11,409,698": {
        "family": "continuation/Fiber/Generator/resume frames (src-scoped)",
        "pattern": r"continuation|Fiber|Generator|resume",
        "paths": ["crates/cgap-bn3/src", "crates/cgap-bont/src"],
    },
    "US11,645,278": {
        "family": "topology switching / dyn dispatch (src-scoped)",
        "pattern": r"topology|Topology|Box<dyn|&dyn ",
        "paths": ["crates/cgap-bn3/src"],
    },
}

# Positive-surface evidence markers: the cgap mechanism must be present.
ROW_EVIDENCE = {
    "US10,817,467": {
        "cite": "crates/cgap-parquet/src/mmap_writer.rs",
        "markers": ["RingSink", "drain_to_spill", "RunOverrun"],
        "extra_cites": [
            {
                "cite": "crates/cgap-ffi/src/sovereign.rs",
                "markers": ["CGAP_ERR_CONCURRENT_WRITER_DENIED",
                            "0x4347_0006"],
            },
            {
                "cite": "crates/cgap-ffi/tests/fa9_commit_gating.rs",
                "markers": ["CGAP_ERR_CONCURRENT_WRITER_DENIED",
                            "0x4347_0006"],
            },
        ],
    },
    "US11,216,456": {
        "cite": "crates/cgap-parquet/src/bitslice.rs",
        "markers": ["bit-slice", "Pack `values` into the vertical"],
    },
    "US11,409,698": {
        "cite": "crates/cgap-bn3/src/unify.rs",
        "markers": ["semi_join_masks", "caller-leased", "RuleCompiler"],
    },
    "US11,645,278": {
        "cite": "crates/cgap-bn3/src/unify.rs",
        "markers": ["R0", "RegisterOverflow"],
    },
}


def parse_claims(path):
    """Pipe-format rows; a line starting with 2 spaces continues the prior row."""
    rows = []
    with open(path, encoding="utf-8") as fh:
        lines = fh.read().splitlines()
    for raw in lines:
        if not raw or raw.startswith("#"):
            continue
        if raw.startswith("  "):
            if not rows:
                raise ValueError("continuation before first row: %r" % raw)
            rows[-1]["raw"] += "\n" + raw.strip()
            continue
        rows.append({"raw": raw})
    out = []
    for r in rows:
        parts = [p.strip() for p in r["raw"].split("|")]
        if len(parts) != 4:
            raise ValueError("malformed row (need 4 pipe fields): %r" % r["raw"])
        out.append({
            "patent": parts[0],
            "element": parts[1],
            "mechanism": parts[2],
            "status": parts[3],
        })
    return out


def run_grep(pattern, paths, repo):
    """Real grep -rnE subprocess; returns (exit, [ 'path:line: text' ])."""
    existing = [os.path.join(repo, p) for p in paths
                if os.path.exists(os.path.join(repo, p))]
    if not existing:
        return 2, []
    argv = ["grep", "-rnE", pattern] + existing
    proc = subprocess.run(argv, capture_output=True, text=True)
    return proc.returncode, [l for l in proc.stdout.splitlines() if l]


def classify_hits(hits):
    """Split grep hits into src vs tests/other. tests-only != practicing."""
    src, tests = [], []
    for h in hits:
        if "/tests/" in h or h.endswith("_test.rs"):
            tests.append(h)
        else:
            src.append(h)
    return src, tests


# Non-mechanism src-hit classes, each named and disclosed in the report
# (an unclassified exclusion would be a silent pass channel):
#   doc-comment prose: the pattern only occurs on a `///`/`//` comment line.
#   error-trait upcast: `&dyn core::error::Error` is error plumbing, not
#     runtime topology dispatch (w2-07-ost-greps.md row4 witness).
NON_MECHANISM_CODE = re.compile(r"&dyn\s+core::error::Error")


def classify_src_hit(hit):
    """Return (is_mechanism, reason) for a src-scoped grep hit."""
    line = hit.split(":", 2)[2] if hit.count(":") >= 2 else hit
    if line.strip().startswith("//"):
        return False, "doc-comment prose (comment-only line)"
    code = line.split("//", 1)[0]
    if not code.strip():
        return False, "doc-comment prose (no code on line)"
    if NON_MECHANISM_CODE.search(code):
        return False, "error-trait upcast (&dyn core::error::Error), not dispatch"
    return True, "pattern on live code line"


def mechanism_src_hits(src_hits):
    out = []
    for h in src_hits:
        is_m, why = classify_src_hit(h)
        out.append({"hit": h, "mechanism": is_m, "reason": why})
    return out


def norm_text(s):
    return re.sub(r"\s+", " ", s).strip().lower()


def verdict_for_row(row, repo):
    patent = row["patent"]
    status = row["status"]
    v = {
        "patent": patent,
        "element": row["element"],
        "mechanism": row["mechanism"],
        "status_raw": status,
        "grep_family": None, "grep_command": None, "grep_exit": None,
        "src_hits": [], "tests_hits": [],
        "evidence_cite": None, "evidence_present": None,
        "evidence_markers": {},
    }
    g = ROW_GREPS.get(patent)
    if g is None:
        v["verdict"] = "UNKNOWN-ROW"
        v["reason"] = "no grep family registered for %s" % patent
        return v
    v["grep_family"] = g["family"]
    argv = ["grep", "-rnE", g["pattern"]] + \
        [os.path.join(repo, p) for p in g["paths"]
         if os.path.exists(os.path.join(repo, p))]
    v["grep_command"] = " ".join(argv)
    if not argv[3:]:
        v["verdict"] = "SKIP-ROW"
        v["reason"] = "grep paths absent"
        return v
    proc = subprocess.run(argv, capture_output=True, text=True)
    v["grep_exit"] = proc.returncode
    hits = [l for l in proc.stdout.splitlines() if l]
    src, tests = classify_hits(hits)
    v["src_hits"] = [h["hit"] for h in mechanism_src_hits(src)]
    v["src_hit_classification"] = mechanism_src_hits(src)
    v["tests_hits"] = tests
    # Negative-surface: patented mechanism must have zero MECHANISM src hits.
    mech = [h for h in v["src_hit_classification"] if h["mechanism"]]
    if mech:
        v["verdict"] = "NOT-DISTINCT"
        v["reason"] = ("patented-mechanism surface present in src "
                       "(%d mechanism hits)" % len(mech))
        return v
    ev = ROW_EVIDENCE.get(patent)
    if ev:
        cite_abs = os.path.join(repo, ev["cite"])
        v["evidence_cite"] = ev["cite"]
        v["evidence_present"] = os.path.isfile(cite_abs)
        if not v["evidence_present"]:
            v["verdict"] = "PENDING"
            v["reason"] = "landed cite missing: %s" % ev["cite"]
            return v
        cite_list = ([{"cite": ev["cite"], "markers": ev["markers"]}] +
                     ev.get("extra_cites", []))
        for c in cite_list:
            c_abs = os.path.join(repo, c["cite"])
            if not os.path.isfile(c_abs):
                v["evidence_present"] = False
                v["verdict"] = "PENDING"
                v["reason"] = "landed cite missing: %s" % c["cite"]
                return v
            for m in c["markers"]:
                mproc = subprocess.run(
                    ["grep", "-rnE", m, c_abs],
                    capture_output=True, text=True)
                v["evidence_markers"]["%s:%s" % (c["cite"], m)] = \
                    mproc.returncode == 0
        if not all(v["evidence_markers"].values()):
            v["verdict"] = "PENDING"
            v["reason"] = ("evidence markers missing in cite: %s"
                           % [k for k, ok in v["evidence_markers"].items()
                              if not ok])
            return v
    if "PENDING" in status:
        if "LANDED" in status:
            v["verdict"] = "PARTIAL"
            v["reason"] = "status split: LANDED component + PENDING component"
        else:
            v["verdict"] = "PENDING"
            v["reason"] = "status column PENDING (no landed evidence recorded)"
        return v
    v["verdict"] = "DISTINCT-MECHANISM"
    v["reason"] = ("zero src hits for patented mechanism; landed cite "
                   "verified" if ev else "zero src hits for patented mechanism")
    return v


def duplicate_mechanism_check(rows):
    """Near-duplicate merged mechanisms break distinctness."""
    seen = {}
    dups = []
    for r in rows:
        key = norm_text(r["mechanism"])
        seen.setdefault(key, []).append(r["patent"])
    for key, pats in seen.items():
        if len(pats) > 1:
            dups.append(pats)
    return dups


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


# Behavioral gate (lane AE6): grep evidence alone is not behavioral —
# the landed cgap behavioral court must ALSO pass. Anti-vacuity mutation
# (witnessed in the /tmp sandbox, lane receipt): bypassing the bound-key
# mask arm in emit_join_masks_view flips row2 to FAIL -> court exit 1 ->
# gate exit 1.
BEHAVIORAL_COURT_CMD = ["cargo", "test", "-p", "cgap",
                        "--test", "patent_behavioral_court"]


def run_behavioral_court(repo):
    """Run the behavioral patent court; returns (exit, stdout tail)."""
    proc = subprocess.run(BEHAVIORAL_COURT_CMD, cwd=repo,
                          capture_output=True, text=True)
    return proc.returncode, (proc.stdout + proc.stderr).splitlines()[-6:]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--claims", default=DEFAULT_CLAIMS)
    ap.add_argument("--corpus", default="/Users/sac/purrdf/corpora")
    ap.add_argument("--report", default=DEFAULT_REPORT)
    args = ap.parse_args()

    claims_abs = os.path.abspath(args.claims)
    corpus = args.corpus
    if not os.path.isfile(claims_abs) or not os.path.isdir(corpus):
        print("[SKIP] %s" % ("claims.txt-absent" if
              not os.path.isfile(claims_abs) else "corpus-absent"))
        return 3
    if not os.path.isdir(os.path.join(REPO, "crates")):
        print("[SKIP] crates-corpus-absent")
        return 3

    rows = parse_claims(claims_abs)
    dups = duplicate_mechanism_check(rows)
    dup_patents = {p for group in dups for p in group}

    results = []
    for row in rows:
        v = verdict_for_row(row, REPO)
        if row["patent"] in dup_patents:
            v["verdict"] = "NOT-DISTINCT"
            v["reason"] = ("duplicate/near-duplicate mechanism shared "
                           "across rows %s" % sorted(dup_patents))
        results.append(v)

    all_distinct = all(r["verdict"] == "DISTINCT-MECHANISM" for r in results)
    court_exit, court_tail = run_behavioral_court(REPO)
    court_ok = court_exit == 0
    print("behavioral court %s (exit %d)"
          % ("PASS" if court_ok else "FAIL", court_exit))
    for line in court_tail:
        print("  | %s" % line)
    for r in results:
        print("%s -> %s (%s)" % (r["patent"], r["verdict"], r["reason"]))
        if r["src_hits"]:
            print("  src hits: %d" % len(r["src_hits"]))
        if r["tests_hits"]:
            print("  tests hits (not practicing): %d" % len(r["tests_hits"]))

    report = {
        "gate": "G7-PATENT",
        "claims_file": claims_abs,
        "claims_sha256": sha256(claims_abs),
        "corpus": corpus,
        "rows": results,
        "duplicate_mechanism_groups": [sorted(g) for g in dups],
        "all_distinct": all_distinct,
        "behavioral_court": {
            "command": " ".join(BEHAVIORAL_COURT_CMD),
            "exit": court_exit,
            "pass": court_ok,
        },
        "gate_exit": 0 if (all_distinct and court_ok) else 1,
    }
    os.makedirs(os.path.dirname(os.path.abspath(args.report)),
                exist_ok=True)
    with open(args.report, "w", encoding="utf-8") as fh:
        json.dump(report, fh, indent=2)
        fh.write("\n")
    if all_distinct and court_ok:
        print("G7-PATENT: PASS 4/4 DISTINCT-MECHANISM + behavioral court (exit 0)")
        return 0
    pending = [r["patent"] for r in results if r["verdict"] != "DISTINCT-MECHANISM"]
    print("G7-PATENT: FAIL (%d/%d DISTINCT, court exit %d) failing rows: %s"
          % (sum(1 for r in results if r["verdict"] == "DISTINCT-MECHANISM"),
             len(results), court_exit, ",".join(pending)))
    return 1


if __name__ == "__main__":
    sys.exit(main())
