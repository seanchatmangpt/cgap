# cgap-bont

Binary ontology (B-ONT) scalar kernel: BLAKE3-addressed slot table for
ontology lexemes with a frozen wire format.

- Spec: `docs/jira/v26.11.1/lanes/A06` (SlotId law, collision trap).
- License: P28-SIL v2.1.0 — `urn:avec:license:p28-sil:v2.1.0-sovereign`.
- Status: **ALIVE** — receipt
  `docs/jira/v26.11.1/lanes/a06-finish-receipt.md` is mirrored at
  `docs/jira/v26.11.1/waves/a06-finish-receipt.md`.

## Authority pins

- SlotId law: `SlotId = truncate64(BLAKE3(lexeme))` with namespace tag
  `b"bont\x01"` and a fixed output window — frozen by the A06 dispatch.
- Collision trap (T5): the REAL witness collision pair for the 64-bit
  window is embedded as a court fixture (not fabricated, not a
  "trap refusing" fallback). T5a negative control (corpus collision-free,
  exactly 1000 slots) and T5c round out the trap.

## Courts

Canonical gate: `cargo test -p cgap-bont` -> **35 passed / 0 failed**
(a06-finish-receipt.md). The T5 trap court proves the collision is the
real crafted pair in the frozen window, not a test double.

## Publish status (witnessed 2026-11-01)

- **PUBLISHED-26.11.1** — crates.io `max_version = 26.11.1`,
  `num_versions = 1` (API read with UA `cgap-publish-receipt`, one-attempt
  publish, receipt `waves/p3-pub-bont-bosym.md`).
