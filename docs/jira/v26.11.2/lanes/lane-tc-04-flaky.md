# lane-tc-04-flaky

Lane TC-04 — octopus 20x stability re-run + pack_demux c7
harden-or-cull (v26.11.2 starDF). Repo /Users/sac/cgap, branch main.
Build root lease: `CARGO_TARGET_DIR=/tmp/tc04-target`.

## Octopus: 20/20, no split (per dispatch arm 1)

Re-ran the exact GAP-09 subject 20x solo, bounded loop, no sleeps:

- `for i in $(seq 1 20); do CARGO_TARGET_DIR=/tmp/tc04-target cargo
  test -p cgap-git --test octopus_merge_court; done` → 20/20 exit 0
  (FAILS=0/20; per-run rc echoed, `run 20 rc=0` last).
- Run-1 log witness: the by-design child re-exec still prints
  `ANDON domain clash ... heads 0 and 2 conflict` with CORRECT
  attribution, `test result: FAILED. 0 passed; 1 failed` (child) then
  `test result: ok. 5 passed; 0 failed` (parent). GAP-08's ~50% witness
  was pre-fix bytes; the :309 prior-INPUT-row fix holds.

Per dispatch: instability did NOT reproduce → both arms stay
unchanged in `crates/cgap-git/tests/octopus_merge_court.rs` — the
self-re-exec 101 witness stays inside
`mutation_conflict_injection_flip` and the in-process DomainClash arm
stays always-run. NO edits made to that file this session (verified:
file untouched by TC-04; `git status` shows no TC-04 diff against it).

## c7 decision: CULL (ignore-gate, subsumer named)

Per audit/cull-03-git-court-census.md flag 2:

- `SERIAL` mutex in pack_demux_court protects only this binary; a
  foreign allocation from a parallel sibling binary inflates the
  zero-alloc delta (the witnessed false-FAILED "feed allocated 1
  times"). soc_full_chip_court already runs the counting allocator
  with a windowed instrument + positive control; c8 covers the static
  no-std scan. Distinct value of c7 was only the ISOLATED-demux claim.
- Moved the isolated-demux zero-alloc claim into
  `crates/cgap-git/tests/soc_full_chip_court.rs` as the ONE disclosed
  line (block at the pipeline zero-alloc assert, `soc_full_chip_court.rs:489-521`):
  builds the clean minimal 66-byte pack on the STACK (no heap), opens
  the existing window instrument around a fresh `Demux::new()` feed +
  1-record fixed-table drain, asserts delta == 0 with a VACUOUS marker;
  the existing positive alloc control (run earlier in the same test,
  asserted `ctrl > 0`) proves the window counter is alive, so this
  window is not pinned-at-zero.
- `c7_parse_path_is_zero_alloc` marked
  `#[ignore = "CULLED per cull-03 flag 2: subsumed by
  soc_full_chip_court demux-stage zero-alloc window + c8 static scan;
  SERIAL mutex does not shield against foreign-binary allocations"]`
  — kept, not deleted, replayable via `-- --ignored`.


## Commands + exits (this session)

- `for i in $(seq 1 20)` 20x octopus loop with per-run rc →
  FAILS=0/20, loop exit 0.
- `cargo test -p cgap-git --test pack_demux_court`
  (CARGO_TARGET_DIR=/tmp/tc04-target) → exit 0, 9 passed / 0 failed /
  **1 ignored** (c7).
- `cargo test -p cgap-git --test pack_demux_court -- --ignored`
  → exit 0, c7 passes in isolation (1 passed; law still holds when
  replayed — culled for isolation failure mode, not for being false).
- `cargo test -p cgap-git --test soc_full_chip_court` → exit 0,
  1 passed (7.80s), including the new isolated-demux zero-alloc block.
- `cargo test -p cgap-git` (whole crate) → see below.

## Fences run (must-not-regress)

- octopus_merge_court 5 courts: green 20/20 — could have regressed
  only if TC-04 had touched octopus_merge.rs or the court; no diff.
- pack_demux_court c1-c6, c8: green — c7 ignore does not touch
  their harness; 9 passed post-change.
- soc_full_chip_court pipeline laws + canary + mutation courts A/B
  + positive control: green — the added block opens/closes the window
  AFTER the pipeline close, cannot disturb the pipeline zero-alloc
  assert (window closed before it).


### Whole-crate run

- `cargo test -p cgap-git` (CARGO_TARGET_DIR=/tmp/tc04-target) →
  exit 0. Every target 0 failed; octopus_merge_court shows its
  by-design `FAILED. 0 passed; 1 failed` child line followed by
  `ok. 5 passed` parent; pack_demux_court `ok. 9 passed; 1 ignored`;
  soc_full_chip_court `ok. 2 passed` (includes the thread-spawned
  wrapper court). No compile aborts (the E0063 soc abort GAP-09 saw
  is gone — GIT-16 landed `ObjectStore::new(cgap_blake3::hash)`).

## Standing

ALIVE for: octopus 20/20 stability verdict, c7 cull + soc claim
  move, all three court targets green this session. UNVERIFIED: none
  outstanding for this lane's scope.

## Divergences

- Dispatch named writing octopus_merge_court.rs as a possible
  target; arm-1 outcome (stable) means NO edit was needed — file
  untouched, zero diff from TC-04.
- soc_full_chip_court.rs edit is exactly the ONE disclosed block
  (isolated-demux zero-alloc window); no other lines touched.

