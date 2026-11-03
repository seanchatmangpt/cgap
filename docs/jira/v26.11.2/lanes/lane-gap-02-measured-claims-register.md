# Lane GAP-02 — measured-claims master register alignment (merge-audit)

Subject: `docs/jira/v26.11.2/11-measured-claims.md` (sole owned file).
Date: 2026-11-02. Expected outcome MINIMAL edits — achieved: ONE edit.

## Per-item status

1. **Rows 5 + 28 (quiet-host ingress / contention split)** —
   ALREADY-PRESENT. Both rows carry the red-10 receipted figures
   (10.20/10.30 quiet; 14.71/19.03/124.06 contention) and cite
   `redteam/red-10-timing-jitter-exploit.md`. No R-15 sweep receipt and
   no fix-18 sweep receipt exist on disk (`ls redteam/` + `ls lanes/ |
   grep` run this session: no fix-18, no r-15 lane file), so there is
   no fresher N=7 citation to register. Numbers NOT overwritten —
   correct per dispatch law (only a receipt naming the run may move
   numbers).
2. **ROI/core-hour/RAM TARGET-fencing** — ALREADY-PRESENT (by absence).
   grep for `ROI|core-hour` over the register: 0 matches. No unbacked
   ROI/core-hour claim exists inside 11-measured-claims.md; the header
   exclusivity law (lines 3-10) already forbids any number absent from
   the table. No edit needed; nothing to add.
3. **Control-plan §5.2 cross-link** — ADDED. §4's cross-instrument-legs
   note now points at the CONTENTION-SUSPECT enforcement row
   (`dflss/06-control-plan.md:143-157`, landed by
   `lanes/lane-r-adv09fix-contention-doc-wiring.md:25-26` — verified on
   disk this session).
4. **Diff discipline** — merge-audit held: no duplicate re-insertion.
   Note: `git diff --stat` on the file shows 52+/4- because the file
   carried PRE-EXISTING uncommitted modifications at session start
   (git status snapshot: `M docs/jira/v26.11.2/11-measured-claims.md`);
   THIS session's delta is only the 5-line pointer block.

## Commands / exits

- `ls redteam/` + lanes grep — exit 0 (no fix-18 / R-15 receipts).
- `grep -n "CONTENTION-SUSPECT" dflss/06-control-plan.md` — exit 0,
  rows at 143-157 + 173-174.
- `awk length>100` sweep — exit 0; all >100-char lines are
  PRE-EXISTING table rows (25-64, 106-109); added lines ≤100.

## Standing

ALIVE for the audit (all three items executed against the exact subject
this session). No courts run — this lane is documentation-only; there
is no anti-vacuity mutation to flip (a court that cannot exist is not
FAIL; the falsifier "register contains an unbacked ROI claim" was
checked and is empty).

## Fences

Must-not-regress: header exclusivity law (lines 3-10) and §2's
classification statements — untouched. FIX-06/R-16/R-12 reconciliation
edits — untouched. Section-5 (control plan) citation verified live on
disk before adding the pointer.

## Divergences

Dispatch said "fix-18's sweep receipt if present" — it is ABSENT; no
edit was fabricated for it. R-15's N=7 numbers likewise have no on-disk
receipt naming them; register unchanged on rows 5/28.
