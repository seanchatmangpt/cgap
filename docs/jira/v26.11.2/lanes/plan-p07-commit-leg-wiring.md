# P-07 — Commit-Leg Receipt Wiring (PLAN)

PLAN (session-delivered, transcribed 2026-11-02; not yet executed).

Register row: RW-09-O4 (DEFECT-REGISTER.md:322). Finder:
`lanes/lane-ocel01-cgap-ocel2-adapter.md:99`; boundary statement
`lanes/lane-fix08-receipt-desync.md:31` ("COMMIT-LEG BOUNDARY,
RED-13 §3").

## 1. Seam

- Post-seal / pre-flush in
  `crates/cgap-ffi/src/sovereign.rs:811-819`: after
  `writer.seal(None)` returns the sealed block and BEFORE
  `sealed.flush_to(&mut lease.ring)` drains it to the ring sink.
  This is the only point where the commit-leg event exists in
  sealed form but has not yet left the lease.

## 2. Chain topology: per-thread, inside CommitLease

- A per-thread `RingReceiptChain` handle carried INSIDE the
  commit lease struct (granted once, reused per transaction —
  same steady-state-zero-alloc discipline as the 64 KiB block
  buffer / ring sink at sovereign.rs:824-829).
- REJECTED: a global/static chain. Ground: the existing
  `RingReceiptChain::verify` contract pairs receipts:blocks 1:1;
  a global chain multiplexes concurrent leases into one chain and
  breaks that 1:1 pairing. Per-thread handles keep the pairing.

## 3. FFI surface: opt-in license binding, 2 new exports

- `cgap_commit_bind_license`: binds the caller's P28 license
  grant to the lease's chain (path-argument grant reuse — the
  same grant-loading path the existing FFI surface uses).
- `cgap_commit_last_receipt`: returns the most recent receipt
  minted at commit to the C caller (the return path FIX-08
  declined to build).
- Rebind semantics: calling `bind_license` again RESETS the
  lease's chain (a new attestation epoch invalidates prior
  receipts).
- Opt-in law: an UNBOUND lease mints nothing. No binding, no
  commit-leg receipt, zero behavior change on the existing path.

## 4. Refusal

- A commit executed against a bound lease whose license check
  fails returns `CGAP_ERR_P28_SPOLIATION`
  (0x5032_38FF, trap.rs:36 pin) BEFORE `flush_to` — the spoliated
  block never reaches the ring.

## 5. Size / blast radius

- ~370 LOC across 9 files. Primary:
  `crates/cgap-ffi/src/sovereign.rs`,
  `crates/cgap-ffi/src/receipt_chain.rs`; consumers include the
  OCEL adapter projection. sovereign.rs is multi-lane-contested
  (fix08 disclosed the ownership boundary) — land as its own
  coherent diff, not a rider.

## 6. Court plan (5 courts)

- C1 baseline: bound commit mints a receipt; `chain.receipts()`
  gains one entry per commit (1:1 pairing holds). Mutation:
  remove `chain.append` from the seam -> court FAILs.
- C2 license-tamper: tampered grant -> `CGAP_ERR_P28_SPOLIATION`
  before flush_to, ring unchanged. Mutation: skip-tamper
  injection -> court must observe the refusal.
- C3 byte-tamper: flip one receipt byte in a /tmp sandbox chain;
  `verify` refuses. This leg IS the mutation (lib court).
- C4 unbound lease mints nothing: no bind -> no receipt; fa9
  path byte-identical. Mutation: wire the append
  unconditionally -> court FAILs.
- C5 rebind-resets-chain: second `bind_license` empties prior
  receipts. Mutation: remove the reset -> court FAILs.

- Mutations on /tmp sandbox copies; sha256-exact restore + mtime
  bump (a02-finish-receipt.md:170-184 mtime law).
- Blast-radius gates to re-run: fa9 7/7
  (`crates/cgap-ffi/tests/fa9_commit_gating.rs`), the receipt
  chain family, ocel2 courts, surface gate N -> N+2 (two new
  exports).
- Residual (disclosed, honest): unbound leases mint nothing —
  commit-leg non-repudiation remains opt-in, not default.

## 7. Standing

UNVERIFIED (plan transcribed, not executed). No command in this
file has been run against compiled code; all cites above re-read
on disk 2026-11-02 (sovereign.rs:811-819, :824-829; trap.rs:36
pin per CLAUDE.md status-code law).
