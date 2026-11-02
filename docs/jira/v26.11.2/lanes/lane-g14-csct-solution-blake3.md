# Lane G14 — CSCT solution courts (BLAKE3 + timing vacuum-fold fix)

Date: 2026-11-02. Repo: /Users/sac/cgap (branch main). Lane files:
`crates/csct/src/solution.rs` (edited on real defect), this receipt.

## Stale-premise reconciliation (confirmed)

- solution.rs compiles clean against the landed cgap_blake3 slice input —
  the E0308 at solution.rs:134-135 was an in-flight snapshot. No fix needed.
- Dead `econ_ok` binding: absent from current solution.rs (already fixed).
- Debug court accepts typed LoudSkip; release must Pass — confirmed both.

## Commands + exits (this session)

| command (cwd /Users/sac/cgap/crates/csct) | exit |
|---|---|
| `CARGO_TARGET_DIR=/tmp/g14-target cargo test solution` (debug) | 0 — 3/3 pass |
| `CARGO_TARGET_DIR=/tmp/g14-rel cargo test --release solution` | 0 — 3/3 pass |
| `/tmp/g14-target/debug/csct_runner --receipt` | 3 (typed LoudSkip, by design) |
| `/tmp/g14-rel/release/csct_runner --receipt` | 0 (Pass) |

## Defect found and fixed (owned file only)

`ingress_filter_per_frame_ns_lt_10` observed 0.00042 ns/frame in the
release receipt (~42 ns total for 100k frames — physically impossible):
release LLVM constant-folded the timing loop. Fix (minimal, in
measure_ingress): `std::hint::black_box` on the frames slice and on each
`filter_frame` result folded into the accumulator, plus black_box on the
accumulator before `start.elapsed()`.

Post-fix release receipts across runs: 3.6375, 1.03167 ns/frame — real,
non-zero, plausible, < 10. Receipt digest UNCHANGED
(927ca472f7a592f070a4a272fe3cdf4be81e2bc7293f74e970db90cbc8520a78)
before and after both edits — behavior identical.

## Debug-vs-release contract (witnessed)

- Debug runner: timing observed 76.00583 ns/frame -> FAIL -> stage exit
  LoudSkip, runner exit 3. Typed, loud, never a fake pass.
- Release runner: timing observed 1.03167 ns/frame -> PASS, stage Pass,
  runner exit 0.

## Economics wiring (step 4)

`crate::drowning` landed with `Ledger` and
`EXPECTED_ANNUALIZED_USD: f64 = 5_068_632.0` — byte-equal to the local
STATUS_QUO_ANNUAL_USD duplicate. Wired: solution.rs now imports
`crate::drowning::EXPECTED_ANNUALIZED_USD`; local duplicate removed.
SUT_ANNUAL_USD (14,400.0) and ANNUAL_SAVINGS_FLOOR_USD (5,050,000.0)
have no drowning counterpart — kept as documented lane-local consts.
Court digests unaffected (economics not in receipt input).

## Anti-vacuity

- `mutation_perturbed_input_flips_digest` runs and passes in both debug
  and release (3/3 above) — one-u64 perturbation flips the BLAKE3 digest.
- The vacuum-fold itself was witnessed as a real defect (0.00042
  observed pre-fix), so the timing court's non-vacuity is empirical.
