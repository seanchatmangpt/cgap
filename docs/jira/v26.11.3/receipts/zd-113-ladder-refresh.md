# ZD-113 — crates-io-ladder-plan §3.2/§5 refresh (post-dry-run reconciliation)
Standing: VERIFIED

Date: 2026-11-04. Subject: /Users/sac/cgap @ 7459848b3b274399d783352019a30452b777c063.
Owned files: `docs/jira/v26.11.3/plans/crates-io-ladder-plan.md` (edited),
this receipt (new). No commit made (per task). No git transitions.

## Trigger

The ZD-43/ZD-97 dry-run reality superseded the plan's predictions:

- §3.2's predicted Stage-1 blocker ("license-file outside package
  root") NEVER fired across 20 dry-runs (ZD-43 @ 37721e2f: 7 green
  full-package builds including the license-file-declaring crates;
  ZD-97 @ 9582131: the 2 dirty-worktree crates re-ran clean-tree, both
  only registry-dep-missing).
- The dominant failure class was registry-dep-missing — 13 crates
  self-healing at Stage 3 publish order (11 in ZD-43 + 2 in ZD-97).

## Edits made (plan file)

1. **§0 baseline**: struck the 17-crate YP-1 count and reconcile note;
   pinned **21 publishable crates** with the full list (see below).
2. **§1 license-file section**: struck as a Stage-1 gate, re-headed
   "FALSIFIED as a gate (historical record)", annotated with ZD-43/ZD-97
   evidence; retained the copy-LICENSE.md recipe only as a contingency
   note; kept the SPDX ban (P28-SIL URN law) intact.
3. **§2 command sequence**: re-verified the sequence still matches
   (Stage 0 preflight, topo-order dry-run ladder, bounded polls —
   unchanged); marked Stage 1 STRUCK; marked Stage 2
   COMPLETE-VERIFIED (20/20 accounted); restated Stage 3 admission gate
   (7 green dry-runs + topo order + bounded registry polls); ladder
   census updated 17 → 21.
4. **§4 receipt shape**: `dry-run exits ×17` → `×21`; dropped the
   license-fix commit SHA field (no license fix will land).
5. **§5 UNVERIFIED details**: refreshed — both items (path-only
   dev-dep stripping, docs.rs rustc-args) remain UNVERIFIED, re-test at
   Stage 3; added a falsified list (license blocker, census counts).
6. **Header standing**: "UNVERIFIED as a plan" → Stage 2
   COMPLETE-VERIFIED, nothing published.

## Publishable count: 21, pinned

Live re-derivation this session, `cargo metadata --no-deps
--format-version 1` at HEAD 7459848: 25 workspace members, 4 excluded
via explicit `publish = []` (wire-shm, cgap-beam, bench, cgap-court) →
**21**. Supersedes plan-history 17 (YP-1) / 20 (ZD-43 task list) /
"22" (ZD-43 pre-sweep census, corrected to 21 in its ZB-16 addendum).
The 21: cgap, cgap-arrow, cgap-aoq, cgap-blake3, cgap-crypto, cgap-core,
cgap-gate, cgap-branchless, cgap-storage, cgap-git, cgap-bn3,
cgap-parquet, cgap-p28, cgap-ffi, cgap-audit, cgap-gpu, cgap-integration,
cgap-bont, cgap-bosym, cgap-fibo, cgap-cloud.

Note: ZB-16's recorded count was at e3233191; this receipt re-derives it
live at 7459848 — same 21.

## Verification

- Command: `cargo metadata --no-deps --format-version 1 | python3
  census` — exit 0, members: 25, publishable: 21, excluded list
  matches. (Ran twice: once crates/-filtered, once full-membership.)
- Edits confirmed on disk (Edit tool success against exact prior
  strings; file state tracked current).
- Standing: plan refresh ALIVE; Stage 3 itself remains UNEXECUTED —
  nothing published.

## Replay

```
cd /Users/sac/cgap && git rev-parse HEAD   # 7459848...
cargo metadata --no-deps --format-version 1 | jq -r
  '.packages[] | select((.publish // ["x"])[0] != false) | .name' | sort
```
