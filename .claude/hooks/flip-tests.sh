#!/bin/bash
# flip-tests.sh — anti-vacuity courts for the three SM-4 hooks.
# Each hook: one fixture that FIRES (observable effect) + one that DOES NOT.
# Exit 0 = all courts pass (both directions flip); nonzero = court FAIL.
set -u
H="/Users/sac/cgap/.claude/hooks"
chmod +x "$H"/*.sh 2>/dev/null
fail=0
J='{"tool_name":"Bash","tool_input":{"command":"%s"}}'

court() { # name expected_exit expected_effect cmd
  local name="$1" exp="$2" eff="$3" cmd="$4" out rc
  out="$(printf "$J" "$cmd" | "$H/$1.sh" 2>&1)"; rc=$?
  if [ "$rc" != "$exp" ]; then echo "[FAIL] $name exit=$rc want=$exp :: $out"; fail=1; return; fi
  case "$eff" in
    fire)   [ -n "$out" ] || { echo "[FAIL] $name expected firing output, got none"; fail=1; return; } ;;
    quiet)  [ -z "$out" ] || { echo "[FAIL] $name expected silence, got: $out"; fail=1; return; } ;;
  esac
  echo "[PASS] $name exit=$rc effect=$eff"
}

# (1) publish-guard
court publish-guard 2 fire  "cargo publish -p cgap-core"
court publish-guard 0 quiet "cargo test -p cgap-core"
# mutation: WITH authority marker, refusal must flip to allow
T="$(mktemp -d)/auth.token"
printf 'OPERATOR-PUBLISH-AUTHORITY abc123 2026-11-01\n' > "$T"
export CGAP_PUBLISH_AUTHORITY_FILE="$T"
out="$(printf "$J" "cargo publish" | bash "$H/publish-guard.sh" 2>&1)"; rc=$?
unset CGAP_PUBLISH_AUTHORITY_FILE
if [ "$rc" = 0 ] && [ -n "$out" ]; then
  echo "[PASS] publish-guard marker-mutation: flips block->allow"
else
  echo "[FAIL] publish-guard marker-mutation rc=$rc out=$out"; fail=1
fi

# (2) cwd-guard
court cwd-guard 0 fire  "cargo test --manifest-path /tmp/sandbox/Cargo.toml"
court cwd-guard 0 quiet "cd /tmp/sandbox && cargo test"
court cwd-guard 0 quiet "cargo test -p cgap-core"

# (3) mtime-restore-guard
court mtime-restore-guard 0 fire  "shasum -c manifest && cp /tmp/orig.rs restore/src/lib.rs # RESTORE"
court mtime-restore-guard 0 quiet "shasum -c manifest"

exit $fail
