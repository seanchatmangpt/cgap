# tc-10-open-recensus

TC-10 OPEN re-census, post ~60-lane landing wave (2026-11-02,
read-only). Read fresh: release-blockers.md, audit-ledger.md,
workstream-receipt-map.md, doctrine-ship-plan.md §D, tps-08-shipgate
plan, DEFECT-REGISTER.md, the cited lane receipts.

## 1. Rows now CLOSED (closing receipt cited)

| row | evidence | closed by |
|---|---|---|
| R1 C6/C3 gates | release-blockers:230-247; lane-mv-05:26-36 (c6 exit 0 min 211.2M f/s vs 150M; c3 exit 0 min 36,918 ms vs 60,000; mutants exit 1) | MV-5 DECIDED re-scope; 800M/25 ms ASPIRATIONAL, never gating |
| R2 parquet default-feature abort | release-blockers:214-216; lane-fix02:24-36 (default 0; std 195/0) | FIX-02 per-item std gates |
| R8 build debris | release-blockers:217-221; lane-gap-11:13-20 | GAP-11 |
| R9 member-count stale | release-blockers:222-224,:145; lane-mv-08 | MV-8 |
| R11 stale commit plan | release-blockers:185 (252 fresh entries) | MV-8 |
| R6 census-race fix | release-blockers:99-104; branchless_census.rs:55-60 | carried |
| P9 PRD-as-spec | doctrine-ship-plan:135 (P9 row); cgapprd-prd-v2.md; lane-cgap-prd-10:8-21 | PRD-10 |

## 2. Rows still OPEN

- R3 coverage floor: lane-gap-06:3-5 (PARTIAL, honest red; floor
  NOT lowered). Close: each script >= 90% branch or operator
  exception receipts.
- R4 G2-PARETO (now measured-but-failing): lane-gap-07:50-58 —
  L1 MET, L2 NOT MET 2.31x, L4@4096 NOT MET 9.35 us/tx, L5 NOT
  MET + fused 29% REGRESSION; :90 L1 flip VACUOUS; :14-17
  divan2receipt.py REFUSED chains-* (exit 1). L3 CLOSED
  (repair-06). Close: repair to threshold with flips, or operator
  gate-change receipts + divan2receipt successor + L1 flip court.
- R5 lint census: old 445 stands; GAP-05 added DOC-RECEIPT class —
  001 2324 / 002 81 / 003 145 (lane-gap-05:76), advisory, exit 0.
  Close: census -> 0 or operator triage + fail-closed floor in
  make gate0.
- R7 narrowed to F10: workstream-receipt-map:24-26 — F5 PARTIAL,
  WG2 ALIVE, WG3 ALIVE, F10 UNVERIFIED. Close: F10 receipt or
  coordinator errata retires the token.
- R10 zero-tax (OPEN-NARROWED; two appends conflict): GAP-04
  claims CLOSED-by-append (lane-gap-04:81,:56); MV-5's later
  append supersedes (release-blockers:248-255) — alias flips
  C3+C6, but C1/C2/C4-runner/C5/C7/C8 ignore zero-tax. Close:
  every present child flips, or formal scope-narrowing receipt.
- P7 doc honesty: doctrine-ship-plan:134 stale-low; honest number
  is R5's DOC-RECEIPT census. Close = R5 close.
- P8 AX substrate: COVERED-LOCAL / DRAFT-UNRUN (tps-08:94-100,
  114-117; AX-0 local-exec GO). P8/P10 rows NOT yet in
  doctrine-ship-plan §D — TPS-8 execution is itself the open step.

## 3. NEW rows this session

- NEW-1 octopus residual (CONDITIONAL on TC-04): GAP-09 fixed :309
  (lane-gap-09:7-16), 20/20; a claimed :366 residual is NOT
  evidenced on disk yet. Close: TC-04 confirms/repairs or retires.
- NEW-2 DOC-RECEIPT-001 precision debt: 2324 findings, recall-first
  (lane-gap-05). Close: TC-09/TC-12 triage to residual-or-0, then
  fail-closed floor.
- NEW-3 AX container legs on the cgap-ax image: AX-1 built it
  (lane-ax-01 W1/W3); legs pending TC-14. Close: TC-14 receipt.
- NEW-4 AX-9 UNVERIFIED-by-design: NO-GO verdict (ax 127, cluster
  exit 1, install NOT APPROVED) — lane-ax-09:8-11; probe landed,
  nothing simulated. Close: operator unblocks cluster, or stays
  permanently loud-aspirational.
- NEW-5 POSH-2 23.35 ns gate decision pending: honest typed FAIL,
  2.3x (lane-posh-02:43,111). Close: operator gate-change receipt
  (R1 precedent) or repair to <=10 ns with flip court.
- NEW-6 AshA2A wiring design-only: cgapprd-08b:79-94 — ingredients
  real out-of-tree, wiring bidirectionally absent. Close: chartered
  wiring lane, or PRD clause retired/rewritten in cgapprd-prd-v2.
- NEW-7 cgap-core manifest_court::dod10_no_spdx failure: full-suite
  exit 101, ONLY failure, PRE-EXISTING (lane-cgap-prd-06:5-8).
  Flagged by PRD-6, owner unassigned. Close: SPDX identifiers
  tree-wide or court scope narrowed by owner.

## 4. Verdict — what blocks a v26.11.2 tag

Publish freeze (waves/publish-halt.md, FROZEN-BY-OPERATOR) stops
the actuator; the tag is downstream of operator lift.

MUST FIX before any tag: R4 (L2/L4/L5 + VACUOUS L1 + parser),
R10 (non-flipping children), NEW-7 (cgap-core 101), NEW-5
(decision, not necessarily repair).

MUST CLOSE before CERTIFIED: R3, R5/NEW-2, R7 (F10), TPS-8
enclosure (P8/P10 + sealing + stitch), NEW-1 (TC-04), NEW-3
(TC-14).

DO NOT BLOCK (parallel-safe): NEW-4, NEW-6, R8/R9/R11 (closed),
P9 (covered).

HARD BLOCK (non-code): operator publish-freeze lift; SL-004
resolved at sealing time (tps-08:30-42).

One-screen: NO tag until R4/R10/NEW-7 are repaired or formally
decided (NEW-5), the operator lifts the freeze, and TPS-8 runs the
enclosure. Net since release-blockers was written: R1/R2/R8/R9/R11
CLOSED; P9 COVERED; R4 measured-honestly-failing; R10 narrowed;
six new rows, one conditional. NOTE: the R10 appends conflict
(GAP-04 :81 vs MV-5 :248-255) — the later supersedes; flagged.

## See Also

audit/release-blockers.md · audit/doctrine-ship-plan.md ·
audit/cull-10-cull-wave-plan.md
