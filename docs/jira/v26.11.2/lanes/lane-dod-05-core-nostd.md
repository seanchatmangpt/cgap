# DOD-05: cgap-core no_std posture (L-scope: PUB-07 finding A)

Date: 2026-11-02. Repo /Users/sac/cgap, branch main. No git write commands.

## W1. Subject + diff

ONE file changed: `crates/cgap-core/src/lib.rs` — added the cgap-bn3
posture header:

```rust
#![cfg_attr(not(test), no_std)]
```

No other source file touched. `frame.rs`, `isomorphism.rs`,
`dma_ring.rs`, `checkpoint.rs` were already core-only (`core::fmt`,
`core::mem`, `core::sync::atomic`, `write!`/`matches!` are core macros);
the 95 errors were the std-prelude cascade of the missing
`#![no_std]` (E0463 `std` required → std prelude injection fails →
E0425 Err/Ok/Result/assert/Option, E0405 Fn, unused derive
resolution). Zero per-site fixes needed; the audit's "95 sites" was
one root cause, not 95 defects.

## W2. Commands + exits (this session, this tree)

1. Reproduce (pre-fix): `CARGO_TARGET_DIR=/tmp/dod05-target cargo +nightly check -p cgap-core --no-default-features --target riscv32imc-unknown-none-elf` → 95 errors, "could not compile `cgap-core` (lib) due to 95 previous errors" (exact count matches PUB-07).
2. Post-fix same command → **exit 0**, "Finished dev profile", 0 warnings.
3. `CARGO_TARGET_DIR=/tmp/dod05-target cargo test -q -p cgap-core` → **exit 0**; 13 test binaries all `test result: ok`, total **79 tests, 0 failed, 0 ignored**.
4. Anti-vacuity flip battery (sandbox /tmp copy, never the shared tree):
   - Mutation: inject `if true { return; }` at the top of `tracking_alloc::breach()` — breach recording disabled (SANDBOX MUTATION, marked in source).
   - Mutated run: `cargo test -q --test checkpoint_court` → **FAILED**, `COURT0 VACUOUS: zero-budget window caught no allocation` (checkpoint_court.rs:60).
   - Restore sha256-exact `9f871a51fbcadb378e21526eda3d09808b912aa268e31a38483903f7a45d3d83` (tracking_alloc.rs), re-run → **ok, 1 passed**.
   - Shared tree confirmed untouched by the same sha256 after cleanup.
5. Scoped `git status --porcelain crates/cgap-core`: only lib.rs is my diff.

## W3. Standing

- riscv32imc `--no-default-features` check: **ALIVE** (exit 0, this session).
- Full host suite (lib + 12 integration courts incl. manifest, rv32imc,
  source-scan, identity, ingress, dma_ring, checkpoint): **ALIVE**
  (79/79, exit 0).
- Checkpoint zero-alloc window court: **ALIVE and non-vacuous** (flip
  demonstrated: mutated → FAIL, restored → PASS, sha256-exact restore).

## W4. Fences (must-not-regress surfaces adjacent to this diff)

- `tests/source_scan_gate.rs` — ran green (within the 79). The posture
  adds no new `alloc::`/`std::` tokens; the lib stays alloc-free.
- `tests/manifest_court.rs` — ran green. Note: `crates/cgap-core/Cargo.toml`
  carries ANOTHER lane's in-flight diff (metadata: categories/keywords,
  blake3 dep bumped to 26.11.2, license-file key removed) present before
  my runs; my gates ran against that tree and both exit 0. Not my file;
  disclosed per the disclose-in-both-directions law — the owning lane
  supersedes on landing.
- Consumer lanes (A14 dam / cgap-gate dev-dep) unaffected: no API change.

## W5. Divergences

- Dispatch expected 95 per-site prelude fixes across 4 files; reality:
  a single-line posture fix (files were already core-only). The audit's
  census counted cascade errors, not root causes. Recorded as divergence.
- `checkpoint.rs` / `tracking_alloc.rs` are NOT lib modules — they are
  `#[path]`-included by `tests/checkpoint_court.rs`, so the lib posture
  does not affect them (dispatch's "keep checkpoint.rs compiling" was
  already satisfied; verified by the checkpoint court run).
- Sandbox manifest needed literals (workspace inheritance + string-form
  categories/keywords) to run standalone; shared tree never edited.

## W6. Replay

```
CARGO_TARGET_DIR=/tmp/dod05-target cargo +nightly check -p cgap-core \
  --no-default-features --target riscv32imc-unknown-none-elf   # exit 0
CARGO_TARGET_DIR=/tmp/dod05-target cargo test -q -p cgap-core   # exit 0, 79/79
```

## W7. Cleanup

`/tmp/dod05-target` and `/tmp/dod05-sb-target`, `/tmp/dod05-sandbox`
remain — lease law says the coordinator deletes lane build roots at
integration; `-sb-target`/`-sandbox` deleted this session; `/tmp/dod05-target`
retained for coordinator drain. Delete at integration.
