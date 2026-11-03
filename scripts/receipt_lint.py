#!/usr/bin/env python3
"""receipt_lint.py — claim-sentence receipt linter (lane W2-3).

Scope: docs/jira/v26.11.*/waves/*.md plus landing docs directly under
docs/jira/v26.11.*/. A claim sentence (declarative line containing a
landing verb) must carry, within PROXIMITY lines, either a validated
waves/ receipt ref (file exists; cited :N line must be inside the file)
or the exact token [TARGET-v26.11.2]. Otherwise:
    UNRECEIPTED-CLAIM <file>:<line>

Exit codes: 0 clean (or census findings without --strict) / 1 findings
under --strict, or VACUOUS self-check / 2 fail-closed (zero docs,
unreadable file, internal exception) / 3 loud skip. Stdlib-only.

Lane RC-09: the strict arm is typed, never a KeyError crash. A scan
finding lacking a required key ("rule"/"file"/"reason") becomes a loud
MISSING-RULE-KEY row naming the file + the missing key: exit 1 under
--strict (STRICT-FAIL cites RECEIPT-LINT-MISSING-KEY), loud row only
under default mode (no crash). KEY-CRASH court in --self-check flips
both arms; a guard-stripped /tmp copy must fail the court.
"""
import argparse
import json
import os
import re
import sys

CLAIM_RE = re.compile(
    r"\b(landed|removed|verified|green|passes|closed|fixed|deleted|stops"
    r"|pruned)\b", re.IGNORECASE)
TARGET_RE = re.compile(r"\[TARGET-v26\.11\.2\]")
REF_RE = re.compile(r"waves/([A-Za-z0-9][A-Za-z0-9._-]*\.md)(?::(\d+))?")
PROXIMITY = 3
DEFAULT_DIRS = ("docs/jira/v26.11.2/waves",)
# Directories never linted; encountered entries are skipped LOUDLY.
EXCLUDED_DIR_NAMES = {"node_modules", "target", ".git", "__pycache__",
                      "patent", "research", "plans"}
VERDICT_CLEAN = "PASS"
VERDICT_FAIL = "FAIL"
VERDICT_SKIP = "SKIP"


def is_claim(line):
    s = line.strip()
    if not s or s.startswith(("#", "|", "-", ">", "```", "UNRECEIPTED")):
        return False
    if set(s) <= set("-=`~."):
        return False
    return bool(CLAIM_RE.search(s))


def resolve_ref(doc_path, root, name, lineno, claim_words=None):
    """Validate a waves/ receipt ref. Returns (ok, reason)."""
    cand = os.path.normpath(os.path.join(os.path.dirname(doc_path), name))
    if not os.path.isfile(cand):
        # W3-R1: also resolve waves/ refs against the campaign roots — a
        # citation like `waves/w2-07-ost-greps.md` from a doc that is not
        # itself in docs/jira/v26.11.2/ must not read as "receipt file
        # missing" when the receipt exists under the campaign dir.
        for base in ("docs/jira/v26.11.2", "docs/jira"):
            cand2 = os.path.normpath(os.path.join(root, base, name))
            if os.path.isfile(cand2):
                cand = cand2
                break
        if not os.path.isfile(cand):
            return False, "receipt file missing: %s" % name
    with open(cand, "r", errors="replace") as f:
        receipt_lines = f.read().splitlines()
    if lineno is not None:
        if not (1 <= lineno <= len(receipt_lines)):
            return False, "cited line %d out of range in %s" % (lineno, name)
        # D1 fix (w2-04): cited line must exist AND the receipt body must
        # actually mention the claim's subject — a ref to an arbitrary
        # in-range line is a WEAK ref, not a valid one.
        cited = receipt_lines[lineno - 1]
        if len(cited.strip()) < 8:
            return False, "cited line %d in %s is empty/near-empty (not a receipt)" % (lineno, name)
        # content check (w2-17 D1 / w3-r3 residual): the cited line or its
        # +-2 line window must share at least one content word with the
        # claim sentence; an arbitrary unrelated line cannot receipt it.
        window = receipt_lines[max(0, lineno - 3):lineno + 2]
        window_words = set("".join(window).lower().split())
        claim_set = {w.strip(".,;:[]()'\"").lower() for w in (claim_words or []) if len(w) > 3}
        if claim_set and not (claim_set & window_words):
            return False, ("cited line %d in %s shares no content with the claim "
                           "(stale/irrelevant receipt)" % (lineno, name))
    else:
        if not any(len(l.strip()) >= 8 for l in receipt_lines):
            return False, "receipt %s has no substantive lines" % name
    return True, "ok"


def lint_doc(path, root):
    findings = []
    try:
        with open(path, "r", errors="strict") as f:
            lines = f.read().splitlines()
    except Exception as e:  # fail-closed on unreadable
        sys.stderr.write("RECEIPT-LINT: unreadable %s (%s)\n" % (path, e))
        sys.exit(2)
    for i, line in enumerate(lines):
        if not is_claim(line):
            continue
        window = lines[max(0, i - PROXIMITY):i + PROXIMITY + 1]
        window_txt = "\n".join(window)
        if TARGET_RE.search(window_txt):
            continue
        ok = False
        reason = "no waves/ ref in proximity"
        for m in REF_RE.finditer(window_txt):
            claim_words = line.split()
            ok, reason = resolve_ref(path, root, m.group(1),
                                     int(m.group(2)) if m.group(2) else None,
                                     claim_words=claim_words)
            if ok:
                break
        if not ok:
            # GAP-05 crash fix (DF-3): every finding carries a rule id. The
            # strict arm at main() joins rules over ALL findings; claim
            # findings previously lacked "rule" -> KeyError 'rule' at :436.
            findings.append({
                "rule": "UNRECEIPTED-CLAIM",
                "file": os.path.relpath(path, root),
                "line": i + 1,
                "snippet": line.strip()[:100],
                "reason": reason,
            })
    return findings


