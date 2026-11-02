#!/bin/bash
# mtime-restore-guard.sh (PostToolUse, Bash matcher) — stale-binary hazard.
# After a command that looks like a sha256-exact restore, remind that mtimes
# must be bumped or the target cleaned, else cargo may reuse stale artifacts
# (witnessed stale-binary hazard).
#   exit 0 = ok (reminder on firing path), exit 2 = error (fail-closed),
#   exit 3 = loud skip (unparseable).
set -u
CMD=""
INPUT="$(cat 2>/dev/null)" || INPUT=""
if command -v jq >/dev/null 2>&1; then
  CMD="$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)" || CMD=""
else
  CMD="$(printf '%s' "$INPUT" | grep -o '"command"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*:[[:space:]]*"//;s/"$//')"
fi
[ -n "$CMD" ] || exit 3
case "$CMD" in
  *sha256*|*shasum*) ;;
  *) exit 0 ;;
esac
case "$CMD" in
  *restore*|*RESTORE*) ;;
  *) exit 0 ;;
esac
echo "mtime-restore-guard REMINDER: sha256-exact restore detected — byte content \
matches but mtimes do NOT. Bump mtimes (touch) on restored files or clean the \
target dir, or cargo/make may skip rebuild against stale artifacts." >&2
exit 0
