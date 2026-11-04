# ZD-3: alloc_report_scopes_and_error_path red (XA-2 census red #5)

Standing: ALIVE (green release + debug)

Date: 2026-11-04 · Lane: zd-3 · Status: ALIVE (green release + debug)

## Subject

`alloc_report_scopes_and_error_path` (crates/integration/tests/error_paths.rs:605) —
deterministic red: "inner scope lost its alloc: AllocReport { count: 0, bytes: 0 }".

## Root cause

TEST bug, not an AllocReport bug. The tracker
(crates/audit/src/alloc_counter.rs:127 `with_alloc_tracking`) is correct: per-thread
`Cell<Tracking>` with prev-state folding across nested scopes — nesting logic verified
sound by reading, and by the tracker's own unit tests.

The failure is LLVM heap-elision in release, profile-gated (debug green, release red,
hence "deterministic ×3"): the inner scope's `vec![1u8; 8]` has no observable use
beyond `.len()` (derivable without allocating), so LLVM removes the malloc/free pair
entirely — the inner scope legitimately records zero allocations. After fixing that
with `black_box`, the outer's `_pad = vec![0u8; 4096]` (dropped unused, same
malloc/free pair elimination) was the next elided allocation: `outer { count: 1 } vs
inner { count: 1 }` at line 640.

## Fix (forward, test-side only)

crates/integration/tests/error_paths.rs — wrap both allocations in
`std::hint::black_box` so their pointers observably escape and the allocations must
happen while tracking is active:

- line 636: `with_alloc_tracking(|| std::hint::black_box(vec![1u8; 8]))`
- line 636 pad: `let _pad = std::hint::black_box(vec![0u8; 4096]);`

Comments in-file explain the elision mechanism so the next reader does not re-diagnose.

## Verification (commands + exits, this session)

- `cargo test -p cgap-integration --test error_paths --release alloc_report` → ok, 1 passed (was FAILED at 636, then at 640 pre-fix)
- `cargo test -p cgap-integration --test error_paths alloc_report` → ok, 1 passed (debug)
- Full suite unregressed:
  - `cargo test -p cgap-integration --release` → 0+2+6+8+4 passed, 0 failed (lib, admission_e2e, cross_crate, error_paths, membranes)
  - `cargo test -p cgap-integration` (debug) → 2+6+8+4 passed, 0 failed

## Falsifier / non-vacuity

Pre-fix release run flipped the test at line 636, then at 640 after the first
black_box — the court flips under the mutation class (removing a black_box would
re-expose elision). Debug profile was already green pre-fix, confirming profile
gating, not tracker state.

## Out-of-scope note (not my file)

crates/audit/src/alloc_counter.rs:188-199 unit test `nested_inner_scope_excludes_outer_allocations`
carries the identical latent fragility (un-boxed `vec![0u8; 4096]` pad and `vec![1u8; 8]`
inner); it will fail identically if audit's tests are ever run in release. Left for the
audit-crate owner.
