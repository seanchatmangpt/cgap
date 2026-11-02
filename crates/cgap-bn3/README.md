# cgap-bn3

Binary N3 (BN3) reasoning kernel: sigil-coded triple store with a
fixed-capacity deductive arena — deterministic, zero heap at
steady state.

- Spec: `docs/jira/v26.11.1/lanes/A11` (24B statement pin, fixed arena).
- License: P28-SIL v2.1.0 — `urn:avec:license:p28-sil:v2.1.0-sovereign`.
- Status: **ALIVE** — receipt
  `docs/jira/v26.11.1/waves/a11-finish-receipt.md`.

## Authority pins

- 24-byte `repr(C)` `BN3Statement` — ABI frozen wave 1; `size_court_statement_is_24_bytes`
  plus const asserts hold it (`2*24+16 = 64` cache-line tiling court green).
- Fixed-capacity deductive arena: `Arena::with_capacity(statements, formulas)`
  refuses above the compile-time max, typed; `Arena::new()` remains the
  growable dynamic arena; one flatten path serves both.
- Canonical tree never shimmed: arena/sigil/unify sha256-unchanged across
  the flip battery.

## Courts

Canonical gate: `cargo test -p cgap-bn3` -> **111 passed / 0 failed / 0
ignored, exit 0** (102 pre-existing + 5 new fixed-arena courts;
a11-finish-receipt.md). Sandbox battery: 2/2 flips exit 0 — the dispatch
mutation on the 24-byte pin aborts via size court, the arena-bound
mutation refuses typed above the compile-time max.

## Publish status (witnessed 2026-11-01)

- Manifest guard `publish = true` (guard flip P3-2b, receipt
  `waves/p3-pub-guard-flip.md`).
- crates.io: **NOT YET PUBLISHED** (API HTTP 404 at read time;
  p3-pub-bn3-gate.md holds the gate receipt). First-version budget
  intact.
- This line states witnessed state, not aspiration.
