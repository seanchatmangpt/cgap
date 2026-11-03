# ADV-16 — M-CTQ promotion deficit & unsubstantiated assertion audit (corrosion lane)

Date: 2026-11-02. Repo: /Users/sac/cgap, branch main. READ-ONLY lane:
zero bytes written outside this file. No git commands run. Every claim
below re-read from disk this session; fail-closed logging ([SKIP] with
reason, never inferred).

## Standing: ALIVE (read-only audit; greps + reads this session, exits
0/1 as listed; grep exit 1 = zero hits, reported as evidence)

## Inputs read first

- `lanes/lane-xc04-mctq-consolidation.md` (full, 117 lines)
- `11-measured-claims.md` §4 + §4.1 (rows 30/31/31a/32, D1/D2/D3,
  landed; :90-127)
- `dflss/02-voc-ctq-matrix.md` M-CTQ cells (:229-251, R-07 fold
  LANDED on disk — dispatch warned it might be mid-flight; it is not)
- `redteam/corrosion/adv-07-ctq-metric-arbitrage.md` (full)
- `lanes/lane-r16-mctq-promotion.md` incl. ERRATA :73-93

## Probe 1 — did the promotion round up? (target-MET claims)

Grep for any doc stating M-CTQ-04/05/06 targets MET on rows 30-32
evidence: `grep -rn "M-CTQ-0[456].*MET\|MET.*M-CTQ"` over docs/jira/
v26.11.2 — ZERO hits. Class is `consolidation-addendum` in every
carrier (claims §4 rows :106-109; matrix cells :229/:243/:258 all
lead "CONSOLIDATION-ADDENDUM ... cross-instrument wire legs
UNVERIFIED"; matrix standing note :270-272 explicitly quotes claims
claim-file :97-99 "NOT full MEASURED promotion").

The 1M/s LSL: technically cleared by run-1 min 3,529,029/s, but NO
cell certifies it on that evidence. The master-table CTQ-04 row
(matrix :17) still reads "median 3,894,347/s | VERIFIED [a]" — that
is the lane-fix04 1M-window fresh run (its own instrument, flip-
witnessed), NOT the XC-04 rows; footnote [a] keeps it
UNVERIFIED-FOR-PAPER. Verdict: **CLEAN — no round-up**.

## Probe 2 — unrealistic-pipeline laundering sweep

XC-04 stations are in-process extern "C" / Rust mmap loop /
single-threaded quiet-host. Full-repo grep for the station numbers
(`3,529,029|3,824,869|4,130,106|4,135,837|124.02|113.37|0.15 ns` and
variants) over docs/ + README.md: the numbers appear in exactly
three files — the XC-04 receipt, claims §4, and the CTQ matrix.
Exec/marketing surfaces (README, REVENUE_GENERATION_COMPENDIUM,
position-paper, dflss rollups 00/11, charters 01/interops): ZERO
hits. `dflss/01-project-charter-interop.md:55-66` and
`audit/cgapprd-prd-v2.md:144-147` quote only b06 1M-window figures
(4,377,052/4,356,280/4,351,961) WITH the contended-fail +
machine-dependent caveats inline — a different instrument, properly
not conflated with XC-04.

D1/D2/D3 are disclosed AT THE POINT OF QUOTE in both repeating
docs: claims §4 carries each caveat in the row class column
(:106-109) AND verbatim in §4.1 (:111-127); the matrix folds each
caveat INLINE into its cell (D2 at M-CTQ-04, D1 at M-CTQ-05, D3 at
M-CTQ-06, each naming the wire-authority receipt: ipc-03 :44,50;
ipc-04 :57; b06 :24,51). Verdict: **CLEAN — no laundering**.

## Probe 3 — run-1 min visibility vs flattering run-2

Run-1 min 3,529,029/s appears wherever the LSL/target is discussed:
claims row 30 (first column, run1 first), matrix M-CTQ-04 cell
(inline, both runs), XC-04 receipt :36, and adv-07 :94. NO doc
anywhere quotes run 2 (4.07-4.14M) without run 1 — grep-verified.
ADV-07's selective-minimum warning (:97-100) remains unexploited:
no post-addendum doc quotes only run 2. Historical figures
(b06 4.35M; lane-fix04 3.89M medians) coexist with their own
windows disclosed. Verdict: **CLEAN**.

## Probe 4 — receipt-chain integrity (XC-04 → claims §4 → matrix)

- XC-04 receipt (:1-117) is the source: commands+exits, both runs,
  positive controls, station-3 flip. Self-consistent.
- Claims §4 cites `lanes/lane-xc04-mctq-consolidation.md:36-37,
  47-49,51-52,66-67` per row — line cites verified against the
  receipt on disk. No self-citation: the claims file cites the
  receipt, not itself. §2's stale "zero measured facts/s" sentence (:100)
  is explicitly flagged as predating the addendum inside §4
  (:99-102) — disclosed, not silent.
- Matrix cells cite claims rows 30/31/31a/32 + the underlying lane
  receipts (b06/ipc-03/ipc-04) + XC-04 line numbers. No doc cites
  itself as evidence anywhere in the chain.
- R-16 errata (:73-93) correctly supersedes its own stale W3/W6
  ("NO consolidation rows added"), cites the XC-04 receipt and
  claims §4 + §4.1, and preserves W3/W6 history unmodified —
  ERRATA-never-rewrite law followed. This closes ADV-07's
  "ERRATA against R-16 NOT filed here" divergence :65-67.
- Attribution note: claims §4 is attributed "lane R-12"; R-16's
  errata describes the resumed promotion pass landing there. The
  chain reads consistently; no circular receipt found.
Verdict: **CLEAN — chain consistent, no circularity**.

## Findings (non-failing, for the record)

1. Master-table footnote [a] (matrix :22-23) still reads
   "UNVERIFIED-FOR-PAPER **pending consolidation**" — consolidation
   has now landed and is folded into the SAME file (:229-251). The
   wording is stale but conservative (it under-states standing);
   owner (R-07) may update to "consolidation receipted; wire legs
   UNVERIFIED" at will. Not a round-up.
2. ADV-07's SPOILER (dod-12:26 / dod-19:58 quoting 23.13 ms L2
   without the row-9-governs qualifier) is a DIFFERENT figure (L2
   chain, not M-CTQ); out of this lane's scope, remains open for
   its owner.
3. Dispatch premise "R-07's update may be mid-flight": FALSE on
   disk — the fold is landed. Dispatch premise "3.9-4.35M/s
   historical claims": referents exist (lane-fix04, b06) and are
   caveat-carried; no laundering found.

## Verdict summary

- Probe 1: CLEAN — no target-MET round-up on rows 30-32 evidence.
- Probe 2: CLEAN — numbers confined to 3 canonical docs; D1/D2/D3
  disclosed at every point of quote.
- Probe 3: CLEAN — run-1 min visible at every target/LSL discuss-
  ion; no run-2-only quote anywhere.
- Probe 4: CLEAN — chain XC-04 → claims §4 → matrix cites
  consistently; R-16 errata landed and lawful; no circular
  receipts.

No SPOILER-CONFIRMED verdicts this lane. Standing: ALIVE.