def collect_dirs(root, dir_args):
    """Returns (docs, skipped) — docs is list of .md files in scope."""
    docs, skipped = [], []
    seen = set()
    for d in dir_args:
        absd = os.path.normpath(os.path.join(root, d))
        base = os.path.basename(absd)
        if base in EXCLUDED_DIR_NAMES:
            skipped.append((absd, "excluded dir name"))
            continue
        if not os.path.isdir(absd):
            skipped.append((absd, "missing"))
            continue
        for entry in sorted(os.listdir(absd)):
            p = os.path.join(absd, entry)
            if os.path.isdir(p):
                if entry in EXCLUDED_DIR_NAMES:
                    skipped.append((p, "excluded dir name"))
                continue
            if entry.endswith(".md") and p not in seen:
                seen.add(p)
                docs.append(p)
        # landing docs: sibling .md files one level above a waves/ dir
        parent = os.path.dirname(absd)
        if os.path.basename(absd) == "waves" and os.path.isdir(parent):
            for entry in sorted(os.listdir(parent)):
                p = os.path.join(parent, entry)
                if (os.path.isfile(p) and entry.endswith(".md")
                        and p not in seen):
                    seen.add(p)
                    docs.append(p)
    return docs, skipped


# ---------------------------------------------------------------------------
# Typed receipt-structure rules (lane DX-4, --strict). Two legal classes:
#
#   class LANE : lane receipts. MUST live in a lanes/ dir, named
#                lane-<lane-id>-<slug>.md; MUST contain >=1 fenced code
#                block and >=1 exit citation. Receipt of record for one
#                lane's contract.
#   class WAVE : wave-level docs under waves/ (any name not starting
#                lane-). Free-form; governed by the claim-sentence lint
#                above, not the lane structure rules.
#
# Irreducible class (NOT mechanical, needs re-derivation): stale-looking
# numeric counts inside claim sentences — a count can only be refreshed by
# re-running the thing it counts, never by editing the sentence.
# ---------------------------------------------------------------------------
EXIT_CITE_RE = re.compile(
    r"\bexit(?:\s+(?:code\s*)?[:=]?\s*)?\d+\b|\brc\s*=\s*\d+\b", re.IGNORECASE)
LANE_NAME_RE = re.compile(r"^lane-[A-Za-z0-9][A-Za-z0-9._-]*\.md$")
STRUCT_RULES = {
    "RECEIPT-STRUCT-001": "lane receipt missing fenced code block",
    "RECEIPT-STRUCT-002": "lane receipt missing exit citation",
    "RECEIPT-STRUCT-003": "lane receipt outside lanes/ (mechanical move)",
    "RECEIPT-STRUCT-004": "wave doc inside lanes/ (mechanical move)",
    "RECEIPT-STRUCT-005": "lane receipt name not lane-<id>-<slug>.md",
}

# ---------------------------------------------------------------------------
# DOC-RECEIPT rule class (lane GAP-05, extends DX-4 per audit/
# doctrine-ship-plan.md section C). Scope: shipped narratives under
# audit/ and waves/ ONLY — lanes/ keeps RECEIPT-STRUCT-*.
#
#   DOC-RECEIPT-001 : quantitative claim (number+unit / count pattern) in
#                     a shipped narrative with no resolvable receipt ref
#                     in the proximity window. NOT mechanical.
#   DOC-RECEIPT-002 : cited receipt path does not resolve. Mechanical.
#   DOC-RECEIPT-003 : missing honesty-header block (status vocabulary +
#                     as-of date). NOT mechanical.
# ---------------------------------------------------------------------------
LANES_REF_RE = re.compile(
    r"lanes/([A-Za-z0-9][A-Za-z0-9._-]*\.md)(?::(\d+))?")
# TC-09 GAP A/B: explicit receipt-cite form. `[receipt: <path>]` names the
# receipting artifact for a claim (001) and is a mechanical path (002).
RECEIPT_CITE_RE = re.compile(
    r"\[?receipt:\s*([A-Za-z0-9][A-Za-z0-9._/-]*\.md)\]?")
# TC-09 GAP D: quantified deferrals parked in ROADMAP-PENDING are declared
# future work, not unreceipted claims.
ROADMAP_PENDING_TOKEN = "ROADMAP-PENDING"
# TC-09 GAP C: interop honesty header vocabulary — sealed-by provenance with
# a named divergence is equivalent to the campaign-local standing vocab.
SEALED_BY_RE = re.compile(r"\bsealed-by\b", re.IGNORECASE)
DIVERGENCE_NAMED_RE = re.compile(r"\bDIVERGENCE-NAMED\b")
VERSION_RE = re.compile(r"\bv?\d+\.\d+(?:\.\d+)?\b")
DATE_STRIP_RE = re.compile(r"\d{4}-\d{2}-\d{2}")
# line-citation / numeric-range artifacts, not quantitative claims:
# "gate-5-silicon.md:27-28", "differential.rs:309", "(:242)", "w2-20"
LINECITE_RE = re.compile(r"[\w.)]:\d+(?:[-:]\d+)*|\b\d+-\d+\b")
QUANT_RE = re.compile(
    r"\b\d+(?:\.\d+)?\s*(?:%|x\b|ms\b|us\b|µs\b|ns\b|KB\b|MB\b|GB\b"
    r"|B\b|bytes?\b|tests?\b|findings?\b|docs?\b|crates?\b|lanes?\b"
    r"|rules?\b|rows?\b|lines\b|counts?\b|checks?\b|files?\b)", re.IGNORECASE)
QUANT_COUNT_RE = re.compile(r"\b\d{2,}\b")
HONESTY_VOCAB_RE = re.compile(
    r"\b(ALIVE|PARTIAL|BLOCKED|UNVERIFIED|REFUSED)\b")
HONESTY_DATE_RE = re.compile(r"\d{4}-\d{2}-\d{2}|\bas[- ]of\b", re.IGNORECASE)
HONESTY_HEADER_WINDOW = 30
DOC_RULES = {
    "DOC-RECEIPT-001": "quantitative claim, no receipt ref in proximity",
    "DOC-RECEIPT-002": "cited receipt path does not resolve (mechanical)",
    "DOC-RECEIPT-003": "missing honesty-header (status vocab + as-of date)",
}

