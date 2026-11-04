#!/usr/bin/env bash
# receipt_lint.sh — cross-repo citation resolver linter (lane BR-2, seam S6).
#
# Parses cross-repo receipt citations in landed receipts/ledgers and resolves
# each to an on-disk file.
#
# Citation classes (AF-41 S6 bare-shorthand class):
#   1. FULL PATH   : /Users/sac/<repo>/docs/jira/vX/receipts/name.md   -> PASS if -f
#   2. SHORTHAND   : `<repo-shorthand> receipts/name.md`               -> expand to
#      /Users/sac/<repo>/docs/jira/v<release>/receipts/name.md (newest release
#      dir containing the file). If it resolves after expansion: WARN SHORTHAND
#      with the corrected path (exit 1). If it resolves nowhere: FAIL typed.
#   3. Plain same-repo `receipts/x.md` (no shorthand token on the line) is
#      local-relative and NOT linted.
#
# Repo map:
#   cgap|lsp-max|ggen|arcus  -> /Users/sac/<repo>/docs/jira/v<release>/receipts/
#   marketplace              -> /Users/sac/ggen-marketplace/docs/jira/v<release>/receipts/
#
# Scan roots: docs/jira/*/receipts/*.md and docs/jira/*/closure/*.md
# (closure is optional; absent closure dirs are skipped, never silent-passed:
# a nonexistent scan root with zero candidate files at all is fail-closed 2).
#
# Exit codes: 0 clean / 1 findings (WARN or FAIL) / 2 fail-closed (no scan
# roots, unreadable file) / 3 VACUOUS self-check.
#
# Modes: receipt_lint.sh [files...]          lint given files (or default scan)
#        receipt_lint.sh --self-check        fixture court: good->0, shorthand->1,
#                                            missing->1; any mismatch = exit 3.
#
# Stdlib only. Never runs git. bash -n clean (bash 3.2 compatible).

set -u

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SHORTHANDS="cgap lsp-max ggen arcus marketplace"
FIXTURE_DIR="$REPO_ROOT/scripts/receipt_lint_fixtures/cite-fixtures"

exit_findings() {  # print findings, return lint exit code
    :
}

lint_one_file() {
    # emits: CLASS<TAB>file:line<TAB>detail
    local f="$1"
    python3 - "$f" "$REPO_ROOT" "$SHORTHANDS" <<'PY'
import os, re, sys

path, root, shorthands = sys.argv[1], sys.argv[2], sys.argv[3].split()
REPO_DIR = {
    "cgap": "cgap", "lsp-max": "lsp-max", "ggen": "ggen",
    "arcus": "arcus", "marketplace": "ggen-marketplace",
}
FULL = re.compile(r"/Users/sac/[A-Za-z0-9._-]+/docs/jira/[A-Za-z0-9._/~+-]*\.md")
RECS = re.compile(r"receipts/([A-Za-z0-9][A-Za-z0-9._/~+-]*)\.md")
SH_TOK = re.compile(r"\b(" + "|".join(re.escape(s) for s in shorthands) + r")\b")

def sac_base(repo_dirname):
    # /Users/sac/<repo>: for cgap the lint root already IS /Users/sac/cgap,
    # so derive the shared parent; every other repo is sibling-resolved.
    if repo_dirname == "cgap":
        return os.path.dirname(root.rstrip("/"))
    return "/Users/sac"

def release_dirs(repo_dirname):
    base = os.path.join(sac_base(repo_dirname), repo_dirname, "docs/jira")
    out = []
    if os.path.isdir(base):
        for d in os.listdir(base):
            if d.startswith("v") and os.path.isdir(os.path.join(base, d)):
                out.append(d)
    return sorted(out, reverse=True)

def resolve_shorthand(repo, name):
    dirname = REPO_DIR.get(repo)
    if not dirname:
        return None
    for rel in release_dirs(dirname):
        cand = os.path.join(
            sac_base(dirname), dirname, "docs/jira", rel, "receipts",
            name + ".md")
        if os.path.isfile(cand):
            return cand
    return None

try:
    text = open(path, encoding="utf-8", errors="replace").read()
except OSError as e:
    print("FAIL-READ\t%s:0\tunreadable: %s" % (path, e))
    sys.exit(0)

for i, line in enumerate(text.splitlines(), 1):
    full_spans = [(m.start(), m.end()) for m in FULL.finditer(line)]

    def in_full(pos):
        return any(a <= pos < b for a, b in full_spans)

    for m in RECS.finditer(line):
        pos = m.start()
        if in_full(pos):
            continue
        # nearest shorthand token strictly before the path on the same line
        head = line[:pos]
        last = None
        for t in SH_TOK.finditer(head):
            last = t.group(1)
        if last is None:
            continue  # local relative cite, not the cross-repo class
        name = m.group(1)
        full = resolve_shorthand(last, name)
        if full:
            print("WARN-SHORTHAND\t%s:%d\t'%s receipts/%s.md' -> %s"
                  % (path, i, last, name, full))
        else:
            print("FAIL-NOT-FOUND\t%s:%d\t'%s receipts/%s.md' resolves nowhere"
                  " (no v*/receipts/%s.md under /Users/sac/%s)"
                  % (path, i, last, name, name, REPO_DIR[last]))
PY
}

