# CSCT Lane 11 Receipt — End-to-End Integration Court

- **Lane**: CSCT-L11
- **Date**: 2026-11-02
- **Subject**: `crates/csct/tests/csct_pipeline_court.rs`
- **Standing**: **UNVERIFIED:compile-pending** — the csct lib does not compile
  (modules owned by lanes L3–L10 not yet landed: `reframe.rs`, `solution.rs`,
  `andon.rs`, `court.rs` are absent). Compile-abort is not a failure count.

## Files written (owned only)

1. `crates/csct/tests/csct_pipeline_court.rs` (8 tests, ~140 lines)
2. `docs/jira/v26.11.2/waves/csct-lane-11-court.md` (this receipt)

## Contract coverage

| # | Contract item | Test fn |
|---|---|---|
| a | 6 stages, indices 1..=6 in order, all Pass, verdict true | `suite_runs_six_stages_in_order_all_pass` |
| b | net delta = 5,068,632 − 14,400; bleed = 5,068,632 | `economic_block_matches_rfc0042` |
| c | `run_mutation_controls()` is Ok | `mutation_controls_pass` |
| d | receipt non-empty, carries stage names + hex digest, byte-identical determinism | `receipt_renders_deterministically_with_stages_and_digest` |
| e | anti-vacuity: no stage passes with zero assertions / ≥1 passing assertion | `no_stage_passes_vacuously` |
| f | `ExitCode::AndonRefusal == 101` (cast + `as_u32`) | `andon_refusal_wire_value_is_101` |
| + | court falsifier: digest moves on stage flip; vacuum guard trips on emptied stage via `catch_unwind` | `court_flips_on_stage_mutation` |

## Commands + exits (this session)

- `cd crates/csct && CARGO_TARGET_DIR=/tmp/csct-l11-target cargo test --test csct_pipeline_court`
  → **exit 101** (compile abort: `error[E0583]: file not found for module 'court'`;
  also missing `reframe`, `solution`, `andon`).
- Build root `/tmp/csct-l11-target` deleted at lane end (shutil.rmtree, done).

## Divergences / notes

- `csct` is NOT a member of the root workspace manifest (coordinator-owned);
  tested standalone in `crates/csct`. Dispatch said "root manifest is
  coordinator-owned" — consistent, no divergence in ownership, only in
  assumption that the lib compiles today.
- One mid-write corruption occurred during initial Write (placeholder tokens
  landed on disk); rewritten via python assembly and re-verified on disk.
- Must-not-regress fences: none new — test-only diff against a not-yet-compiling
  lib; no lib/manifest edits, no git commands, shared tree untouched.
- Chicago discipline held: real `run_suite()` execution, assertions on final
  state, no mocks, no `Process.sleep`; the guard's non-vacuity is itself
  witnessed (emptied stage must trip the guard via `catch_unwind`).
