# Lane B06 — CSCB-C7 receipt sealing (v26.11.2)

Date: 2026-11-02. Repo: /Users/sac/cgap, branch main. No git write commands run.

## Files written (owned)

- `crates/csct/src/bin/cscb_c7_receipt_sealing.rs` (218 lines on disk)
- `docs/jira/v26.11.2/lanes/lane-b06-cscb-c7.md` (this receipt)

## Framing decision (per RFC)

The cgap P28 lease layout (`cgap-crypto/src/lease.rs`) is not directly importable as a
byte frame from the csct crate (type is private to that crate's lease module), so the
80-byte framing is defined per the lane RFC: nonce u64 LE + parent digest 32B +
payload 32B (splitmix64 from nonce, canonical multiplier seed
`0xBF58_476D_1CE4_E5B9` used as splitmix64 multiplier, never conflated with the
corpus seed) + tag u64 LE `0xC5CB_C700_0000_0080`. Documented in the bin header.
Hashing uses the real `cgap_blake3::hash` — no reimplementation.

## Commands + exits (this session)

- `cd crates/csct && CARGO_TARGET_DIR=/tmp/b06-target cargo build --release --bin cscb_c7_receipt_sealing` → exit 0
- `/tmp/b06-target/release/cscb_c7_receipt_sealing` (clean, 3 runs) → exit 0 each:
  median 4,377,052 / 4,356,280 / 4,351,961 receipts/s; min 4.25M/s; p99 ~4.4M/s
- `/tmp/b06-target/release/cscb_c7_receipt_sealing` under machine contention →
  exit 1, typed `[C7-FAIL] rate median 727,755/s < floor 1,000,000/s` (gate flips
  when the box is loaded — non-vacuity of the rate gate itself witnessed)
- `CSCB_MUTATE=1 .../cscb_c7_receipt_sealing` (release) → exit 0:
  `[C7-PASS] anti-vacuity: mutated certificate rejected`
- Debug build run → typed `[C7-DEBUG]` phase lines witnessed; typed
  `[C7-FAIL] rate median 204,511/s < floor` → debug_exit=1 (expected: unoptimized
  build below the floor; typed, loud, not silent).

## Chain-validity court (real state assert)

Full post-run verification of all 1,000,000 receipts: nonces strictly increment
from 1, every parent equals the previous digest (genesis parent = zero digest),
every stored digest re-derives via `cgap_blake3::hash`. PASSES every clean run.

## Anti-vacuity

`CSCB_MUTATE=1` flips byte 40 bit 0 (payload region) of the nonce-N+1 certificate
before hashing; the chain court must reject. Two flip layers proven: (1) bitflip
changes the digest (identical-digest = typed FAIL); (2) mutated certificate
ACCEPTED = typed FAIL. Mutation court flips (exit 0, PASS witnessed this session).
The court also self-fails if the bitflip lands back on the same bit value — the
land check compares full frames, not the bit value.

## Standing

ALIVE for the release court: command+exit run this session, median 4.35M receipts/s
>= 1M floor (3.9-4.4x headroom uncontended), mutation court flips. The contended
run (exit 1) and debug run (exit 1) are honest typed failures — the gate is
load-sensitive; rate claims are for the uncontended release build on this box.

## Divergences

- Dispatch first named `docs/jira/v26.11.2/ledger.md` then corrected in-line to the
  lanes/ receipt path — this file is the receipt of record.
- Debug build does not clear the 1M/s floor (204k/s) — by design of the court; the
  gate is a release-profile gate. No Process.sleep used; timing via `Instant`.
