# cgap-core

Zero-dependency, `no_std` byte-layout kernel: the canonical 128-byte
`CgapFrameHeader` (`frame.rs`), DMA ring (`dma_ring.rs`), and the
triumvirate isomorphism harness (`isomorphism.rs`).

- Spec: `docs/jira/v26.11.1/lanes/A04-triumvirate-isomorphism.md` (§6 is the
  normative `CgapFrameHeader` layout, referenced byte-for-byte by lanes A14,
  A17, A19).
- License: P28-SIL v2.1.0 — `urn:avec:license:p28-sil:v2.1.0-sovereign`
  (no SPDX identifiers).
- Status: **ALIVE** — receipt
  `docs/jira/v26.11.1/waves/a04-finish-receipt.md`.

## Authority pins

- `FRAME_HEADER_LEN == 128` — compile-time const pin
  (`frame.rs`), backed by `static_assert` suites: mutating the constant to
  129 aborts compilation (`E0080`) before any test runs.
- `CgapFrameHeader::project(&[u8; 128])` — typed projection of a frame slot;
  misaligned ingress is refused typed (`FrameError::Misaligned`).
- P28-SIL license URN byte-frozen (rename-receipt decision D2).

## Courts

Canonical gate: `cargo test -p cgap-core` -> exit 0, **75 passed / 0 failed**
across 11 suites (a04-finish-receipt.md; baseline pre-finish was 69/10).
Anti-vacuity: the dispatch-named mutation `FRAME_HEADER_LEN 128 -> 129` in a
/tmp sandbox copy flipped to compile-abort exit 101 with the exact layout
assertion; canonical tree sha256-unchanged.

## Publish status (witnessed 2026-11-01)

- Manifest guard `publish = true` (guard flip P3-2b, receipt
  `waves/p3-pub-guard-flip.md`).
- crates.io: **NOT YET PUBLISHED** (API HTTP 404 at read time). Publish chain
  was gated on dependency order (p3-pub-core-storage.md verdict
  UNSUPPORTED-structural at its read time; first-version budget intact).
- This line states witnessed state, not aspiration.
