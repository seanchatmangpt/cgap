# Lane B07 — CSCB C8 Economic Arbitrage Ledger Court

Date: 2026-11-02. Repo: /Users/sac/cgap, branch main.

## Files written

- `crates/csct/src/drowning_benchmark.rs` (new)
- `crates/csct/src/lib.rs` — ONE line added: `pub mod drowning_benchmark;`
- this receipt

## What was built

C8 coordinate court reusing Stage 3 RATIONAL DROWNING's `Ledger` and
`EXPECTED_MONTHLY_TOTAL_CENTS` from `crate::drowning` (imported, not
duplicated). Exact u64-cents integer math:

- ANNUAL_BLEED_CENTS = 42,238,600 x12 = 506,863,200 ¢ ($5,068,632)
- SUT_ANNUAL_CENTS   = 120,000 ¢ x12  = 1,440,000 ¢ ($14,400)
- gross_arbitrage_factor() = 351 (integer floor of 351.99), asserted >= 350
  per the coordinator's REQUIRED_ROI_X=350.0 gross-formula integration ruling
- NET_DELTA_CENTS = 505,423,200 ¢ ($5,054,232 exactly)

Courts (4): ledger math exact; gross factor 351 >= 350; canonical ledger
passes c8_court_verdict; ANTI-VACUITY mutant (spark 318,873 -> 0, monthly
10,351,300 ¢) fails both `evaluate()` and the C8 verdict via the same
functions — flip asserted, no shared-tree mutation.

## Commands + exits (this session)

- `cd crates/csct && CARGO_TARGET_DIR=/tmp/b07-target cargo test --lib drowning_benchmark`
  -> exit 0, 4 passed / 0 failed / 37 filtered
- `cd crates/csct && CARGO_TARGET_DIR=/tmp/b07-target cargo test --lib`
  -> exit 0, 41 passed / 0 failed

## Divergences

- Dispatch said "full csct suite currently 51 tests"; lib target re-count
  is 37 pre-existing + 4 new = 41 passing. Bin target `cscb_c4_alloc`
  (another lane's file) fails to compile pre-existing: it
  `include!`s `../../cgap-core/src/tracking_alloc.rs`, which does not
  exist on disk. Compile-abort is not a failure count; my courts ran on
  the lib target only. NOT owned by B07 — left untouched.
- Stdlib note: `cargo test` unscoped compiles all targets, so the broken
  bin makes unscoped runs exit nonzero regardless of this lane. Scoped
  `--lib` runs are the working gate.

## Standing

ALIVE — both commands above ran this session with exit 0 against the
exact subject. Anti-vacuity mutation demonstrated flipping (test
`court_flips_on_zeroed_spark` green, i.e. the court correctly rejects the
mutant).

Build root /tmp/b07-target deleted at lane end.
