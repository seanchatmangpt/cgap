#!/usr/bin/env bash
# m3_version_skew.sh — Aoq1Version 3→2 version skew into the codec gate.
# Subject: the GENERATED layout projection crates/cgap-aoq/src/
# generated_aoq_header_layout.rs — mutated ONLY in the git-archive sandbox,
# never in place (frozen-file law). Court = aoq1_layout_drift_court.rs
# magic_and_version_pins (asserts gen::AOQ1_VERSION == 3).
set -euo pipefail
. "$(dirname "$0")/lib.sh"
mc_self_check
mc_stage_cargo

subject="$MC_CARGO_SANDBOX/crates/cgap-aoq/src/generated_aoq_header_layout.rs"

before="cargo test -q --manifest-path $MC_CARGO_SANDBOX/Cargo.toml -p cgap-aoq --test aoq1_layout_drift_court"
after="$before"

apply_mutation() {
    python3 - "$1" <<'PYEOF'
import sys
p = sys.argv[1]
t = open(p).read()
old = "pub const AOQ1_VERSION: u16 = 3;"
new = "pub const AOQ1_VERSION: u16 = 2; // SHIM-COURT: Aoq1Version 3->2 skew"
assert old in t, "AOQ1_VERSION pin not found"
open(p, "w").write(t.replace(old, new, 1))
PYEOF
}

mc_with_mutation m3_version_skew "cgap-aoq/tests/aoq1_layout_drift_court.rs (codec version gate)" \
    "$subject" "$before" "$after"
