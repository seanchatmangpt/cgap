#!/usr/bin/env bash
# m5_allowlist_tamper.sh — duplicate allowlist entry injected into the
# census allowlist (profile_guard_allowlist.toml). Witnesses the census
# gate's fail-closed allowlist validation — the leg that backs every
# registered allowlist ticket (ZP-18-C/D, ffi-ZP18-D-*, ...) in the
# meta-court registry.
set -euo pipefail
. "$(dirname "$0")/lib.sh"
mc_self_check
mc_stage_full

subject="$MC_SANDBOX/scripts/profile_guard_allowlist.toml"

before="python3 $MC_SANDBOX/scripts/profile_guard_census.py"
after="$before"

apply_mutation() {
    python3 - "$1" <<'PYEOF'
import sys
p = sys.argv[1]
t = open(p).read()
first = t.find("[[allowlist]]")
second = t.find("[[allowlist]]", first + 1)
assert first != -1 and second != -1, "allowlist entries not found"
block = t[first:second]
open(p, "w").write(t + "\n# SHIM-COURT: duplicate allowlist entry injected\n" + block)
PYEOF
}

mc_with_mutation m5_allowlist_tamper "scripts/profile_guard_census.py (allowlist validation leg)" \
    "$subject" "$before" "$after"
