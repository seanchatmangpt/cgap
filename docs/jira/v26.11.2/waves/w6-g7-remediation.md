# W6-G7 Remediation Receipt

Lane: G7R. Repo: /Users/sac/cgap, branch main. 2026-11-02.

## Deliverables

- `scripts/verify_ost_design_around.py` — ROW_EVIDENCE block only:
  added `US11,409,698` entry (cite
  crates/cgap-bn3/src/unify.rs; markers `semi_join_masks`,
  `caller-leased`, `RuleCompiler` — all grep-verified in the cite).
- `docs/jira/v26.11.2/patent/claims.txt` — status columns only:
  row 2 re-scoped to the verified bit-slice surface (PLNF becomes a
  named open item); row 3 flipped to
  LANDED-crates/cgap-bn3/src/unify.rs. Row 1 stays PENDING.
- `docs/jira/v26.11.2/03-ost-design-around-register.md` — row
  updates: row 1 typed KNOWN-DIVERGENCE, row 2 re-scope record,
  row 3 landed record with line cites, receipt standing block.
- Receipt: docs/jira/v26.11.2/waves/w6-g7-remediation.md.

## Per-row status transitions

- Row 1 US10,817,467: PENDING -> PENDING (UNCHANGED, deliberately).
  No landed artifact matches the design-around ("partition-disjoint
  snapshot-delta; each partition seals its own delta pages"):
  session search found zero partition-sealing surface in cgap-storage
  / cgap-parquet src (only unrelated `partition_point` calls). The
  RingSink ring commit leg (mmap_writer.rs:706-840) + receipt chain
  (receipt_chain.rs:34-76) is a SINGLE ring, NOT per-partition
  sealed delta pages — registering it would fabricate the mapping.
  Typed KNOWN-DIVERGENCE row registered in the register doc; the
  gate honestly fails on this row.
- Row 2 US11,216,456: PARTIAL -> DISTINCT-MECHANISM via re-scope.
  Consistency check against the element mapping (register row 2):
  the patent element is "runtime tuple-binding propagation"; the
  vertical bit-slice masks with bounds fixed before execution answer
  that element on their own (no runtime propagation) — the PLNF
  cardinality component was cgap-side design-around extra, not part
  of the element, so narrowing does not misrepresent coverage.
  PLNF remainder = named open item in the register.
- Row 3 US11,409,698: PENDING -> DISTINCT-MECHANISM via landed
  evidence. Cite crates/cgap-bn3/src/unify.rs; markers
  `semi_join_masks` (unify.rs:783), `caller-leased` (unify.rs:650),
  `RuleCompiler` (unify.rs:474). Negative surface: sole src hit is a
  doc-comment "Generator" mention, classified non-mechanism.
  Disclosed wording divergence: "stratified" maps to semi-naive
  delta-round strata (cgap-storage/src/closure.rs:94).
- Row 4 US11,645,278: unchanged DISTINCT-MECHANISM.

## Gate run (real, this session)

    python3 scripts/verify_ost_design_around.py --claims docs/jira/v26.11.2/patent/claims.txt --corpus /Users/sac/purrdf/corpora --report docs/jira/v26.11.2/patent/ost_report.json
    -> exit 1: US10,817,467 PENDING; rows 2/3/4 DISTINCT-MECHANISM (3/4)

## FLIP court (anti-vacuity, /tmp sandbox, sha256-exact restore)

- Mutation: row 3's mechanism field replaced with row 4's
  ("branchless register unification + static instruction schedule
  (R0..=R255 ABI)") in /tmp/claims_flipped_g7r.txt -> duplicate
  mechanism register.
- -> exit 1; US11,409,698 and US11,645,278 dropped
  DISTINCT-MECHANISM -> NOT-DISTINCT (duplicate/near-duplicate
  mechanism shared across rows) — flip witnessed.
- Shared tree untouched: claims.txt sha256 before == after
  (8387d20824b83705..., restore_exact=True).

## Standing

- Gate: FAIL 3/4 DISTINCT (exit 1) — row 1 correctly failing on a
  registered known-divergence; rows 2/3/4 DISTINCT with verified
  cites. FLIP court flips. Not 4/4: no fabrication was performed to
  reach it; closing row 1 requires a landed partition-sealed
  delta-page mechanism or a charter-owner re-scope.

<!-- JSON
{"lane":"G7R","files_written":["scripts/verify_ost_design_around.py (ROW_EVIDENCE block only)","docs/jira/v26.11.2/patent/claims.txt (status columns only)","docs/jira/v26.11.2/03-ost-design-around-register.md (row updates)","docs/jira/v26.11.2/waves/w6-g7-remediation.md"],"commands":["python3 scripts/verify_ost_design_around.py --claims docs/jira/v26.11.2/patent/claims.txt --corpus /Users/sac/purrdf/corpora --report docs/jira/v26.11.2/patent/ost_report.json -> exit 1 (3/4 DISTINCT, row 1 PENDING)","FLIP court: /tmp/claims_flipped_g7r.txt (merged duplicate mechanism) -> exit 1, rows 3+4 DISTINCT->NOT-DISTINCT witnessed, claims.txt sha256 restore_exact=True (8387d20824b83705...)"],"courts_run":["FLIP merged-near-duplicate-claims (flipped: US11,409,698 + US11,645,278 NOT-DISTINCT)"],"standing":"G7-PATENT FAIL 3/4 DISTINCT (exit 1, this session) — row 1 honestly PENDING on a typed known-divergence; rows 2/3/4 DISTINCT-MECHANISM with grep-verified cites; gate machinery ALIVE (baseline + FLIP flip witnessed)","divergences":["row 1 design-around has NO landed artifact — RingSink/receipt-chain candidate rejected as a mapping fabrication; divergence registered in 03-ost-design-around-register.md instead of flipping","row 2 narrowed per operator order; PLNF remainder recorded as named open item (deemed consistent with the element mapping — element is runtime propagation, bit-slice pre-execution bounds answer it alone)","'stratified' wording vs landed semi-naive delta-round strata disclosed in register row 3"]}
-->
