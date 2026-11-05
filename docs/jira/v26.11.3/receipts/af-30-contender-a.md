# AF-30 Receipt — Cage Match Contender A harness (cgap-aoq)
Standing: ALIVE

Lane: AF-30 (implementation lane; comparative report hosted in arcus:
`/Users/sac/arcus/docs/jira/v26.11.3/receipts/af-30-cage-match.md`).
Date: 2026-11-03. Subject: `/Users/sac/cgap`. No git operations run.

## What landed

- `crates/cgap-aoq/tests/af30_cage_match.rs` — Contender A runner:
  1M frames/round × 3 rounds, per frame = AB-6 `codec::decode` + AB-4
  `seal_detached` (real kernel via `cgap-crypto` dev-dep) + AB-7
  `AoqChain::append` (Mix32 digest seam, disclosed — not BLAKE3), plus
  a counting-global-allocator census over an 8192-frame window
  asserting 0 allocs / 0 bytes.
- Receipt rows in arcus's `docs/jira/v26.11.3/RECEIPTS.md` (comparative
  report row) and this repo's README receipts table (below).

## Exit records (this session, exact subject)

- Run 1: `cargo test -p cgap-aoq --test af30_cage_match --release -- --nocapture` → exit 0, 1 passed / 0 failed, `best_us_per_frame=0.5630`, census 0 allocs / 0 bytes.
- Run 2: same command → exit 0, 1 passed / 0 failed, `best_us_measured=0.5551` µs/frame, census 0 allocs / 0 bytes.
- AF-14 self-scan: `cargo test -p cgap-aoq --test cbmqs_runner` → exit 0, verdict PASS (G3 mock scan clean over src/, G1 positive control live, G7 BLAKE3 seal `b4766185e2efd769…` verified).

## Measured (release, Apple M3 Max, this session)

- Per-frame pipeline (decode + AEAD seal + chain append): 0.5630 /
  0.5551 µs/frame (min-of-3 rounds per run; two green runs).
- Alloc census: 0 allocs / 0 bytes per frame — the 0-alloc claim is
  witnessed by this session's census, not cited.
- Divergence disclosed: the digest seam is Mix32 (AB-7 landed-bench
  seam), not BLAKE3 (AF-26's substrate). The AB-7 0.291µs/link receipt
  used the same Mix32-family seam.

## Standing

ALIVE (commands + exits above, this session, exact subject).
