# cgap-storage

WAL-framed transactional storage engine: `atomic_commit.rs` two-slot
atomic commit with power-cut safety across the full erase/program
surface (1,336 LOC: `atomic_commit.rs` 1,035, `dred.rs` 298).

- Crate version: **26.11.2** (`version.workspace = true`; workspace
  `[workspace.package]` version — reality receipt drift-check MATCH).
- Spec: `docs/jira/v26.11.1/lanes/A05` (WAL framing; power-cut sweep).
- License: P28-SIL v2.1.0 — `urn:avec:license:p28-sil:v2.1.0-sovereign`.
- Status: **ALIVE** — reality receipt
  `docs/jira/v26.11.2/waves/crate-reality/cgap-storage.md`; historical
  receipt `docs/jira/v26.11.1/waves/a05-finish-receipt.md`.

## Authority pins

- Two-slot atomic commit (C0-C4 commit protocol, I1-I3 invariants).
- Zero torn writes under power cut at ANY erase/program offset.

## Courts

Canonical gate: `cargo test -p cgap-storage` — **24 passed / 0 failed**
across 6 suites, zero ignored (unittests 5; court_crc_identity 3;
court_faults 5; court_flash 3; court_mutations 6; court_sweep 2).
Reality-receipt census: 24/0, zero compile-aborts.

- `sweep_per_offset_rollback`: 113 cut offsets x per-offset rollback sweep
  (t1) — every cut lands in a typed recovery state, no torn k.
- `sweep_both_slots`: 113 offsets over both slots (t2 / F2, divergence D2).
- `cut_during_erase_sweep`: 2x113 erase offsets on a standing seq-2 state.
- Anti-vacuity: the court set flips on real sandbox mutations of the
  commit path (see a05-finish-receipt.md for the named flips).

## Publish status (as last witnessed, v26.11.1 wave — 2026-11-01)

- Manifest guard `publish = true` (guard flip P3-2b, receipt
  `waves/p3-pub-guard-flip.md`).
- crates.io: **NOT YET PUBLISHED** (API HTTP 404 at read time). Publish chain
  gated on dependency order (p3-pub-core-storage.md verdict
  UNSUPPORTED-structural at its read time; first-version budget intact).
- This line states witnessed state, not aspiration.
