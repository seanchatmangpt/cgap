# repair-17: bitplane court stack overflow (structural fix)

Lane STK, 2026-11-02. Codifies H7's environmental finding; replaces C6's
disclosed SLA unblock (64 MB explicit-stack thread) in
`crates/cgap-storage/tests/court_bitplane_seen.rs`.

## Root cause

`BitPlaneSeenSet<16384>` is `128 planes x 16384 slots x 8 B = 16 MB`
inline. In debug builds the 16 MB value is stack-staged by the caller
before the move into `Box::new(BitPlaneSeenSet::new())` would take
effect — so boxing at the call site does NOT fix the overflow. The
original court staged it as a plain local on the default 2 MB test
thread: SIGABRT, witnessed by H7 and C6.

## Fix

- `crates/cgap-storage/src/bitplane_seen.rs`: new `new_boxed()`
  constructor — `alloc_zeroed(Layout::new::<Self>())` + `Box::from_raw`.
  The 16 MB is allocated directly on the heap; no stack staging at any
  point.
- `crates/cgap-storage/tests/court_bitplane_seen.rs`:
  `court_membership_matches_btreeset_oracle_10k` now uses
  `let mut seen: Box<BitPlaneSeenSet<16_384>> = BitPlaneSeenSet::new_boxed();`
  and runs on the DEFAULT test-thread stack; the 64 MB thread scaffold
  (C6 SLA unblock) removed.

## Verification (this session)

- Default-stack green run (no RUST_MIN_STACK):
  `CARGO_TARGET_DIR=/tmp/stk-target cargo test -p cgap-storage --test court_bitplane_seen`
  exit 0 — 3 passed / 0 failed (membership court 241 s; `contains` is
  O(slots) x O(planes) in debug, expected).
- Anti-vacuity flip: /tmp sandbox copy reverted to inline
  `BitPlaneSeenSet::new()` -> `has overflowed its stack ... SIGABRT
  (signal 6)`. Court flips; not vacuous. Shared tree untouched
  (sandbox sha256-exact before mutation; restore verified by mutation
  never leaving /tmp).
- Build-root leases `/tmp/stk-target`, `/tmp/stk-sandbox-target`,
  `/tmp/stk-sandbox` deleted.

## Standing

ALIVE — command + exit run this session against the exact subject.
