# Lane C04 — branchless boundary coverage courts

Wave: v26.11.2 starDF. Date: 2026-11-02. Branch: main (no git writes).

## Files written (exact ownership)
- `crates/cgap-branchless/tests/boundary_sweep_courts.rs` (NEW, 409 lines)
- `docs/jira/v26.11.2/lanes/lane-c04-branchless-boundary-cov.md` (this receipt)

## Public APIs covered (read this session, cite-don't-redefine)
- `scan.rs`: `prefix_or_u64x8([u64;8]) -> [u64;8]`, `prefix_and_u64x8`,
  `prefix_xor_u64x8` (same shape), `prefix_sum_u32x8([u32;8]) -> [u32;8]`,
  `segmented_prefix_sum_u32x8([u32;8],[bool;8]) -> [u32;8]`,
  `prefix_max_u8x8([u8;8]) -> [u8;8]`, `count_leading_eq_u8([u8;8],u8) -> usize`
- `transpose.rs`: `transpose_8x8(u64) -> u64`,
  `transpose_64x64([u64;64]) -> [u64;64]`
- `swar.rs`: `swar_lt`, `swar_eq_u64`, `swar_has_zero_byte`,
  `swar_range_mask(u64,u8,u8)`, `swar_splat(u8)`

## Divergence from dispatch
Dispatch names `clamp.rs`; no such file exists in
`crates/cgap-branchless/src/` (saturated clamp lives in `arithmetic.rs`,
outside G01/G02's re-lowering surface). Covered the on-disk surface:
scan + swar + transpose. Recorded here, not silently.

## Courts landed (tests/boundary_sweep_courts.rs)
- `scan_boundary_sweep_court` — all-ones/all-zeros/0x5555../0xAAAA.. over
  every scan primitive vs scalar reference models (real loops, not
  restated implementation); segmented sums over 6 boundary flag shapes.
- `scan_high_entropy_differential_court` — 4,096 SplitMix64 rounds
  (test-space seed 0xC04B_0747_1DEA_0001; NOT the frozen corpus seed),
  every scan primitive vs reference.
- `swar_boundary_sweep_court` — corner words x corner splats (0x00/01/7F/
  80/FE/FF), full edge-pair range sweep, empty-range lane zeros,
  20,000-round differential.
- `transpose_boundary_sweep_court` — 8x8 corner words + involutions;
  64x64 identity, 49 single-bit matrices (r step 9 x c step 7), 256
  high-entropy rounds + involutions, corner fills, vs bit-level
  reference.
- ANTI-VACUITY twins (must flip, proven via #[should_panic]):
  `scan_reference_model_has_teeth`, `transpose_reference_model_has_teeth`,
  `swar_reference_model_has_teeth` — each injects a deliberately wrong
  expected value and MUST fail; all three flipped (should_panic ok).

## Commands + exits (this session, exact subject)
- `CARGO_TARGET_DIR=/tmp/c04-target cargo test -p cgap-branchless` — exit 0.
  Suites: 34+7+3+8+7 = 59 passed, 0 failed (incl. 7 new C04 courts,
  3 should_panic teeth). Existing suites stayed green.
- `python3 scripts/verify_disassembly_invariants.py` — **exit 0**, verdict
  "AE4 disassembly-invariants court: CLEAN (census + linked witness +
  mutant + self-check + binary mutant all held)". transpose_64x64/8x8
  show 0 forbidden branches on G01/G02's mid-flight state — no
  BLOCKED-on-sibling hits.

## Standing
ALIVE. Both commands ran and passed this session against the exact
on-disk subject.

## Fences (must-not-regress)
- Existing branchless suites (branchless_census, scan_courts,
  transpose_courts): re-run in the same command, green.
- verify_disassembly_invariants.py: re-run, exit 0 CLEAN. G01/G02's
  gate flip to 0 remains their receipt's claim; C04 only records the
  observed state.
- Shared tree untouched outside the two owned files; /tmp/c04-target
  deleted at lane end.

## Seeds
Test-space SplitMix64 seed 0xC04B_0747_1DEA_0001 (local PRNG, declared
test-space by dispatch). Frozen corpus seed 0xA20C_0FFE_E5A7_E000 NOT
used; SplitMix64 multiplier 0xBF58_476D_1CE4_E5B9 used as algorithm
constant only.
