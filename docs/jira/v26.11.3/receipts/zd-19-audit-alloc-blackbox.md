# ZD-19 — audit alloc_counter: black_box the scope-exclusion test's allocations

Date: 2026-11-04 · Repo: /Users/sac/cgap · Crate: `cgap-audit`
Status: ALIVE (all commands below run this session, exits shown)

## Defect (ZD-3 fix class)

`nested_inner_scope_excludes_outer_allocations` (crates/audit/src/alloc_counter.rs:188)
carried the same LLVM-heap-elision fragility that reded `error_paths` in release
(ZD-3): the outer scope's `_pad` allocation is observably unused, so LLVM may elide
the malloc/free pair and the outer count can hit zero, failing
`outer.count > inner.count`.

## Change (test-only, no lib behavior change)

- crates/audit/src/alloc_counter.rs:190 — wrapped the pad allocation in
  `std::hint::black_box(...)` so the pointer observably escapes and the
  malloc/free pair survives release optimization.

No other file touched. No git commands run.

## Verification (exact commands + exits)

All from /Users/sac/cgap:

| # | Command | Result |
|---|---|---|
| 1 | `cargo test -p cgap-audit --lib nested_inner_scope --release` (run 1) | ok, 1 passed / 0 failed, exit 0 |
| 2 | `cargo test -p cgap-audit --lib nested_inner_scope --release` (run 2) | ok, 1 passed / 0 failed, exit 0 |
| 3 | `cargo test -p cgap-audit --lib nested_inner_scope` (debug) | ok, 1 passed / 0 failed, exit 0 |
| 4 | `cargo test -p cgap-audit --lib --release` (full crate) | 37 passed / 0 failed, exit 0 |
| 5 | `cargo test -p cgap-audit --lib` (full crate, debug) | 37 passed / 0 failed, exit 0 |

The release profile is where the elision bites; two consecutive release runs
green post-fix.

## Standing

- ALIVE for post-fix green ×2 release + ×1 debug, full crate unregressed both
  profiles.
- UNVERIFIED (not reproduced this session): the pre-fix red in release. The
  failing baseline was not run before the edit landed; the defect claim comes
  from the ZD-3 flag, not a witnessed release failure on this exact subject.
- Anti-vacuity note: a future flip check (revert line 190, expect release red)
  was not executed — the black_box guard is asserted structurally, not
  mutation-tested here.
