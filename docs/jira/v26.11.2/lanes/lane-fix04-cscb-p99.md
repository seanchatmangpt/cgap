# lane-fix04-cscb-p99 — Receipt (2026-11-02)

Finding (EXP-04, audit/dod-cgap-blake3.md weak point 1): the "p99" over
N=7 run-level rate samples computed `ceil(0.99*7)=7 -> sorted[6]` = the
MAX, mislabeled as a tail quantile.

## Fix (owned file only)

`crates/csct/src/bin/cscb_c7_receipt_sealing.rs`

- Removed `percentile()`; added `median()` (sorted[N/2], identical value
  to the old `percentile(.., 50.0)` for odd N=7 — gate semantics
  unchanged, gate still on the median vs RATE_FLOOR).
- Reporting now honest: `[C7-RATE] min/median/max runs=7` plus an
  explicit note that no p99 is computed (run-level sample population of
  7 is too small for a tail quantile). Per-frame samples are not
  collected, so the "true p99" branch was not applicable.
- Doc comment updated to match.

## Commands + exits (this session, exact subject)

- `cd crates/csct && cargo build --release` → exit 0 (pre-existing
  cgap-core dep warning in manifest, not lane-introduced)
- `cargo run --release --bin cscb_c7_receipt_sealing` → exit 0;
  output: `[C7-RATE] min=4223111/s median=4303094/s max=4367782/s runs=7 (note: ... no p99 computed ...)` + `[C7-PASS]` rate gate + chain validity (1,000,000 receipts)
- `CSCB_MUTATE=1 cargo run -q --release --bin cscb_c7_receipt_sealing`
  → exit 0; `[C7-PASS] anti-vacuity: mutated certificate rejected`
- `cargo test` (crates/csct, full suite) → exit 0; totals: 41+1+7+9+7
  passed, 0 failed across all test binaries.

## Sibling audit (report-only, not owned)

- `cscb_c1_c2_ingress.rs:190-194` — HONEST: p99 over
  `n_batches = FRAMES/64` (thousands of batch-amortized per-frame
  samples); genuine tail quantile. No fix needed.
- `cscb_c6_retraction.rs:87-100` — SAME DEFECT: `secs[6]` over 7 runs
  labeled `p99` in both timing and throughput lines = the max, not a
  quantile. Owner should apply the same min/median/max/N rename.

## Fences

Could not regress: C7's rate gate (median-based, unchanged), chain
validity court, anti-vacuity CSCB_MUTATE court — all re-run, all pass.
csct full test suite green (exit 0).

Standing: ALIVE (all commands + exits run this session).
