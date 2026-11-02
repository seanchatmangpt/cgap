# lane-07-alloc-interceptor — AE7 backfill note (receipt was missing; landed state + open failure)

Date: 2026-11-02. Lane: F3 (AE-lane receipt backfill, per W7 metagate).
Branch: main, canonical checkout /Users/sac/cgap. Doc-only lane:
read-only vs source, no git transitions, no source edits.

## W7 metagate finding (why this file exists)

W7 found AE7's lane receipt missing. This section backfills the landed
state as of F3's session; repair ownership stays with F1 (repairs/
repair-08 is F1's file — F3 did not touch it).

## What is landed (verified on disk this session)

- `crates/cgap-core/src/tracking_alloc.rs` — the enclave tracking
  allocator interceptor (counting + budgeting GlobalAlloc type).
  Registration divergence recorded in the module doc: the
  `#[global_allocator]` REGISTRATION is deliberately per-binary (rustc
  E0152 collision with per-binary counting allocators already landed
  across cgap test targets); the landed court binary is
  `crates/cgap/tests/alloc_boundary.rs`.
- `crates/cgap/tests/alloc_boundary.rs` — AE7 court binary
  (`ae7_alloc_boundary_courts` at :188, plus the positive-control
  court), 2 `#[test]` fns on disk.
- Coordinator DEPTH fix LANDED: DEPTH LAW doc at
  `tracking_alloc.rs:44-47` (breach zeroes DEPTH while guards are
  alive; plain fetch_sub wraps to usize::MAX and leaks ARMED into later
  windows) and the saturating disarm compare_exchange loop at
  `tracking_alloc.rs:119-130` — the breach-depth-underflow defect is
  fixed in code.

## Open failure (witnessed this session, F3)

Two-court failure per W7/CL2 has narrowed to ONE failing test fn.
Current state witnessed this session:

    $ CARGO_TARGET_DIR=/tmp/f3-target cargo test -p cgap --test alloc_boundary
    thread 'ae7_alloc_boundary_courts' panicked at
    crates/cgap/tests/alloc_boundary.rs:287:9:
    test result: FAILED. 0 passed; 1 failed; exit 101

- The line-287 panic is Court 4 (H8 honesty — bn3 fixpoint is NOT
  zero-alloc; documented ceiling AE7_FP_BUDGET_ALLOCS=512 /
  AE7_FP_BUDGET_BYTES=2MiB, `alloc_boundary.rs:295-298`): the fixpoint
  window breached, so `last_breach()` is Some at the assert. The
  remaining failure is the court-4 fixture/ceiling mismatch, matching
  the dispatch's "court-4 fixture" item.
- The other historical panic (breach-depth underflow,
  `tracking_alloc.rs:156`) is addressed by the landed DEPTH fix; the
  current run no longer shows that panic text.

## Repair ownership

F1 is finishing AE7 (repairs/repair-08). F3 records state only; the
court-4 ceiling/fixture reconciliation is F1's to land with its own
witness run.

## W4 static diagnosis

