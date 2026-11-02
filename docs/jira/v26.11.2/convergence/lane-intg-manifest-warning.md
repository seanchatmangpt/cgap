# Lane INTG — cgap-integration unused-dep manifest warning (W15/W6-b7)

2026-11-02. Owner file: `crates/integration/Cargo.toml`.

## Diagnosis

`cgap-arrow` was declared in BOTH `[dependencies]` and `[dev-dependencies]`.
All consumption is in `tests/` (`cross_crate.rs`, `admission_e2e.rs`,
`membranes.rs`); `src/lib.rs` mentions `cgap-arrow` only in a doc comment.
So the `[dependencies]` entry was genuinely unused — integration tests link
the dev-dep — and the warning was NOT stale.

## Fix

Removed the `[dependencies] cgap-arrow = "26.11.1"` line (replaced with a
comment noting the test-only consumption). Dev-dep unchanged, so publish
surface is unaffected (dev-deps are never compiled by `cargo publish`
verify).

## Witnesses

Before (shared tree, this session):
```
warning: unused dependency `cgap-arrow`
  --> crates/integration/Cargo.toml:11:1
warning: `cgap-integration` (manifest) generated 1 warning
```

After:
- `cargo check -p cgap-integration` → exit 0, ZERO warnings
  ("Finished `dev` profile ... in 0.04s", no warning lines).
- `cargo test -p cgap-integration` → exit 0; all suites passed incl.
  cross_crate/membranes/admission_e2e (tail witness: 4 tests in the last
  suite, `test result: ok`, 0 failed; doc-tests 0 failed).

Usage grep receipt:
`grep -rn cgap_arrow crates/integration/src tests` → 12 hits, ALL in
`tests/` (cross_crate.rs:11,12,286,292; admission_e2e.rs:20;
membranes.rs:30-36); src/lib.rs hit is a `//!` doc comment only.

## Standing

ALIVE (commands + exits run this session, exact subject).

## Divergences

None. The dispatch's "if a test consumes it, investigate wrong-section"
branch resolved to: dep declared in the WRONG SECTION SET — it was in
both; the normal-dep copy was the redundant one and was removed.
