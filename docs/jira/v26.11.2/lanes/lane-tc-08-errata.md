# lane-tc-08-errata

TC-08 errata-append lane (2026-11-02). Spec of record:
audit/cull-08-doc-retirement.md (10-row action table + TPS-1 5.70 ns
addendum). ERRATA APPENDS ONLY; no history rewritten; no git commands.

## Files appended (dated ERRATA/TOMBSTONE/RIDER blocks)

1. docs/REVENUE_GENERATION_COMPENDIUM.md — TWO erratas: (a) Sigma_PQC
   at :103 (INVENTED LORE, citing cgapprd-02:47-57,70-81); (b) 5.70 ns
   penstock at :59,:237,:288 FAILED re-derivation (citing
   audit/tps-01-reality-ledger.md:21,28: manual 5.70 FAILED; re-derived
   fastest 7.35 / median 8.13 / receipted 7.71; 175.5M fps derivative
   flagged same defect).
2. docs/jira/v26.11.2/12-ard-prd-cgap.md:19 — Sigma_PQC errata citing
   PRD-2.
3. docs/jira/v26.11.2/paper/position-paper-v3.md:231,233 — Sigma_PQC
   errata (paper's own TARGET qualifier noted as already honest).
4. docs/jira/v26.10.31/07-chain-economy-8020.md:36 — TOMBSTONE
   redirect to 12-ard-prd-cgap.md + cgapprd-02:66+.
5. LICENSE-P28-SIL.md:69 — ERRATA RIDER (legal text; rider governs on
   divergence), citing PRD-2.
6. lanes/lane-b05-cscb-c6.md:21,47 — 800M-as-operative-gate errata,
   citing lane-mv-05-sla-reconcile.md:44 + audit/release-blockers.md:233
   (decided gate 150M full-cascade min-of-7; 800M = ASPIRATIONAL_FPS).
7. lanes/lane-git-11-retract.md:90 — LOW-priority re-scope pointer
   errata (prior framing receipt-true as-of-writing; predates MV-5).
8. lanes/lane-mv-05-sla-reconcile.md:44 + audit/release-blockers.md:237
   — VERIFIED, NO APPEND: "p99" does not appear at either cited line
   (mv-05 C6 block states min/median/max; release-blockers R1 append
   same). grep sweep: 'p99' in mv-05 only at :39 (C3, genuine quantile
   wording) and in release-blockers only at :268 (same C3 run). Per
   dispatch law (append only if "p99" actually appears): loud no-action.
9. audit/dod-csct-cscb.md:35 — "one timing flake" C2 errata: FAIL 3/3
   direct runs under load (doctrine-commercial-proofs.md:29) AND a
   fresh quiet-host PASS re-derived this session: C2 p99 5.00 ns, exit
   0, release, batch64-amortized-instant disclosed in [META]
   (command: cd crates/csct && CARGO_TARGET_DIR=/tmp/tc08-target cargo
   build --release --bin cscb_c1_c2_ingress, then
   ./target/release/cscb_c1_c2_ingress > /tmp/tc08-c2-direct.log; exit
   0). Honest row = variance across host-load states, not a flake.

## Commands

- C2 fresh witness: exit 0 (log /tmp/tc08-c2-direct.log, quoted in 9).
- grep sweeps below (Verify section), all run this session.

## Verify (grep sweep of false claims)

Remaining-hit law: hits must be inside errata/qualification contexts.
- 'Sigma_PQC' remaining hits: compendium :103 (original line, now
  followed by Errata 1), 12-ard :19 (+errata), paper :231/233 (+errata),
  8020 :36 (+tombstone), LICENSE :69 (+rider), A16 (honest UNKNOWN),
  claims-appendix :37 (TARGET-qualified), cgapprd-02 itself — all now
  errata-qualified. [PENDING final sweep below]
- '5.70' remaining hits in compendium: :59,:237,:288 originals, each
  superseded by Errata 2. tps-01:21,28 are the REFUTING receipt
  (expected).
- '800M': b05 (:21,:47 originals + errata; :45-46 are the honest
  FAIL-vs-800M report, retained), git-11 :90 (+errata), mv-05/release-
  blockers ASPIRATIONAL_FPS contexts (receipt-true), doctrine :87.
- 'p99' at mv-05:44 / release-blockers:237: ZERO hits (verified, row 8).

## Standing

ALIVE (per-file appends verified on disk; grep sweep run this session;
C2 fresh PASS witnessed exit 0). Divergence: row 8 resolved as
verified-no-action rather than append, per dispatch's own conditional.
