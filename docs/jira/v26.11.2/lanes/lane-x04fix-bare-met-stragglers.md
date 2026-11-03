# Lane X-04FIX — bare-MET stragglers closed (2026-11-02)

W1 Subject: docs/CHALLENGER_SALES_REPORT.md, docs/REVENUE_GENERATION_COMPENDIUM.md §9,
docs/jira/v26.11.2/12-ard-prd-cgap.md:55 region, audit/cgapprd-prd-v2.md:82 region,
docs/jira/v26.10.31/20-prd.md:36 region (borderline annotation).

W2 Diff:
- CHALLENGER_SALES_REPORT.md — RW-02-style scope banner replaces "Every metric is
  ground truth" (:31); row-37 BN3 row rewritten: "<100 MB/s" incumbent column marked
  UNSOURCED-BASELINE (no engine/method ever cited; per paper-comparison-audit R4/R6),
  "82x margin over 1.0 GB/s wire rate" restated as "82.2x its own >1.0 GB/s charter
  target — a byte-throughput multiple, NOT a wire-rate margin"; BN3-internal
  isolated-scan scope + UNVERIFIED-WITHOUT-METHOD for v26.11.2 + row-28 contention
  pointer all carried in the banner.
- COMPENDIUM §9 Playbook 2 Step 2 (~:422): inline X-04FIX scope qualifier on the
  82.24 GB/s bullet (quiet-host, BN3-internal isolated scan, bench-bn3-receipt.md:45,69,
  no v26.11.2 register row, bench-chains-bn3.md:37 chain caveat). §6.1 RW-02 banner
  left untouched (not owned); §9 extended locally instead.
- 12-ard-prd-cgap.md P1 + cgapprd-prd-v2.md P1: ANNOTATE chosen consistently over
  strike (both carry "STAYS per R9" provenance; annotate preserves it); appended
  qualifier citing 11-measured-claims.md row 29 (UNSUPPORTED; BR3 — typed-refusal
  branches lawful).
- 20-prd.md:36: one-line HISTORICAL MARKER (superseded v26.10.31 PRD, row 29).

W3 Commands/exits:
- grep -c "X-04FIX" over the five files → 1/1/1/1/1, exit 0 (anti-vacuity: every
  site carries its qualifier).
- Cite re-reads exit 0: bench-bn3-receipt.md:45 (82.24 GB/s Divan fastest row) and :69
  (statement_scan_24b MET vs >1.0 GB/s); bench-chains-bn3.md:37 (80.32 GB/s isolated);
  paper-comparison-audit.md:31 (R4 category-error flag, verbatim).
- 11-measured-claims.md rows 5/7/12/28/29 re-read from disk this session (line 29/32/36/52/53).

W4 Standing: ALIVE (all edits on disk, grep-verified exit 0 this session).
No builds run — doc-only lane; compile fence N/A.

W5 Fences: the RW-02 §6.1 banner (COMPENDIUM:290) is the governing banner — this lane
extended §9 locally rather than editing §6.1 (not owned). No gates/courts touch these
docs; the only regression surface was re-clobbering FIX-13/FIX-16 regions of
12-ard-prd-cgap.md — untouched (single P1 inline edit only).

W6 Divergences: dispatch said banner "quiet-host divan scope; 82.24 GB/s is
BN3-internal, no v26.11.2 register row → UNVERIFIED-WITHOUT-METHOD, cite bench
receipt; row-28 divergence pointer" — all carried. Dispatch allowed strike-or-annotate
for the PRDs; chose annotate consistently, reason recorded above.

W7 No cross-lane unblocks applied; no git commands run.
