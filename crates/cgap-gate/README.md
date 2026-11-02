# cgap-gate

**Status: BROKEN-MID-EDIT (v26.11.2 reality wave, 2026-11-01)** —
`cargo test -p cgap-gate` aborts at compile: 22 errors in the lib-test
target (`Vec`/`String`/`ToString`/unsafe-impl class), 0 tests executed.
Per census law, compile-abort is not a failure count. Workspace-local
moonshot crate; consumed in-tree by `cgap-court`, `cgap-ffi`, and the
wave receipts under `docs/jira/v26.11.1/waves/`. Reality receipt:
`docs/jira/v26.11.2/waves/crate-reality/cgap-gate.md`.

## What it is

The dam/penstock ingress gate (lane A14): a `#![no_std]`, zero-alloc
admission instrument in `src/penstock.rs` that classifies every CgapFrame
(128-byte header, lane A04) as accept or reject before any downstream
memory is touched. The dam analogy is load-bearing, not decorative —
penstock = the single ingress pipe, dam = the wall of static asserts and
courts that must hold before a frame passes.

## Substance census (v26.11.2 reality wave)

`penstock.rs` 1,869 LOC + `lib.rs` 13 LOC = 1,882 LOC; 2 pub fns in
`penstock.rs` (plus the unsafe `GlobalAlloc` impl surface); 12 `#[test]`
in `penstock.rs`, all blocked at compile. Manifest drift-check: NO DRIFT
(crateName/crateVersion 26.11.2/description/publish all match).

## Test receipt (historical, v26.11.1 wave — as-of-date, not current)

```
cargo test -p cgap-gate   # exit 0
  11 passed; 0 failed  (unit tests in src/penstock.rs; no tests/ dir)
```

Receipt of record: `docs/jira/v26.11.1/waves/a14-finish-receipt.md`
(command-exit table; `penstock.rs` untouched all session).

## The corpus and the zero-alloc law

- **10M-frame corpus** (`§8.1`, deterministic SplitMix64 seed, line 763):
  the exit gate is the 10M reject + zero-alloc court — 10,000,000 frames,
  **0 misclassified, 0 allocations** across the whole corpus.
- Zero-alloc is witnessed **per-call**, not process-wide. The original
  process-counter witness false-FAILED under a concurrent test's
  allocation inside a process-wide bracket; the fix (R3b) reads
  always-on per-thread allocator hooks plus an in-court positive control
  (a deliberate allocation MUST move the counter — the court that kills
  the vacuous-witness mutation).
- Mutation battery (named in-file): magic-removal FLIP (5 courts),
  allocation-injection FLIP (exactly the two zero-alloc courts),
  order-shuffle FLIP (order-pin court). A court that cannot flip is FAIL.

## Layout

- `src/lib.rs` — crate root, `#![no_std]`.
- `src/penstock.rs` — the gate: struct/const definitions with size/align
  static asserts (§6), byte-offset table (§6.1), flags (§6.3), SplitMix64
  corpus (§8.1), and the unit battery (12 `#[test]` at the 26.11.2
  census).
