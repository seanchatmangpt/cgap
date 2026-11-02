# cgap-bosym

Binary symbol archive kernel (`slab.rs`): typed parse of the BOSYM
container with an ingress limiter guarding the parser.

- Spec: `docs/jira/v26.11.1/lanes/A07` (ingress limiter, diode courts).
- License: P28-SIL v2.1.0 — `urn:avec:license:p28-sil:v2.1.0-sovereign`.
- Status: **ALIVE** — receipt
  `docs/jira/v26.11.1/waves/a07-finish-receipt.md`.

## Authority pins

- Ingress limiter (`slab.rs`): bounds `Vec::with_capacity` and all
  parser allocations BEFORE parse; rejects oversized/malformed archives
  with the parser's own typed errors (`Truncated`/`BadMagic`/...). The
  limiter is a diode: what it refuses can never reach the parser.
- Limiter-first ordering inside `parse_archive` — frozen by A07.

## Courts

Canonical gate: `cargo test -p cgap-bosym` -> **14 passed / 0 failed**
with the ingress limiter wired in (a07-finish-receipt.md). Diode courts
prove the limiter ADMITS everything the plain parser admits and refuses
oversized inputs as the identical typed error; the named mutation
("limiter ADMITTED forbidden-flags — must drop") flips the court.

## Publish status (witnessed 2026-11-01)
- **PUBLISHED-26.11.1** — crates.io `max_version = 26.11.1`,
  `num_versions = 1` (API read with UA `cgap-publish-receipt`, one-attempt
  publish, receipt `waves/p3-pub-bont-bosym.md`).
