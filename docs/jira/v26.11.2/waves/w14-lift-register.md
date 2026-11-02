# w14-lift-register — publish-lift work register (missing-author backfill)

C10/F11 flagged: the W14 lift register was cited by release/SKILL.md,
C9 rules, and w-commitplan-v2 but never authored. This file IS the
register. Grounded in publish-halt.md:31-42 (readiness matrix + owed
repairs), repair-10 (header parity CLOSED), repair-13 (Cpk measured),
and the C-lane hook upgrades (publish-guard.sh enforced).

## Standing at authoring (2026-11-02)

- 12 crates LIVE at 26.11.2 (p4-terminus Table 3; publish-halt says 8
  pre-halt — Table 3's x8 API witness supersedes).
- FROZEN at 26.11.1: audit, cloud, ffi, fibo, gpu, wasm-footprint.
- ABSENT: cgap-integration (first publish pending, dry-run READY).
- Registry drift accrued this session: bn3 no_std flip (republish
  candidate), parquet RunOverrun rows bound (republish candidate),
  ffi ABI +0x4347_0006 + enclave_128k.ld artifact (republish
  candidate), blake3 D1 NEON UB fix in tree (republish candidate),
  arrow D2 stale generated_version (republish candidate).
- include/cgap.h parity: CLOSED by repair-10 (0x4347_0006 + M13 corpus).
- Publish-guard hook: ENFORCED (C4/C8; unconditional freeze, marker
  bypass removed).

## Lift sequence (operator order required)

(a) cgap-parquet 26.11.3 — RunOverrun rows bound (H5) + RingSink
    (D8) + stats/prune machinery — then cgap-ffi unfreezes.
(b) cgap-ffi 26.11.x — +0x4347_0006 ABI, enclave_128k.ld shipped,
    spill_gate, receipt-chain (D8 unblocked by (a)).
(c) cgap-bn3 26.11.3 — no_std flip + radix default + SplitStore.
(d) cgap-blake3 26.11.3 — D1 NEON UB fix in tree.
(e) cgap-arrow 26.11.3 — D2 generated_version + validate ladder.
(f) cgap-integration FIRST publish (never on registry; dry-run READY).
(g) READY set sequenced: crypto, p28, gpu, audit, fibo, cloud
    (dry-run READY per publish-halt matrix); wasm-footprint LAST
    (D9 host-link diagnosis owed).
Each tier publishes only after the prior tier's verification exits 0;
ONE attempt per crate; stop-on-slide + 1 hour backoff.

## Still owed before (a)

- wasm-footprint D9 diagnosis (new defect class).
- Cpk CTQ-06 MEASURED-FAIL (drift): re-run after load < 2 or accept.
- F-lane final integration commit (w-commitplan-v2 sequence).
