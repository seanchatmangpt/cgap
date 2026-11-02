# Lane G11 — csct court fix: independent verification receipt

- Date: 2026-11-02
- Lane: G11 (verification-only; shared tree untouched)
- Subject: `crates/csct` (sha256 tests/csct_pipeline_court.rs =
  `f1401e0f...3d905a`, src/lib.rs = `ecb653d5...2139` at receipt time)

## Verdict

Standing: **ALIVE**. All six witnesses pass this session.

| # | Witness | Command | Exit | Result |
|---|---------|---------|------|--------|
| 1 | Full suite | `cd crates/csct && CARGO_TARGET_DIR=/tmp/g11-target cargo test` | 0 | 37 lib + 7 runner + 7 court, 0 failed |
| 2 | Placeholder sweep | `grep -rnE 'PLACEHOLDER\|TODO\|FIXME' tests/csct_pipeline_court.rs` | 1 (no match) | 0 tokens |
| 3 | catch_unwind guard read | read tests file:85-147 | — | `court_flips_on_stage_mutation` (L132-147): digest moves on GateFail flip (L134-140), emptied stage 2 trips guard via `catch_unwind` (L143-146), asserted `tripped` |
| 4 | No `crate::court` refs | `grep -rn 'crate::court\|mod court' crates/csct/` | 0 | Only matches: a comment in lib.rs:18 ("`pub mod court` was dropped") and a STALE cargo fingerprint diagnostic in `target/` (pre-repair debris, not source). No live source reference. `src/` has no court.rs. |
| 5 | Release witness | `cargo run --release --bin csct_runner -- --suite=cloud-cartel-1360shard-vs-cgap --receipt` | 0 (captured un-piped) | `final_commercial_verdict \| PASS`, 6 stages Pass, ROI 352x, net delta $5,054,232 |
| 6 | Anti-vacuity mutation | /tmp sandbox copy, stage-1 assertions emptied | 1 | `no_stage_passes_vacuously` FAILED: "stage THE_REFRAME passed with ZERO assertions" — guard trips in landed form |

## Anti-vacuity (witness 6 detail)

Sandbox: `/tmp/g11-csct-sandbox` (python copytree of the crate; dep
repointed to absolute `/Users/sac/cgap/crates/cgap-blake3`; mutation
marked `SHIM-COURT` in the copied tests file, emptying
`report.stages[1].assertions` inside `no_stage_passes_vacuously`).
`cargo test --test csct_pipeline_court` = 6 passed / 1 failed
(`no_stage_passes_vacuously`) — the vacuum guard is non-vacuous in its
landed form. Sandbox + both build roots deleted at lane end; shared
tree never mutated.

## Fences

Must-not-regress surfaces this diff could break: none — the lane wrote
one doc file. Fences run anyway (witness 1 = full csct suite; witness
5 = release runner). Witness 1 was run twice: first invocation piped
to `tail`, so the exit code shown was tail's (pipe law); re-run
captured the true exit 0 un-piped.

## Divergences

- None vs the stale-premise notice. The `crate::court` grep hit in
  `target/` is stale build debris, not a source defect; no test-file
  edit was needed.
