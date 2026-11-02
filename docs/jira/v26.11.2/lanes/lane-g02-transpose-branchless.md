---
lane: G02
crate: cgap-branchless
standing: ALIVE
---

# Lane G02 — Transpose Branchless

Subject: `/Users/sac/cgap/crates/cgap-branchless/src/transpose.rs`
(64x64 bit-matrix transpose, riscv32imc/aarch64 0-branch).

## Charter

Dispatch: eliminate the `bne` the AE4 disassembly-invariants court
(`scripts/verify_disassembly_invariants.py`) flags in
`transpose_64x64`; keep `transpose_courts` 7/7 green under
`--release`; keep the census hot-cell courts flipping.

## Root cause

The pre-edit `transpose_64x64` drove six quadrant passes through a
runtime `while` nest (h=32..1, groups, inner i). At `-C opt-level=3`
LLVM partially unrolls but leaves residual loop-guard branches; the
AE4 arm-A census attributed `bne x1` to the
`...branchless9transpose15transpose_64x64B3_` symbol and exited 1.

## Fix

Full source-level unroll: 192 literal-index `swap_pair!` macro
invocations (32 pairs/pass x 6 passes at h = 32/16/8/4/2/1), masks
still single-sourced through the unchanged `quad_mask(h)` const fn
(const-folded at each call site). `transpose_8x8`, `dswap`,
`quad_mask` bodies unchanged; no loops remain in the IR, so the
bodies lower to pure xor/and/sll/srl. No heap, no allocation,
`const` preserved, `#![no_std]` preserved. Generated via python
assembly per the long-write corruption law; 192 invocation count
verified on disk (`grep -c`).

## Commands + exits (this session)

- BEFORE (pre-edit tree): `python3 scripts/verify_disassembly_invariants.py`
  -> EXIT=1; arm A rows included
  `[FORBIDDEN] transpose_64x64: [('...15transpose_64x64B3_', 'bne')]`
  (full log: /tmp persist, tool-results bkrdllt4j.txt).
- `cargo test -p cgap-branchless --release` (CARGO_TARGET_DIR=/tmp/g02-target,
  cwd crates/cgap-branchless) -> EXIT=0; transpose_courts 7/7 ok
  (court_8x8_differential_oracle, court_8x8_involution,
  court_64x64_differential_oracle,
  court_64x64_involution_and_stage_selfinverse, census_host_hot_cells,
  census_riscv32imc_hot_cells,
  anti_vacuity_mutant_flips_differential_court); scan_courts 8/8 ok.
- AFTER: `python3 scripts/verify_disassembly_invariants.py` -> EXIT=1;
  arm A row now `[PASS] transpose_64x64: 0 forbidden branches` and
  `[PASS] transpose_8x8: 0 forbidden branches`; anti-vacuity M-AE4-1
  and branchy self-check still PASS (court still flips).

## Disasm witness (riscv32imc, nightly rustc, opt-level=3,
inline-threshold=0)

BEFORE (loop-form reconstruction compiled standalone in
/tmp/g02-sandbox/before): 5x `bne` back-edge guards in the emitted
asm (e.g. `bne\ta2, a3, .LBB1_1` at before.s:69). The AE4 crate
census attributed 1x bne to the transpose_64x64 symbol.

AFTER (shared-tree transpose.rs, same wrapper
/tmp/g02-sandbox/after): `census_t64` region = 3261 instructions,
0 branch ops (bne/beq/blt/bge/bltu/bgeu/beqz/bnez/j/jal/jalr all
zero) -- xor/and/sll/srl/sw/lui/addi only.

## Anti-vacuity (courts still flip)

- `anti_vacuity_mutant_flips_differential_court`: ok (pristine twin
  exit 0, M-TS4-1 mask bit-flip twin exit nonzero) -- the mutation
  anchor `0x00AA_00AA_00AA_00AA` is untouched by this diff.
- AE4 mutant court M-AE4-1 (if-injection into cmov_u64): PASS both
  runs.
- AE4 branchy self-check (naive_if_cmov): flagged BRANCHY both runs.
- census_host_hot_cells / census_riscv32imc_hot_cells: ok. The
  unrolled form measures 0 branches at census_t64, under the
  `T64_LOOP_RESIDUAL = 2` ceiling asserted by the courts (tests not
  edited -- the court budget is a ceiling, satisfied at 0).

## Fences (must-not-regress surfaces crossed)

- transpose_courts 7/7 under --release: re-run post-edit, green.
- branchless_census + scan_courts binaries in the same crate: green
  (same `cargo test -p cgap-branchless --release` run, exit 0).
- AE4 court: re-run post-edit (exit 1 remains, but no longer from
  transpose; see honest residual below).

## Honest residual (out of lane)

AE4 still exits 1 AFTER this lane: `[FORBIDDEN] count_leading_eq_u8
(beqz)` and `[FORBIDDEN] prefix_max_u8x8 (bgeu/bltu)` -- both in
`crates/cgap-branchless/src/scan.rs`, NOT in this lane's file list
(dispatch scoped G02 to transpose.rs only). These are pre-existing
rows, untouched by this diff; the owning lane must close them for
AE4 to reach exit 0. AE4 on-disk also gained an "arm E" mid-session
(634 lines at after-run vs 445 at before-run read): a concurrent
lane edited scripts/verify_disassembly_invariants.py during G02;
arm E emitted `[SKIP] arm E binary-mutant court did not run (loud)`.

## Divergences

- Dispatch said "before (exit 1, bne hit)" for AE4: confirmed, but
  the before exit-1 also carried the two scan.rs FORBIDDEN rows.
- transpose.rs is not in HEAD (uncommitted wave state), so the
  before-disasm witness is a reconstruction of the pre-edit loop
  form compiled in /tmp/g02-sandbox/before, not a `git show HEAD`
  rebuild.
- tests/transpose_courts.rs header comment still describes the
  6-loop-back-edge residual and T64_LOOP_RESIDUAL=2; tests are
  outside this lane's file ownership, so the comment is stale but
  the assertion (<= 2) holds at the measured 0.

## Standing

ALIVE for the lane claim: transpose_64x64 branchless form landed,
courts green, AE4 transpose row flipped FORBIDDEN -> PASS, all
commands run this session against the exact subject with exits
recorded above. AE4 overall exit 0 is BLOCKED out-of-lane on scan.rs.