# ---------------------------------------------------------------------------
# LAYER-QUALIFIER rule class (lane TC-12, per tps-07 rank 3 (C1) + the
# witnessed drift in audit/tps-01-reality-ledger.md: the manual's 5.70
# ns/frame kernel-layer figure was quoted unqualified and refuted by
# re-derivation; AX-5 policed "containerized execution" by hand). Scope:
# DOC-RECEIPT-scoped docs (audit/ + waves/), CLAIM ROWS ONLY (markdown
# table rows — the register shape the drift lived in).
#
#   LAYER-QUALIFIER-001 : a latency unit token (ns/frame, ns/call, ns/op,
#     us/batch) in a claim row with NO co-occurring layer token
#     (bare, kernel, membrane, containerized, in-process) within the
#     PROXIMITY window. NOT mechanical: the fix is re-deriving or
#     qualifying the figure, never editing the row blind.
# ---------------------------------------------------------------------------
LAYER_UNIT_RE = re.compile(
    r"\b\d+(?:\.\d+)?\s*(?:ns/(?:frame|call|op)|us/batch)\b")
LAYER_TOKEN_RE = re.compile(
    r"\b(?:bare|kernel|membrane|container\w*|in-process)\b", re.IGNORECASE)
LAYER_RULES = {
    "LAYER-QUALIFIER-001": ("latency figure in claim row without layer "
                            "qualifier (bare|kernel|membrane|containerized"
                            "|in-process) in proximity"),
}


def layer_qualifier_scan(path, root):
    """TC-12: unqualified latency figures in claim rows -> typed finding."""
    out = []
    rel = os.path.relpath(path, root)
    try:
        with open(path, "r", errors="replace") as f:
            lines = f.read().splitlines()
    except Exception as e:
        sys.stderr.write("RECEIPT-LINT: unreadable %s (%s)\n" % (path, e))
        sys.exit(2)
    in_fence = False
    for i, line in enumerate(lines):
        if line.strip().startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        if not line.strip().startswith("|"):
            continue  # claim ROWS only (table register shape)
        if not LAYER_UNIT_RE.search(line):
            continue
        window = lines[max(0, i - PROXIMITY):i + PROXIMITY + 1]
        if LAYER_TOKEN_RE.search("\n".join(window)):
            continue
        out.append({"rule": "LAYER-QUALIFIER-001", "file": rel,
                    "line": i + 1, "snippet": line.strip()[:100],
                    "reason": LAYER_RULES["LAYER-QUALIFIER-001"],
                    "fixable": False, "fix": None})
    return out


def layer_flip_check():
    """TC-12 anti-vacuity arm: LAYER-QUALIFIER-001 must fire on an
    unqualified latency row, stay silent on a layer-qualified row, and
    stay silent on tps-01-reality-ledger.md's own corrected rows
    (self-compliance). All fixtures live in /tmp."""
    import shutil
    import tempfile
    tmp = tempfile.mkdtemp(prefix="receipt_lint_layerflip_")
    try:
        audit = os.path.join(tmp, "audit")
        os.makedirs(audit)
        doc = os.path.join(audit, "ledger.md")
        header = ("# ledger\n\nStanding: ALIVE (2026-11-02).\n\n")

        def rows(firing):
            return [f for f in layer_qualifier_scan(doc, tmp)
                    if f["rule"] == "LAYER-QUALIFIER-001"] if firing else []

        # FLIP A: bare "6.04 ns/frame" row -> fires.
        with open(doc, "w") as f:
            f.write(header + "| x | 6.04 ns/frame |\n")
        fs = layer_qualifier_scan(doc, tmp)
        if not fs or fs[0]["rule"] != "LAYER-QUALIFIER-001":
            sys.stderr.write("VACUOUS GATE: LAYER-QUALIFIER-001 did not "
                             "fire on bare '6.04 ns/frame' row\n")
            return 1
        # FLIP B: layer-qualified row -> silent.
        with open(doc, "a") as f:
            f.write("| y | 6.04 ns/frame (membrane, containerized) |\n")
        fs = layer_qualifier_scan(doc, tmp)
        if any(f["line"] == 5 for f in fs):
            sys.stderr.write("VACUOUS GATE: layer-qualified row flagged\n")
            return 1
        # FLIP C: tps-01's own corrected rows -> silent (self-compliance).
        src = ("/Users/sac/cgap/docs/jira/v26.11.2/audit/"
               "tps-01-reality-ledger.md")
        with open(doc, "w") as f:
            f.write(open(src, "r", errors="replace").read())
        fs = layer_qualifier_scan(doc, tmp)
        if fs:
            sys.stderr.write("VACUOUS GATE: tps-01 corrected rows flagged "
                             "— %s\n" % fs[0]["snippet"])
            return 1
        print("RECEIPT-LINT LAYER-FLIP: PASS (bare row fires; "
              "membrane/containerized row silent; tps-01 self-compliant)")
        return 0
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def doc_receipt_scope(path):
    """DOC-RECEIPT rules apply to audit/ and waves/ docs only."""
    parent = os.path.basename(os.path.dirname(path))
    return parent in ("audit", "waves")


def resolve_any_ref(doc_path, root, name):
    """Pure path resolution for DOC-RECEIPT-002 (no content checks)."""
    cand = os.path.normpath(os.path.join(os.path.dirname(doc_path), name))
    if os.path.isfile(cand):
        return True
    for base in ("docs/jira/v26.11.2", "docs/jira"):
        cand2 = os.path.normpath(os.path.join(root, base, name))
        if os.path.isfile(cand2):
            return True
    # TC-09: root-relative receipt paths (receipts/ sibling of the doc tree).
    cand3 = os.path.normpath(os.path.join(root, name))
    return os.path.isfile(cand3)


