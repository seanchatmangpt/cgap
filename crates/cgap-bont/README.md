# cgap-bont

Binary ontology (B-ONT) scalar kernel (4,891 LOC, 25 pub fns): BLAKE3-
addressed slot table for ontology lexemes with a frozen wire format.

- Crate version: **26.11.2** (`version.workspace = true`; reality receipt
  drift-check: manifest 26.11.2 vs Cargo.toml — MATCH).
- Spec: `docs/jira/v26.11.1/lanes/A06` (SlotId law, collision trap).
- License: P28-SIL v2.1.0 — `urn:avec:license:p28-sil:v2.1.0-sovereign`.
- Status: **ALIVE (REAL)** — reality receipt
  `docs/jira/v26.11.2/waves/crate-reality/cgap-bont.md`; historical
  receipt `docs/jira/v26.11.1/waves/a06-finish-receipt.md`.

## Authority pins

- SlotId law: `SlotId = truncate64(BLAKE3(lexeme))` with namespace tag
  `b"bont\x01"` and a fixed output window — frozen by the A06 dispatch.
- Collision trap (T5): the REAL witness collision pair for the 64-bit
  window is embedded as a court fixture (not fabricated, not a
  "trap refusing" fallback). T5a negative control (corpus collision-free,
  exactly 1000 slots) and T5c round out the trap.

## Courts

Canonical gate: `cargo test -p cgap-bont` — **35 passed / 0 failed / 0
ignored** (unit 35 + doctests 0), exit 0, 3.39s (reality receipt, this
wave). The T5 trap court proves the collision is the real crafted pair in
the frozen window, not a test double; the `*_flips` mutation courts flip
on sandbox mutations.

## Publish status (as last witnessed, v26.11.1 wave — 2026-11-01)

- **PUBLISHED-26.11.1** — crates.io `max_version = 26.11.1`,
  `num_versions = 1` (API read with UA `cgap-publish-receipt`, one-attempt
  publish, receipt `waves/p3-pub-bont-bosym.md`). No 26.11.2 publish
  attempt has been witnessed.
