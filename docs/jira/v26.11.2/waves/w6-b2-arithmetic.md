---
title: "w6-b2-arithmetic"
lane: B2
status: ALIVE
date: 2026-11-02
---

# Lane B2 — branchless arithmetic (saturated_subs_p28, poison_mask)

## Files written

- `crates/cgap-branchless/src/arithmetic.rs` (sole owned file)
- this receipt

## Final implementation

- `saturated_subs_p28(counter, delta) -> (u64, u64)`: wrapping diff,
  borrow bit = exact 128-bit borrow-out `((full >> 64) & 1)` of
  `(counter as u128) - (delta as u128)`; tripwire =
  `0u64.wrapping_sub(borrow_bit)`; `new = diff & !tripwire` (saturate).
- `poison_mask(quota)`: `nz = ((quota | quota.wrapping_neg()) >> 63) & 1`;
  `0u64.wrapping_sub(nz ^ 1)` — all-ones iff quota == 0.

## Divergences recorded

1. Dispatch poison formula
   `((quota.wrapping_sub(1)) as i64 >> 63) as u64 ^ u64::MAX` is wrong
   twice over: (a) it is all-ones iff quota > 2^63 or quota == 0 (a
   sign-bit test of quota-1, not a zero test; CORPUS_SEED 0xA20C... has
   bit 63 set and triggered it); (b) its XOR is the complement of the
   B1 oracle. Implemented the exact branchless zero test instead; doc
   and code agree in-file.
2. An interim u128 unblock fix by another lane extracted the borrow as
   `(full >> 64) as u64` (missing `& 1`) — the differential court
   caught it (0-1 returned tripwire=1, new=MAX-1). Fixed to
   `& 1`. Disclose-in-both: this receipt discloses; the fixing lane
   should mirror.
3. A later unblock wrote `0u64.wrapping_sub(nz)` (inverted: all-ones
   iff quota != 0); differential court caught it; fixed to `nz ^ 1`.

## Courts run (anti-vacuity: branchless == naive on every input)

Gate: `CARGO_TARGET_DIR=/tmp/b2-target cargo test -p cgap-branchless
--lib arithmetic` → `test result: ok. 4 passed; 0 failed; 22 filtered
out` — exit 0 (witnessed: `EXIT=0`).

- differential_fuzz_branchless_equals_naive: 4096 SplitMix64 pairs,
  corpus seed 0xA20C_0FFE_E5A7_E000 (multiplier 0xBF58_476D_1CE4_E5B9
  stays the DERIVE constant inside the finalizer).
- exhaustive_edge_cases_match_oracles: {0,1,CORPUS_SEED,MAX-1,MAX}²,
  plus mutation assertions (tripwire clear on 100-40; (0,MAX) on 1-2;
  poison(0)=MAX, poison(1)=0).
- saturation_and_tripwire_shape: shape invariant over 512 pairs.

## Branchlessness census (this lane's probe; full census is lane B6)

Probe: scratch caller crate at /tmp/b2-asmprobe calling both fns via
black_box, `cargo rustc --manifest-path /tmp/b2-asmprobe/Cargo.toml
--release -- --emit=asm -Ccodegen-units=1` →
/tmp/b2-target/release/build/b2-asmprobe/e6a14a27183bfd76/out/b2_asmprobe.s.
Emitted aarch64 for the inlined fns: `subs` + `ngc x9, xzr` (tripwire,
carry-derived), `bic x8, x8, x9` (saturate mask), `cmp x10, #0` +
`csetm x10, eq` (poison mask — predicated set, not a branch).
Zero conditional branch instructions in the inlined bodies. The lib
itself emits no standalone symbols for the `#[inline]` fns (no
callers), so a caller crate was the honest way to force codegen.

## Standing

ALIVE — command + exit run this session against the exact subject
(arithmetic.rs final md5s witnessed across the green run; gate exit 0).
Caveat: concurrent lanes (B1/B3/B6) rewrote arithmetic.rs repeatedly
during this lane; the final on-disk content was repaired and verified
in the same command as the green gate. Later mutation of the file by
another lane voids this receipt.

## Cleanup

Build lease /tmp/b2-target and scratch /tmp/b2-asmprobe deleted below.
