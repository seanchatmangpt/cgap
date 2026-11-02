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
            findings.append({
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
                    help="exit 1 on any UNRECEIPTED-CLAIM")
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
    findings.sort(key=lambda f: (f["file"], f["line"]))
    verdict = VERDICT_FAIL if findings else VERDICT_CLEAN

    if args.as_json:
        print(json.dumps({
            "gate": "receipt_lint",
            "verdict": verdict,
            "files_scanned": len(docs),
            "findings": findings,
            "unreceipted_claims": len(findings),
            "seed": None,
        }))
    else:
        for f in findings:
            print("UNRECEIPTED-CLAIM %s:%d %s [%s]"
                  % (f["file"], f["line"], f["snippet"], f["reason"]))
        print("RECEIPT-LINT: %s %d (%d docs scanned)"
              % (verdict, len(findings), len(docs)))

    if findings and args.strict:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
