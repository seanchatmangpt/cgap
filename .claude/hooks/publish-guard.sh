#!/bin/bash
# publish-guard.sh (PreToolUse, Bash matcher) — FROZEN-PUBLISH guard.
# Operator order 2026-11-02: all publishes FROZEN ("do not publish anything yet").
# Cite: docs/jira/v26.11.2/waves/publish-halt.md (standing FROZEN-BY-OPERATOR).
# Blocks cargo publish / npm publish unconditionally — the freeze supersedes the
# former authority-marker bypass. Fail-closed: any error path exits 2 (block).
#   exit 0 = allow, exit 2 = block, exit 3 = loud skip (non-Bash input shape).
set -u
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
  *cargo\ publish*|*cargo\ \-p*\ publish*|*"cargo +"*publish*|*npm\ publish*|*pnpm\ publish*|*yarn\ publish*|*twine\ upload*)
    ;;
  *) exit 0 ;;
esac
REASON="publish-guard: REFUSED — publishes are FROZEN by operator order (2026-11-02). \
See docs/jira/v26.11.2/waves/publish-halt.md (standing FROZEN-BY-OPERATOR). \
No publish attempts are authorized until the operator lifts the freeze in this hook. \
Blocked command class: cargo/npm/pnpm/yarn publish, twine upload."
printf '{"decision":"block","reason":"%s"}\n' "$REASON"
exit 2
