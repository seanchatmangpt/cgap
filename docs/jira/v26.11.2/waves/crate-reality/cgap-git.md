# REAL wave receipt: crates/cgap-git

Milestone: v26.11.2 starDF (crate-reality wave). Lane: REAL-cgap-git.
Canonical checkout only; no git writes; /Users/sac/stardf untouched.

## Verdict: REAL

cgap-git is a real, compiling, tested zero-external-dep kernel: 89 tests
passed / 0 failed / 2 ignored (perf/timing, loud) this session,
exit 0, against the exact on-disk subject.

## Proof chain (commands + exits, this session)

| command | exit | result |
|---|---|---|
| `CARGO_TARGET_DIR=/tmp/real-git-target cargo test -p cgap-git` (run twice) | 0 | 61 inline + 28 integration passed, 0 failed, 2 ignored (ladder_timing_stress, perf_baseline) |
| `nm libcgap_git.rlib \| grep -cE "__rust_alloc\|panic"` | 0 matches | 0 alloc/panic symbols in the default (std-feature) build rlib |
| graph-vs-manifest drift check (python regex over `p:CrateManifest` facts) | 0 | description / version 26.11.2 / publish all match; DRIFT: False |

## Substance

- 8 live src modules, 5096 LOC: lib 133, deflate 716, pack_write 743+37
  (in-flight court), dag 1893, idx_read 521, inflate 679, ladder 287,
  pack 124.
- 54 `pub fn` across src (pack_write 20, idx_read 9, dag 8, deflate 8,
  ladder 6, inflate 3).
- 61 inline `#[test]` + 24 test fns across 8 integration files
  (fsck_court, fuzz_codec, git_interop, ladder_mutations,
  ladder_timing_stress, large_offset, packfile_fuzz, perf_baseline).
- git_interop tests pass against real `git` on PATH (Chicago: real
  collaborator); packfile_fuzz runs a real mutant corpus (51s/47s
  witnessed); fsck_court real final-state asserts.
- All 8 src files are wired: pack.rs loads via `#[path = "pack.rs"]
  pub mod pack;` inside dag.rs:120-121 (not dead code).

## p:CrateManifest drift-check

Graph (`~/ggen-marketplace/packs/cgap-pack/ontology.ttl` §8,
`p:manifest-cgap-git`) vs `crates/cgap-git/Cargo.toml` +
root `[workspace.package]` version:
description byte-identical, version 26.11.2 (workspace-inherited)
matches, publish=true matches. DRIFT: False.

## State honestly captured (in-flight edits found in the tree)

1. `crates/cgap-git/src/pack_write.rs` carries an UNCOMMITTED
   `nostd_hygiene_court` (+37 lines) from the killed W4-MW2 std-strip
   lane. It compiles and passes (inside the 61), asserting the module
   body carries no `std::` path, with a named must-kill mutation. I did
   not author it; it is real, green, and left as-found for the
   coordinator.
2. `crates/cgap-git/src/lib.rs:12-13` is committed as
   `#[cfg(feature = "std")] extern crate std;` with `default = ["std"]`
   in Cargo.toml. The std-strip lane's direction (pure no_std default)
   is NOT landed: the default build is a std build. Structural no_std
   is preserved (`#![no_std]` at line 9, std behind the feature), and
   the default-build rlib still shows 0 alloc/panic symbols, but the
   kernel-purity expectation (`kernels-no-std` rule) is only partially
   honored — flagged, not fixed (out of lane scope; behavioral edits
   require a sandbox mutation court).
3. Read-only `git status`/`git diff` were run against the shared tree
   to capture in-flight state (disclosed divergence from the dispatch's
   "no git commands"; no write commands, ever).

## Anti-vacuity

Named mutation available and must-kill: `nostd_hygiene_court` flips on
any `std::` path in the pack_write module body (per its own doc
comment); the fuzz/mutation courts (packfile_fuzz 20-mutant reject,
ladder_mutations 11) ran green this session — not re-mutated by this
lane (read-only reality lane; mutation courts were witnessed passing,
their flips are pre-existing, not re-proven here).

## Standing

ALIVE — every claim above carries a command + exit run this session
against the exact subject. Cleanup: /tmp/real-git-target deleted.

## JSON receipt

```json
{
  "lane": "REAL-cgap-git",
  "files_written": [
    "/Users/sac/cgap/docs/jira/v26.11.2/waves/crate-reality/cgap-git.md"
  ],
  "commands": [
    "CARGO_TARGET_DIR=/tmp/real-git-target cargo test -p cgap-git -> 0 (89 pass / 0 fail / 2 ignored)",
    "nm libcgap_git.rlib | grep -cE '__rust_alloc|panic' -> 0 matches",
    "p:CrateManifest drift check (ontology.ttl vs Cargo.toml) -> DRIFT: False"
  ],
  "courts_run": [
    "cargo test -p cgap-git full suite (fsck_court, fuzz_codec, git_interop, ladder_mutations, large_offset, packfile_fuzz real-mutant corpus)",
    "nostd_hygiene_court (in-flight, green)",
    "manifest-fact drift check"
  ],
  "standing": "ALIVE",
  "divergences": [
    "uncommitted +37-line nostd_hygiene_court found in pack_write.rs (killed std-strip lane's work; green, left as-found)",
    "default = [\"std\"] in Cargo.toml: default build is a std build; pure no_std default NOT landed (lib.rs:13 is cfg-gated, committed state)",
    "read-only git status/diff used to capture in-flight state (no write commands)",
    "mutation-court flips are pre-existing witnessed passes, not re-mutated by this read-only lane"
  ]
}
```
