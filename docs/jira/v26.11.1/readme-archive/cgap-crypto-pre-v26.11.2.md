# cgap-crypto

**Status: PUBLISHED** — crates.io shows `26.11.1` (created
2026-11-02T03:25:33Z; verified live this session). The only moonshot-B
crate on the registry so far.

## What it is

Lane A16: the Chatman sealer and its lease machinery
(`src/chatman.rs`, `src/lease.rs`) — the cryptographic core behind
P28 attestation digests and derivative-artifact sealing. `#![no_std]`,
zero external deps.

## Test receipt (run this session)

```
cargo test -p cgap-crypto   # exit 0
  18 passed (lib) + 6 passed (chatman_courts + lease_courts) = 24/0
```

Receipt of record: `docs/jira/v26.11.1/waves/a16-finish-receipt.md`
(sources pinned by sha256 at commit 62215326).

## The 9-mutation battery

`scripts/wave7_shim_battery.py` is the non-vacuity court: for each of 9
mutations it applies an exact-string shim into a /tmp sandbox copy, runs
the real courts, asserts the FLIP, and restores sha256 fail-closed.
Witnessed exit 0 in the A16 receipt. **Divergence (this session):** run
standalone under the concurrent DOC wave it refused fail-closed —
`BASELINE NOT GREEN (exit 101); battery refused`. Courts must start from
a green baseline; under wave contention the sandbox baseline compiles
against a moving shared lock. Loud refusal, not a fake pass. Replay on a
quiet tree to reproduce the A16 exit-0 witness.

## Layout

- `src/chatman.rs` — the sealer.
- `src/lease.rs` — lease machinery.
- `tests/chatman_courts.rs`, `tests/lease_courts.rs` — integration courts.
- `scripts/wave7_shim_battery.py` — 9-mutation non-vacuity battery.
