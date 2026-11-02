# cgap-crypto

**Status: PUBLISHED-26.11.1 (as last witnessed, v26.11.1 wave)** —
crates.io showed `26.11.1` (created 2026-11-02T03:25:33Z). No 26.11.2
publish attempt has been witnessed.

## What it is

Lane A16: the Chatman sealer and its lease machinery
(`src/chatman.rs` 724 LOC, `src/lease.rs` 188 LOC — 1,823 LOC total) —
the cryptographic core behind P28 attestation digests and
derivative-artifact sealing. `#![no_std]`, zero external deps.

- Crate version: **26.11.2** (`version.workspace = true`; workspace
  26.11.2 — reality receipt drift-check yes).
- Status: **ALIVE (REAL)** — reality receipt
  `docs/jira/v26.11.2/waves/crate-reality/cgap-crypto.md`; historical
  receipt `docs/jira/v26.11.1/waves/a16-finish-receipt.md`
  (sources pinned by sha256 at commit 62215326).

## Test receipt (v26.11.2 reality wave)

```
cargo test -p cgap-crypto   # exit 0
  1 (lib) + 18 (chatman_courts) + 6 (lease_courts) = 25 passed; 0 failed
```

## The 9-mutation battery

`scripts/wave7_shim_battery.py` is the non-vacuity court: for each of 9
mutations it applies an exact-string shim into a /tmp sandbox copy, runs
the real courts, asserts the FLIP, and restores sha256 fail-closed.
Witnessed exit 0 in the A16 receipt. **Divergence (recorded in the
v26.11.1 wave):** run standalone under the concurrent DOC wave it refused
fail-closed — `BASELINE NOT GREEN (exit 101); battery refused`. Courts
must start from a green baseline; under wave contention the sandbox
baseline compiles against a moving shared lock. Loud refusal, not a fake
pass. Replay on a quiet tree to reproduce the A16 exit-0 witness.

## Layout

- `src/chatman.rs` — the sealer.
- `src/lease.rs` — lease machinery.
- `tests/chatman_courts.rs`, `tests/lease_courts.rs` — integration courts.
- `scripts/wave7_shim_battery.py` — 9-mutation non-vacuity battery.
