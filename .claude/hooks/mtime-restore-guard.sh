#!/bin/bash
# mtime-restore-guard.sh (PostToolUse, Bash matcher) — stale-binary hazard.
# Law (CLAUDE.md mtime-bump restore law): a sha256-exact restore does NOT
# invalidate cargo's fingerprint — cargo compares mtimes. A restore that
# PRESERVES old mtimes (cp -p/-a, ditto, rsync -a, touch -r, git checkout/restore
# of paths) leaves a stale fingerprint: the mutated binary survives and the
# green run is VACUOUS. Such restores FAIL CLOSED (exit 2). Restores that bump
# mtimes (plain cp/mv, touch) are allowed with a reminder (exit 0).
#   exit 0 = allowed (reminder on firing path), exit 2 = fail-closed,
#   exit 3 = loud skip (unparseable input).
set -u
INPUT="$(cat 2>/dev/null)" || INPUT=""
CMD=""
if command -v jq >/dev/null 2>&1; then
  CMD="$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)" || CMD=""
fi
if [ -z "$CMD" ]; then
  CMD="$(printf '%s' "$INPUT" | grep -o '"command"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*:[[:space:]]*"//;s/"$//')"
fi
[ -n "$CMD" ] || exit 3
# Only restore-flavored sha256 commands are in scope.
case "$CMD" in
  *sha256*|*shasum*) ;;
  *) exit 0 ;;
esac
case "$CMD" in
  *restore*|*RESTORE*|*rsync*|*cp\ *|*mv\ *|*ditto*|*checkout*|*git\ show*) ;;
  *) exit 0 ;;
esac
# Fail-closed: mtime-PRESERVING restore verbs defeat cargo's fingerprint.
case "$CMD" in
  *cp\ -p*|*cp\ -a*|*"cp -Rp"*|*ditto*|*rsync\ -a*|*touch\ -r*|*checkout\ --*|*"git restore"*)
    echo "mtime-restore-guard FAIL-CLOSED: sha256-exact restore uses an \
mtime-preserving verb (cp -p/-a, ditto, rsync -a, touch -r, git checkout --). \
Byte content matches but mtimes do NOT — cargo's fingerprint stays valid and a \
stale (mutated) binary survives. Restore with plain cp/mv, then touch the \
restored files, or cargo clean -p <crate>." >&2
    exit 2 ;;
esac
# mtime-bumping (or fingerprint-cleaning) restores are lawful: reminder only.
echo "mtime-restore-guard REMINDER: sha256-exact restore detected — verify \
mtimes were bumped (touch) or the target cleaned, else cargo may reuse stale \
artifacts." >&2
exit 0