def doc_receipt_scan(path, root):
    out = []
    rel = os.path.relpath(path, root)
    try:
        with open(path, "r", errors="replace") as f:
            lines = f.read().splitlines()
    except Exception as e:
        sys.stderr.write("RECEIPT-LINT: unreadable %s (%s)\n" % (path, e))
        sys.exit(2)
    # TC-09 GAP C: interop honesty vocabulary — sealed-by provenance with a
    # named divergence is equivalent to the campaign-local standing vocab.
    header = lines[:HONESTY_HEADER_WINDOW]
    has_vocab = any(HONESTY_VOCAB_RE.search(l) for l in header)
    has_sealed = (any(SEALED_BY_RE.search(l) for l in header)
                  and any(DIVERGENCE_NAMED_RE.search(l) for l in header))
    if not (has_vocab or has_sealed):
        out.append({"rule": "DOC-RECEIPT-003", "file": rel, "line": 0,
                    "snippet": os.path.basename(path),
                    "reason": DOC_RULES["DOC-RECEIPT-003"],
                    "fixable": False, "fix": None})
    elif not any(HONESTY_DATE_RE.search(l) for l in header):
        out.append({"rule": "DOC-RECEIPT-003", "file": rel, "line": 0,
                    "snippet": os.path.basename(path),
                    "reason": DOC_RULES["DOC-RECEIPT-003"],
                    "fixable": False, "fix": None})
    in_fence = False
    for i, line in enumerate(lines):
        if line.strip().startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        unresolved = False
        for m in REF_RE.finditer(line):
            if not resolve_any_ref(path, root, m.group(1)):
                unresolved = True
        for m in LANES_REF_RE.finditer(line):
            if not resolve_any_ref(path, root, m.group(1)):
                unresolved = True
        # TC-09 GAP B: receipt: cites are mechanical paths — a dangling
        # cite must fire 002 (was a silent false NEGATIVE before TC-09).
        for m in RECEIPT_CITE_RE.finditer(line):
            if not resolve_any_ref(path, root, m.group(1)):
                unresolved = True
        if unresolved:
            out.append({"rule": "DOC-RECEIPT-002", "file": rel,
                        "line": i + 1, "snippet": line.strip()[:100],
                        "reason": DOC_RULES["DOC-RECEIPT-002"],
                        "fixable": False, "fix": None})
        prose_q = LINECITE_RE.sub("", DATE_STRIP_RE.sub(
            "", VERSION_RE.sub("", line)))
        # TC-09 GAP A: CTQ-table rows ARE the receipt (the emitted template
        # self-receipts its measurements) — 001 stays silent on them.
        if line.strip().startswith("|"):
            continue
        if QUANT_RE.search(line) or QUANT_COUNT_RE.search(prose_q):
            window = lines[max(0, i - PROXIMITY):i + PROXIMITY + 1]
            window_txt = "\n".join(window)
            if TARGET_RE.search(window_txt):
                continue
            if "```" in window_txt:
                continue
            # TC-09 GAP D: a quantified ROADMAP-PENDING deferral is declared
            # future work, not an unreceipted claim.
            if ROADMAP_PENDING_TOKEN in line:
                continue
            has_ref = any(resolve_any_ref(path, root, m.group(1))
                          for m in REF_RE.finditer(window_txt))
            has_ref = has_ref or any(resolve_any_ref(path, root, m.group(1))
                                     for m in LANES_REF_RE.finditer(window_txt))
            # TC-09 GAP A: [receipt: <path>] cite names the receipting
            # artifact; a RESOLVED receipt cite receipts the claim.
            has_ref = has_ref or any(
                resolve_any_ref(path, root, m.group(1))
                for m in RECEIPT_CITE_RE.finditer(window_txt))
            if not has_ref:
                out.append({"rule": "DOC-RECEIPT-001", "file": rel,
                            "line": i + 1, "snippet": line.strip()[:100],
                            "reason": DOC_RULES["DOC-RECEIPT-001"],
                            "fixable": False, "fix": None})
    return out


def doc_class(path):
    parts = path.split(os.sep)
    parent = parts[-2] if len(parts) >= 2 else ""
    name = parts[-1]
    if parent == "lanes" or LANE_NAME_RE.match(name):
        return "lane"
    if parent == "waves":
        return "wave"
    return "landing"


def lane_rule_scan(docs, root):
    """Typed structural findings. Each finding carries rule, fixable."""
    out = []
    for p in docs:
        rel = os.path.relpath(p, root)
        name = os.path.basename(p)
        parent = os.path.basename(os.path.dirname(p))
        cls = doc_class(p)
        if parent == "lanes" and cls != "lane":
            out.append({"rule": "RECEIPT-STRUCT-004", "file": rel, "line": 0,
                        "snippet": name,
                        "reason": "wave doc inside lanes/; move to waves/",
                        "fixable": True, "fix": "move:%s" % name})
            continue
        if cls != "lane":
            continue
        if parent != "lanes":
            out.append({"rule": "RECEIPT-STRUCT-003", "file": rel, "line": 0,
                        "snippet": name,
                        "reason": "lane-named doc outside lanes/; "
                                  "move to lanes/",
                        "fixable": True, "fix": "move:lanes/%s" % name})
            continue
        if not LANE_NAME_RE.match(name):
            out.append({"rule": "RECEIPT-STRUCT-005", "file": rel, "line": 0,
                        "snippet": name,
                        "reason": "lane receipt name not "
                                  "lane-<id>-<slug>.md", "fixable": False,
                        "fix": None})
            continue
        try:
            with open(p, "r", errors="replace") as f:
                text = f.read()
        except Exception as e:
            sys.stderr.write("RECEIPT-LINT: unreadable %s (%s)\n" % (p, e))
            sys.exit(2)
        if "```" not in text:
            out.append({"rule": "RECEIPT-STRUCT-001", "file": rel, "line": 0,
                        "snippet": name,
                        "reason": STRUCT_RULES["RECEIPT-STRUCT-001"],
                        "fixable": False, "fix": None})
        if not EXIT_CITE_RE.search(text):
            out.append({"rule": "RECEIPT-STRUCT-002", "file": rel, "line": 0,
                        "snippet": name,
                        "reason": STRUCT_RULES["RECEIPT-STRUCT-002"],
                        "fixable": 2, "fix": None})
        else:
            pass
    return out


