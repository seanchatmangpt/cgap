# v26.11.2 OST Design-Around Register (4/4 DISTINCT-MECHANISM)

Transcription of CHARTER-CGAP-STARDF-v26.11.2 §OST DESIGN-AROUND REGISTER.
Repo: /Users/sac/cgap. Lease: docs/jira/v26.11.2/03-ost-design-around-register.md
(this file, NEW; no other file written; no git writes performed).
Explicitly NOT legal advice — engineering register only.

## Law (binding on every v26.11.2 lane)

Any lane restating a patented mechanism as a design-around = typed REFUSED
(OVERNIGHT-LOOP.md §6, gate-enforced). Parity is achieved by independent
mechanisms, not renamed ones. Each row below names the mechanism that is
DIFFERENT, not a paraphrase of the claim.

## Register

| # | Patent | OST patented mechanism | v26.11.2 design-around (distinct mechanism) | Refused alternative |
|---|--------|------------------------|---------------------------------------------|---------------------|
| 1 | US 10,817,467 | shared-memory CAS over snapshot state | partition-disjoint snapshot-delta: each partition seals its own delta pages; no shared-memory CAS anywhere | any CAS/lock-based snapshot merge |
| 2 | US 11,216,456 | runtime tuple-binding propagation | compile-time PLNF cardinality + vertical bit-slice masks; bounds fixed before execution | runtime sideways information passing |
| 3 | US 11,409,698 | runtime subquery continuation frames | stratified semi-naive bit-matrix cascades in caller-leased scratch; no continuation frames | runtime subquery continuation |
| 4 | US 11,645,278 | runtime topology switching | branchless register unification + static instruction schedule (R0..=R255 ABI) | dynamic topology dispatch |

## Cross-check (read-only, this session)

Per-row implementation-evidence status (PENDING = named for v26.11.2, no
cgap implementation yet; statement-evidence = landed receipt naming the
mechanism).

1. US 10,817,467 — PENDING in cgap. Adjacent landed statement: stardf
   RDF-5 incremental (/Users/sac/stardf/arrow/docs/ost/incremental.md)
   seals immutable footer-sequenced delta pages, recomputes views
   PER-DELTA-PAGE — page-granularity, no shared-memory CAS. cgap grep
   for snapshot/CAS in cgap-storage/cgap-parquet src: zero hits.
2. US 11,216,456 — statement evidence LANDED: stardf SIP-ALT
   (/Users/sac/stardf/arrow/docs/ost/sip-alt.md) cites
   "NOT-practiced: US 11,216,456 / US 11,409,698 claim classes" via
   precomputed structural planning from sealed per-file per-predicate
   count tables. cgap partial impl evidence: crates/cgap-parquet/src/
   bitslice.rs (vertical bit-slice masks). Compile-time PLNF
   cardinality: PENDING.
3. US 11,409,698 — cited NOT-practiced in sip-alt.md (same receipt as
   row 2, distinct mechanism column). cgap semi-naive cascade impl:
   grep over cgap-bn3/cgap-bont src = zero hits. PENDING.
4. US 11,645,278 — implementation evidence LANDED: cgap-bn3
   crates/cgap-bn3/src/unify.rs (lane A12, v26.11.1 wave 9-10):
   register-file ABI R0..=R255, Wave 2 branchless-selection kernel,
   static rule-word compilation; typed UnifyError::RegisterOverflow.
   No runtime topology switch surface in the crate.

### errc-backlog + dia-5 scout cross-check

- docs/jira/v26.11.1/waves/errc-backlog.md: read this session. Zero
  patent/design-around rows (grep for patent/design-around/467/278:
  zero hits). Its only grounded candidate is the docs-diataxis-pack
  generalization — no overlap with any register row.
- docs/jira/v26.11.1/waves/dia-5-marketplace-scout.md: read this
  session. Documents the readme-diataxis-pack consumer run + FM-PACK-013
  mutation court. No patent content; no overlap with any register row.
- Related gate: docs/jira/v26.11.2/01-release-gates.md G7-PATENT
  (verify_ost_design_around.py, 4/4 DISTINCT-MECHANISM verdict);
  docs/jira/v26.11.2/patent/ does not exist yet — PENDING.

## Receipt

- Written this session by V2-PATENT (court-auditor), 2026-11-01, repo
  /Users/sac/cgap, branch main. This file IS the marker.
- Standing: transcription ALIVE (transcribed from the operator's
  charter text supplied with the dispatch, cross-checked against
  on-disk receipts); implementation rows 1 and 3 PENDING,
  row 2 half-landed (bitslice.rs yes, PLNF no), row 4 landed (A12).
