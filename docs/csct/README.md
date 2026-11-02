# csct

CSCT (Challenger Sales Chicago Test, RFC-0042) is the commercial-qualification
court for cgap: a 6-stage pipeline that runs Challenger-sale evidence against a
Chicago-school test discipline — every stage carries a real collaborator, a
metric contract, and an anti-vacuity mutation. It converts the campaign's
economic story ($5,068,632/yr status-quo bleeding vs a $1,200/mo ceiling) into
executable gates with the repo's exit-code contract (0/1/2/3/101).

Landing: `crates/csct` (standalone crate, own `[workspace]`, `publish = false`,
single dep `cgap-blake3`).

## Crate map

- `crates/csct/src/lib.rs` — scaffold: module declarations, economics constants
  (`STATUS_QUO_ANNUAL_USD = 5_068_632.0`, `SUT_ANNUAL_USD = 14_400.0`,
  `REQUIRED_ROI_X = 352.0`), `run_suite()` executing the 6 stages in order with
  early-return on first failing stage.
- `src/types.rs` — `ExitCode` (Pass=0, GateFail=1, SubjectMissing=2,
  LoudSkip=3, AndonRefusal=101), `StateAssertion`, `StageResult`, `Report`.
- `src/warmer.rs`, `src/reframe.rs`, `src/drowning.rs`,
  `src/emotional.rs`, `src/new_way.rs`, `src/solution.rs` — the 6 stage modules.
- `src/mutation.rs` — anti-vacuity mutation controls: a court run without a
  demonstrated flip is VACUOUS (exit 1).
- `src/andon.rs` — typed refusal: stop-the-line exits with `AndonRefusal = 101`
  when a stage cannot be evaluated honestly (loud skip, never silent pass).
- `bin/csct_runner` — runner binary (`--suite=... --receipt`).
- `tests/csct_pipeline_court.rs` — court asserting stage sequencing and the
  final commercial verdict gate.

Status note: `bin/csct_runner` and `src/court.rs` are declared by lanes in
flight and are absent from the tree at writing time (see
`docs/jira/v26.11.2/waves/csct-lane-13-docs.md` divergences).

## How to run

```bash
cargo test -p csct
cargo run -p csct --bin csct_runner -- --suite=cloud-cartel-1360shard-vs-cgap --receipt
```

Both commands target the standalone crate (own `[workspace]`), not the root
workspace. Standing of these commands this session: NOT RUN (docs lane — see
receipt).

## Exit-code contract

| code | variant | meaning |
|---|---|---|
| 0 | `Pass` | all 6 stages passed; verdict gate evaluated |
| 1 | `GateFail` | a metric contract failed; Report carries the failing stage |
| 2 | `SubjectMissing` | suite subject absent from the corpus |
| 3 | `LoudSkip` | stage skipped, loudly, with typed reason |
| 101 | `AndonRefusal` | stage refused: cannot evaluate honestly; stop the line |

## Dogfood law

No protobuf, no JSON, no serde anywhere in the crate. Types are plain Rust
structs; receipts are deterministic markdown rendered by `src/receipt.rs`
(`render_report`) and fingerprinted with `cgap-blake3`. This keeps the court
byte-replayable with zero-dep kernels only.