def apply_fixes(struct_findings, root, dry_run=True):
    """Mechanical class only: path moves lanes/ <-> waves/ by clear name."""
    moved, planned = [], []
    for f in struct_findings:
        if not f.get("fixable") or not (f.get("fix") or "").startswith("move:"):
            continue
        src = os.path.join(root, f["file"])
        dst_name = f["fix"].split(":", 1)[1].split("/")[-1]
        dst_dir = "lanes" if f["rule"] == "RECEIPT-STRUCT-003" else "waves"
        dst = os.path.normpath(os.path.join(
            root, "docs/jira/v26.11.2", dst_dir, dst_name))
        planned.append((f["file"], os.path.relpath(dst, root)))
        if dry_run:
            continue
        if not os.path.isfile(src):
            sys.stderr.write("RECEIPT-LINT: fix source missing %s\n" % src)
            continue
        if os.path.exists(dst):
            sys.stderr.write("RECEIPT-LINT: fix dest exists, refusing %s\n"
                             % dst)
            continue
        os.rename(src, dst)
        moved.append((f["file"], os.path.relpath(dst, root)))
    return moved, planned


def normalize_finding_keys(findings, struct):
    """RC-09: every finding lacking a required key becomes a loud typed
    MISSING-RULE-KEY row (appended to struct so --strict exits 1 citing
    RECEIPT-LINT-MISSING-KEY). Returns the typed rows; never raises."""
    required = ("rule", "file", "reason")
    rows = []
    for f in list(findings) + list(struct):
        missing = [k for k in required if k not in f]
        if not missing:
            continue
        rows.append({
            "rule": "RECEIPT-LINT-MISSING-KEY",
            "file": f.get("file", "<unknown>"),
            "line": f.get("line", 0),
            "snippet": "",
            "reason": "finding lacks required key(s) %s — typed refusal, "
                      "not a KeyError crash"
                      % ",".join("'%s'" % k for k in missing),
        })
    return rows


