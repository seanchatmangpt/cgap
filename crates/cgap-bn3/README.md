# cgap-bn3

Binary N3 (BN3) reasoning kernel (9,354 LOC / 71 pub fns): sigil-coded
triple store with a fixed-capacity deductive arena — deterministic,
zero heap at steady state.

- Crate version: **26.11.2** (`version.workspace = true`; equals root
  workspace version 26.11.2 — reality receipt drift-check True).
- Spec: `docs/jira/v26.11.1/lanes/A11` (24B statement pin, fixed arena).
- License: P28-SIL v2.1.0 — `urn:avec:license:p28-sil:v2.1.0-sovereign`.
- Status: **ALIVE (REAL)** — reality receipt
  `docs/jira/v26.11.2/waves/crate-reality/cgap-bn3.md`; historical
  receipt `docs/jira/v26.11.1/waves/a11-finish-receipt.md`.

## Authority pins

- 24-byte `repr(C)` `BN3Statement` — ABI frozen wave 1; `size_court_statement_is_24_bytes`
  plus const asserts hold it (`2*24+16 = 64` cache-line tiling court green).
- Fixed-capacity deductive arena: `Arena::with_capacity(statements, formulas)`
  refuses above the compile-time max, typed; `Arena::new()` remains the
  growable dynamic arena; one flatten path serves both.
- Canonical tree never shimmed: arena/sigil/unify sha256-unchanged across
  the flip battery.

## Courts

Canonical gate: `cargo test -p cgap-bn3` — **126 passed / 0 failed / 0
skipped**, exit 0 (unittests 119 + `tests/lever_courts.rs` 7, including
`l3_antivacuity_probe_drop_flips`; reality receipt, this wave; compile
clean — the W3-L3 kill is closed). Sandbox battery witnessed in the
a11 receipt: the 24-byte pin mutation aborts via size court, the
arena-bound mutation refuses typed above the compile-time max.

## Publish status (as last witnessed, v26.11.1 wave — 2026-11-01)

- Manifest guard `publish = true` (guard flip P3-2b, receipt
  `waves/p3-pub-guard-flip.md`).
- crates.io: **NOT YET PUBLISHED** (API HTTP 404 at read time;
  p3-pub-bn3-gate.md holds the gate receipt). First-version budget
  intact.
- This line states witnessed state, not aspiration.
