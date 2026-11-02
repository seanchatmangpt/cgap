# W6 — GITFIX lane receipt: cgap-git typed refusals

Date: 2026-11-02. Lane: GITFIX (GS4 §3+§4 quick-fixes). Repo: /Users/sac/cgap (main).

## Files written

- `crates/cgap-git/src/ladder.rs` — probe instrumentation cfg narrowed from
  `#[cfg(debug_assertions)]` to `#[cfg(all(debug_assertions, feature = "std"))]`
  at all 5 sites (probe module, `debug_probes` both arms, `debug_tick`,
  unrolled-pass tick at ladder.rs:142). Note: dispatch named one attribute;
  narrowing only the module would have orphaned 4 use sites (E0433
  `probe_count` unresolved), so all five were narrowed together.
- `crates/cgap-git/src/pack_write.rs` — `read_varint_header` now returns
  `Err(GitError::UnsupportedDeltaEncoding)` for type codes 6/7 (previously
  fail-open: delta payload consumed as zlib body). Types 0/5 keep
  `BadVarint`; 1-4 unchanged.
- `crates/cgap-git/src/lib.rs` — new `GitError::UnsupportedDeltaEncoding`
  variant + honest Display arm ("delta type code 6 (OFS_DELTA) or 7
  (REF_DELTA): ... would be misparsed as a zlib body — re-pack with only
  full (stored/zlib) objects").
- `crates/cgap-git/tests/delta_refusal_court.rs` — new Chicago court (4
  tests, real bytes in /tmp tempdir, crate's own `encode_object_header` /
  `zlib_compress` / `sha1` as collaborators; final-state asserts).
- `crates/cgap-git/tests/fuzz_codec.rs` — 1-line minimal unblock: added the
  new variant to `git_bucket`'s exhaustive match (E0004 otherwise; disclosed
  here as an in-lane fix to a file under my `tests/` ownership).

## Fix 1 — ladder.rs cfg narrowing

- std feature verified in `crates/cgap-git/Cargo.toml` (`default = ["std"]`,
  `std = []`).
- Verified (nightly toolchain — repo has no rust-std for riscv32imc on
  stable; workspace is nightly-active):
  - `cargo check -p cgap-git --target riscv32imc-unknown-none-elf
    --no-default-features` (debug): exit 0
  - same, `--release`: exit 0
- DIVERGENCE: the dispatch's "default features, DEBUG profile" case (std
  feature on, riscv32imc) cannot exit 0 for any ladder fix: `extern crate
  std` at lib.rs:13 is gated only on `feature = "std"`, which default
  enables, and riscv32imc ships no std. Pre-existing structural failure
  (E0463 at lib.rs:13), witnessed pre- and post-fix, outside this lane's
  lib.rs ownership scope ("error variant if needed" only). The actually
  failing case the spec described (no-default-features + debug,
  E0433 at ladder.rs:31) is fixed and flipped.

## Fix 2 — typed delta refusal

Pre-fix fail-open witness (HEAD pack_write.rs in /tmp/gitfix-sandbox
sandbox, sha256-vs-tree irrelevant — sandbox-only, shared tree untouched):

    test type6_ofs_delta_header_is_refused_typed ... FAILED
    thread '...' panicked at tests/delta_refusal_court.rs:57:37:
    type-6 must be refused, not parsed:
    [ParsedObject { obj_type: 6, size: 21, offset: 12, crc: 1033723598 }]
    test type7_ref_delta_header_is_refused_typed ... FAILED
    [ParsedObject { obj_type: 7, size: 20, offset: 12, crc: 3081789012 }]
    test result: FAILED. 2 passed; 2 failed; 0 ignored
    PREFIX-COURT-EXIT:101

Post-fix (shared tree, CARGO_TARGET_DIR=/tmp/gitfix-target):

    cargo test -p cgap-git → exit 0; delta_refusal_court: 4 passed.
    Full suite: 12 test binaries, all `test result: ok`, 0 failures.

## gate0 (criterion: identical before/after)

Identical pre-existing failure both runs — fails at the ggen sync step,
before any cargo compile:

    FM-PACK-008 pack `cgap-pack` content hash mismatch: ggen.lock has
    blake3:e085b8d1... vs disk blake3:22ba9f10...; make: *** [gate0] Error 1

Before-run exit captured as 0 through a `| tail` pipeline (capture
artifact); after-run exit 2; both runs print the identical FM-PACK-008
signature at 18:23 and 18:27 — identical-before/after satisfied by
signature, not exit-code pair.

## Dependents regression

`cargo check -p cgap-storage -p cgap -p cgap-beam`: exit 0 (new enum
variant is additive; no downstream match breakage found in libs —
fuzz_codec.rs was the only exhaustive match, fixed above).

## Commands (exits this session)

- make gate0 (pre-edit): FM-PACK-008, make Error 1 (18:23)
- make gate0 (post-edit): identical FM-PACK-008, exit 2 (18:27)
- cargo +nightly check -p cgap-git --target riscv32imc-unknown-none-elf
  --no-default-features: exit 0 (debug and --release)
- cargo +nightly test -p cgap-git: exit 0
- cargo +nightly check -p cgap-storage -p cgap -p cgap-beam: exit 0

## Standing: ALIVE

Both fixes landed, flipped (court cannot pass pre-fix; riscv no-default
debug cannot compile pre-fix), regression-green, dependents green.

## Divergences

1. Default-feature riscv32imc check remains structurally broken at
   lib.rs:13 (extern std vs no std target) — pre-existing, out of scope.
2. Working tree already carried pre-lane modifications in cgap-git
   (dag.rs, deflate.rs, inflate.rs, pack.rs, fuzz_codec.rs) from other
   lanes; untouched by this lane.
3. Other-lane sweep: `git status` at lane end shows cgap-git files beyond
   my five marked M — attributable to concurrent lanes, not this one.

## Cleanup

`/tmp/gitfix-target`, `/tmp/gitfix-sandbox-target`,
`/tmp/gitfix-sandbox` removed at lane end (python shutil.rmtree).
