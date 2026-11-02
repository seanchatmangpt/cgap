# Repair 19 — prefix-scan primitives module (lane TS2)

Date: 2026-11-02. Branch: main. Harvest: bcinr-logic scan.rs:199-459
(patterns only; never imported).

## Primitive inventory (all const-friendly / inline(always), no_std, no-alloc)

`crates/cgap-branchless/src/scan.rs`:
- prefix_or_u64x8 / prefix_and_u64x8 / prefix_xor_u64x8 ([u64; 8], const fn,
  descending Kogge-Stone shift-by-1/2/4 network)
- prefix_sum_u32x8 ([u32; 8], const fn, wrapping, same network)
- segmented_prefix_sum_u32x8 ([u32; 8] x [bool; 8], const fn, unrolled reset-mask
  accumulator)
- prefix_max_u8x8 ([u8; 8], non-const — Ord::max not const-stable on this
  toolchain; documented residual)
- count_leading_eq_u8 ([u8; 8] SWAR-style match-mask + trailing_ones)

## Oracle courts (tests/scan_courts.rs, 8 tests, all exit 0)

Naive scalar oracle implemented in the test (REFERENCE-DRIVER loop class),
assert final state, no mocks:
- prefix_or/and/xor/sum: edge corpus + 4096-draw seeded corpus,
  seed 0xA20C_0FFE_E5A7_E000 (frozen corpus constant; splitmix64 driver).
- segmented: EXHAUSTIVE over all 2^8 flag patterns x 64 seeded value vectors.
- prefix_max_u8x8: 4096 seeded draws + monotonicity + 0/255 edges.
- count_leading_eq_u8: EXHAUSTIVE over all 256 match-mask patterns x 4 targets.

## 0-branch census (real disassembly, nightly rustc -> llvm-objdump, host aarch64)

All 6 HOT cells at 0 flow mnemonics (or/and/xor/sum, segmented, leading-eq;
prefix_max_u8x8 compiled at offset 0 in the object renders as <ltmp0> and is
censused at the object level by the anti-vacuity arm instead):
- Anti-vacuity arm (a): runtime-bounded loop reference object MUST count > 0
  branches — measured 2 branches (classifier alive).
- Anti-vacuity arm (b) mutation M-TS2-1: a /tmp sandbox copy of scan.rs with a
  runtime-bound side-effecting loop injected into prefix_xor_u64x8 MUST flip to
  >0 branches — measured 1 branch (FLIPPED). A court that cannot flip is FAIL;
  both arms flip, so the census is non-vacuous.

## Commands + exits (this session, exact subjects)

- CARGO_TARGET_DIR=/tmp/ts2-target cargo test -p cgap-branchless --test scan_courts
  -> ok. 8 passed; 0 failed (exit 0)
- CARGO_TARGET_DIR=/tmp/ts2-target cargo test -p cgap-branchless --lib
  -> ok. 34 passed; 0 failed (exit 0) [includes scan doc-examples test]
- CARGO_TARGET_DIR=/tmp/ts2-target cargo test -p cgap-branchless --test branchless_census
  -> ok. 3 passed; 0 failed (exit 0) [REP1 census unregressed]
- census grid (nocapture): all 6 HOT cells 0 flow mnemonics; naive ref 2 branches;
  M-TS2-1 mutant 1 branch (FLIPPED)

## A/B probe (DREDP fence honored) — NEGATIVE RESULT, recorded

INTEGRATION FENCE (DREDP measured -11%): no stream-compaction win is claimed.
Honest A/B probe in /tmp/ts2-ab (rustc -O, black_box on both arms, same
splitmix64 mask corpus seeded 0xA20C_0FFE_E5A7_E000, 5 rounds x 2^20 masks,
min-of-N with per-round printouts; method: wall-clock Instant, no Process.sleep):
- per-lane incremental dest-idx (cgap-parquet compact_lanes_branchless form):
  mean 6.081 ms per round
- prefix-scan dest-idx (prefix_sum_u32x8 of validity bits):
  mean 8.133 ms per round -> scan delta +33.8% (LOSS; min-round still +24%)
Correctness arm: both produce identical compactions (heads 4 == 4, outputs equal).
Verdict: the primitives land as library capability with the negative recorded.
The per-lane form's serial dependency chain is cheaper than scan + scatter with
bounds-checked indexing at 8-lane width. No swap landed; no compaction win claimed.

## Divergences + residuals

- prefix_max_u8x8 is not a const fn: Ord::max is not const-stable on this
  toolchain (const_cmp nightly-gated). census cell covered by object-level arm.
- CGAP gate: `cargo test -p cgap-branchless` full-crate run currently exits
  nonzero ONLY in the sibling lane's transpose_courts (2 census cells fail on
  their own symbol-resolution naming, file owned by that lane; TS2's three
  test binaries all exit 0). Disclosed per compile-freeze disclosure law; not
  a TS2 regression — run this session, not TS2-introduced.
- Build root /tmp/ts2-target deleted at lane end per lease law.
