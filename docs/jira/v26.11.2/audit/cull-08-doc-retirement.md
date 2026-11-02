# cull-08-doc-retirement

CULL-08 doc retirement audit (2026-11-02, read-only): documents whose
claims are now FALSE (not merely stale). Actions: rewrite (errata
append) or tombstone (redirect + superseded-by). Receipts of record:
cgapprd-01/02/03, lane-cgap-prd-05, lane-dflss-04-cpk, lane-dfx-04-msa,
lane-gap-03-c6-p99, lane-mv-05-sla-reconcile.

## (a) Sigma_PQC stated as landed conservation law

Owning receipt: cgapprd-02-equation-grounding.md:57-66 ("mod
Sigma_PQC" is INVENTED LORE, :54; no in-tree referent).
REWRITE (errata append):
- docs/REVENUE_GENERATION_COMPENDIUM.md:103 (unqualified equation;
  PRD-2's rewrite text applies verbatim).
- docs/jira/v26.11.2/12-ard-prd-cgap.md:19 (conservation block).
- docs/jira/v26.11.2/paper/position-paper-v3.md:231,233 (prose +
  LaTeX).
- LICENSE-P28-SIL.md:69 (statutory restatement; legal text needs an
  errata rider citing PRD-2, not a silent tombstone).
TOMBSTONE: docs/jira/v26.10.31/07-chain-economy-8020.md:36
(prior-milestone; redirect to PRD-2).
Verify-only (no action): claims-appendix.md:37 (TARGET-qualified);
A16-chatman-pqc.md:28,97-99 (already honest UNKNOWN).

## (b) pre-MV-5 800M-gate prose

Owning receipts: lane-mv-05-sla-reconcile.md:15; release-blockers
R1 append :233-238 (landed); doctrine-commercial-proofs.md:60,81.
REWRITE:
- lanes/lane-b05-cscb-c6.md:21,47 — presents 800M as the operative
  gate post-re-scope; errata append citing release-blockers:233.
- lanes/lane-git-11-retract.md:90 — aspiration framing is
  receipt-true; LOW priority (annotate re-scope pointer).
DONE/no-action: release-blockers:23 (superseded by :233 append);
dod-cgap-storage.md (already corrected); MEGA-REPORT (no false
800M row found).

## (c) pre-FIX-04/GAP-03 "p99" labeling

Owning receipt: lane-gap-03-c6-p99.md:4-17 (C6 secs[6] "p99" over
N=7 is the MAX; source relabeled).
REWRITE candidates:
- lanes/lane-mv-05-sla-reconcile.md:44 + audit/release-blockers.md
  :237 — both quote the same run; verify label wording; errata
  append if "p99" appears.
NO action: the "p99 ~4.4M/s" figure is C7/b06:24 — a genuine
sorted-sample p99, not the C6 max bug. dod-csct-cscb.md:35 C1/C2
p99 is genuine (lane-b01); BUT its "one timing flake" row is
contradicted by doctrine-commercial-proofs.md:29 (C2 FAIL 3/3
direct runs) — errata append flagged.

## (d) L2/L5-era causal claims

DF-4/DFX-4 landed ERRATAs (dflss/05-cpk-analysis.md:47-48,103;
09-ax-cpk-msa.md:67). Sweep for surviving uncorrected phrasing:
none found. NO new action.

## (e) lane-b03 stale 64-byte claim — DONE

PRD-5 errata landed (lane-b03:65-68; DFLSS_CHARTER:30-33 scoped;
notes in 00-ard-prd/15-ctq/14-dflss). Grep sweep clean. No further
work.

## (f) sub-ms checkpointing / <2 ms closure as capability

Zero offenders found: PRD-1's rows are DEFECT-OPEN/ASPIRATIONAL;
cgapprd-09:167-173 already bans the claims; the compendium's
88-byte sealed-checkpoint sketch (:204) is a feature sketch, not a
perf claim. NO action.

## Action table

| file | line | false claim | action |
|---|---|---|---|
| REVENUE_GENERATION_COMPENDIUM.md | 103 | mod Sigma_PQC law | rewrite |
| 12-ard-prd-cgap.md | 19 | mod Sigma_PQC law | rewrite |
| paper/position-paper-v3.md | 231,233 | mod Sigma_PQC law | rewrite |
| v26.10.31/07-chain-economy-8020.md | 36 | mod Sigma_PQC law | tombstone |
| LICENSE-P28-SIL.md | 69 | mod Sigma_PQC statute | rewrite (rider) |
| lanes/lane-b05-cscb-c6.md | 21,47 | 800M operative gate | rewrite |
| lanes/lane-git-11-retract.md | 90 | 800M framing | rewrite (low) |
| lanes/lane-mv-05-sla-reconcile.md | 44 | C6 max labeling | verify/rewrite |
| audit/release-blockers.md | 237 | same labeling | verify/rewrite |
| audit/dod-csct-cscb.md | 35 | "one timing flake" C2 | rewrite |

## See Also

audit/cgapprd-02-equation-grounding.md ·
audit/doctrine-commercial-proofs.md · audit/cull-10-cull-wave-plan.md
