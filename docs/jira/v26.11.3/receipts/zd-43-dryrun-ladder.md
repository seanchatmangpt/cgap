# ZD-43 — crates.io Dry-Run Ladder (CARRY-17 Stage 2)

Date: 2026-11-04. Subject: /Users/sac/cgap at HEAD 37721e2fa67132e917f6c6addf0c4c239e3777bf.
Standing: PARTIAL — 7/20 dry-run exits 0; two typed failure classes; the predicted
license-file blocker never fired.

## Method

Read-only probing. `cargo publish -p <crate> --dry-run` per crate, sequential, topo order;
per-crate logs at /tmp/zd43/<crate>.log. No source edits, no git transitions, no real
publishes (dry-run aborts at upload: "aborting upload due to dry run"). No --allow-dirty
(it would have masked the dirty-worktree class).

## Topo order and census reconcile

`cargo tree -p cgap --workspace --edges normal --depth 1` re-derived.

- The task's L0-L3 order was confirmed for the 20 listed crates; ladder run in that order.
- Census discrepancy (recorded, not repaired): 22 manifests carry publish = true — the
  task's 20 plus crates/integration and crates/wasm-footprint (both P3-10 flips,
  registry-only dep surfaces, workspace members). Both excluded per the task list; flagged
  for the Stage-3 owner to adjudicate.
- bench appears as a cargo-tree root but is publish = false — tree roots are not the
  publish census; manifests are.

## Ladder table (all 20 exits)

| crate | dry-run exit | error class (first error line) |
|---|---|---|
| cgap-arrow | 0 | — |
| cgap-blake3 | 0 | — |
| cgap-bosym | 0 | — |
| cgap-branchless | 0 | — |
| cgap-aoq | 0 | — |
| cgap-core | 101 | registry-dep-missing: failed to select cgap-blake3 ^26.11.3 (candidates 26.11.2, 26.11.1, 26.10.29) |
| cgap-git | 0 | — |
| cgap-parquet | 0 | — |
| cgap-bont | 101 | registry-dep-missing: cgap-blake3 ^26.11.3 |
| cgap-bn3 | 101 | registry-dep-missing: cgap-blake3 ^26.11.3 |
| cgap-storage | 101 | registry-dep-missing: cgap-blake3 ^26.11.3 |
| cgap-crypto | 101 | registry-dep-missing: cgap-blake3 ^26.11.3 |
| cgap-gate | 101 | registry-dep-missing: cgap-branchless ^26.11.3 |
| cgap-audit | 101 | dirty-worktree: 1 uncommitted file (crates/audit/src/alloc_counter.rs) |
| cgap-gpu | 101 | registry-dep-missing: cgap-arrow ^26.11.3 |
| cgap-fibo | 101 | registry-dep-missing: cgap-blake3 ^26.11.3 |
| cgap-p28 | 101 | registry-dep-missing: cgap-blake3 ^26.11.3 |
| cgap-cloud | 101 | registry-dep-missing: cgap-blake3 ^26.11.3 |
| cgap-ffi | 101 | dirty-worktree: 8 uncommitted files under crates/cgap-ffi/ |
| cgap | 101 | registry-dep-missing: cgap-arrow ^26.11.3 |

## Error classes

Two classes only. No license-file error fired on any of the 20 crates — the plan doc's
primary Stage-1 blocker did not materialize; packaging metadata passed everywhere. The
task's STOP condition (all-fail-on-license) did not fire, so the ladder ran to completion.
E0583 count: 0. Doc-test failures: 0 observed (dry-run verify compiles in dev profile;
doctest behavior of the verify step is UNVERIFIED as-of-37721e2f, re-test at Stage 3).

1. registry-dep-missing (11 crates): dry-run resolves sibling deps from the crates.io
   index, and no 26.11.3 sibling is published yet. Self-healing: the Stage-3 real ladder
   publishes L0 first, then dependents resolve. Exact unresolved req per crate in the
   table above.
2. dirty-worktree (2 crates): cargo publish refuses to package from a dirty worktree.
   cgap-audit: crates/audit/src/alloc_counter.rs. cgap-ffi: Cargo.toml, src/lib.rs,
   src/sovereign.rs, src/xip_tables.rs, src/aoq_session.rs, src/bin/aoq_sidecar.rs (+2
   more per /tmp/zd43/cgap-ffi.log). These are other lanes' in-flight edits — not touched
   (read-only probing mandate); owning lanes must commit before Stage 3 can include them.

## Conclusion and handoff to Stage 3

- 7 ALIVE / 11 PARTIAL (registry-dep-missing, self-healing) / 2 BLOCKED (dirty-worktree)
  = 20. L0 is fully green (arrow, blake3, bosym, branchless, aoq, git, parquet).
- License fix (plan §1 Stage 1) is NOT a dry-run gate — dry-runs passed packaging
  metadata on all 20 manifests; Stage 1 can be deferred or re-scoped.
- Handoff items: (1) owning lanes commit crates/audit + crates/cgap-ffi in-flight edits;
  (2) adjudicate the 22-vs-20 publish census excess (integration, wasm-footprint);
  (3) Stage 3 may start immediately on the green L0, topo order with bounded polls.
- Plan §5 falsifiers: §5.1 (path-only dev-dep stripping) and §5.2 (docs.rs rustc-args in
  verify) both remain UNVERIFIED as-of-37721e2f — re-test at Stage 3.

## Correction (2026-11-04, ZB-16 re-baseline)

The "22 publish=true" census recorded above was pre-sweep. Live count at
e3233191: **21 publishable crates** in the root workspace (4 excluded via
publish = []: wire-shm, cgap-beam, bench, cgap-court; cgap-mcp is a
separate workspace with publish=false per yd-3). Stage 3 ladder operates
on 21.
