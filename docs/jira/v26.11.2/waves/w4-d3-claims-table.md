# W4-D3 — Measured-claims consolidation (position-paper-v2 forward leg)

Lane W4-D3 (lane-runner). Date 2026-11-01. Repo /Users/sac/cgap (main),
no git commands. Never touched /Users/sac/stardf.

## Subject + sole writes

- `docs/jira/v26.11.2/11-measured-claims.md` (NEW) — the exclusive
  numeric source for position-paper-v2: 27-row claims table, RDFox
  classification section, loud-skip section.
- This receipt.

## Method

Swept `docs/jira/v26.11.2/waves/` for receipts carrying measured
numbers, per dispatch list: w1-02, w1-03, w2-09, w3-l1, w3-l2, w3-l5,
w3-t2, w3-t3, w3-t5, w3-t7. Each number carried into the table with
claim | number | unit | method | receipt file:line | class. Verified
all cited line anchors with grep -n against the on-disk receipts this
session (10 receipts, all present, all anchors matched).

## Commands + exits (this session)

| Command | Exit | Evidence |
|---|---|---|
| `ls waves/` + targeted `wc -l` over 10 dispatch receipts | 0 | all 10 present |
| `grep -n` line-anchor verification over 6 receipts | 0 | all anchors matched (w3-l1:63,69-70; w3-l2:26-28,61; w3-l5:38-42,63; w3-t2:23-29; w3-t3:36-38,50,56-62; w3-t5:20,50,74-75; w2-09:14-15; w1-02:19-29; w1-03:14-22,39; w3-t7:10-14) |
| bounded 60 s poll for `waves/w4-*` receipts (10 s interval) | — | [SKIP] none present; w4-p1/p2/p3/mw2/k1 not consolidated (loud, recorded in claims doc §3) |
| python byte-integrity read-back of 11-measured-claims.md | 0 | on-disk bytes = written content |

## Table contents summary

- 22 MEASURED rows (regen digests x2, build/test x2, L1 ns/frame x4,
  L2 chain + target-fail, L5 fused regression + target-fail, asm
  census, riscv flash/purity census x4, pack 14/14 + lock hash,
  hygiene credential/seed x2, court flips x3).
- 2 REFUTED-ATTRIBUTION rows (L2 FNV-dominance, L5 span-walk model —
  both measured, both flipped the plan's bottleneck story).
- 3 TARGET rows (MW4/RDFox comparative, w4 lanes not-yet-landed).
- §2 RDFox classification: no MEASURED row is RDFox-comparative today
  (zero measured facts/s in the campaign); GB/s-or-ns/frame vs facts/s
  is CATEGORY-ERROR; MW4 vs RDFox 2-6.1M published envelope is
  UNVERIFIABLE-BY-ABSENCE until an MW4 receipt lands; g20 incumbent
  headlines UNVERIFIABLE-BY-ABSENCE per 06-bench-protocol header.

## Standing

ALIVE — consolidation ran this session against the on-disk receipts
with the exits above; every row cites receipt file:line verified by
grep this session. The claims doc is the exclusive numeric source for
position-paper-v2; no number may appear in the paper that is absent
from `docs/jira/v26.11.2/11-measured-claims.md`.

## JSON

```json
{"lane":"W4-D3","files_written":["docs/jira/v26.11.2/11-measured-claims.md","docs/jira/v26.11.2/waves/w4-d3-claims-table.md"],"commands":["ls/wc-l receipt census (exit 0, 10/10 present)","grep -n line-anchor verification over 10 receipts (exit 0, all matched)","bounded 60s poll for w4-* receipts ([SKIP] none present)","python read-back integrity check (exit 0)"],"courts_run":["line-anchor verification sweep (all 27 rows cite-matched)"],"standing":"ALIVE","divergences":["[SKIP] w4-p1/p2/p3/mw2/k1 absent at consolidation time (bounded 60s poll) — recorded loud, not silent","dispatch named 06-bench-protocol.md §6; verified present (lines 109-120 MW4 like-for-like, §7 unit law)"]}
```
