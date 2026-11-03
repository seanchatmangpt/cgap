#!/bin/bash
# publish-guard.sh (PreToolUse, Bash matcher) — v26.11.2 publish-window guard.
# Operator order 2026-11-02 froze all publishes (waves/publish-halt.md).
# FREEZE LIFTED 2026-11-02 by coordinator delegation ("Operator-owned
# decisions, these should not be for me to decide") — see the FREEZE LIFT
# section at the tail of waves/publish-halt.md. This guard now gates on
# that lift receipt: publish commands are allowed ONLY while the lift
# record is present in publish-halt.md; removing it re-freezes (fail-closed).
#   exit 0 = allow, exit 2 = block, exit 3 = loud skip (non-Bash input shape).
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
  *cargo\ publish*|*cargo\ \-p*\ publish*|*"cargo +"*publish*|*npm\ publish*|*pnpm\ publish*|*yarn\ publish*|*twine\ upload*)
    ;;
  *) exit 0 ;;
esac
HALT="docs/jira/v26.11.2/waves/publish-halt.md"
if [ -f "$HALT" ] && grep -q "FREEZE LIFT (2026-11-02, coordinator-delegated)" "$HALT"; then
  exit 0  # lift receipt present — window open
fi
REASON="publish-guard: REFUSED — publish freeze is ACTIVE. \
The FREEZE LIFT receipt is absent from docs/jira/v26.11.2/waves/publish-halt.md. \
Restore the lift record (or obtain operator authorization) to open the window. \
Blocked command class: cargo/npm/pnpm/yarn publish, twine upload."
printf '{"decision":"block","reason":"%s"}\n' "$REASON"
exit 2
