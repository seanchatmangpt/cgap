# Lane B08 — CSCB runner + corpus (v26.11.2)

Standing: PARTIAL — runner ALIVE, ladder aggregate exit 1 (two honest
fails), anti-vacuity flip court FAIL (cannot flip; close condition
named).

## Files written (exactly the dispatch list)

- `scripts/generate_cscb_corpus.py` — stdlib-only, SplitMix64 seed
  `0xA20C_0FFE_E5A7_E000` (multiplier `0xBF58_476D_1CE4_E5B9`), 1M
  triples x 24B LE, `--validate` byte-identical replay court.
- `crates/csct/src/bin/cscb_runner.rs` — executes C1..C8 in RFC order,
  typed SubjectMissing (exit 2, loud `[SKIP]`) for absent siblings,
  CSCB_MUTATE + CSCB_CORPUS propagation, BLAKE3 transcript root,
  markdown receipt emission.
- `docs/csct/cscb-certified-run-v1.md` — emitted by the runner (not
  hand-authored); regenerated on every run.
- `docs/jira/v26.11.2/lanes/lane-b08-cscb-runner.md` — this receipt.

## Commands + exits (this session, exact subjects)

| command | exit |
|---|---|
| `python3 scripts/generate_cscb_corpus.py` | 0 — sha256 `f5069a892eb04c30682636c128f62ee14301296ef756b7609745806912659ec2`, count 1000000, path `/tmp/cscb_corpus.bin` |
| `python3 scripts/generate_cscb_corpus.py --validate` | 0 — identical sha256 (byte-identical replay) |
| `cargo build --release` (crates/csct) | 0 — all 6 cscb bins + runner |
| `target/release/cscb_runner --receipt-out docs/csct/cscb-certified-run-v1.md` (all bins missing) | 0 — 8x `[SKIP] exit 2`, typed SubjectMissing path exercised |
| same, siblings landed | 1 — C3 exit 1, C6 exit 1 (honest FAIL, not rounded up); C1/C2/C4/C5/C7/C8 exit 0 |
| `CSCB_MUTATE=zero-tax` ladder | 1 — but flip NOT witnessed (see court) |

## Anti-vacuity court: CSCB_MUTATE=zero-tax — FAIL (cannot flip)

- Aggregate-refusal assertion: ALIVE. Witnessed twice: all-missing run
  exits 0 (skips recorded, no fail masking), present-child-refuses run
  exits 1 (C3 exit 1 → aggregate 1).
- Mutation flip: FAIL. Under `CSCB_MUTATE=zero-tax`, NO child flips:
  C1/C2/C4/C7 exits are byte-identical to baseline; C3/C6 fail both
  runs (mutation confounded). A court that cannot flip is FAIL.
- Close condition: (a) B02 fixes C3 closure and B05 fixes C6 retraction
  to pass baseline; (b) at least one passing child implements zero-tax
  refusal semantics reading CSCB_MUTATE; then
  `CSCB_MUTATE=zero-tax <runner>` must produce a strictly different
  exit profile than baseline (aggregate stays nonzero, but a
  previously-passing coordinate must refuse).
- Current flip runs receipted at `/tmp/b08-certified-run-baseline.md`
  (root `6405fbf0...`) and `/tmp/b08-certified-run-mutate.md` (root
  `ba35f705...`); canonical certified receipt root
  `79965bc2b6eeffb9335f3d6ca6c7e050504824f0cf7f0bea5fec9875f0ad9df8`.

## C8 harness limitation (recorded in certified receipt context)

C8 is B07's test-module court, not a bin. The runner invokes
`cargo test drowning_benchmark` with cwd = the csct manifest dir
(`crates/csct`), because cargo requires the manifest dir as cwd when
invoked from a binary; `-p csct` is implied by the standalone-crate
[workspace]. If the filter is absent from sources, the runner types it
SubjectMissing (exit 2) via a source scan for `drowning_benchmark`.
C8 witness (direct run, this session): 4 passed / 0 failed / 37
filtered out, including the flip court
`drowning_benchmark::tests::court_flips_on_zeroed_spark`.

## Fences (must-not-regress surfaces)

- crates/csct is a standalone-crate [workspace]; the root workspace
  gate is untouched by this lane (no root member edited). Observed:
  csct release build exit 0 with all sibling bins present.
- Runner shares the sibling bins' target dir (standalone crate target/
  release); no `CARGO_TARGET_DIR` lease was opened — the csct crate's
  own `target/` holds the artifacts, built by `cargo build --release`
  in crates/csct.
- `docs/csct/cscb-certified-run-v1.md` is runner-emitted; hand edits
  would be overwritten by the next run by design.
