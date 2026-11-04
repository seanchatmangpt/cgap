#!/usr/bin/env bash
# m4_census_debug_profile.sh — debug-profile substitution into the census
# gate. Subject: crates/cgap-blake3/tests/percall_latency.rs (a census-
# SCANNED, fenced TIMING file). The mutation substitutes BOTH guard markers
# out of the file — `profile_guard::assert_release()` is replaced with a
# debug-permissive call form and every `profile_guard::fence` with
# `std::hint::black_box` — so the file reads as unguarded TIMING and the
# census MUST refuse (exit 1).
#
# REGISTERED FINDING (witnessed by the first draft of this leg): the census
# never scans `src/main.rs` bin targets — crates/bench/src/main.rs composes
# Instant::now + assert_release entirely outside the gate's subject surface
# (tests/benches/src-bin only). Disclosed in the lane receipt; census scope
# law (tests/benches/src-bin) is the owner lane's to widen.
set -euo pipefail
. "$(dirname "$0")/lib.sh"
mc_self_check
mc_stage_full
# Baseline disclosure: HEAD left the census standing RED (committed
# aoq_flip_ladder_court.rs had no allowlist ticket). Lane-2 repaired the
# allowlist in the working tree (ffi-ZP18-D-5, bounded-poll deadline class,
# matching siblings D-1..4). Overlay the repaired allowlist into the sandbox
# so the BEFORE leg witnesses the post-repair law.
cp "$CGAP_ROOT/scripts/profile_guard_allowlist.toml" \
   "$MC_SANDBOX/scripts/profile_guard_allowlist.toml"
mc_log OVERLAY "repaired allowlist (ffi-ZP18-D-5 entry) overlaid into sandbox"

subject="$MC_SANDBOX/crates/cgap-blake3/tests/percall_latency.rs"

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
t = t.replace(old, new, 1)
n = t.count("cgap_aoq::profile_guard::fence")
assert n >= 1, "no fence marker found"
t = t.replace("cgap_aoq::profile_guard::fence", "core::hint::black_box")
open(p, "w").write(t)
PYEOF
}

mc_with_mutation m4_census_debug_profile "scripts/profile_guard_census.py (census gate)" \
    "$subject" "$before" "$after"
