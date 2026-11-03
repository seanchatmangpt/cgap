# ADV-07 — VOC/CTQ numeric discrepancy & metric arbitrage (corrosion lane)

Date: 2026-11-02. Repo: /Users/sac/cgap, branch main. READ-ONLY lane:
zero bytes written outside this file. No git commands run. Every claim
below was re-read from disk this session; fail-closed logging (a probe
with no on-disk referent is [SKIP], never inferred).

## Standing: ALIVE (read-only audit; all commands exit 0/1 as listed)

## Probe 1 — unqualified "23.13 ms" in exec/marketing-facing docs

Full-repo grep `23\.13` over docs/ + README.md. Hits and verdicts:

- `dflss/02-voc-ctq-matrix.md:252-265` — QUALIFIED. The registered-
  conflict paragraph says explicitly "consolidated row 9 = 42.58 ms
  governs" and "the lane figure stays UNVERIFIED-FOR-PAPER until
  consolidated".
- `audit/dod-12-g2-decision.md:26,81,85,93,261` — UNQUALIFIED use:
  dod-12 adopts 23.13 ms as the operative L2 number ("NOT MET (23.13
  ms @1M vs <10 ms; 2.31x over)") citing only
  `lanes/lane-gap-07-g2-pareto.md:52` — no row-9-governs qualifier,
  no mention of the registered conflict.
- `audit/dod-19-shipgate-final.md:58` — UNQUALIFIED: the SHIPGATE
  final doc quotes "23.13 ms vs <10 ms" as the L2 state with no
  row-9-governs note.
- `lanes/lane-posh-02-batch.md:64` — lane receipt, self-referential,
  not exec-facing; qualified by its own receipt context.
- Exec/marketing surfaces clean: README.md, REVENUE_GENERATION_
  COMPENDIUM.md, CHALLENGER_SALES_REPORT.md, dflss/00-dmadv-rollup.md,
  dflss/01-project-charter*.md, position-paper-v2.md, paper-sip.md —
  ZERO 23.13 hits (grep exit 1 on each).

VERDICT: **SPOILER-CONFIRMED (narrow)** — not in exec-facing docs, but
in the G2 decision and the SHIPGATE final: two audit-class documents
carry the faster un-reproduced lane figure as the operative L2 number
without the registered-conflict / row-9-governs qualifier. This is a
live arbitrage channel: a reader of dod-19 inherits the favorable
23.13 figure and the registered conflict is invisible there.

## Probe 2 — R-07 registered-conflict paragraph + M-CTQ promotion caveats

- R-07's paragraph (02-voc-ctq-matrix.md:252-265) is INTACT: it states
  row 9 (42.58 ms, `11-measured-claims.md` §1 row 9, divan, 100
  samples, `waves/w3-l2-swar-fnv.md:26-28`) governs for the paper and
  the lane figure stays UNVERIFIED-FOR-PAPER. No corruption.
- M-CTQ-04/05/06 promotion state (matrix :229-251) — **STALE, both
  directions**. Each cell says "no row in 11-measured-claims.md
  (checked on disk this session)". That is now FALSE: rows 30, 31,
  31a, 32 landed in the §4 consolidation addendum
  (`11-measured-claims.md:86-127`, attributed "lane R-12,
  2026-11-02"). The cells cite the dossier figures (4.35M seals/s,
  85 ns/call, 4.52 ns/op) and do NOT carry the addendum's instrument-
  divergence caveats D1/D2/D3. Honest direction of the staleness: the
  cells UNDERSTATE (they hold UNVERIFIED-FOR-PAPER when consolidation
  evidence now exists), so this is not an overclaim — but the
  instrument caveats live only in the claims-file addendum; the
  matrix cells would mislead a reader post-promotion if promoted
  without importing D1/D2/D3 verbatim (row 31 is in-process extern "C"
  ≠ cgo 85 ns wire floor; row 32 is a Rust L1-hot loop ≠ ipc-04's Go
  harness 4.52 ns/op).
