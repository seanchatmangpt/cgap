# Repair-09: F7 Compiler Warnings Sweep (CL2 census + W10 findings)

Standing: ALIVE. Date: 2026-11-02. Lane: F7.
Build root lease: `/tmp/f7-target` (deleted at lane end).

## Baseline (this session, before edits)

`CARGO_TARGET_DIR=/tmp/f7-target cargo build --workspace` — 9 build
warnings: 5 in-code (below) + 4 manifest-level unused-dependency warnings
(`crates/bench/Cargo.toml`: cgap-ffi/git/storage; `crates/integration/
Cargo.toml`: cgap-arrow). Manifest warnings are NOT in F7 ownership and
were left untouched.

In-code baseline (all eliminated):

- `crates/cgap-branchless/src/swar.rs:22,25` — deprecated `use core::u64;`
  + dead `LO` const.
- `crates/cgap-gate/src/penstock.rs:208-209` — unused `String`/`ToString`
  /`Vec` imports at module scope (test-only consumers).
- `crates/cgap/src/pipeline.rs:65-67` — dead fields `hash`/`pack_write`.
- Test-side: unused `SIGIL_SHIFT` (self_play_arrow_radix_join.rs:37),
  unused `f` (branchless_census.rs:434), unused `CompiledRule`
  (cgap-bn3/tests/lever_courts.rs:211).

## Fixes (7 files edited)

1. `crates/cgap-branchless/src/swar.rs` — removed `use core::u64;` and the
   dead `LO` const. No behavior change.
2. `crates/cgap-gate/src/penstock.rs` — moved the `alloc::string::{String,
   ToString}` / `alloc::vec::Vec` imports from module scope into
   `#[cfg(test)] mod courts` (only consumers are tests at :1114+).
   NOTE: first attempt deleted them outright and broke the cgap-gate
   lib-test compile (17 E0425/E0433/E0599 errors); repaired by cfg-scoped
   relocation within the same ~10-min SLA window. Final compile clean.
3. `crates/cgap/src/pipeline.rs` — `hash`/`pack_write` fields RETAINED and
   converted to read API: new `hash_fn()`/`pack_write_fn()` accessors
   mirroring the existing `ingress_fn()` surface. Grep of `crates/cgap/**`
   found no external readers and no serde derives on the struct (the
   struct carries no derive attributes at all). Added court
   `pack_write_wiring_is_kernel_entry` (anti-vacuity mutation: replacing
   the seam with any other fn pointer flips `core::ptr::fn_addr_eq`);
   extended `blake3_wiring_matches_kernel` to call through `pip.hash_fn()`
   (the field was previously never invoked through the wiring).
4. `crates/cgap/tests/self_play_arrow_radix_join.rs` — removed `SIGIL_SHIFT`
   from the sigil import group (remaining use at lever_courts.rs:127 is
   fully-qualified, unaffected).
5. `crates/cgap-branchless/tests/branchless_census.rs:434` — loop binding
   `f` -> `_f`.
6. `crates/cgap-bn3/tests/lever_courts.rs:211` — removed `CompiledRule`
   from the unify import group.
7. `crates/cgap-bn3/src/unify.rs` — (a) module header lines 1-16 rewritten
   from stale Wave-2 in-progress prose to landed state per W10;
   (b) `FixpointHasher` doc re-framed past-tense as frozen artifact
   (W6-BN3NS seen-set moved to BTreeSet; seed lineage pinned to
   0xBF58_476D_1CE4_E5B9); (c) removed redundant
   `use std::collections::BTreeSet` from the `#[cfg(test)] mod tests`
   block (alloc `BTreeSet` via `use super::*` is the one in force).

## After (this session)

- `cargo build --workspace` — exit 0, ZERO in-code warnings; only the 4
  out-of-ownership manifest unused-dependency warnings remain.
- `cargo test -p cgap-branchless -p cgap-gate -p cgap -p cgap-bn3` — exit 0;
  all 23 targets ok (cgap 32 tests, cgap-bn3 130 incl. 2 ignored,
  branchless 35, gate 15). No behavior change.
  - TRANSIENT, disclosed: one intermediate run aborted with a stack
    overflow in `alloc_boundary` while lane F1's in-flight edit to
    `crates/cgap-core/src/tracking_alloc.rs` (included via `#[path]` into
    that test binary) was landing — file mtime 11:30, failure observed
    11:31, re-run after F1 settled: exit 0. Not an F7 regression; F1's
    file is EXCLUDED from F7 ownership by dispatch.
  - RESIDUAL, not F7-owned: `crates/cgap-branchless/src/eytzinger.rs:140`
    unused import `format` (lib-test cfg). Left for the owning lane.

## Divergences from dispatch

1. `crates/cgap/tests/alloc_boundary.rs:34` — dispatch named an unused
   `na_unify_radix` import. NO such import exists in the current tree
   (file uses fully-qualified `cgap_bn3::noalloc::na_unify_radix` at
   :254/:260); the warning does not reproduce, so the file was left
   unedited — same stale-census class as the SP9 finding the dispatch
   already declared stale for self_play_radix_dred_cascade.rs. Verified:
   final `cargo test` compile shows zero warnings for cgap test targets.
2. First penstock.rs edit broke the lib-test compile (deleted imports
   that `#[cfg(test)]` code used); repaired in-session by relocating the
   imports into the test module. Disclosed here per the disclosure law.
