#!/usr/bin/env bash
# m4_census_debug_profile.sh — debug-profile substitution into the census
# gate. Subject: crates/bench/src/main.rs release guard — the
# `cgap_aoq::profile_guard::assert_release()` composition is substituted
# with a debug-permissive call form (removing the exact guard marker the
# census classifies as "fenced TIMING"). Gate =
# scripts/profile_guard_census.py run over a full git-archive sandbox.
set -euo pipefail
. "$(dirname "$0")/lib.sh"
mc_self_check
mc_stage_full

subject="$MC_SANDBOX/crates/bench/src/main.rs"

before="python3 $MC_SANDBOX/scripts/profile_guard_census.py"
after="$before"

apply_mutation() {
    python3 - "$1" <<'PYEOF'
import sys
p = sys.argv[1]
t = open(p).read()
old = "if let Err(v) = cgap_aoq::profile_guard::assert_release() {"
new = ("if let Err(v) = assert_release_debug_permissive() "
       "{ /* SHIM-COURT: debug-profile substitution, guard marker removed */")
assert old in t, "assert_release anchor not found"
open(p, "w").write(t.replace(old, new, 1))
PYEOF
}

mc_with_mutation m4_census_debug_profile "scripts/profile_guard_census.py (census gate)" \
    "$subject" "$before" "$after"