- Receipt-integrity finding: `lanes/lane-r16-mctq-promotion.md` states
  standing PARTIAL, "NO consolidation rows were added" (:29-34, :58),
  and its W7 names the exact unblock condition. On disk the §4
  addendum EXISTS, attributed to "lane R-12". R-16's receipt is
  therefore stale/superseded by a landing it predicted but did not
  make. ERRATA against R-16 NOT filed here (read-only lane; owner or
  coordinator must file). Logged as the divergence of record.

## Probe 3 — bare VERIFIED sweep (family vs master tables)

- Master table (matrix :14-20): CTQ-04 reads "VERIFIED [a]" — the
  [a] footnote (:22-23) carries "UNVERIFIED-FOR-PAPER pending
  consolidation". Not bare.
- Section headers: CTQ-04 header (:85-86) repeats the qualifier;
  CTQ-05 "GATE VERIFIED [b]" with re-scope receipt cite; CTQ-07
  (:169-170) honestly splits "VERIFIED by receipt" from "end-to-end
  re-run UNVERIFIED-this-session". No bare VERIFIED found on any row
  whose standing is UNVERIFIED-FOR-PAPER.
- Downstream cites: `09-ax-cpk-msa.md:19` and `10-ax-control-plan.md:27`
  quote 85 ns/call as the membrane floor WITHOUT the in-process-vs-cgo
  caveat — lawful, because the addendum's D1 (claims file :119-123)
  itself pins ipc-03's 85 ns as "the wire-level authority"; the two
  docs agree with the addendum, so family and master tables are
  CONSISTENT today.

## Probe 4 — "3.95M/s LSL" vs XC-04 run-1 min 3,529,029/s

[SKIP] The dispatch's premise figure does not exist on disk: grep
`3\.95|3,95` over docs/ returns only µs-class bench hits (3.957 µs
challenger_parquet) and unrelated flake-register rows. No doc declares
a 3.95M receipts/s LSL, so no doc can claim it met on consolidation
evidence. Nearest on-disk referents:
- CTQ-04 target/LSL = median >= 1,000,000 receipts/s (matrix :95);
  XC-04 run-1 min 3,529,029/s is above it, and no doc claims the
  1M LSL "met on XC-04 evidence" — the matrix CTQ-04 cell still rests
  on the lane-fix04 fresh runs (3,894,347 / 3,891,452 median).
- Arbitrage risk noted for the record: run-1 min < run-2 min by 15%
  across runs of the same harness; any future promotion that quotes
  only run 2 (4.07-4.14M) without run 1 would be selective-minimum
  laundering. The addendum correctly carries both runs (row 30).

## Divergences (dispatch vs disk)

1. Dispatch premise "3.95M/s LSL": no on-disk referent ([SKIP] above).
2. Dispatch premise "R-16's resume landed the consolidation rows":
   the rows landed attributed to lane R-12, while R-16's own receipt
   on disk still says no rows added. The rows are real (re-read at
   use time); the attribution chain is broken, not the numbers.
3. R-07 owns 02-voc-ctq-matrix.md; this lane did not edit it even
   though its M-CTQ cells are stale post-addendum. Owner action: fold
   rows 30/31/31a/32 + D1/D2/D3 into the M-CTQ-04/05/06 cells.

## Verdict summary

- Probe 1: SPOILER-CONFIRMED (narrow — dod-12:26, dod-19:58).
- Probe 2: R-07 paragraph INTACT; M-CTQ cells STALE (understate);
  R-16 receipt superseded by R-12's addendum on disk.
- Probe 3: CLEAN — no bare VERIFIED; family/master tables agree.
- Probe 4: [SKIP] no 3.95M LSL referent; no met-on-consolidation
  claim exists to arbitrage.

Lane standing: ALIVE (read-only; greps + reads this session, exit
codes in the commands above: grep exit 1 = zero hits, reported as
evidence, not failure).
