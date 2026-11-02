# Model 12 — Legal & Patent Clearance (Enterprise Equivalent)

Lane AU12, Phase 2. Grounded in cgap's ACTUAL IP posture; enterprise
modeling labeled ASSUMPTION. NOT legal advice.

## Actual IP posture (receipts, 2026-11-02)

- Register: docs/jira/v26.11.2/03-ost-design-around-register.md —
  4/4 DISTINCT-MECHANISM (US 10,817,467 / 11,216,456 / 11,409,698 /
  11,645,278), gate `verify_ost_design_around.py` exit 0.
- Honest limitation (verbatim, FA11 M2 residual, closure/lane-05-g7-
  patent-witness.md §3): "The gate is a register-consistency gate.
  The element column remains self-authored — no patent text exists on
  disk to compare against, so element wording is unfalsifiable inside
  the gate and ZZQQ-class corruption still passes."
- Behavioral rows: crates/cgap/tests/patent_behavioral_court.rs —
  row 2 (bound-key mask prunes pre-evaluation; MASK_ALL flip fails
  the court), row 3 (cyclic derivation refused typed
  FixpointBudgetExceeded, never a hang).
- Row 1 mechanical enforcement: CGAP_ERR_CONCURRENT_WRITER_DENIED
  0x4347_0006 (sovereign.rs:87, pre-IO denial :764) + race court
  crates/cgap-ffi/tests/fa9_commit_gating.rs; RingSink/drain_to_spill/
  RunOverrun mmap_writer.rs:721-842; BLAKE3 chain receipt_chain.rs.
- w5/w6 receipts: w5-ring-rewitness.md (L4 courts + bench
  re-witnessed); w6-g7-patent-gate-flip.md (row 1 re-scoped to the
  landed single-ring mechanism; merged-duplicate flip -> exit 1).

## 1. Clean-room program requirements per claim family (ASSUMPTION)

1. Prior-art / claim-chart review: file-history + claim-element chart
   against the design-around mechanism, by counsel or patent agent.
   cgap equivalent in place: register DISTINCT-MECHANISM rows
   (engineering-side claim chart), gate + flip court.
2. Design-around spec review: counsel reviews the spec text (register
   row wording + landed cites) element-by-element. cgap equivalent:
   register cross-check + claims.txt consistency.
3. Outside-counsel non-infringement opinion (mitigates willfulness
   exposure): element-by-element, privileged. cgap has NO equivalent
   — the honest asymmetry (§3).

## 2. Calendar model (ASSUMPTION ranges)

- Outside-counsel non-infringement opinion: commonly 6–12 weeks per
  claim family. ASSUMPTION — not measured in this repo.
- Committee / IP-review-board cycle: quarterly cadence typical;
  2–4 weeks per round. ASSUMPTION.
- Prior-art search by counsel: 1–3 weeks per family. ASSUMPTION.

## 3. The honest asymmetry

cgap's gate is register-consistency + behavioral courts + mechanical
enforcement, NOT legal opinions. It proves the mechanism is distinct
and enforced in code; it cannot prove non-infringement (the FA11 M2
residual is explicit: element text self-authored, patent text not on
disk, ZZQQ-class corruption still passes). An enterprise ADDS counsel
review on top — it does not replace what cgap has. The behavioral
row-2/3 and fa9_commit_gating race courts are engineering evidence
counsel would CONSUME (input to an opinion), not substitutes.

## 4. Claim-family table (ranges ASSUMPTION; engineering state receipt-backed)

| Claim family | Eng. state (receipt) | Prior-art (wks) | Counsel opinion (wks) | Committee |
|---|---|---|---|---|
| US 10,817,467 | LANDED row 1: single ring + BLAKE3 chain + 0x4347_0006 + fa9 race court | 1–3* | 6–12* | quarterly, 2–4 wks* |
| US 11,216,456 | DISTINCT row 2: bitslice.rs; behavioral court + MASK_ALL flip | 1–3* | 6–12* | quarterly, 2–4 wks* |
| US 11,409,698 | LANDED row 3: unify.rs semi-join cascades; behavioral court | 1–3* | 6–12* | quarterly, 2–4 wks* |
| US 11,645,278 | DISTINCT row 4: R0..=R255 ABI, branchless unification | 1–3* | 6–12* | quarterly, 2–4 wks* |

* = ASSUMPTION. Serial critical path 8–16 weeks/family (ASSUMPTION);
families run in parallel — wall-clock ~ one committee season.
