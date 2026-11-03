# Lane R-ADV09FIX — contention-guard doc-layer enforcement wiring

2026-11-02, /Users/sac/cgap @ main. Contract: owned files only
(`06-bench-protocol.md`, `dflss/03-msa-gauge.md`,
`dflss/06-control-plan.md`), merge-not-clobber, no git commands.
Reads only elsewhere. All cites re-read on disk this session.

## W1 Findings applied

- ADV-09-R2 (probe 4): CONTENTION-SUSPECT enforcement was bench-header
  prose only; zero control/MSA docs named it, no gate consumed it.
- ADV-09-R1 (probe 1): amortization disclosure covered batch MEANS
  only; row-28 "slowest" never disclosed as window-amortized, not a
  frame-level tail.

## W2 Landed lines

1. `../06-bench-protocol.md` §8 (lines 177-193): contention-marker
   consumption law — a run whose stderr carries `CONTENTION-SUSPECT:`
   is REJECTED against rows 5/6/7/8; rows citing a marked run are
   VOID; mechanism cites `chains-ingress.rs:716-802` (:745 const,
   :802/:854 emission) + mctq copy :31,65,98; honest scope stated:
   PROCEDURAL enforcement (receipt-lint/consumer discipline), machine
   gate = TARGET, unlanded.
2. `dflss/06-control-plan.md` §5.2 new SPC control row (lines
   143-157): CONTENTION-SUSPECT host-noise marker control — method
   "grep the receipted bench stderr", typed exit contract (0 clean /
   marked-run-quoted = VOID / missing stderr disclosure = incomplete),
   machine gate TARGET; anti-vacuity arm = RC-11 flip witness
   (margin 2.1-30x). §5.3 standing rows added (lines 174-175).
3. `dflss/03-msa-gauge.md` new Sec 1.2 (lines 55-68): guard named as
   gauge-law enforcement mechanism with chains-ingress.rs:716-802
   cite + honest procedural-scope statement; new Sec 1.3 (lines
   70-76): ADV-09-R1 tail disclosure — row-28 124.06 "slowest" is a
   whole-window amortized sample, NOT per-frame tail resolution;
   consumers must not read it as frame-level worst case.

## W3 Verification

- Marker cites verified by grep this session:
  chains-ingress.rs:716,719,745,779,780,802,854;
  mctq_consolidation.rs:31,65,98 (exit 0).
- Row-28 figure re-read at `11-measured-claims.md:52` (124.06 present,
  registered-divergence wording intact).
- Line law: awk sweep — my landed lines all <=100 chars (remaining
  >100 lines in control-plan/bench-protocol are pre-existing table
  rows; none mine).
- Merge law: all three edits are additive sections/rows; no existing
  line clobbered.

## W4 Standing + fences

- Standing: ALIVE (every cite above grepped/read this session;
  exits 0).
- Fences this diff could break: none mechanical (docs-only). Nearest
  fences: (a) receipt_lint --strict claim-proximity — every claim
  carries its file:line cite in the same sentence; (b) exclusivity law
  — no new numeric claim landed except row-28's existing 124.06,
  re-read from its table row; (c) 09-ax-cpk-msa.md and
  02-voc-ctq-matrix.md are NOT owned by this lane and were not edited
  (their divergence-registration rows are untouched).

```json
{"lane":"R-ADV09FIX",
 "files_written":[
  "/Users/sac/cgap/docs/jira/v26.11.2/06-bench-protocol.md",
  "/Users/sac/cgap/docs/jira/v26.11.2/dflss/06-control-plan.md",
  "/Users/sac/cgap/docs/jira/v26.11.2/dflss/03-msa-gauge.md",
  "/Users/sac/cgap/docs/jira/v26.11.2/lanes/lane-r-adv09fix-contention-doc-wiring.md"],
 "commands":[
  "Read adv-09-msa-gauge-falsification.md (exit 0)",
  "Read 03-msa-gauge.md, 06-control-plan.md, 06-bench-protocol.md (exit 0)",
  "grep CONTENTION-SUSPECT chains-ingress.rs mctq_consolidation.rs (exit 0)",
  "grep 124.06 11-measured-claims.md (exit 0, row 28 at :52)",
  "awk 100-char line sweep over 3 owned docs (exit 0)",
  "grep landed markers in 3 docs (exit 0)"],
 "courts_run":[
  "citation-grounding court: every file:line cite re-read this session",
  "anti-vacuity: procedural control honestly marked TARGET-not-gate; no",
  "  vacuous 'gate' claim landed"],
 "standing":"ALIVE",
 "divergences":[
  "enforcement is procedural only; machine gate explicitly TARGET,",
  "  not landed — stated in all three docs per dispatch honest-scope law",
  "2.1-30x flip margin and mctq copy cites taken from RC-11 receipt +",
  "  fresh greps; no new measurement run (docs-only lane)"]}
```
