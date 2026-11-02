# v26.11.2 DFSS — PFMEA Risk Matrix (04)

Phase: Measure->Analyze (DFSS lane DF-3).
Method: RPN = SEV x OCC x DET, 1-10 scales each.
DET columns are re-scored from ACTUAL gate evidence, each detection
control citing its witnessed flip (receipt + fresh re-run where stated).
Charter-level FMEA lives at `docs/jira/v26.11.2/14-dflss-charter.md:94-105`
— that table is retained there; this matrix supersedes its DET column
with witnessed evidence. Cross-link, no duplication.

## Detection-evidence fresh runs (this session, 2026-11-02)

| control | command | exit | note |
|---|---|---|---|
| disassembly-invariants gate | `python3 scripts/verify_disassembly_invariants.py` | 0 | CLEAN; arm E binary mutant `[PASS] anti-vacuity M-G03-E`; original archive sha256-verified unchanged |
| SRAM gate baseline | `python3 scripts/verify_cscb_sram.py` | 0 | 36,048 B <= 131,072 B |
| SRAM gate mutant | `python3 scripts/verify_cscb_sram.py --mutate=sram-bloat` | 0 | measured 136,068 B > 131,072 B — overflow flips in OUTPUT; process exit stays 0 (report-only mode, see divergences) |
| receipt_lint strict | `python3 scripts/receipt_lint.py --strict` | 1 | 444 claim findings on 189 docs; then CRASHED in summary (`KeyError: 'rule'`, scripts/receipt_lint.py:436) — defect recorded, not fixed (not owned) |
| n/a | fresh `cargo build`/`cargo test` | not run | not required by dispatch; standing claims re-read from receipts |

## Matrix (ranked by RPN, desc)

| # | Risk | SEV | OCC | DET | RPN | Detection control (witnessed flip) | Rank |
|---|------|-----|-----|-----|-----|-----------------------------------|------|
| R1 | Forbidden branch in generated branchless kernel code | 8 | 3 | 2 | 48 | disassembly-invariants court: census + linked witness + source-mutant + arm E binary mutant (lane-g03-disasm-gate.md:47-71); fresh exit 0 this session | 4 |
| R2 | Heap alloc in no_std kernel path | 8 | 2 | 2 | 32 | CountingAlloc breach court `cscb_c4_alloc`, `CSCB_MUTATE=alloc` -> exit 1 (lane-b03-cscb-c4.md:18-20) | 6 |
| R3 | SRAM/.text envelope overflow (131,072 B) | 7 | 3 | 2 | 42 | sram-bloat mutant measured 136,068 B > bound, overflow flips (lane-b04-cscb-c5.md:40); fresh re-measure this session, output flip witnessed, exit 0 (report-only) | 5 |
| R4 | DRed retraction leaves phantom survivors | 7 | 2 | 2 | 28 | skip-sweep mutant detects exactly 10 injected phantoms -> exit 1 (lane-b05-cscb-c6.md:26-35,61-62) | 8 |
| R5 | Certificate bitflip accepted by chain court | 8 | 1 | 2 | 16 | c7 bitflip court rejects mutated certificate; self-fails if bitflip no-ops (lane-b06-cscb-c7.md:40-46) | 10 |
| R6 | Shared-fixture race: fixed /tmp work dir -> false gate FAIL | 8 | 6 | 7 | 336 | CURRENTLY MANUAL: DX-6 classifier court named by dispatch but `lanes/lane-dx-06` NOT PRESENT on disk (checked this session); DX-8 census receipt carries no classifier court line; per-process `work()` fix UNVERIFIED | 1 |
| R7 | Agent long-file write corruption (~1 in 5) | 7 | 6 | 2 | 84 | write-integrity PostToolUse hook DX-2 LANDED (lane-dx-02-write-integrity.md:7-10), court battery incl. fire->clean byte-restore flips (lines 26-43) | 3 |
| R8 | Stale-binary / rustc-error save reads as 0 refusals | 9 | 4 | 8 | 288 | CURRENT STATE HONEST: no mechanical gate yet; `lanes/lane-lspa-02` and `lane-lspa-04` NOT PRESENT on disk (checked this session; only lspa-01/03 exist). DET=8 until LSPA-2/4 land | 2 |
| R9 | Receipt drift (stale/unbacked claims in landed docs) | 5 | 7 | 2 | 70 | receipt_lint --strict fires (exit 1, 444 findings) — fresh run this session; NOTE: strict summary path crashes KeyError 'rule' (scripts/receipt_lint.py:436); findings list itself printed fully before crash | 7 |
| R10 | 429 lane death mid-wave (silent partial edits) | 6 | 5 | 6 | 180 | CURRENTLY MANUAL: retry lanes GIT-03/GIT-07 re-landed after API 429 (lane-git-03-ingress-ring-receipt.md:1; lane-git-07-egress-ring.md:3); detection is receipt-absence + coordinator inventory, no mechanical court | 9 |

## Top-3 for DF-6 control plan

1. R6 shared-fixture race — RPN 336, detection MANUAL, OCC witnessed in
   session history. Control: land the DX-6 classifier court + per-process
   work() dir; re-score DET 7->2.
2. R8 stale-binary false-pass — RPN 288, SEV 9, no gate until LSPA-2/4
   land. Control: mechanical rustc-error/refusal-count parity gate;
   re-score DET 8->2.
3. R10 429 lane death — RPN 180, detection manual inventory. Control:
   receipt-absence watchdog at wave drain; re-score DET 6->3.

R7 (write corruption, RPN 84) is held out of top-3 only because its
control DX-2 is already landed and witnessed; it is DF-6's cheapest win
(verify hook fires in settings.json, re-score DET 2->1).

## Anti-vacuity of this matrix

Mutation named: if any DET evidence cell is deleted, RPNs rise and the
top-3 set changes (R1 RPN 48->96 without the court cite) — the ranking
is falsifiable by receipt absence. Fresh runs above are the non-vacuity
witness that DET=2 scores were re-derived, not carried from the charter.

## Divergences (doc-yields-to-reality)

- SRAM mutant run exits 0 (report-only); the flip is in the measured
  byte count, not the exit code. lane-b04-cscb-c5.md:30 lists exit 0
  for the final court run consistently with this. A gate that reports
  a flip but exits 0 is a latent R8-class defect for DF-6.
- `receipt_lint.py --strict` crashes at scripts/receipt_lint.py:436
  (KeyError 'rule') after printing findings — DX-4 owner notified via
  this receipt (cross-lane disclosure; file not owned by DF-3).
- Dispatch-named `lanes/lane-dx-06` (classifier court) does not exist
  on disk; R6 DET scored for the CURRENT state, not the planned one.
- Dispatch-named `lanes/lane-lspa-02`/`lane-lspa-04` do not exist on
  disk; R8 DET=8 reflects current ungated state honestly.
- rep1-census incident (R6 OCC evidence, per dispatch): grep across
  waves/ and lanes/ found no matching receipt line — recorded as
  session-hearsay OCC evidence, receipt locate UNVERIFIED.
