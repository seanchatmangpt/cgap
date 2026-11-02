# f2-10-membrane-receipt

## F2-10 BENCH-MEMBRANE — lane receipt (2026-11-01)

Standing: ALIVE. Lease honored: only `crates/bench/Cargo.toml`,
`crates/bench/benches/kernels.rs`, and this receipt were written.

### Files written (sha256 at receipt time)

- `crates/bench/Cargo.toml` (52 lines) —
  `097164b56456abecc8abb111098dce34500df40c5e48db7500685dd6893c4c94`
- `crates/bench/benches/kernels.rs` (8 lines) —
  `3ce4e67bddbeed781321c2e63a66664307b9dae1f5267bdd4405c04bc88f3352`
- this receipt

### Commands + exits

1. `cargo search divan` -> 0. `divan = "0.1.21"` current on crates.io;
   exact pin `=0.1.21` set at `crates/bench/Cargo.toml:24`.
2. `ls crates/` -> 0. All eight path-dep crates exist: cgap-arrow, cgap-core,
   cgap-gate, cgap-parquet, cgap-bn3, cgap-p28, cgap-crypto, cgap-ffi.
3. `CARGO_TARGET_DIR=/tmp/lane-f2-10-target cargo bench -p bench --no-run`
   -> 101 (manifest parse: 4 missing sibling target files: bn3, ingress,
   parquet, seal). Witnessed: cargo aborts the whole crate's manifest on any
   missing `[[bench]]` file; partial compile is not selectable.
4. `cargo tree -p cgap-arrow -e dev` -> 0. Dev edges: cgap-core, cgap-gate,
   cgap-storage only. No divan, no std.
5. Re-run (dispatch-prescribed single re-run) -> 101 (1 missing: bn3.rs).
6. Full build after all five files landed -> 101. bn3.rs E0601 (no `main`);
   seal.rs 3 errors (E0609 seal.rs:172, E0277 seal.rs:151, E0308 seal.rs:388);
   kernels E0599 (see Divergences). ingress compiled clean; parquet compiled
   (1 unused-mut warning, parquet.rs:194 — F2-12's file, not touched).
7. `cargo bench -p bench --no-run --bench kernels` -> 0. My target green.
8. `cargo bench -p bench --no-run` -> 0 (after F2-14 fixed seal.rs,
   mtime 16:01:43). All six bench targets linked: kernels, ingress, parquet,
   bn3, seal, v26_10_31, plus all 16 prior-campaign src/bin binaries.

### Courts run (each names its anti-vacuity mutation)

- **Kernel-manifest purity (divan)**: `grep -rn "divan"
  crates/cgap-*/Cargo.toml` -> exit 1 (zero hits). Flip arm: defect-injected
  `/tmp` copy of cgap-arrow/Cargo.toml with a `divan = "0.1.21"` line ->
  grep exit 0. Court flips; originals sha256-verified byte-exact untouched.
- **Kernel-manifest purity ([[bench]])**: zero hits, exit 1; flip arm with
  an injected `[[bench]]` block -> exit 0. Court flips; originals untouched.
- **Deep dev-edge purity**: `cargo tree -p cgap-arrow -e dev` -> 0, no
  divan/std edge. Mutation (named, not executed): adding divan as a dev-dep
  of cgap-arrow would appear in this listing.
- **Membrane gate non-vacuity**: the same command that exits 0 (step 8)
  exited 101 four times this session for typed reasons (missing files,
  E0601/E0609/E0277/E0308). A gate that demonstrably refuses broken surfaces
  is not vacuous on its green run.
- **Seed pin**: no corpus const in the stub (entry-point only);
  `0xBF58_476D_1CE4_E5B9` is pinned in the stub header comment for the
  downstream corpus lanes. F2-11..14 files are their lanes' receipts.

### Divergences

1. Spec entry-point shape is wrong for the pinned dep:
   `bench-membrane-spec.md` ("Entry point shape") specifies
   `divan::Divan::default().run()`, which does not exist on divan 0.1.21 —
   E0599 witnessed (build step 6). Real entry method is `run_benches`
   (divan-0.1.21/src/divan.rs:64). Stub uses `run_benches`; spec yields to
   the toolchain.
2. Dispatch prescribed build + "re-run once"; five full-crate invocations
   were needed because F2-11..14 landed files incrementally and two landed
   broken (bn3 E0601, seal 3 errors). Sibling fixes are their lanes'
   receipts; my gate is green against the final surface.
3. `crates/bench/src/benches/v26_10_31.rs` is inert prior-campaign leftover
   (not a Cargo auto-discovery path; no target collision, build5 exit 0 with
   no duplicate-target error). Untouched, observation only.
