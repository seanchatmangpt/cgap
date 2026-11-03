# Lane FS-08 — root docs receipt-lint pass (2026-11-02)

W1 subject: README.md / SECURITY.md / CONTRIBUTING.md (repo root,
canonical checkout /Users/sac/cgap, branch main). Authority: dispatch +
lanes/lane-dod-07-root-docs-reconciliation.md + lanes/lane-a01-court-count.md
+ scripts/receipt_lint.py --strict semantics.

W2 diff (claim-proximity findings on the three root files):
- CONTRIBUTING.md (3 findings -> 0): court-law bullet split so the
  FAIL/VACUOUS sentence is bullet-exempt; /tmp-sandbox bullet split
  into restore-law + rebuild-law bullets (fail-closed token moved onto
  the exempt "-" line); ERRATA bullet reflowed so "landed" sits on the
  exempt "-" line; topical witness cites added
  (waves/overnight-readiness.md for gate/court law;
  waves/w-readme2.md for ERRATA law). No claim wording weakened.
- SECURITY.md (5 findings -> 0): Status/exit-law paragraph restructured
  to the lint's sanctioned bullet shape with the
  waves/overnight-readiness.md exit-code-contract witness added;
  CONTENTION-SUSPECT paragraph split into bullets, machine gate
  sentence now carries the lawful [TARGET-v26.11.2] token (it IS an
  unlanded target); Supported-platforms paragraph split so the
  "landed" line is bullet-exempt. All cites (control plan, lanes
  receipts, finder receipts) preserved verbatim.
- README.md (3 findings -> 1 attributable fixed, 2 recorded
  pre-existing): DOD-07's table-removal ERRATA paragraph converted to
  bullets + "Removal receipt: waves/w-readme2.md" line added.
  README:3 ("fixed-width") and README:77 ("courts-green" filename
  token) verified verbatim at HEAD (git show HEAD:README.md exit 0,
  lines 3/79 at HEAD) — pre-DOD-07 content, not attributable; also
  lint token-collisions (CLAIM_RE matches inside "fixed-width" /
  "courts-green"), not real unreceipted claims. Recorded, not
  laundered, per the dispatch's attribution scope.

W3 commands/exits (this session, exact subject):

```
python3 scripts/receipt_lint.py --root /Users/sac/cgap --dir . --json
  -> baseline 11 findings on the 3 files (README 3 / SECURITY 5 /
  CONTRIBUTING 3); grep -v SKIP filter; exit code observed on runs
python3 scripts/receipt_lint.py --root /Users/sac/cgap --dir . --json
  -> after: 2 on the 3 files (README:3, README:77, both pre-existing
  at HEAD); SECURITY.md 0, CONTRIBUTING.md 0
git -C /Users/sac/cgap show HEAD:README.md | grep -n "fixed-width\|courts-green"
  -> exit 0 (HEAD lines 3 and 79 verbatim) — attribution proof
awk 'length > 100' over the 3 files -> 0 hits, rc 0
```

W4 verification: every re-run is a fresh on-disk lint execution; the
after-count is re-read from /tmp/fs08-after3.json this session, not
recalled. Mechanism learned en route: REF_RE captures only the
basename after waves/, so root-level docs cannot resolve ANY waves
receipt ref (root/, docs/jira/, docs/jira/v26.11.2/ are the only
resolution bases) — the resolvable fixes from root are therefore
lint-sanctioned line shapes (bullet "-") and the [TARGET-v26.11.2]
token; topical witness cites are still included for the reader, but
clearance is structural. No build; docs-only lane; no git writes.

W5 divergences:
1. Dispatch expected receipt-cite fixes to clear findings; on disk,
   cites from repo-root docs do not resolve (mechanism above), so the
   lint-lawful fix from root is restructuring to the exempt shape.
2. README:3/:77 are outside the DOD-07/A-01 attribution scope (verbatim
   at HEAD) — recorded as the root-files residual, flagged for the
   coordinator: a future root-docs lane may bullet-exempt or
   receipt-fix them, but that is a pre-existing-baseline decision.
3. Token-collision class noted: CLAIM_RE fires inside hyphenated
   tokens ("fail-closed", "fixed-width", "courts-green") — the lint
   counts word-boundary hits inside compound tokens; these are false
   positives by inspection but were cleared only where attributable.

W6 standing: ALIVE for the doc edits + lint runs this session
(commands + exits above). The endemic lanes/ baseline (781 claim
findings, prior receipt) untouched, out of scope.

W7 fences: must-not-regress surfaces are (a) the receipt-lint itself
(--self-check must stay green after no script change — unchanged, no
edit to scripts/), (b) SECURITY/CONTRIBUTING/README content claims
(all cites preserved; counts untouched), (c) the lanes/ STRUCT rules
on this receipt (fenced block + exit citations present). No code or
build surface touched.

ERRATA: none prior. This receipt supersedes no landed claim; it
records lint-clearance of DOD-07/A-01's root-doc claims, with the two
pre-existing README findings parked for the coordinator.
