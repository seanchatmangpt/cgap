# YD-5 Receipt: cgap v26.11.3 wave-index backfill (XD-3 finding)

Lane: YD-5 (index-only). Date: 2026-11-04. Repo: /Users/sac/cgap.
Constraint honored: no git commands run.

## Subject

- `docs/jira/v26.11.3/README.md` — the wave index (only index file in the
  dir; no RECEIPTS.md exists).

## Counts

- Receipt files in `receipts/` after this lane's receipt lands: **55**
  (`ls receipts/ | wc -l` = 55; was 52 at lane start).
- Table rows after backfill + self-index + mid-pass arrivals: **55**
  (awk row census; the 56th `^| [A-Z]` match is the `| Lane` header
  row).
- Rows pre-existing (cl-close-2, 2026-11-03): 6 (CG-1, GA-5, GA-5B,
  SA-4, AF-30, SA-5).
- Rows added this lane: **49** (46 backfill + 1 self-index YD-5 row +
  2 mid-pass arrival rows).
- Cross-repo pointer exclusions: **0** — every receipt in `receipts/`
  is a cgap-subject receipt; the e80-06/e80-07 rows hosted in
  claude-code-config-lsp are unrelated pre-existing placement notes,
  not exclusions from this count. The 55-row count closes over the
  receipts dir exactly; the self-index (the receipt indexing itself)
  is disclosed, not hidden.
- Mid-pass arrivals (disclosed): `yd-3-publish-lines.md` (01:15) and
  `yd-1-licenses-clarify.md` landed in `receipts/` from concurrent
  lanes while this lane ran; both were read from disk and indexed
  (YD-3 ALIVE, YD-1 ALIVE, per their own bodies) — the closure is
  against the post-arrival dir state, not the lane-start snapshot.
- `yd-*` receipts: **0 files exist** in `receipts/` as of 2026-11-04
  (the dispatch named xd-*/yd-* receipts landed since — on disk only
  xd-1 and xd-5 exist; no yd-* file was present to index; disclosed,
  not invented).

## Method

- Standing words re-read from each receipt BODY on disk this lane
  (grep for the standing vocabulary over all 52 files, then full reads
  of the standing sections for the ambiguous ones); UNSTATED recorded
  where a receipt carries no standing word — 4 files: ab-19
  (defers to its Verification Ladder), af-12, af-24, xc-9. Never
  invented.
- Gate/exit-record column transcribed from each receipt's cited
  commands + exits (cargo "N passed; 0 failed" lines or process exits,
  per the index's exit-record law).

## Standing distribution (as transcribed)

ALIVE-led rows dominate; PARTIAL rows: AB-4, AB-8 (crate-level), AF-29,
AF-34, GD-1 (overall); mixed disclosed rows: AB-1 (suite BLOCKED at
write time), AF-32 (1B run UNVERIFIED), AF-27 (scale claims
UNVERIFIED), RX-3 (honest DISQUALIFIED verdict on a green exit — never
rounded up). UNSTATED: 4 rows.

## Verification

- `ls receipts/ | wc -l` → 55 (real command, exit 0, run after the
  receipt + both mid-pass arrivals landed on disk).
- `awk` table-row census → 55 receipt rows; duplicate-label census → 0;
  row-label families map 1:1 onto the receipts dir (55 = 55).
- Index file re-read after edit; row set matches the file list.

## Standing

UNSTATED→ALIVE for the index edit itself: the edit is on disk and the
census commands above ran this session with the exits cited. Standing
of the INDEXED receipts is theirs, not this lane's.

## Falsifier

- `ls docs/jira/v26.11.3/receipts/ | wc -l` vs an awk row census of the
  README table — any drift reopens this lane.
