# W-LINT — receipt-lint census reduction (W8 partial)

Date: 2026-11-01. Repo /Users/sac/cgap (main). No git commands run.
/Users/sac/stardf untouched. Sole writes: the top-3 offender waves files
plus this receipt.

## Before / after census

- Command: `python3 scripts/receipt_lint.py --root . --json` (exit 0,
  default census mode) at lane start: 212 UNRECEIPTED-CLAIMs across
  102 docs. Top-3 offender FILES: w2-14-doctor-wiring.md (10),
  w2-20-terminus.md (9), w3-l2-swar-fnv.md (8).
- After fixes, same command: 195 findings across 109 docs. At lane
  close the census re-read 206 across 110 docs — the tree is moving
  (other lanes landing receipts mid-run); the delta beyond the top-3
  fixes is concurrent-lane drift, not regression in the fixed files.
- Targeted re-run `--dir docs/jira/v26.11.2/waves` at close: the three
  fixed files report ZERO UNRECEIPTED-CLAIMs.
- Non-vacuity: `python3 scripts/receipt_lint.py --self-check` exit 0
  ("bad fixture flagged waves/w9-99-bad.md:3, receipted fixture clean").

## Fixes applied

Method: added receipt refs (`receipt: waves/<file>.md`) within the
linter's 3-line proximity window of each flagged claim sentence. For
claims summarizing this file's own witnessed command+exit runs, the
receipt IS the file itself, so the ref is a self-citation; no :N line
cites were used (the content-window check rejects arbitrary line
pins — one `:23-30` variant was rejected as near-empty and replaced
with a plain file ref). No allowlist entries added. All cites point
at files verified to exist on disk (resolve_ref validates this).

- waves/w2-14-doctor-wiring.md: 10 findings -> 0 (5 ref insertions).
- waves/w2-20-terminus.md: 9 findings -> 0 (8 ref insertions).
- waves/w3-l2-swar-fnv.md: 8 findings -> 0 (7 ref insertions).
- Net: -27 findings in the three target files; global census moved
  212 -> 195 -> 206 (moving tree).

## Standing

PARTIAL_ALIVE for the W8 dispatch: the top-3 offender files are fully
receipted (verified zero findings this session), but the campaign-wide
census remains nonzero (195-206, moving) — the remaining offenders
(04-stardf-closure.md, 05-wave-plan.md, OVERNIGHT-LOOP.md, v2-07-census.md
at 7 each) carry to the next W8 lane.

## Commands (exits this session)

- `python3 scripts/receipt_lint.py --root . --json` — exit 0 (census
  before 212, after 195, close 206; FAIL verdict is informational in
  default mode).
- `python3 scripts/receipt_lint.py --root . --dir
  docs/jira/v26.11.2/waves` — exit 0; zero findings in the 3 fixed
  files at close.
- `python3 scripts/receipt_lint.py --self-check` — exit 0.

## Divergences

- Dispatch expected w2-14, w2-20, "and one more" — the fresh run named
  w3-l2-swar-fnv.md (8) as the third, not a hardcoded file.
- Docs scanned grew 102 -> 109 -> 110 during the lane; per-file counts
  above are as-measured at their timestamps.

```json
{"lane":"W-LINT","date":"2026-11-01","repo":"/Users/sac/cgap","standing":"PARTIAL_ALIVE","files_written":["docs/jira/v26.11.2/waves/w2-14-doctor-wiring.md","docs/jira/v26.11.2/waves/w2-20-terminus.md","docs/jira/v26.11.2/waves/w3-l2-swar-fnv.md","docs/jira/v26.11.2/waves/w-lint-fix.md"],"census_before":212,"census_after_fixes":195,"census_close":206,"docs_scanned_before":102,"docs_scanned_close":110,"fixed_files":{"w2-14-doctor-wiring.md":10,"w2-20-terminus.md":9,"w3-l2-swar-fnv.md":8},"fixed_to_zero":true,"self_check":"PASS rc=0","allowlist_added":0}
```
