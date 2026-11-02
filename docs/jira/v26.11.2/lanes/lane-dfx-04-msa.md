# Lane DFX-4: AX-container-court Gage R&R / MSA (09-ax-cpk-msa)

Repo /Users/sac/cgap @ main, 2026-11-02. Write scope honored:
`docs/jira/v26.11.2/dflss/09-ax-cpk-msa.md` (new) + this receipt.
No git commands. No build lease needed (docs-only lane).

## W1 Subject

Gage R&R for the AX containerized court + the darwin-host Cpk story,
re-derived from landed receipts (ipc-03, g09, dflss-04, ax-00, ipc-08,
b01, repair-13). All receipts re-read fresh from disk this session;
cited line numbers re-verified present.

## W2 Commands + exits

| # | Command | Exit |
|---|---|---|
| 1 | grep/reads of the seven cited receipts | 0 (all hits found at cited lines) |
| 2 | python3 assembly of 09-ax-cpk-msa.md (2 chunks, 88 lines on disk) | 0 |

## W3 Courts run

None executable for a docs-only lane; the lane's falsifier is citation
grounding: every cited figure was re-read at its cited line this session
(41 ns @ lane-ipc-03:43, 85 ns @ :44, batch 179-181 ns @ :46, GO/NO-GO
rows @ lane-ax-00:44-48, IPC-7 PENDING @ lane-ipc-08:13,27, repair-13
0.855/2.131 @ :36,44, g09 Cpl 28.97, DF-4 Cpl 0.694/Cpu 0.408). A broken
citation would be the flip; none observed.

## W4 Content summary (09-ax-cpk-msa.md)

1. Gauge law: darwin timer 41 ns min monotonic delta
   (lane-ipc-03-latency-court.md:43) cannot resolve a ~10 ns frame; the
   gauge is batch-amortized timing, disclosed. Gauges inventoried:
   85 ns membrane-floor proxy (:44), batch-1365 179-181 ns/frame (:46),
   CTQ-02 BATCH=64 Instant (05-cpk-analysis.md:63).
2. In-container N=30 Cpk: ASPIRATIONAL-UNVERIFIED. IPC-7 PENDING
   (lane-ipc-08-index.md:13,27); cluster NO-GO (kubectl exit 1 conn
   refused, lane-ax-00-gate.md:17); exact unblock paths named: (a)
   cluster GO flipping AX-2/AX-9, or (b) local docker 28.0.4 re-run of
   the IPC-3 linux port once AX-1/AX-3 image builds land
   (lane-ax-00-gate.md:19-22,44-48).
3. Cpk story: G09 quiet Cpl 28.97 vs DF-4 loaded Cpl 0.694 / Cpu 0.408
   — both receipts stand, host-load variance recorded as a measurement
   property; control implication deferred to 06-control-plan.md.
4. REFUTED the "0.855 -> 1.57 via optimization" causal story: 12.4/6.00
   ns are lane-b01 CSCB C2 p99 numbers (lane-b01-cscb-c1-c2.md:19);
   true root cause is load-induced between-sample drift (within 2.131
   vs overall 0.855, repair-13:36,44), repaired by G09's n=30 sampling
   methodology, not codegen; the 1.572 intermediate claim remains
   UNVERIFIED (no receipt on disk).

## W5 Standing + fences

- Standing: ALIVE for the citation-grounding pass (commands in W2,
  exit 0, this session). All capability figures are carried with their
  originating receipts' standing, not upgraded by this lane. The
  in-container study is ASPIRATIONAL-UNVERIFIED by design.
- Fences this diff could break: none mechanical — docs-only; nearest
  fences are the receipt lint (lane-dx-04) and no-overclaiming doc
  rules. Zero source/manifest/Makefile surface touched.
- Divergences from dispatch: (1) the receipt file landed as
  lane-dfx-04-msa.md under lanes/ as directed; (2) the dispatch's
  "lane-dflss-04-msa.md if landed" does not exist on disk — the landed
  CPK receipt is lane-dflss-04-cpk.md, used instead (recorded, not
  silently corrected).
