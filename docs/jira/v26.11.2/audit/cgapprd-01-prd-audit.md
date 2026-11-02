# cgapprd-01-prd-audit

CGAP-PRD-1 PRD/ARD claims audit (2026-11-02, read-only lane; verdicts
from landed receipts at file:line or fresh session commands).

## On-disk referent

PRD/ARD exists: docs/jira/v26.11.2/12-ard-prd-cgap.md (161 lines,
10,871 B; canonical operator charter, receipt waves/v2-17-ard-landing.md
ALIVE, :159-161). Numbering is P1-P6 + NFR table (§3.2), not FR-01..06
— FR/NFR claims audited as listed, mapped to charter sections.

## Fresh commands this session (all exit 0)

verify_disassembly_invariants.py (AE4 CLEAN incl. binary mutant);
verify_cscb_sram.py (36,048 B, gate PASS); cargo test -p cgap-blake3
(31/31 incl. chain_mutation_court); posh_alloc_sentry_court (0/0
window + MEASURED honesty window).

## Per-claim table

| claim | verdict | receipt | gap |
|---|---|---|---|
| FR-01 penstock/scan branchless | VERIFIED (fresh AE4 exit 0) | lane-g01:3; lane-g02:118-121 | — |
| FR-02 bn3 unify RADIX_BUCKETS=4096 | VERIFIED | unify.rs:840; audit/dod-cgap-bn3.md:17 | — |
| FR-03 blake3 chains + in-crate tamper court | VERIFIED (fresh 31/31) | chain_mutation_court.rs:1; lane-gap-02:1 | — |
| FR-04 andon 101 + exit ABI | VERIFIED | andon.rs:17 + :109-110 court | as_u32 helper not located in csct; rests on runner boundary (:44) — minor |
| FR-05 zero-alloc SCOPED | VERIFIED (scoped only) | posh_alloc_sentry_court.rs:8-24; lane-posh-04-sentry.md:37 (2,662 allocs / 650,294,994 B measured) | whole-pipeline zero claim would be VACUOUS — the court fences it |
| FR-06 sub-ms checkpointing | DEFECT-OPEN (net-new) | no subsystem in-tree (nearest: dflss/02-ax-charter-ctq.md:68 NO-GO; dred is anti-join not snapshot) | register in DEFECT-REGISTER |
| NFR-01 10 ns/fact | HONEST FAIL | lane-posh-02-batch.md:43,111 (23.35 ns median, 2.3x over) | 10 ns marketing unsupported |
| NFR-02 <2.0 ms closure | ASPIRATIONAL | 1.873 ms is DEBUG best-of-3 (lane-git-12:60); release witness non-repro (lane-b02:32); 36.9 s standalone (lane-mv-05:66) | profile-ambiguous |
| NFR-03 SRAM | VERIFIED (fresh) | verify_cscb_sram.py exit 0; 36,048 B, headroom 95,024 B | stack bound UNVERIFIED (self-declared) |
| NFR-04 scoped zero-alloc | VERIFIED (scoped) | same court/fence as FR-05 | same fence applies upward |
| NFR-05 >4M seals/s | CONDITIONAL PASS | lane-b06:24 (medians) + :52-54 (contended FAIL witnessed) | must travel with load caveat |
| NFR-06 branchless + no_std | VERIFIED w/ residuals | fresh AE4 exit 0; cgap-blake3 Cargo.toml:5 no_std; residuals cmov_u64/eytzinger_restore/select_slice allowlisted RV32 ceiling | not hidden |

## 5 weakest links

1. FR-06 checkpointing absent in-tree — the only outright missing
   capability; DEFECT-OPEN net-new.
2. NFR-01 10 ns/fact honest FAIL (23.35 ns median, 2.3x).
3. NFR-02 <2 ms profile-ambiguous — only DEBUG clears it.
4. NFR-05 >4M/s uncontended-only (contended FAIL witnessed).
5. FR-05/NFR-04 zero-alloc scope fence — whole-pipeline restatement
   is vacuous per the court's own fence.

Secondary: FR-04's exit-ABI half (as_u32 court) has no csct referent.

## See Also

audit/cgapprd-02-equation-grounding.md ·
audit/cgapprd-03-frame-layout.md · 12-ard-prd-cgap.md
