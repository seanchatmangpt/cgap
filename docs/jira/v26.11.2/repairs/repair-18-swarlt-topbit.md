# Repair 18 — swar_lt 8-bit correctness (BH2 top-bit lead)

Lane TS1, 2026-11-02. Owner files: `crates/cgap-branchless/src/swar.rs`,
`crates/cgap-branchless/tests/branchless_census.rs` (read-only), this receipt.

## Finding under test

BH2 (bcinr `reduce.rs:105-118`): top-bit-naive SWAR byte-lt forms
(`((x|HI) - (y & !HI)) & HI` without folding the genuine top bits) are wrong
for bytes >= 0x80 — adjacent-lane corruption.

## Verdict: NO CHANGE to the landed formulation

The landed `swar_lt` (`crates/cgap-branchless/src/swar.rs:33-39`) is ALREADY
the top-bit-fold form, structurally identical to bcinr's
`swar_byte_ge_mask` (harvest-cite, read-only, no import):

- `low_ge = (x | HI).wrapping_sub(y & !HI) & HI` — lane-safe borrow: minuend
  bytes lie in `[0x80, 0xFF]`, subtrahend bytes in `[0x00, 0x7F]`, so no
  cross-byte carry; bit 7 is exactly `lo7(x) >= lo7(y)`.
- `high_gt | (high_eq & low_ge)` — folds the GENUINE top bits in
  (`x7 & !y7` for >, `x7 == y7` deferring to the low-7 verdict).
- `!ge & HI` — the strict-lt mask.

Per the dispatch contract ("if current form is already exhaustive-correct,
record that and STOP"), the function body is untouched.

## Exhaustiveness court (new, the only source change)

`swar::tests::swar_lt_exhaustive_full_byte_domain` (swar.rs): all 65,536
`(a, b)` byte pairs, each splatted across all 8 lanes = 524,288 lane
instances, each asserted against the naive byte-loop oracle
`naive_lt_mask`. Full domain, not sampled.

## Anti-vacuity mutation (sandbox, /tmp python model — shared tree untouched)

Top-bit-naive mutant `((~((x|HI) - (y & !HI))) & HI`:
- wrong on 16,384 of 65,536 pairs (e.g. a=0x00, b=0x80: mutant 0x0,
  oracle 0x8080808080808080) — the court MUST flip for any such landing.
- Landed form: 0 mismatches over the full domain in the same sandbox model.

## Census

`swar_lt` is an existing HOT cell in `branchless_census.rs` GRID
(0-branch required on host-aarch64 AND riscv32imc, symbol presence
required). No census extension needed; the 0-branch property is unchanged
(body untouched).

## Receipt (commands + exits, this session)

- `CARGO_TARGET_DIR=/tmp/ts1-target cargo test -p cgap-branchless` — exit 0.
  33 unit tests passed (incl. `swar_lt_exhaustive_full_byte_domain`) +
  3 census tests passed (host-aarch64 + riscv32imc grids, M-REP1-1 and
  M-INL-1 mutation flips witnessed).
- Sandbox exhaustive model (python3, /tmp): landed 0/65,536 wrong; mutant
  16,384/65,536 wrong. Exit 0 both runs.

Standing: ALIVE for the exhaustive-correctness claim on the landed
`swar_lt`; the function body is unchanged from the pre-lane state.
