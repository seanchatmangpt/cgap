# ZD-57 — index refresh 3 (six new receipt rows + af-43 row refinement)
Standing: PARTIAL

Lane: ZD-57 (index-only). Date: 2026-11-04. Repo: `/Users/sac/cgap`.
Owns: `docs/jira/v26.11.3/README.md` (rows + refresh paragraph +
Placement notes) and this receipt. No source files, no git operations.

## Method (zd-22 precedent)

Census-first: `ls docs/jira/v26.11.3/receipts/` (69 files at pass
start), diffed against the README table's receipt rows (63 at pass
start). Six on-disk files had no row: zd-6, zd-13, zd-26, zd-36,
zd-43, zd-55. Standing for each row was re-read from the receipt body
on disk this pass, never from memory; UNSTATED where the body carries
no standing-vocabulary word (ALIVE / PARTIAL / BLOCKED / UNVERIFIED /
REFUSED).

## Rows added

| receipt | standing read from body |
|---|---|
| zd-6-w2a-session-ffi.md | ALIVE (Status header) |
| zd-13-w2c-sidecar.md | ALIVE (Status line) |
| zd-26-frozen-drift.md | ALIVE (Standing section) |
| zd-36-dod-final.md | UNSTATED overall (per-gate MEASURED ×10 / REFUSED ×2; yd-7 precedent) |
| zd-43-dryrun-ladder.md | PARTIAL (body header, verbatim "7/20") |
| zd-55-dod-commands.md | UNSTATED overall (Standing section: dod-011/012 MEASURED; skip disclosed) |

af-43's existing row was refined, standing re-read from its body's
Standing section this pass: threat-model content UNVERIFIED (AF-17
reconstruction) and CBMQS G2 fail-honest open defect are in the body
but were absent from the row — disclosed drift, now surfaced.

## Absent files — no rows, disclosed

`ls receipts/` at pass start carries no zd-8, zd-53, zd-56, or xd-16
file (grep → no match each). No rows written for absent files. zd-42
exists nowhere in this tree — cross-repo pointer to the arcus tree,
disclosed in README Placement notes, not counted. All five yd-*
receipts (yd-1/3/5/6/7) verified already rowed by diff at this pass,
not from memory.

## Coverage loop closure

- `ls receipts/ | wc -l` at pass start: **69**.
- README receipt rows before this lane: **63**. After: **70**
  (63 + 6 new + this receipt's self-indexed row, YD-5 precedent).
- Post-write census: `ls receipts/ | wc -l` = **70** (69 + this
  receipt); table rows = 70. Count closes over the dir exactly.
- Pointer-only rows (e80-06/e80-07 in claude-code-config-lsp; zd-42
  in arcus) are not counted in either number, per the zd-22 note.
- Duplicate-lane-label check: `grep -o '^\| [A-Z0-9-]* \|' README.md`
  census — no duplicate lane labels (uniq -d → empty).

## Repairs carried

- Removed the duplicated `## Placement notes (pointer-only)` heading
  (lines 111/113 pre-existing) — one heading now.

## Standing

ALIVE (this row is the refresh's own receipt; the census commands
above ran this session against the exact tree; README edits verified
on disk by the Edit tool's byte-level application).
