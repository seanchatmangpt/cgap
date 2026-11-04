#!/usr/bin/env bash
# m2_digest_bitflip.sh — 1-bit digest corruption into the BLAKE3 pin court.
# Subject: cgap_blake3::hash output (single-chunk path); court =
# crates/cgap-blake3/tests/chain_mutation_court.rs (chained-receipt court
# re-deriving every digest with the real hash).
set -euo pipefail
. "$(dirname "$0")/lib.sh"
mc_self_check
mc_stage_cargo

subject="$MC_CARGO_SANDBOX/crates/cgap-blake3/src/lib.rs"

before="cargo test -q --manifest-path $MC_CARGO_SANDBOX/Cargo.toml -p cgap-blake3 --test chain_mutation_court"
after="$before"

apply_mutation() {
    python3 - "$1" <<'PYEOF'
import sys
p = sys.argv[1]
t = open(p).read()
old = ("        let mut res = [0u8; 32];\n"
       "        for i in 0..8 {\n"
       "            res[i * 4..(i + 1) * 4].copy_from_slice(&out[i].to_le_bytes());\n"
       "        }\n"
       "        return res;")
new = ("        let mut res = [0u8; 32];\n"
       "        for i in 0..8 {\n"
       "            res[i * 4..(i + 1) * 4].copy_from_slice(&out[i].to_le_bytes());\n"
       "        }\n"
       "        res[0] ^= 0x01; // SHIM-COURT: 1-bit digest corruption\n"
       "        return res;")
assert old in t, "hash() single-chunk return anchor not found"
open(p, "w").write(t.replace(old, new, 1))
PYEOF
}

mc_with_mutation m2_digest_bitflip "cgap-blake3/tests/chain_mutation_court.rs (BLAKE3 pin)" \
    "$subject" "$before" "$after"
