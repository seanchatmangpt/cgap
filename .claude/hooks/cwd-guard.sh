#!/bin/bash
# cwd-guard.sh (PreToolUse, Bash matcher) — silent-canonical-build hazard.
# Warns loudly when a cargo command references a /tmp sandbox without an
# explicit leading `cd`. Warn-only on the firing path (exit 0 + stderr),
# fail-closed on malformed input (exit 2).
#   exit 0 = allow (warn if firing), exit 2 = block (error), exit 3 = loud skip.
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
  *cargo*) ;;
  *) exit 0 ;;
esac
# References /tmp?
case "$CMD" in
  */tmp/*|*TMPDIR*) ;;
  *) exit 0 ;;
esac
# Explicit leading `cd` into the sandbox?
case "$CMD" in
  cd\ /tmp/*|*"&& cd /tmp/"*) exit 0 ;;
esac
echo "cwd-guard WARNING: cargo command references /tmp without a leading 'cd /tmp/...' \
— risk of silently building the canonical tree (witnessed: PP-1/VAC-2/P3-13). \
Prefix the command with 'cd <sandbox> && '." >&2
exit 0
