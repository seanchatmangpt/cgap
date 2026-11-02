# Lane C02 — CSCT cold-path coverage lift

Standing: ALIVE (all commands run this session, exits recorded below).

## Files written

- `crates/csct/tests/csct_coverage_court.rs` (NEW, 356 lines, 9 courts)
- `docs/jira/v26.11.2/lanes/lane-c02-csct-coverage-lift.md` (this receipt)

No src edits. The dispatch permitted `#[cfg(test)]` additions in
`src/types.rs`/`src/receipt.rs` for demonstrated-cold private paths, but every
cold path found is reachable through the public API, so none were needed.
`tests/csct_pipeline_court.rs` untouched.

## Cold paths covered (cited, verified unexercised at baseline)

Baseline before lane: 51 tests / 0 failed (37 lib + 7 bin + 7 integration).

- `receipt.rs:91-93` — push_roi "undefined" branch (zero post-adoption cost):
  never rendered by any prior test (all pinned 14,400).
- `receipt.rs:101-103` — push_assertions "(none)" branch (empty assertions):
  never rendered.
- `receipt.rs:123-129` — push_f64 fractional `{:?}` branch: prior samples all
  integral-valued; exercised with 0.5 / 1.5 / 4.25 observed values.
- `receipt.rs:38` — Debug render of `SubjectMissing`, `LoudSkip`,
  `AndonRefusal`: only Pass/GateFail were rendered before.
- `receipt.rs:69-70` — verdict "FAIL" render branch: prior tests always
  rendered PASS.
- `types.rs:63-117` — report_digest over multi-stage reports; sensitivity to
  stage_index, stage_name, per-stage exit, assertion append, wall_clock,
  alloc_count, receipt bytes, stage removal, stage reorder.

Not covered here (pre-covered elsewhere, not duplicated):
`ExitCode::as_u32` all variants was already pinned by
`src/types.rs:191-197`; runner `classify()` edge shapes (empty args, unknown
flag, bad `--mutate` id) were already pinned by
`src/bin/csct_runner.rs:123-159`. The bin's `exit_for()` is private to the
bin target and unreachable from an integration court — UNVERIFIED, left to
the bin owner. This is the one divergence from the dispatch text: classify()
coverage was requested but already existed; duplicating it would be
redundant, so the lane went where the actual cold paths were.

## Commands + exits

```bash
cd crates/csct && CARGO_TARGET_DIR=/tmp/c02-target cargo test
# baseline: 51 tests, 0 failed (37+7+7), exit 0
# interim:  2 FAILED (wrong byte pins: threshold 2.0 renders "2" via the
#           integral f64 branch; verdict line has no trailing " |")
# final:    60 tests, 0 failed (37 lib + 7 bin + 9 integration), exit 0
```

Final: `cargo test` exit 0, 60 passed, 0 failed. New courts: 9 (each with
its anti-vacuity twin or twin-equivalent matrix/structural mutation).

## Fences (must-not-regress surfaces checked)

- `tests/csct_pipeline_court.rs` (coordinator-owned): green in every run
  (7 passed), byte-untouched on disk.
- src/types.rs, src/receipt.rs, src/bin/csct_runner.rs: byte-untouched.
- Rustfmt hook reformatted the new file; compile + tests confirm intact
  bytes.

## Anti-vacuity discipline

Every court names its mutation: exit-variant swap matrix (20 ordered pairs,
wire + rendered bytes must both move), heal-mutation on the failing stage,
populate-the-empty-stage + set-cost mutation, 7 per-field digest flips,
structural removal/reorder. Digest collision on any mutated field trips a
court. No skips; no Process.sleep; mocks: none.
