#!/usr/bin/env bash
# m2_digest_bitflip.sh — 1-bit digest corruption into the BLAKE3 pin courts.
#
# One mutation (uniform 1-bit corruption of cgap_blake3::hash output,
# single-chunk path), two legs:
#
#   leg A  crates/cgap-blake3/tests/chain_mutation_court.rs — WITNESSED
#          VACUITY (court FAILS by governance): the chain court re-derives
#          every digest with the SAME hash function it is pinning, so a
#          uniform output corruption cancels on both sides of the
#          `hash(&r.frame) != r.digest` assertion and the court stays
#          green. Real defect found by this corpus; repair owned by the
#          court's owner lane (cross-substrate pin needed — e.g. compose
#          official-vector pins into the chain court).
#
#   leg B  crates/cgap-blake3/tests/official_vectors.rs — the official
#          BLAKE3 vector pins; the same mutation MUST flip it (the pins
#          are substrate-independent constants), proving the mutation
#          itself is live.
#
# mtime-bump restore law (a02-finish-receipt): every restore is a fresh
# cp (new mtime), so cargo never replays a mutated binary.
set -euo pipefail
. "$(dirname "$0")/lib.sh"
mc_self_check
mc_stage_cargo

subject="$MC_CARGO_SANDBOX/crates/cgap-blake3/src/lib.rs"

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

courtA="cgap-blake3/tests/chain_mutation_court.rs (BLAKE3 chain pin)"
cmdA="cargo test -q --manifest-path $MC_CARGO_SANDBOX/Cargo.toml -p cgap-blake3 --test chain_mutation_court"
courtB="cgap-blake3/tests/official_vectors.rs (BLAKE3 official-vector pin)"
cmdB="cargo test -q --manifest-path $MC_CARGO_SANDBOX/Cargo.toml -p cgap-blake3 --test official_vectors"

# ---- leg A: chain pin court (vacuity expected) --------------------------------
MC_NAME="m2_chain_digest_bitflip"
bash -uc "$cmdA" || mc_fail BEFORE "chain court not green on pristine sandbox — corpus leg broken"
baseline="$(mc_sha "$subject")"
backup="$(mktemp "${TMPDIR:-/tmp}/mc.m2.orig.XXXXXX")"
cp "$subject" "$backup"
apply_mutation "$subject"
set +e; bash -uc "$cmdA"; rcA=$?; set -e
cp "$backup" "$subject"; rm -f "$backup"
restore_sha="$(mc_sha "$subject")"
[ "$restore_sha" = "$baseline" ] || mc_fail RESTORE "leg-A restore sha mismatch"
if [ "$rcA" -eq 0 ]; then
    printf '[VACUOUS] chain_mutation_court stayed green under 1-bit digest corruption — court FAILS; vacuity receipt written\n'
    mc_write_receipt m2_chain_digest_bitflip "$courtA" "$subject" false "$baseline" "$restore_sha" "$rcA"
else
    mc_log AFTER "leg A: chain court DID flip under mutation (exit $rcA) — vacuity hypothesis refuted; writing flip receipt"
    mc_write_receipt m2_chain_digest_bitflip "$courtA" "$subject" true "$baseline" "$restore_sha" "$rcA"
fi

# ---- leg B: official-vector pin court (must flip) -----------------------------
cp "$CGAP_ROOT/crates/cgap-blake3/src/lib.rs" "$subject"
cp "$CGAP_ROOT/crates/cgap-blake3/tests/official_vectors.rs" \
   "$MC_CARGO_SANDBOX/crates/cgap-blake3/tests/"
mc_with_mutation m2_official_vectors_bitflip "$courtB" "$subject" "$cmdB" "$cmdB"