def key_crash_court():
    """RC-09 anti-vacuity arm: (1) a scan emitting a finding without a
    'rule' key must yield a typed MISSING-RULE-KEY row naming the file +
    key under --strict (exit 1, NO traceback); (2) the same mutated /tmp
    script copy with the normalize guard stripped must FAIL this court
    (no typed row == vacuous guard); (3) a complete fixture still exits
    0. All fixtures and mutated copies live in /tmp; the shared tree is
    never mutated."""
    import shutil
    import tempfile
    import subprocess
    tmp = tempfile.mkdtemp(prefix="receipt_lint_keycrash_")
    try:
        root = os.path.join(tmp, "repo")
        waves = os.path.join(root, "waves")
        os.makedirs(waves)
        # Malformed-finding fixture: dangling cite -> DOC-RECEIPT-002.
        with open(os.path.join(waves, "fixture.md"), "w") as f:
            f.write("# fixture\n\nStanding: ALIVE (2026-11-02).\n\n"
                    "See waves/w9-nope-missing.md for proof.\n")
        # Complete fixture root: only w-ref.md (honest header, receipted).
        good = os.path.join(tmp, "good")
        gwaves = os.path.join(good, "waves")
        os.makedirs(gwaves)
        with open(os.path.join(gwaves, "w-ref.md"), "w") as f:
            f.write("# w-ref\n\nStanding: ALIVE (2026-11-02).\n\n"
                    "substantive receipt body line here\n")

        def run(script, root_arg, extra=()):
            return subprocess.run(
                [sys.executable, script, "--root", root_arg,
                 "--dir", "waves"] + list(extra),
                capture_output=True, text=True)

        real = os.path.abspath(__file__)
        # Arm 3: complete fixture -> strict exit 0 (typed PASS).
        p = run(real, good, ["--strict"])
        if p.returncode != 0 or "Traceback" in p.stderr:
            sys.stderr.write("VACUOUS GATE: complete fixture failed "
                             "--strict rc=%d %s\n" % (p.returncode,
                                                      p.stderr[-200:]))
            return 1
        # Arm 1: mutate a /tmp copy so a scan emits a finding WITHOUT the
        # 'rule' key — the real script must answer with a typed
        # MISSING-RULE-KEY row naming the file + key, never a traceback.
        mut = os.path.join(tmp, "mutated.py")
        src = open(real, "r").read()
        m1 = src.replace('"rule": "DOC-RECEIPT-002"',
                         '"x-rule": "DOC-RECEIPT-002"')
        if m1 == src:
            sys.stderr.write("VACUOUS GATE: mutation 1 (drop rule key) "
                             "is a no-op\n")
            return 1
        with open(mut, "w") as f:
            f.write(m1)
        p = run(mut, root, ["--strict"])
        if "Traceback" in p.stderr:
            sys.stderr.write("VACUOUS GATE: strict arm crashed with "
                             "KeyError instead of typed refusal\n")
            return 1
        if p.returncode != 1 or "RECEIPT-LINT-MISSING-KEY" not in p.stderr \
                or "fixture.md" not in p.stdout + p.stderr \
                or "'rule'" not in p.stdout + p.stderr:
            sys.stderr.write("VACUOUS GATE: missing-key finding not typed "
                             "(rc=%d stderr=%r out=%r)\n"
                             % (p.returncode, p.stderr[-200:],
                                p.stdout[-200:]))
            return 1
        # Default mode: loud row, no crash, no strict exit.
        p = run(mut, root)
        if "Traceback" in p.stderr or "RECEIPT-LINT-MISSING-KEY" \
                not in p.stdout + p.stderr:
            sys.stderr.write("VACUOUS GATE: default mode crash or silent "
                             "missing-key finding\n")
            return 1
        # Arm 2 (anti-vacuity): strip the normalize guard from the /tmp
        # copy — the court must REFUSE (typed row absent == guard strip
        # undetected would be a vacuous gate).
        stripped = m1.replace(
            "missing_key_rows = normalize_finding_keys(findings, struct)",
            "missing_key_rows = []")
        if stripped == m1:
            sys.stderr.write("VACUOUS GATE: guard-strip mutation is a "
                             "no-op\n")
            return 1
        mut2 = os.path.join(tmp, "stripped.py")
        with open(mut2, "w") as f:
            f.write(stripped)
        p = run(mut2, root, ["--strict"])
        if "RECEIPT-LINT-MISSING-KEY" in p.stderr:
            sys.stderr.write("VACUOUS GATE: guard-stripped copy still "
                             "typed — court cannot flip\n")
            return 1
        print("RECEIPT-LINT KEY-CRASH: PASS (missing-key finding typed, "
              "named file+'rule', no traceback; default mode loud row; "
              "guard-stripped copy refused)")
        return 0
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def strict_check():
    """DX-4 anti-vacuity arm: a lane receipt with zero exit citations MUST
    fail --strict citing RECEIPT-STRUCT-002; a compliant receipt MUST
    pass. Runs the real CLI over a /tmp fixture."""
    import shutil
    import tempfile
    tmp = tempfile.mkdtemp(prefix="receipt_lint_strict_")
    try:
        lanes = os.path.join(tmp, "lanes")
        os.makedirs(lanes)
        bad = os.path.join(lanes, "lane-dx9-bad.md")
        with open(bad, "w") as f:
            f.write("# lane-dx9-bad\n\n```\nplain block\n```\n\n"
                    "All work done.\n")  # fence, NO exit citation
        rc = main(["--root", tmp, "--dir", "lanes", "--strict"])
        if rc != 1:
            sys.stderr.write("VACUOUS GATE: exit-citation-less receipt "
                             "passed --strict (rc=%d)\n" % rc)
            return 1
        with open(bad, "a") as f:
            f.write("\nexit 0\n")
        rc = main(["--root", tmp, "--dir", "lanes", "--strict"])
        if rc != 0:
            sys.stderr.write("VACUOUS GATE: compliant receipt failed "
                             "--strict (rc=%d)\n" % rc)
            return 1
        # mechanical-move flip: lane-named doc in waves/ must be planned.
        waves = os.path.join(tmp, "waves")
        os.makedirs(waves)
        stray = os.path.join(waves, "lane-stray.md")
        with open(stray, "w") as f:
            f.write("# lane-stray\n\nexit 0\n\n```\nx\n```\n")
        rc = main(["--root", tmp, "--dir", "lanes", "--dir", "waves",
                   "--dry-run"])
        return 0 if rc == 0 else rc
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def doc_flip_check():
    """GAP-05 anti-vacuity arm: each DOC-RECEIPT rule must flip. All
    fixtures live in /tmp; the shared tree is untouched."""
    import shutil
    import tempfile
    tmp = tempfile.mkdtemp(prefix="receipt_lint_docflip_")
    try:
        waves = os.path.join(tmp, "waves")
        os.makedirs(waves)
        with open(os.path.join(waves, "w-ref.md"), "w") as f:
            f.write("# w-ref\n\nStanding: ALIVE (2026-11-02).\n\n"
                    "substantive receipt body line here\n")
        doc = os.path.join(waves, "w-doc.md")
        header = ("# w-doc\n\nStanding: ALIVE (2026-11-02).\n\n")

        def strict_rc():
            return main(["--root", tmp, "--dir", "waves", "--strict"])

        def rules_cited():
            import subprocess
            p = subprocess.run(
                [sys.executable, os.path.abspath(__file__), "--root", tmp,
                 "--dir", "waves", "--strict"],
                capture_output=True, text=True)
            line = [l for l in p.stderr.splitlines()
                    if "STRICT-FAIL" in l]
            return line[0] if line else ""

        # 001 flip: quant claim, no ref -> exit 1; add resolvable ref -> 0.
        with open(doc, "w") as f:
            f.write(header + "The kernel is 3.4x faster.\n")
        if strict_rc() != 1 or "DOC-RECEIPT-001" not in rules_cited():
            sys.stderr.write("VACUOUS GATE: DOC-RECEIPT-001 did not fire "
                             "on '3.4x faster' with no ref\n")
            return 1
        with open(doc, "a") as f:
            f.write("\nSee waves/w-ref.md:3 for the measurement.\n")
        if strict_rc() != 0:
            sys.stderr.write("VACUOUS GATE: receipted quant claim still "
                             "fails --strict\n")
            return 1
        # 002 flip: cited path that does not resolve.
        with open(doc, "w") as f:
            f.write(header + "See waves/w9-nope-missing.md for proof.\n")
        if strict_rc() != 1 or "DOC-RECEIPT-002" not in rules_cited():
            sys.stderr.write("VACUOUS GATE: DOC-RECEIPT-002 did not fire "
                             "on a non-resolving citation\n")
            return 1
        # 003 flip: honesty header missing.
        with open(doc, "w") as f:
            f.write("# w-doc\n\nJust narrative prose, no standing "
                    "vocabulary anywhere.\n")
        if strict_rc() != 1 or "DOC-RECEIPT-003" not in rules_cited():
            sys.stderr.write("VACUOUS GATE: DOC-RECEIPT-003 did not fire "
                             "on a header-less doc\n")
            return 1
        print("RECEIPT-LINT DOC-FLIP: PASS (001/002/003 each fire and "
              "001 clears on receipt)")
        return 0
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def gap_flip_check():
    """TC-09 anti-vacuity arm (GAPs A-D): each fixture in
    scripts/receipt_lint_fixtures/ must be SILENT on its rule as shipped
    (or, for the negative fixture, FIRE as shipped), then FLIP to typed
    exit 1 citing its rule under the named mutation, applied to /tmp
    sandbox copies only. One sandbox root per gap — fixtures do not
    contaminate each other's baselines. The shared tree is never mutated."""
    import shutil
    import tempfile
    fixtures = os.path.normpath(os.path.join(
        os.path.dirname(os.path.abspath(__file__)),
        "receipt_lint_fixtures"))
    if not os.path.isdir(fixtures):
        sys.stderr.write("VACUOUS GATE: fixture dir missing %s\n" % fixtures)
        return 1

    def rules_cited(root):
        import subprocess
        p = subprocess.run(
            [sys.executable, os.path.abspath(__file__), "--root", root,
             "--dir", "waves", "--strict"],
            capture_output=True, text=True)
        line = [l for l in p.stderr.splitlines() if "STRICT-FAIL" in l]
        return line[0] if line else ""

    def sandbox(tag, doc_name):
        root = tempfile.mkdtemp(prefix="receipt_lint_gapflip_%s_" % tag)
        shutil.copytree(os.path.join(fixtures, "waves"),
                        os.path.join(root, "waves"))
        shutil.copytree(os.path.join(fixtures, "receipts"),
                        os.path.join(root, "receipts"))
        for stray in os.listdir(os.path.join(root, "waves")):
            if stray != doc_name:
                os.remove(os.path.join(root, "waves", stray))
        return root

    results = []
    # --- GAP A: CTQ rows + resolved [receipt:] cite -> silent as shipped;
    # flattened row / dropped cite -> DOC-RECEIPT-001 fires.
    root = sandbox("a", "fixture-gap-a.md")
    fd = os.path.join(root, "waves", "fixture-gap-a.md")
    try:
        base = open(fd).read()
        rc = main(["--root", root, "--dir", "waves", "--strict"])
        if rc != 0:
            sys.stderr.write("VACUOUS GATE: GAP-A fixture not silent as "
                             "shipped (%s)\n" % rules_cited(root))
            return 1
        mut1 = base.replace(
            "| ingress | 100ms | 12ms |\n",
            "ingress measured 100ms target 12ms observed.\n").replace(
            "- [receipt: receipts/w9-gap-a-receipt.md] measurement "
            "recorded, 12ms observed.\n", "")
        if mut1 == base:
            sys.stderr.write("VACUOUS GATE: GAP-A mutation A1 is a no-op\n")
            return 1
        open(fd, "w").write(mut1)
        if main(["--root", root, "--dir", "waves", "--strict"]) != 1 or \
                "DOC-RECEIPT-001" not in rules_cited(root):
            sys.stderr.write("VACUOUS GATE: GAP-A flattened CTQ row did "
                             "not fire 001\n")
            return 1
        open(fd, "w").write(base.replace(
            "[receipt: receipts/w9-gap-a-receipt.md]",
            "[receipt: receipts/w9-nope-missing.md]"))
        if main(["--root", root, "--dir", "waves", "--strict"]) != 1 or \
                "DOC-RECEIPT-001" not in rules_cited(root):
            sys.stderr.write("VACUOUS GATE: GAP-A dropped receipt cite "
                             "did not fire 001\n")
            return 1
        open(fd, "w").write(base)
        shutil.rmtree(root, ignore_errors=True)
        results.append("A")
    finally:
        shutil.rmtree(root, ignore_errors=True)
    # --- GAP B: dangling receipt: cite -> DOC-RECEIPT-002 fires as
    # shipped (was a silent false NEGATIVE); creating the receipt -> silent.
    root = sandbox("b", "fixture-gap-b.md")
    fb = os.path.join(root, "waves", "fixture-gap-b.md")
    try:
        base = open(fb).read()
        if main(["--root", root, "--dir", "waves", "--strict"]) != 1 or \
                "DOC-RECEIPT-002" not in rules_cited(root):
            sys.stderr.write("VACUOUS GATE: GAP-B dangling receipt cite "
                             "did not fire 002\n")
            return 1
        os.makedirs(os.path.join(root, "receipts"), exist_ok=True)
        with open(os.path.join(root, "receipts", "w9-nope-missing.md"),
                  "w") as f:
            f.write("# w9-nope-missing\n\nsubstantive receipt body\n")
        if main(["--root", root, "--dir", "waves", "--strict"]) != 0 or \
                "DOC-RECEIPT-002" in rules_cited(root):
            sys.stderr.write("VACUOUS GATE: GAP-B still fires with the "
                             "receipt present\n")
            return 1
        results.append("B")
    finally:
        shutil.rmtree(root, ignore_errors=True)
    # --- GAP C: sealed-by + DIVERGENCE-NAMED + dated caveat -> 003 silent
    # as shipped; stripping the interop vocabulary -> 003 fires.
    root = sandbox("c", "fixture-gap-c.md")
    fc = os.path.join(root, "waves", "fixture-gap-c.md")
    try:
        base = open(fc).read()
        fs = doc_receipt_scan(fc, root)
        if any(f["rule"] == "DOC-RECEIPT-003" for f in fs):
            sys.stderr.write("VACUOUS GATE: GAP-C interop honesty header "
                             "fired 003\n")
            return 1
        stripped = "\n".join(
            l for l in base.splitlines()
            if "sealed-by" not in l.lower()
            and "DIVERGENCE-NAMED" not in l) + "\n"
        open(fc, "w").write(stripped)
        fs = doc_receipt_scan(fc, root)
        if not any(f["rule"] == "DOC-RECEIPT-003" for f in fs):
            sys.stderr.write("VACUOUS GATE: stripped header did not fire "
                             "003\n")
            return 1
        results.append("C")
    finally:
        shutil.rmtree(root, ignore_errors=True)
    # --- GAP D: quantified ROADMAP-PENDING deferral -> silent as shipped;
    # dropping the token -> DOC-RECEIPT-001 fires.
    root = sandbox("d", "fixture-gap-d.md")
    fx = os.path.join(root, "waves", "fixture-gap-d.md")
    try:
        base = open(fx).read()
        if main(["--root", root, "--dir", "waves", "--strict"]) != 0:
            sys.stderr.write("VACUOUS GATE: GAP-D deferral fixture not "
                             "silent as shipped\n")
            return 1
        mut = base.replace("Deferred 3 lanes to ROADMAP-PENDING.",
                           "Deferred 3 lanes.")
        if mut == base:
            sys.stderr.write("VACUOUS GATE: GAP-D mutation is a no-op\n")
            return 1
        open(fx, "w").write(mut)
        if main(["--root", root, "--dir", "waves", "--strict"]) != 1 or \
                "DOC-RECEIPT-001" not in rules_cited(root):
            sys.stderr.write("VACUOUS GATE: GAP-D un-parked deferral did "
                             "not fire 001\n")
            return 1
        results.append("D")
    finally:
        shutil.rmtree(root, ignore_errors=True)
    print("RECEIPT-LINT GAP-FLIP: PASS (A ctq-row+receipt-cite silent then "
          "fires 001; B dangling cite fires 002 then clears; C sealed-by "
          "silent then fires 003; D deferral silent then fires 001)")
    return 0


