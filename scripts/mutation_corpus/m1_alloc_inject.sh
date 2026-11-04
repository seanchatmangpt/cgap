#!/usr/bin/env bash
# m1_alloc_inject.sh — steady-state 1-byte allocation injected into the
# alloc-fenced path (cgap_blake3::hash), court = crates/cgap-blake3/tests/
# zero_alloc.rs (counting GlobalAlloc wrapper).
#
# PLANE DISCLOSURE: cgap-local. The arcus ZP-19 court does not exist in
# /Users/sac/arcus (no ZP-19 referent found); the cgap-local alloc fence
# court is the sandbox-runnable equivalent, run against a git-archive
# sandbox of cgap HEAD.
set -euo pipefail
. "$(dirname "$0")/lib.sh"
mc_self_check
mc_stage_cargo

subject="$MC_CARGO_SANDBOX/crates/cgap-blake3/src/lib.rs"

before="cargo test -q --manifest-path $MC_CARGO_SANDBOX/Cargo.toml -p cgap-blake3 --test zero_alloc"
after="$before"

apply_mutation() {
    python3 - "$1" <<'PYEOF'
import sys
p = sys.argv[1]
t = open(p).read()
old = "pub fn hash(data: &[u8]) -> [u8; 32] {\n    let len = data.len();"
new = ("pub fn hash(data: &[u8]) -> [u8; 32] {\n"
       "    // SHIM-COURT: steady-state 1-byte heap allocation injected into\n"
       "    // the alloc-fenced path.\n"
       "    {\n"
       "        extern crate alloc;\n"
       "        let _steady = alloc::vec::Vec::<u8>::with_capacity(1);\n"
       "    }\n"
       "    let len = data.len();")
assert old in t, "hash() anchor not found"
open(p, "w").write(t.replace(old, new, 1))
PYEOF
}

mc_with_mutation m1_alloc_inject "cgap-blake3/tests/zero_alloc.rs (alloc fence)" \
    "$subject" "$before" "$after"
