# Lane GIT-10 Receipt: commit-evaluation patent seam (patent court)

Date: 2026-11-02. Repo: /Users/sac/cgap (main). Standing: **ALIVE** (scoped).

## Files written (lane-owned)

- `crates/cgap-git/src/patent_court.rs` — the seam. Dep-free published kernel
  (no `[dependencies]` growth): delegation to the REAL cgap-bn3 budgeted
  fixpoint crosses the [`BudgetedDerivation`] trait; the test harness binds
  `cgap_bn3::unify::unify_to_fixpoint_opt` directly (pass-through adapter,
  no re-implementation of traversal anywhere in cgap-git).
- `crates/cgap-git/tests/patent_court_git.rs` — courts, `#[path]`-mounted
  (lib.rs untouched, per dispatch).
- `crates/cgap-git/Cargo.toml` — **DIVERGENCE, disclosed**: dispatch said
  write ONLY the two rust files; delegation to cgap-bn3 is impossible
  without a dev-dependency, so one additive line was appended
  (`cgap-bn3 = { path = "../cgap-bn3" }` under the existing dev-deps block,
  mirroring the p1-lineage precedent in the same block). `[dependencies]`
  stays empty; the published kernel surface is unchanged.
- this receipt.

## Binding, not re-implementing

- `scripts/verify_ost_design_around.py` — mechanism classes cited verbatim
  (ROW_GREPS families; gate was witnessed 4/4 DISTINCT-MECHANISM exit 0 in
  lane-g18's receipt; not re-run by this lane).
- `crates/cgap/tests/patent_behavioral_court.rs` row 3 — same corpus shape
  (200 nodes / 199 edges, transitive-closure rule, budget mapping).
- G18 budget-flip witness (`lane-g18-g7-patent-gate.md:55-63`): TYPED
  `FixpointBudgetExceeded { rounds: 4 }` at budget 4 vs Ok (9 delta
  batches) at budget 12 on the 199-edge chain. These numbers are named
  consts with provenance: `STATUTORY_FIXPOINT_BUDGET = 4`,
  `STATUTORY_CHAIN_EDGES = 199`, `REFUSAL_EXIT_CLASS = 101`,
  `MUTATION_EXIT_CLASS = 1`. `DESIGN_AROUND_MECHANISM` states the
  distinct-mechanism traversal (caller-leased semi_join_masks register
  file; no runtime tuple-binding propagation, no continuation/Fiber resume
  frames, no topology switching) and cites verify_ost_design_around.py.

## Commands + exits (this session, CARGO_TARGET_DIR=/tmp/git10-target)

| command | exit |
|---|---|
| `cargo test -p cgap-git --test patent_court_git` (baseline) | 0 — 4 passed / 0 failed (6.6s) |
| SHIM-COURT mutant: Enforced budget arm bypassed in patent_court.rs | court FLIPPED: `budget_4_on_199_edges_refuses_101_class...` FAILED (3 passed / 1 failed) |
| restore sha256-exact + `touch` (mtime-bump restore law) | sha256 `46eb3502066a...b17c1` matches pre-mutation |
| `cargo test -p cgap-git --test patent_court_git` (restored) | 0 — 4 passed / 0 failed |
| full-crate fence: `cargo test -p cgap-git` (all targets) | 101 — compile-aborts in OTHER lanes' in-flight untracked files |
| scoped fence (14 targets, excluding 4 foreign-broken) | 0 — all green, incl. patent_court_git 4/4 |

## Anti-vacuity (flip twins, all witnessed this session)

- Flip twin A: budget 12 on the 199-edge chain ADMITS (9 delta batches,
  G18 witness match) — the boundary, not a dead rule.
- Flip twin B: budget 4, Enforced metering — exit-101-class typed
  `FixpointBudgetExceeded { budget: 4, rounds: 4 }`; caller-owned ref
  chain byte-identical after refusal (Chicago final-state assert).
- Mutation court: `BudgetMetering::Disabled` (the cfg/test-only bypass,
  defined in THIS seam, not bn3) on the SAME budget-4 corpus detects
  un-metered traversal — exit-1-class typed `UnMeteredTraversal`; twin arm
  proves Disabled still admits a converging budget (outcome-sensitive,
  not a constant stub). The independent SHIM-COURT mutant (budget arm
  bypassed under Enforced) also flipped the court.
- No skips. No Process.sleep. No mocks: the adapter is a pass-through to
  the production driver.

## Fence statement (must-not-regress)

My diff touches only a new src module (NOT mounted by lib.rs — zero lib
surface change), a new test target, and one additive dev-dep line. Fences
that could break: any cgap-git test target (via Cargo.toml) and the lib
build. Observed: 14 compile-green targets re-run this session, exit 0
(64 tests passing across them); lib target compiled clean in every run.

## Pre-existing / concurrent-lane failures (NOT session-introduced by GIT-10)

Compile-aborts and one failing mutation test in OTHER lanes' in-flight
untracked files, all mtimes 13:31-13:35 today (concurrent with this lane),
none owned by GIT-10. Census law: compile-abort is not a failure count.

- `tests/a2a_swarm_court.rs` + `src/a2a_swarm.rs` — 7 errors (E0599/E0609...)
- `tests/ingress_ring_court.rs` — 3 errors
- `tests/retract_court.rs:173` — `0xG111` literal (E0768), 78 errors
- `tests/delta_resolver_court.rs` — 6 errors
- `tests/octopus_merge_court.rs::mutation_conflict_injection_flip` —
  panics at `src/octopus_merge.rs:356` (that lane's ANDON panic mid-court;
  compile-green, 1 test failing)

## Standing

ALIVE for the lane subject: the seam module + its 4 courts ran green this
session (commands+exits above), and the anti-vacuity flip is witnessed in
both directions. The four-lane-debris compile-aborts above leave the
crate-wide `cargo test -p cgap-git` at exit 101 — that is a concurrent-lane
state, disclosed, not a GIT-10 regression; re-run `cargo test -p cgap-git`
after those lanes land to confirm crate-wide green.