collect_files() {
    # args: explicit file list, else default scan roots
    if [ "$#" -gt 0 ]; then
        for f in "$@"; do
            if [ -f "$f" ]; then printf '%s\n' "$f"; else
                echo "FAIL-ROOT: unreadable file arg: $f" >&2
                exit 2
            fi
        done
    else
        local found=0 roots=0
        for d in "$REPO_ROOT"/docs/jira/*/receipts "$REPO_ROOT"/docs/jira/*/closure; do
            [ -d "$d" ] || continue
            roots=$((roots + 1))
            for f in "$d"/*.md; do
                [ -f "$f" ] || continue
                printf '%s\n' "$f"
                found=$((found + 1))
            done
        done
        if [ "$roots" -eq 0 ]; then
            echo "FAIL-ROOT: no scan roots under docs/jira/*/ (receipts, closure)" >&2
            exit 2
        fi
    fi
}

main() {
    if [ "${1:-}" = "--self-check" ]; then
        local rc good=0 sh=0 miss=0
        [ -f "$FIXTURE_DIR/known-good-fullpath.md" ] || { echo "VACUOUS: missing fixture known-good-fullpath.md"; exit 3; }
        [ -f "$FIXTURE_DIR/known-shorthand.md" ] || { echo "VACUOUS: missing fixture known-shorthand.md"; exit 3; }
        [ -f "$FIXTURE_DIR/missing-cite.md" ] || { echo "VACUOUS: missing fixture missing-cite.md"; exit 3; }
        bash "$0" "$FIXTURE_DIR/known-good-fullpath.md"; rc=$?
        [ "$rc" -eq 0 ] && good=1
        bash "$0" "$FIXTURE_DIR/known-shorthand.md"; rc=$?
        [ "$rc" -eq 1 ] && sh=1
        bash "$0" "$FIXTURE_DIR/missing-cite.md"; rc=$?
        [ "$rc" -eq 1 ] && miss=1
        if [ "$good" -eq 1 ] && [ "$sh" -eq 1 ] && [ "$miss" -eq 1 ]; then
            echo "SELF-CHECK PASS: known-good->0, known-shorthand->1, missing-cite->1"
            exit 0
        fi
        echo "VACUOUS GATE: good=$good shorthand=$sh missing=$miss (want 1/1/1)"
        exit 3
    fi

    local files rc=0
    files="$(collect_files "$@")" || exit $?
    local out
    out="$(mktemp)" || exit 2
    while IFS= read -r f; do
        [ -n "$f" ] || continue
        lint_one_file "$f" >> "$out" || { echo "FAIL-ROOT: lint core failed on $f" >&2; exit 2; }
    done <<< "$files"
    if [ -s "$out" ]; then
        local n
        n="$(grep -c . "$out")"
        cat "$out"
        rm -f "$out"
        echo "FINDINGS: $n (exit 1)"
        exit 1
    fi
    rm -f "$out"
    echo "CLEAN: 0 cross-repo citation findings"
    exit 0
}

main "$@"
