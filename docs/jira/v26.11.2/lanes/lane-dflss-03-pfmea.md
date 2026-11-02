# Lane DF-3 — PFMEA risk matrix

## W1 Subject
`docs/jira/v26.11.2/dflss/04-pfmea-risk-matrix.md` (NEW, 73 lines).

## W2 Authority
Dispatch: DF-3, write ONLY the matrix + this receipt. Charter
14-dflss-charter.md:94-105 is retained, cross-linked, DET superseded.
NO git commands run.

## W3 Consequence
RPN-scored PFMEA with DET re-scored from witnessed gate evidence;
top-3 rows handed to DF-6 control plan.

## W4 Commands + exits (this session)
| command | exit |
|---|---|
| `python3 scripts/verify_disassembly_invariants.py` | 0 |
| `python3 scripts/verify_cscb_sram.py` | 0 (36,048 B <= 131,072 B) |
| `python3 scripts/verify_cscb_sram.py --mutate=sram-bloat` | 0 (136,068 B measured — flip in output, exit report-only) |
| `python3 scripts/receipt_lint.py --strict` | 1 (444 findings, then KeyError 'rule' crash at scripts/receipt_lint.py:436) |
| on-disk checks: lanes/lane-dx-06, lane-lspa-02/04 absent; lane-dx-02/04, lane-b03/b04/b05/b06, lane-git-03/07 receipts re-read | n/a |

## W5 Verification ladder
Fresh gate re-runs (3) over the exact subjects named by dispatch;
receipt re-reads for B03/B05/B06/B04/DX-2/DX-4/GIT-03/07; absence
checks for DX-6/LSPA-2/4 (ls of lanes/). Markdown lint: not run
(receipt_lint crashed before struct rules — itself recorded).

## W6 Standing
ALIVE for the three fresh runs + on-disk receipt re-reads; the matrix
rows R1-R5, R7, R9 detection cells cite witnessed flips; R6/R10 DET
honestly marked MANUAL. UNVERIFIED: rep1-census incident receipt
locate; DX-6/LSPA-2/4 future DET scores.

## W7 Anti-vacuity / fences
Falsifier named in matrix section "Anti-vacuity of this matrix":
deleting any DET cite changes RPN and top-3 ranking. Fences my diff
could break: none (new files only; charter table untouched, only
cross-linked). Two cross-lane disclosures (SLA unblocks NOT applied —
these are defect findings, no files touched):
1. scripts/receipt_lint.py:436 KeyError 'rule' crash in --strict —
   disclosed for DX-4 owner (lane-dx-04-receipt-lint.md); no fix
   applied, file not owned.
2. verify_cscb_sram.py --mutate exits 0 (report-only flip) — latent
   R8-class gate-exit defect, disclosed for B04 owner
   (lane-b04-cscb-c5.md); no fix applied.
