# PUB-07: no_std matrix — riscv32imc-unknown-none-elf (fresh, nightly)

Date: 2026-11-02. Method: for each crate,
`CARGO_TARGET_DIR=/tmp/pub07b-target cargo +nightly check -p <crate>
--no-default-features --target riscv32imc-unknown-none-elf`
(fresh target dir; floating nightly per rust-toolchain.toml).
`--no-default-features` exercises the pure no_std posture (all
crates that expose a `std` feature keep it opt-in).

## Matrix

| Crate         | Result  | Notes                                          |
|---------------|---------|------------------------------------------------|
| cgap-blake3   | PASS    | clean, 0 warnings                              |
| cgap-core     | FAIL    | 95 errors — no `#![no_std]`; see A             |
| cgap-bn3      | PASS    | clean; `cfg_attr(not(test), no_std)` posture   |
| cgap-branchless | PASS  | clean, 0 warnings                              |
| cgap-parquet  | PASS    | clean, 0 warnings                              |
| cgap-git      | FAIL    | 4 errors, lib-mounted whole; see B             |
| cgap-p28      | PASS    | clean, 0 warnings                              |
| cgap-gate     | PASS    | clean, 0 warnings                              |
| cgap-court    | VACUOUS | no lib target; see C                           |
| cgap-crypto   | PASS    | clean, 0 warnings                              |

Verdict: 7/9 real lib checks pass; 2 wire-in breakages (cgap-core,
cgap-git); 1 vacuous check (cgap-court).

## A. cgap-core — not no_std at all (95 errors)

`crates/cgap-core/src/lib.rs` has NO `#![no_std]`. The riscv build
fails at the front door:

    error[E0463]: can't find crate for `std`
      = note: `std` is required by `cgap_core` because it does not
        declare `#![no_std]`

Without the std prelude the crate collapses (95 errors across 4
files: lib-referenced frame.rs, isomorphism.rs, dma_ring.rs,
checkpoint.rs):

- isomorphism.rs: 27x E0425 `Err`, 12x `Ok`, 13x `Result`,
  3x `Option`, 3x `Some`, 4x `None`, 22x "cannot find macro
  `assert`", 1x `write`, 1x `matches`, 6x "cannot find attribute
  `derive`"
- frame.rs:324: E0405 `cannot find trait Fn` —
  `digest: impl Fn(&[u8]) -> [u8; 32]`
- dma_ring.rs, checkpoint.rs: prelude-import resolution errors

Census of the 95: 27 Err, 12 Ok, 13 Result, 22 assert, 10
Option/Some/None, 6 derive, 1 Fn, 1 write, 1 matches, 1 E0463,
1 prelude resolution — every error is std-prelude reliance.

Fix shape: adopt the cgap-bn3 posture (`#![cfg_attr(not(test),
no_std)]` + `extern crate alloc;` + explicit `core::` prelude
imports). Migration, not a one-liner: 4 files, 95 sites.

## B. cgap-git — lib-mounted wire-in breakage (4 errors)

Confirmed: cgap-git's modules were audited per-lane but NEVER
checked as a lib-mounted whole on riscv32imc. The lib check
exposes exactly 4 errors, all in ref_store.rs:

    error[E0425]: cannot find type `AtomicU64` in module
      `core::sync::atomic`
      --> crates/cgap-git/src/ref_store.rs:96:34
    error[E0425]: cannot find type `AtomicU64` in module
      `core::sync::atomic`
      --> crates/cgap-git/src/ref_store.rs:99:32
    error[E0433]: cannot find `AtomicU64` in `atomic`
      --> crates/cgap-git/src/ref_store.rs:107:42
    error[E0433]: cannot find `AtomicU64` in `atomic`
      --> crates/cgap-git/src/ref_store.rs:109:40

Root cause: riscv32imc provides no 64-bit atomics (largest is
AtomicU16 — rustc itself suggests `AtomicU16` as nearest alias),
so `core::sync::atomic::AtomicU64` does not exist on this target.
The per-lane audits never mounted all modules into the lib target
together on riscv, so this slipped through. Fix shape: gate the
`counter`/`chain` fields on
`#[cfg(target_has_atomic = "64")]` with a U16/U32 fallback or a
critical-section spinlock, or adopt `portable-atomic`.

Non-fatal warnings (cgap-git only):
- src/mcp_driver.rs:36: inner `#![no_std]` on a non-root module —
  "the `#![no_std]` attribute can only be used at the crate root"
  (unused_attributes; has no effect)
- src/mcp_driver.rs:42: unused import `alloc::vec::Vec`

## C. cgap-court — vacuous check (no lib target)

`crates/cgap-court/` has no `src/` at all. It is a court-only
crate: one `[[test]]` target (`court_verdict`, path
`../../tests/court_verdict.rs`) plus dev-deps only. `cargo check -p
cgap-court --no-default-features --target riscv32imc-...` finishes
in 0.01s with exit 0 — nothing is compiled, so exit 0 proves
nothing. Its dev-deps additionally force `std`
(`cgap-parquet features = ["std"]`), so the court cannot run on
riscv regardless. No no_std claim is testable for this crate; the
"zero-external-dependency no_std" line in its Cargo.toml comment
describes its DEPENDENCY TREE, not the crate itself.

## Lease note

No lease file was found on disk (searched docs/jira/**/audit and
docs/jira/ roots for *lease*): nothing to delete. The scratch
target dir /tmp/pub07b-target remains (outside the repo,
self-reclaiming).

## Reproduce

    for c in cgap-blake3 cgap-core cgap-bn3 cgap-branchless \
      cgap-parquet cgap-git cgap-p28 cgap-gate cgap-crypto \
      cgap-court; do
      CARGO_TARGET_DIR=/tmp/pub07b-target cargo +nightly check -q \
        -p $c --no-default-features \
        --target riscv32imc-unknown-none-elf; echo "$c $?"; done