def self_check():
    """Non-vacuity arm: claim-less fixture in /tmp MUST fail."""
    import shutil
    import tempfile
    tmp = tempfile.mkdtemp(prefix="receipt_lint_selfcheck_")
    try:
        waves = os.path.join(tmp, "waves")
        os.makedirs(waves)
        with open(os.path.join(waves, "w9-09-receipt.md"), "w") as f:
            f.write("# w9-09-receipt\n\n| Standing | ALIVE |\n\n")
        bad = os.path.join(waves, "w9-99-bad.md")
        with open(bad, "w") as f:
            f.write("# w9-99-bad\n\nThe gate landed and everything is "
                    "green in production this morning.\n")
        good = os.path.join(waves, "w9-98-good.md")
        with open(good, "w") as f:
            f.write("# w9-98-good\n\nSee waves/w9-09-receipt.md:3 — "
                    "the fix landed this wave.\n\nAlso deferred: "
                    "[TARGET-v26.11.2] removal of the shim.\n")
        found = lint_doc(bad, tmp)
        clean = lint_doc(good, tmp)
        if not found:
            sys.stderr.write("VACUOUS GATE: claim-less fixture not "
                             "flagged; linter cannot flip\n")
            return 1
        if clean:
            sys.stderr.write("VACUOUS GATE: receipted fixture flagged "
                             "as unreceipted\n")
            return 1
        print("RECEIPT-LINT SELF-CHECK: PASS (bad fixture flagged %s:%d, "
              "receipted fixture clean)" % (found[0]["file"],
                                            found[0]["line"]))
        rc_key = key_crash_court()
        if rc_key != 0:
            return rc_key
        rc_strict = strict_check()
        if rc_strict != 0:
            return rc_strict
        rc_doc = doc_flip_check()
        if rc_doc != 0:
            return rc_doc
        rc_layer = layer_flip_check()
        if rc_layer != 0:
            return rc_layer
        rc_gap = gap_flip_check()
        if rc_gap != 0:
            return rc_gap
        return 0
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def main(argv=None):
    ap = argparse.ArgumentParser(
        description="Claim-sentence receipt linter for cgap wave docs.")
    ap.add_argument("--root", default=os.getcwd())
    ap.add_argument("--dir", action="append", default=None,
                    help="receipts dir (repeatable; default %s)"
                    % (DEFAULT_DIRS,))
    ap.add_argument("--strict", action="store_true",
                    help="exit 1 on unreceipted claims + typed structure "
                         "rules (RECEIPT-STRUCT-*)")
    ap.add_argument("--fix", action="store_true",
                    help="apply mechanical path moves (RECEIPT-STRUCT-003/004)")
    ap.add_argument("--dry-run", action="store_true",
                    help="print planned moves without applying")
    ap.add_argument("--json", action="store_true", dest="as_json")
    ap.add_argument("--self-check", action="store_true")
    args = ap.parse_args(argv)

    if args.self_check:
        rc = self_check()
        if rc != 0:
            print("RECEIPT-LINT: FAIL self-check rc=%d" % rc)
        return rc

    root = os.path.abspath(args.root)
    dir_args = args.dir if args.dir else [
        os.path.join("docs/jira/v26.11.2/waves")]
    try:
        docs, skipped = collect_dirs(root, dir_args)
    except Exception as e:
        sys.stderr.write("RECEIPT-LINT: collection failure %s\n" % e)
        return 2
    for p, why in skipped:
        print("[SKIP] %s (%s)" % (p, why))
    if skipped and not docs:
        print("RECEIPT-LINT: SKIP (no lintable dirs)")
        return 3
    if not docs:
        sys.stderr.write("RECEIPT-LINT: zero docs scanned under %s — "
                         "fail-closed\n" % root)
        return 2

    findings = []
    for d in docs:
        findings.extend(lint_doc(d, root))
    struct = lane_rule_scan(docs, root)
    for p in docs:
        if doc_receipt_scope(p):
            struct.extend(doc_receipt_scan(p, root))
            struct.extend(layer_qualifier_scan(p, root))
    missing_key_rows = normalize_finding_keys(findings, struct)
    struct.extend(missing_key_rows)
    findings.sort(key=lambda f: (f.get("file", ""), f.get("line", 0)))
    struct.sort(key=lambda f: (f.get("rule", ""), f.get("file", "")))
    verdict = VERDICT_FAIL if (findings or struct) else VERDICT_CLEAN

    if args.fix or args.dry_run:
        moved, planned = apply_fixes(struct, root,
                                     dry_run=not (args.fix and not args.dry_run))
        if args.dry_run or not args.fix:
            for a, b in planned:
                print("PLAN-MOVE %s -> %s" % (a, b))
            print("RECEIPT-LINT: %d planned move(s) (dry-run)"
                  % len(planned))
        else:
            for a, b in moved:
                print("MOVED %s -> %s" % (a, b))
            print("RECEIPT-LINT: %d move(s) applied" % len(moved))
        return 0

    if args.as_json:
        print(json.dumps({
            "gate": "receipt_lint",
            "verdict": verdict,
            "files_scanned": len(docs),
            "findings": findings,
            "struct_findings": struct,
            "unreceipted_claims": len(findings),
            "struct_counts": {r: sum(1 for f in struct
                                     if f.get("rule") == r)
                              for r in list(STRUCT_RULES) + list(DOC_RULES)},
            "seed": None,
        }))
    else:
        for f in findings:
            print("UNRECEIPTED-CLAIM %s:%d %s [%s]"
                  % (f.get("file", "?"), f.get("line", 0),
                     f.get("snippet", ""), f.get("reason", "")))
        for f in struct:
            print("%s %s [%s]"
                  % (f.get("rule", ""), f.get("file", ""),
                     f.get("reason", "")))
        print("RECEIPT-LINT: %s %d claim + %d struct (%d docs scanned)"
              % (verdict, len(findings), len(struct), len(docs)))

    if (findings or struct) and args.strict:
        # GAP-05: rule ids are carried on every finding (claim findings are
        # UNRECEIPTED-CLAIM); --strict is a typed exit 1, never a panic.
        ids = sorted({f.get("rule", "") for f in findings} |
                     {f.get("rule", "") for f in struct})
        sys.stderr.write("RECEIPT-LINT: STRICT-FAIL citing %s\n"
                         % ",".join(ids))
        return 1
    if args.strict:
        print("RECEIPT-LINT: STRICT-PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
