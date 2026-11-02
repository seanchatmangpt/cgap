# lane-git-15-a2a-swarm

## Standing

ALIVE (courts + sandbox mutation court ran this session, exits cited below).

## Files written (lane lease honored)

- `crates/cgap-git/src/a2a_swarm.rs` (new, untracked)
- `crates/cgap-git/tests/a2a_swarm_court.rs` (new, untracked)
- this receipt

No git commands run. lib.rs untouched; module wired into courts via
`#[path = "../src/a2a_swarm.rs"]` (loud duplicate-definition if the
coordinator later adds `pub mod a2a_swarm;`).

## Scope deltas (disclosed)

- GIT-13 `ref_store` did NOT land (no `ref_store.rs` in
  `crates/cgap-git/src` at lane start), so GIT-15 ships its own minimal
  per-agent CAS head ref (`propose_commit`, expected/new compare-set
  over `[u8; 20]`). Coordinator may later compose/refactor onto
  GIT-13's store.
- Concurrency model stated honestly: single-threaded court over a
  single-writer-per-agent table. No `core::sync::atomic` used, no
  SMP/lock-free claim made — "zero lock contention" is by construction
  (no locks exist), not a measured SMP property.

## Surface

- `MAX_AGENTS = 32` fixed lease table, zero heap, `#![no_std]`-clean
  (inherits crate-level forbid(unsafe), no std use).
- `ZoneMask([u64; 4])` = 256 coordinates; `intersects` = AND across
  words (const fn).
- `register` / `revoke` (tombstoned — revocation survives slot reuse
  and blocks re-registration) / `check_zone` / `propose_commit` /
  `head` / `active_count`.
- Refusal order (all pre-state): registered -> revoked ->
  ForeignRefPath -> ForeignZone -> CAS; head mutates only if every
  gate passes.

## Courts (all run this session)

| command | exit | result |
|---|---|---|
| `cargo test -p cgap-git --test a2a_swarm_court` | 0 | 4 passed / 0 failed |
| `cargo build -p cgap-git` | 0 | lib green (shared compile intact) |
| all other cgap-git test targets (11 named targets, individual runs) | 0 | green; `ladder_timing_stress` + `perf_baseline` each 0 passed / 1 ignored (env-gated, exit 0 re-verified directly) |
| `cargo test -p cgap-git` (aggregate) | 1 | **foreign in-flight failure**: `tests/retract_court.rs` (87 compile errors, another lane's file + `src/retract.rs`) — compile-abort, NOT a failure count; zero errors in GIT-15 files |

### Mutation court (sandbox, /tmp, shared tree untouched)

- Baseline sha256 both sides: `25d317d3a43bfe8fd6f889a1515fd6dcd38bbc35d18cf8259458e49625a56999`.
- Mutation (`SHIM-COURT`): removed the proposer-skip clause
  (`s.agent != agent`) in `check_zone` in the /tmp sandbox copy.
- Flip: `court_cross_zone_push_refused_andon` +
  `court_clean_20_agent_interleaved_ingress` FAIL (2 failed), 2 pass.
  A court that cannot flip is FAIL — this one flips.
- Restore: byte-exact copy back, sha256 equal, mtime bumped
  (restore law), rerun = 4 passed / 0 failed, exit 0.
- Build-root lease `/tmp/git15-target` and `/tmp/git15-sandbox` deleted
  at lane end.

## Bench (measured this session, stated method)

Sequential interleaved ingress, 20 agents x 3 CAS rounds = 60 ops in
37.333µs (~1.61M ops/s), debug profile, single thread. This is a
debug-build sequential number, not an SMP benchmark.

## Anti-vacuity flip twins (in-test, named mutations)

1. cross-zone court: delete `s.agent != agent` (or the AND) -> inverts
   to Ok — proven to flip by the sandbox run above.
2. foreign-ref-path court: delete `ref_owner != agent` gate -> inverts
   (mutation asserted in test doc; not separately sandbox-flipped —
   UNVERIFIED as a sandbox flip, in-test assertion only).
3. revoke court: drop the revoked gate / tombstone append -> inverts
   (same register: in-test assertion, sandbox flip not run).

## Fences that could regress / checked

- Shared `lib.rs` and `Cargo.toml` untouched; zero edits to existing
  cgap-git sources (git status shows only my two untracked files in
  my ownership set).
- All pre-existing green cgap-git test targets re-run green above.
- `retract_court.rs` breakage is foreign, pre-dates/parallel to this
  lane; not repaired (outside lane lease; fixing it would edit another
  lane's mutation-court semantics).

## Divergences

1. GIT-13 ref_store absent -> own minimal CAS (disclosed above).
2. Full `cargo test -p cgap-git` exits 1 on foreign `retract_court.rs`
   compile errors; per-target runs all green except that file.
3. Bench is debug-profile sequential; perf numbers are targets until
   a release-profile method is stated.
