#!/bin/bash
# publish-guard.sh (PreToolUse, Bash matcher) — R2-REFUSED pattern.
# Refuses `cargo publish` unless the operator set an explicit authority marker.
# Fail-closed: any error path exits 2 (block).
#   exit 0 = allow, exit 2 = block, exit 3 = loud skip (non-Bash input shape).
set -u
AUTH_FILE="${CGAP_PUBLISH_AUTHORITY_FILE:-/Users/sac/cgap/.claude/hooks/publish-authority.token}"
CMD=""
INPUT="$(cat 2>/dev/null)" || INPUT=""
# Extract tool_input.command (jq if present, else grep fallback).
if command -v jq >/dev/null 2>&1; then
  CMD="$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)" || CMD=""
else
  CMD="$(printf '%s' "$INPUT" | grep -o '"command"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*:[[:space:]]*"//;s/"$//')"
fi
[ -n "$CMD" ] || exit 3
case "$CMD" in
  *cargo\ publish*|*cargo\ \-p*\ publish*|*"cargo +"*publish*) ;;
  *) exit 0 ;;
esac
# Authority edge: marker file must exist AND carry the operator's SHA receipt.
if [ -f "$AUTH_FILE" ] && grep -q 'OPERATOR-PUBLISH-AUTHORITY' "$AUTH_FILE" 2>/dev/null; then
  echo "publish-guard: authority marker present ($AUTH_FILE) — allowed" >&2
  exit 0
fi
REASON="publish-guard: REFUSED — cargo publish requires operator authority. \
Coordinator executes publishes. To authorize, operator writes marker: \
printf 'OPERATOR-PUBLISH-AUTHORITY <sha> <date>\n' > $AUTH_FILE"
printf '{"decision":"block","reason":"%s"}\n' "$REASON"
exit 2
