# Lane H3 — commit durability / concurrency verification + rotation writeback

Standing: ALIVE (all commands + exits this session, 2026-11-02)

## (1) FA9FIX as-of verification

- `CGAP_ERR_CONCURRENT_WRITER_DENIED = 0x4347_0006` at
  `crates/cgap-ffi/src/sovereign.rs:87` — verified on disk.
- `spill_gate::claim` pre-IO denial (claimed fail-closed before any
  `File::create`), per-(path-hash, thread-id) registry:
  `sovereign.rs` mod spill_gate (FNV-1a over path bytes, thread-local
  OwnersGuard release on thread exit).
- `cargo test -p cgap-ffi` exit 0 — full suite: 50 lib (wave9 courts incl.
  zero-string gate over built crates, 59.7s) + 4 alloc + 3 fa9
  (fa9_gate_register_pin, fa9_release_after_owner_thread_exit,
  fa9_race_grid_one_writer_no_truncation) + 2 lever + 5 zero_copy. 0 failed.

## (2) Rotation writeback — drain_to_spill now power-fail durable at the rotation boundary

Diff (crates/cgap-parquet/src/mmap_writer.rs):

- New `pub trait SpillSync` (`drain_bytes(&mut self, bytes)`): durability
  contract stated once. `impl SpillSync for std::fs::File` =
  write_all + flush + `self.sync_all()` — device-pinned before the drain
  reports Ok.
- `RingSink.spill: Option<Box<dyn SpillSync>>` (was `Option<File>`);
  `RingSink::new(Option<File>)` API UNCHANGED for all callers
  (sovereign.rs COMMIT_LEASE, chains-sovereign bench — both recompiled
  green).
- `drain_to_spill` routes through the seam; comment updated: rotation
  boundary power-fail durable; REMAINING WINDOW stated — undrained ring
  bytes are memory-only, staging-latch semantics retained and stated.
- sovereign.rs FA9 DURABILITY DISCLOSURE updated to match (doc block
  only; FA9FIX gate logic untouched).
- New court `fa9_rotation_drain_is_sync_durable_and_bytes_reach_the_spill`
  in `mod courts`: Part 1 real recorder (Rc<RefCell>) counts writes+syncs
  through the shared drain path, drained bytes == committed region;
  Part 2 real File spill in /tmp, reopened from disk, byte-identical.

### Anti-vacuity mutation court (sandbox /tmp/h3-sandbox, sha256-exact restore)

- BASELINE: sandbox lib run `cargo test --features std --lib fa9_rotation`
  exit 0.
- MUTANT: `self.sync_all()` removed from `impl SpillSync for File`
  (sandbox copy only). First attempt WITHOUT the structural arm: court
  still PASSED (recorder is a separate impl; cannot witness the File
  barrier) — vacuous, honestly marked and FIXED before landing: a
  structural arm over the production impl region was added
  (include_str! pattern, same as the file's MUT-FOOT-4 witness).
- MUTANT re-applied on the updated court: `cargo test` exit 101 —
  FAILED at "production SpillSync for File must issue sync_all". FLIP
  WITNESSED.
- RESTORE: file re-copied from the shared tree; sha256
  9a094d70f00727c570770a53f731e3c75cb2e21788f8442c034639e7cbed661f exact
  match both sides; sandbox re-run exit 0 after restore. Shared tree was
  never mutated.

## (3) Relabel verification

- sovereign.rs:753 (post-edit): "the correct label is 'staged transaction
  latch'" — landed in this lane's doc block.
- w6-fa9-commit-gating.md:45-47 (landed FA9FIX): "measure the in-memory
  STAGING LATCH, not a durable commit" — verified on disk.
- Sweep: `grep -r "durable commit"` over crates/ + docs/jira/v26.11.2:
  only the two honest disclosure sites above; zero remaining
  durability-overclaim sites. benches (chains-sovereign.rs): zero
  "durable" hits.

## Final verification ladder

- `cargo test -p cgap-ffi` exit 0 (full suite incl. fa9 3/3 legs).
- `cargo test -p cgap-parquet --features std` exit 0: 194 lib (includes
  the new fa9_rotation court) + 15 integration test-result-ok lines,
  0 failed anywhere.

## Divergences

- The original court design (recorder-count only) was VACUOUS against
  the production File impl mutation; fixed with the structural arm
  before landing (recorded above, not hidden).
- Sandbox Cargo.toml needed workspace-inheritance inlining + dev-dep
  drop for the standalone /tmp build; sandbox-only, not a tree change.