The dispatch directs citing W4's static diagnosis of the breach-depth
underflow. NO W4 wave receipt naming this diagnosis exists on disk
(searched docs/jira/v26.11.2/** for DEPTH/underflow/tracking_alloc/
alloc_boundary). The static diagnosis that survives on disk is the
DEPTH LAW module doc itself at `crates/cgap-core/src/tracking_alloc.rs:44-47`,
which carries the mechanism (breach zeroes DEPTH -> disarm wraps ->
ARMED leak) and the fix shape (saturating disarm). F3 cites that in
place of a missing W4 receipt; if W4's receipt lands later, it
supersedes this citation.

## F3 session receipts (reads + exits)

| Command (cwd /Users/sac/cgap) | Exit | Note |
|---|---|---|
| CARGO_TARGET_DIR=/tmp/f3-target cargo test -p cgap --test alloc_boundary | 101 | court-4 panic at alloc_boundary.rs:287 (pipestatus-verified 101) |
| Read crates/cgap-core/src/tracking_alloc.rs, crates/cgap/tests/alloc_boundary.rs | 0 | DEPTH fix + court-4 ceiling cites verified on disk |

## Standing

PARTIAL. Landed state ALIVE-cited as of this session's run: interceptor
type + court binary + coordinator DEPTH fix on disk, one failing court
remaining (court-4 fixture/ceiling), repair in flight with F1. This
backfill does NOT mark AE7 done — repair-08/F1 closes it.
- `CARGO_TARGET_DIR=/tmp/ae7-target cargo test -p cgap --test alloc_boundary`
  -> exit 0, `test result: ok. 1 passed` (landed gate; all four
  courts + positive control). Rerun witness `/tmp/ae7-final.txt`.
- `CARGO_TARGET_DIR=/tmp/ae7-target cargo build -p cgap-core` -> exit 0.
- `CARGO_TARGET_DIR=/tmp/ae7-target cargo test -p cgap-core` -> exit 1:
  PRE-EXISTING failure `dod10_no_spdx_license_identifiers_anywhere`
  (crates/cgap-core/tests/manifest_court.rs:56) — the court scans
  Cargo.toml/README.md, neither touched by this lane; the landed
  manifest carries `license-file = "../../LICENSE.md"`, which that
  court's letter forbids. Recorded as observed, NOT session-introduced;
  the court and manifest contradict each other and neither file is in
  this lane's ownership.
- The dispatch's `cargo test -p cgap-core --features host` does not
  exist as written (cgap-core declares no features; see §2 divergence).

## 5. Anti-vacuity mutant witness (SHIM-COURT)
The topology guard refuses repo copies into /tmp, so the lane-doc
"/tmp sandbox mutant" was executed as the landed shim-court pattern
(in-tree mutate, sha256 fail-closed restore, mtime bump). Divergence
recorded. Mutation: one `Vec::with_capacity(4)` push inside the
penstock ingress court loop (tests/alloc_boundary.rs, SHIM-COURT-
marked insert).
- BEFORE sha256(crates/cgap/tests/alloc_boundary.rs) =
  2aa7927bb5483d9eeff39c6df60a009ac67496040177454796fb64ee3a76fa33
- MUTANT run: exit 101; stderr names the flip:
  `ALLOC BUDGET BREACH: site="ae7-penstock-ingress-16" allocs=1 bytes=4`
  `AE7 COURT FLIP: penstock ingress window breached: Breach { site:
  "ae7-penstock-ingress-16", allocs: 1, bytes: 4, offending_bytes: 4 }`
  — the interceptor trips and the court fails naming the site.
- RESTORE sha256 = 2aa7927bb5483d9eeff39c6df60a009ac67496040177454796
  fa33 (byte-exact), mtime bumped (cargo rebuilt; final run exit 0).
- Shared tree otherwise untouched; no git writes; publishes frozen.

## 6. Standing
ALIVE (commands + exits above, run this session against the exact
subject). Perf numbers: none claimed — this lane measures allocation
counts, not ns/frame; the fixpoint ceiling is a measured, pinned
allocator budget (278 allocs / 14,216,246 B witnessed unbounded; a
budget-0 court on the fixpoint would be dishonest per H8 and would
breach instantly — the ceiling court is the honest form).

## 7. Divergences (all recorded)
1. Module not declared in cgap-core lib.rs (lane-frozen); court
   includes it via `#[path]`. `--features host` verify command does
   not exist (cgap-core has no features; SP1's host gate lives in the
   cgap umbrella manifest).
2. "/tmp sandbox mutant" executed as in-tree shim-court (topology
   guard refuses /tmp repo copies); sha256-exact restore + mtime bump.
3. Registration divergence: `#[global_allocator]` registration lives
   in the court binary, not the lib (E0152 vs landed per-binary
   allocators; documented in the module doc).
4. "thread-local tracking" per dispatch -> process-global atomics
   (thread_local! is std-only; keeping the type core-clean matters
   more, and the single-driver sequencing law preserves the
   isolation thread-locals would have given).
5. cgap-core pre-existing manifest_court failure observed this session
   (SPDX-court vs license-file manifest contradiction); not introduced
   by this lane.
