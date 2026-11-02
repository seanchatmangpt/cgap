#!/bin/bash
# write-integrity.sh — PostToolUse Edit|Write corruption scan (DX-2).
# Scope: CODE files only — .rs .py .sh .toml .c .h .cc .cpp .hpp .js .ts
# that are >100 lines OR live under crates/ or scripts/. Never fires on
# docs prose (.md/.txt/.rst/.adoc, docs/, ontologies/) — false-positive
# breeding ground (lane DX-2 dispatch).
# Checks: (a) NUL/control bytes (excl \n\t); (b) brace/paren balance
# DELTA vs pre-edit HEAD state (tracked only; untracked -> typed skip);
# (c) stray tokens: PLACEHOLDER, FIXME-PLACEHOLDER, mojibake runs
# (U+FFFD, latin1-desync pairs like Ã©/â€).
# clean -> exit 0 silent; suspicious -> exit 2 with finding + re-verify.
set -u
IN="$(cat)"
FP="$(printf '%s' "$IN" | python3 -c '
import sys, json
try:
    d = json.load(sys.stdin)
    print(d.get("tool_input", {}).get("file_path", ""))
except Exception:
    print("")
')"
[ -n "$FP" ] && [ -f "$FP" ] || exit 0
case "$FP" in
  *.md|*.txt|*.rst|*.adoc|*/docs/*|*/ontologies/*) exit 0 ;;
esac
case "$FP" in
  *.rs|*.py|*.sh|*.bash|*.zsh|*.toml|*.c|*.h|*.cc|*.cpp|*.hpp|*.js|*.ts) ;;
  *) exit 0 ;;
esac
LINES="$(wc -l < "$FP")"
case "$FP" in
  */crates/*|*/scripts/*) : ;;
  *) [ "$LINES" -gt 100 ] || exit 0 ;;
esac
FINDINGS=""
# (a) NUL / control bytes (exclude \n and \t)
if python3 -c '
import sys
data = open(sys.argv[1], "rb").read()
bad = [b for b in data if (b < 9) or (b in (11, 12)) or (13 < b < 32) or b == 127]
sys.exit(0 if bad else 1)' "$FP"; then
  FINDINGS="$FINDINGS
CONTROL_BYTES: NUL/control byte in $FP"
fi
# (b) brace/paren balance DELTA vs pre-edit HEAD state (tracked only)
REL="$(git -C /Users/sac/cgap ls-files --full-name -- "$FP" 2>/dev/null | head -1)"
if [ -n "$REL" ]; then
  BASE="$(git -C /Users/sac/cgap show "HEAD:$REL" 2>/dev/null | wc -c)"
  if [ "$BASE" -gt 0 ]; then
    BAL="$(python3 -c '
import sys
d = open(sys.argv[1], "rb").read()
print(d.count(b"{")-d.count(b"}"), d.count(b"(")-d.count(b")"))' "$FP")"
    BAL0="$(git -C /Users/sac/cgap show "HEAD:$REL" | python3 -c '
import sys
d = sys.stdin.buffer.read()
print(d.count(b"{")-d.count(b"}"), d.count(b"(")-d.count(b")"))')"
    if [ "$BAL" != "$BAL0" ]; then
      FINDINGS="$FINDINGS
BRACE_PAREN_DELTA: balance post-edit ($BAL) differs from HEAD ($BAL0) in $FP"
    fi
  fi
else
  echo "[write-integrity:typed-skip] balance check skipped (untracked or git-blocked): $FP" 1>&2
fi
# (c) stray tokens / mojibake
TOK="$(python3 -c '
import sys, re
d = open(sys.argv[1], "rb").read()
hits = []
if re.search(rb"PLACEHOLDER|FIXME-PLACEHOLDER", d):
    hits.append("stray-token PLACEHOLDER/FIXME-PLACEHOLDER")
if b"\xef\xbf\xbd" in d:
    hits.append("mojibake U+FFFD replacement run")
if b"\xc3\x83\xc2" in d or b"\xc3\xa2\xe2\x82\xac" in d or b"\xc3\x82" in d:
    hits.append("latin1-desync pattern (Ã©-class / â€-class double-encoding)")
sys.exit(3 if hits else 0)' "$FP" 2>&1)"
TOKRC=$?
if [ "$TOKRC" -eq 3 ]; then
  FINDINGS="$FINDINGS
STRAY_TOKENS: $TOK in $FP"
elif [ "$TOKRC" -ne 0 ]; then
  FINDINGS="$FINDINGS
STRAY_TOKEN_SCAN_ERROR: $TOK"
fi
if [ -n "$FINDINGS" ]; then
  echo "[write-integrity] SUSPICIOUS — re-verify with: LC_ALL=C grep -n $'\xef\xbf\xbd\|PLACEHOLDER' '$FP' && python3 -c \"import sys; d=open(sys.argv[1],'rb').read(); print(sum(b<9 or (13<b<32) or b==127 for b in d))\" '$FP'"
  printf '%s\n' "$FINDINGS" 1>&2
  exit 2
fi
exit 0
