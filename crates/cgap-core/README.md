# cgap-core

Zero-dependency, `no_std` byte-layout kernel: the canonical 128-byte
`CgapFrameHeader` (`frame.rs`), DMA ring (`dma_ring.rs`), and the
triumvirate isomorphism harness (`isomorphism.rs`).

- Crate version: **26.11.2** (`version.workspace = true`; root
  `Cargo.toml:42` `version = "26.11.2"`).
- Spec: `docs/jira/v26.11.1/lanes/A04-triumvirate-isomorphism.md` (§6 is the
  normative `CgapFrameHeader` layout, referenced byte-for-byte by lanes A14,
  A17, A19).
- License: P28-SIL v2.1.0 — `urn:avec:license:p28-sil:v2.1.0-sovereign`
  (no SPDX identifiers).
- Status: **ALIVE (PARTIAL — one pre-existing failing court)** — reality
  receipt `docs/jira/v26.11.2/waves/crate-reality/cgap-core.md`;
  historical receipt
  `docs/jira/v26.11.1/waves/a04-finish-receipt.md`.

## Authority pins

- `FRAME_HEADER_LEN == 128` — compile-time const pin
  (`frame.rs`), backed by `static_assert` suites: mutating the constant to
  129 aborts compilation (`E0080`) before any test runs.
- `CgapFrameHeader::project(&[u8; 128])` — typed projection of a frame slot;
  misaligned ingress is refused typed (`FrameError::Misaligned`).
- P28-SIL license URN byte-frozen (rename-receipt decision D2).

## Courts

Canonical gate: `cargo test -p cgap-core` — **63 collected, 62 passed /
1 failed** (the `manifest_court` failure is pre-existing and
session-observed; matches W4-P3, not a compile abort). All other suites
green: unittests 17, cross_lane_gate_court 8, guard_flip_court 11,
rv32imc_court 2, source_scan_gate 5, static_assert_court 4,
unit_dma_ring 13, and the rest — full census in the reality receipt.

Anti-vacuity: the dispatch-named mutation `FRAME_HEADER_LEN 128 -> 129` in a
/tmp sandbox copy flipped to compile-abort exit 101 with the exact layout
assertion; canonical tree sha256-unchanged.

## Publish status (as last witnessed, v26.11.1 wave — 2026-11-01)

- Manifest guard `publish = true` (guard flip P3-2b, receipt
  `waves/p3-pub-guard-flip.md`).
- crates.io: **NOT YET PUBLISHED** (API HTTP 404 at read time). Publish chain
  was gated on dependency order (p3-pub-core-storage.md verdict
  UNSUPPORTED-structural at its read time; first-version budget intact).
- This line states witnessed state, not aspiration.
