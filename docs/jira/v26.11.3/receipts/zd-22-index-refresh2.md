# ZD-22 Receipt: receipt-index second refresh (yd-6, yd-7, zd-1..zd-4)

- Date: 2026-11-04
- Repo: /Users/sac/cgap
- Lane: ZD-22 (create/update agent, index-refresh2)
- Owned files: docs/jira/v26.11.3/README.md (index rows + refresh note
  only) and this receipt (new). No git commands run; nothing committed.
- Subject of the refresh: the wave index
  `docs/jira/v26.11.3/README.md` receipts table, second pass after
  YD-5's 55-row pass.

## Scope

Six receipts landed since the yd-5 pass; all six indexed this pass:

- yd-6-buffer-too-short.md — standing UNSTATED (no standing-vocabulary
  word in body; the Verification section reports green runs).
- yd-7-dod-rerun.md — standing UNSTATED as an overall word (body
  carries per-gate MEASURED ×11 / RED ×1, not an overall word).
- zd-1-dod10-manifest-red.md — ALIVE (Status header).
- zd-2-fa9-race-grid.md — ALIVE (Standing section: "ALIVE for the
  court"; lib unregressed 58/58).
- zd-3-alloc-scope.md — ALIVE (Status header: green release + debug).
- zd-4-dod-ontology-f2.md — ALIVE (Status header).

Verified already indexed from YD-5's mid-pass absorb (re-read in the
table this pass): yd-1-licenses-clarify.md, yd-3-publish-lines.md,
yd-5-cgap-index.md (self-index). xc-9, xd-1, xd-5 also verified
already present. xd-* census: both xd-1 and xd-5 indexed; no unindexed
xd-* file remains in `receipts/`.

zd-8: no `zd-8*` file exists on disk at this pass (`ls receipts/ |
grep zd-8` → no match, exit 1). No zd-8 row written — never a row for
an absent file. If zd-8 lands later, it is a future refresh's row.

Standing words re-read from each receipt BODY on disk at this pass,
never from memory. UNSTATED where the body carries no
standing-vocabulary word.

## Census / verification (commands + outputs, this session)

- `ls docs/jira/v26.11.3/receipts/ | wc -l` → **61** at pass start;
  **62** after this receipt landed.
- Receipt rows in the README table (awk `/receipts\//` over table
  lines matching `^\| [A-Z]`) → **62** including this receipt's
  self-row (YD-5 self-index precedent: the receipt indexes itself);
  the concurrent zd-19 receipt then landed mid-pass and was absorbed
  (standing re-read from its body), final closure 63 = 63 (dir =
  table).
  Disclosed pointer-only rows: none in this table (e80-06/e80-07 are
  hosted in the claude-code-config-lsp index and counted there, per
  the Placement notes — outside both numbers).
- Duplicate lane-label census: `awk -F'|' '/^\| [A-Z]/ {print $2}'
  README.md | sort | uniq -d` → empty (0 duplicates).
- zd-8 absence: `ls receipts/ | grep zd-8` → exit 1 (no match).
- Edits confined to README.md rows + refresh note; table structure,
  Placement notes, See Also untouched (a pre-existing duplicated
  "## Placement notes" heading at lines ~83-85 predates this lane and
  was left as-is — index rows are this lane's scope).

## Standing

ALIVE for the index refresh itself: the six rows were written this
session against on-disk receipt bodies, and every census command above
was executed this session with its real output recorded. The indexed
receipts carry their own standings (re-read, not re-run); this lane
ran no cargo/ggen commands against the crates — re-execution of the
indexed gates belongs to those lanes' receipts.
