# p2-receipts — ring-commit receipt chaining witness

Date: 2026-11-02. Lane p2-receipts, canonical checkout /Users/sac/cgap,
no git commands run. New module
`crates/cgap-ffi/src/receipt_chain.rs` (registered in lib.rs): a
per-transition BLAKE3 Merkle receipt chain over the W3-L4 ring commit
leg, bound to the P28 license verify. The C-ABI surface is UNCHANGED
(frozen four-verb boundary respected; the chain is a host-surface Rust
module, gated `#![cfg(not(target_os = "none"))]` so the XIP build keeps
its pre-existing dep set).

## Chain law

- Genesis prev-hash = all-zero digest (`GENESIS`).
- Receipt i: `hash = BLAKE3(prev_hash || sealed block bytes || license
  wire bytes)`; stored fields: index, block_digest (BLAKE3 of block
  bytes), license_digest, prev_hash, hash.
- License binding, both directions: `append` refuses an attestation
  failing `validate_intrinsic` (typed refusal, chain unchanged); the
  license wire bytes are a hash input, so a post-hoc license tamper
  flips `verify` even when intrinsically valid.
- One opaque refusal class (`ReceiptChainRefusal::Spoliation`) for
  every tamper (p28 §5 no-fingerprinting law); `EmptyChain` is a shape
  verdict, not tampering.
- The sealer's own `SealedBlock::digest` is the FNV-1a-64 content
  address (footer.rs:34-38), deliberately non-cryptographic; the chain
  records recomputed BLAKE3 instead (recorded in-code at append).

## Courts (all ran this session, exit 0)

`CARGO_TARGET_DIR=/tmp/p2-target cargo test -p cgap-ffi --lib` —
50 passed / 0 failed. Per court:

1. `chain_appends_and_verifies` — 4 REAL sealed blocks (real A02
   writer + real seal), genesis anchor, prev-hash linkage, verify OK.
2. `tampered_receipt_or_block_is_typed_refusal` — negative control
   (pristine verify OK, anti-vacuity) then mutation battery: flips in
   hash / block_digest / prev_hash / license_digest each -> exactly
   `Spoliation`; swapped block order -> `Spoliation`; post-hoc license
   tamper (intrinsic-valid but different bytes) -> `Spoliation`; bad
   magic -> `Spoliation`. Mutations run on in-memory copies only.
3. `invalid_license_refused_at_append` — bad magic and over-ceiling
   records refused at mint; chain length stays 0.
4. `empty_chain_verifies_as_empty` — EmptyChain, not Spoliation.
5. `lease_exhaustion_is_typed_refusal` — REAL `cgap_commit_parquet`:
   16-lane call commits (CGAP_OK), 70,000-lane call refuses exactly
   CGAP_ERR_LEASE_EXHAUSTED (0x43470004) at the 64 KiB lease ceiling.
   Landed `lever_courts` re-run: both l4_* courts pass (court B is the
   70k-lane lease-edge witness, court A the spoliation flip witness).

## Full C-ABI chain bench (divan, exit 0)

`CARGO_TARGET_DIR=/tmp/p2-target cargo bench -p bench --bench
chains-sovereign` — per-tx = stage time / batch:

| stage | @256 | @4096 |
|---|---|---|
| admit_frame | 0.83 us/tx | 0.83 us/tx |
| eval_bn3 | 3.75 us/tx | 3.75 us/tx |
| commit_parquet | 0.83 us/tx | 0.82 us/tx |
| verify_p28 | 0.92 us/tx | 0.91 us/tx |
| FULL CHAIN | 6.33 us/tx | 6.31 us/tx |

Commit leg 0.82-0.83 us/tx is consistent with the A3 receipt's
0.59-0.60 us/tx (same magnitude, run variance). **The end-to-end
<4 us/tx target is NOT met: measured 6.31-6.33 us/tx; the eval_bn3 leg
(3.75 us/tx) is the blocking lever.** Perf stays [TARGET-v26.11.2]
until the eval leg lands its reduction; commit leg is inside budget.

## Standing

PARTIAL. The receipt chain is ALIVE (courts + bench witnessed this
session, commands + exits above); the lane's <4 us/tx end-to-end
clause is BLOCKED by the eval_bn3 leg at 3.75 us/tx (budget-table row:
eval lever, v26.11.2 campaign). Pre-existing, not this lane's: 2
unused-import warnings in cgap-gate/penstock.rs; unused-dep warnings
from manifest patches; 1 in-flight cross-lane defect.

## Disclosed cross-lane compile unblock (SLA, ~10 min)

`crates/cgap-arrow/src/ipc.rs` was mid-edit by another lane (unclosed delimiter at the
`impl fmt::Display for IpcError` body, build-frozen all dependent
crates). Minimal unblock: added the one missing `}` (no semantic
change). Disclosed in THIS receipt per the fan-out law; owner lane
should review the surrounding in-flight edit.

---

```json
{"lane":"p2-receipts","files_written":["crates/cgap-ffi/src/receipt_chain.rs","crates/cgap-ffi/src/lib.rs","crates/cgap-arrow/src/ipc.rs (one-brace compile unblock, disclosed)","docs/jira/v26.11.2/waves/p2-receipts-ring-receipt-chain.md"],"commands":["cargo test -p cgap-ffi --lib (exit 0, 50 passed)","cargo test -p cgap-ffi (exit 0, all targets)","cargo test -p cgap-ffi --test lever_courts (exit 0, 2 passed)","cargo test -p cgap-arrow --lib (exit 0, 134 passed)","cargo build --workspace (exit 0)","cargo bench -p bench --bench chains-sovereign (exit 0, two runs)"],"courts_run":["chain_appends_and_verifies","tampered_receipt_or_block_is_typed_refusal","invalid_license_refused_at_append","empty_chain_verifies_as_empty","lease_exhaustion_is_typed_refusal","l4_committed_header_flip_is_exactly_spoliation","l4_status_flip_witness_at_lease_edge"],"standing":"PARTIAL","divergences":["<4 us/tx end-to-end NOT met: measured 6.31-6.33 us/tx; eval_bn3 leg 3.75 us/tx is the blocking lever (typed BLOCKED, budget row eval lever)","receipt-chain module is Rust-level host surface, NOT wired into cgap_commit_parquet (frozen four-verb ABI; wiring would add chain allocs past the zero-alloc commit courts)","append records recomputed BLAKE3, not the sealer FNV-1a digest (footer.rs:34-38 non-crypto content address)","cross-lane one-brace compile unblock in cgap-arrow/src/ipc.rs (disclosed, no semantic change)"]}
```
