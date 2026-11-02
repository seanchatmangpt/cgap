# w6-b5-radix — lane B5 finish receipt (2026-11-02)

Files: `crates/cgap-branchless/src/radix.rs` (ONLY lane file; + this receipt).

## Kernel

`radix_bucket_12(key: u64) -> usize` = mask-first replica of shipped
`radix_bucket` (`crates/cgap-bn3/src/unify.rs:728-731`, sha256
`d662efd924e50fcfcaadbf694fbdd25d03330a4b594a464736989c723835c785`,
pinned 2026-11-02): `y = key & 0x00FF_FFFF_FFFF_FFFF; (y ^ (y>>16)) & 0x0FFF`.
`RADIX_BUCKETS=4096` (4096 x 4 B heads = 16 KiB, asserted).

## Dep choice (recorded per dispatch)

Path dev-dep on cgap-bn3 REJECTED as infeasible: `radix_bucket` is a
**private** fn in unify.rs — a `[dev-dependencies]` path dep compiles but
cannot name the fn. Fallback per contract: exact copy + sha256-pinned
comment; differential court compares replica vs an independently restated
`shipped_semantics` in the test (unify.rs:729-730 verbatim), 100k-word
corpus on seed `0xA20C_0FFE_E5A7_E000`, plus all-zero/all-ones and all
8 sigil-bit edges (bits 56-63 proven non-perturbing).

## Commands + exits (this session, subject = exact shared-tree bytes)

- Sandbox court over shared tree via `#[path]` module include:
  `cd /tmp/b5-sandbox && CARGO_TARGET_DIR=/tmp/b5-target cargo test`
  → **8 passed; 0 failed, exit 0**.
- Shim-court M-add (xor→wrapping_add, 4 sites): run → **FAILED (2 tests
  flip)**; restore sha256-exact `f5324f58…b08ed`, RESTORE_OK.
- Shim-court M-shift (16→13, 4 sites): run → **FAILED (1 test flips)**;
  restore sha256-exact, RESTORE_OK.
- Post-restore baseline re-run → 8 passed, exit 0; shared tree sha256
  `f5324f589bb6bb19d1cf95dbdb3c7b5284f2f8607d48dff7ab0e1eb2484b08ed`.
- In-repo gate `cargo test -p cgap-branchless radix` → error "package ID
  `cgap-branchless` did not match any packages" — the crate is NOT yet a
  root-workspace member (root Cargo.toml off-limits to this lane).

## Anti-vacuity finding (retired mutation)

Mutation M1 (drop PAYLOAD_WINDOW_MASK) CANNOT flip: the 12-bit fold reads
only bits 0..=27, so sigil bits 56-63 are inert even unmasked — the mask
is belt-and-braces for THIS fold (load-bearing in the shipped kernel for
the wider probe path). A court that cannot flip is FAIL → M1 retired,
replaced by flippable M-add and M-shift (both witnessed flipping above).
Finding documented in radix.rs module docs.

## Divergences

1. cgap-branchless not yet a workspace member → the dispatch's literal
   gate command cannot resolve the package; gate run against the exact
   shared-tree file bytes via /tmp `#[path]` sandbox instead. Re-run
   `cargo test -p cgap-branchless radix` after B1 lands membership.
2. Mid-lane collision: B1's naive stub overwrote this file mid-flight;
   final file keeps B1's two tests (they hold under the mask-first
   convention; verified by the passing run) around the B5 kernel.
3. M1 court retired per cannot-flip law (above), not silently skipped.

## Standing

**ALIVE** for the kernel + courts (commands + exits above, this session).
**BLOCKED-on-B1** for the in-repo `-p cgap-branchless` gate (workspace
membership).
