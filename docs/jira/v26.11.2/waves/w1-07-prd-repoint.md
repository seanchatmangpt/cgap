# W1-07 PRD repoint receipt

Lane W1-07 (lane-runner), 2026-11-01. Repo /Users/sac/cgap.

Subject: docs/jira/v26.11.2/00-ard-prd.md (edits only) + this receipt.

## Diff summary

- §4 rewritten: stale G1–G8 one-liners replaced with on-disk gate names and
  definitions transcribed from 01-release-gates.md — G1-PUBLISH, G2-PARETO,
  G3-MAT-BENCH, G4-DRED, G5-SILICON, G6-NO-DF, G7-PATENT, G8-STARDF-F1 —
  each carrying its threshold numbers and FLIP mutation. Exit-code contract
  (0/1/2/3, loud [SKIP]) appended.
- §6 rewritten: pointer index now lists the on-disk file set (01, 02, 03, 04,
  05, OVERNIGHT-LOOP.md, HANDOFF.md) plus the landing set (06–09, being
  written by sibling lanes, marked do-not-block-on) and a research/
  subsection listing g01–g20 with one-line hooks read from each file's
  first H2.
- §1–§3, §5 untouched. Charter language otherwise intact.

## Verification commands (this session)

- `awk 'length > 100 {print FILENAME":"FNR}' 00-ard-prd.md` — exit 0, zero
  over-length lines (file is 135 lines).
- Read-back of both edited sections confirmed on disk (Edit tool succeeded;
  line-length sweep above is the on-disk check).
- research/ hooks transcribed from live `head -15 | grep -m1 '^## '` output
  over all 20 files, same session.

## Divergences

- Old §4 referenced 02-bench-protocol/03-court-corpus/04-registry-sweep/
  05-overnight-mutation-set numbering; the actual on-disk numbering is
  02-pareto-levers / 03-ost-design-around-register / 04-stardf-closure /
  05-wave-plan, with 06–09 as the in-flight landing set. §6 now matches disk.
- 06–09 were not yet on disk at edit time; referenced as the landing set per
  dispatch instruction, not verified present.

{"lane":"W1-07","files_written":["docs/jira/v26.11.2/00-ard-prd.md","docs/jira/v26.11.2/waves/w1-07-prd-repoint.md"],"commands":["awk length sweep over 00-ard-prd.md (exit 0, 0 over-length lines, 135 lines)","head/grep hook extraction over research/g01-g20"],"courts_run":[],"standing":"PARTIAL_ALIVE","divergences":["on-disk numbering of 02-05 differs from old PRD pointers; PRD now matches disk","06-09 referenced as landing set, not yet on disk at edit time"]}
