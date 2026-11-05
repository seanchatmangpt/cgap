# BR-2 Receipt: Cross-Repo Citation Resolver Lint (seam S6)
Standing: ALIVE

Lane: BR-2 IMPLEMENTATION. Date: 2026-11-03. Repo: /Users/sac/cgap.
Seam: S6 (cross-repo citation resolution, AF-41 S6 bare-shorthand class).

## Subject

- `scripts/receipt_lint.sh` (new, executable, bash 3.2-compatible, stdlib-only
  core via embedded python3; never runs git).
- `scripts/receipt_lint_fixtures/cite-fixtures/{known-good-fullpath.md,
  known-shorthand.md,missing-cite.md}` (anti-vacuity fixture court).
- This receipt.

## Behavior

Lints cross-repo receipt citations in `docs/jira/*/receipts/*.md` and
`docs/jira/*/closure/*.md` (closure optional, never silent-skipped: zero scan
roots is fail-closed exit 2). Classes:

- **PASS** — full path `/Users/sac/<repo>/docs/jira/vX/receipts/name.md` exists → no row.
- **WARN-SHORTHAND** — `<shorthand> receipts/name.md` resolves only after
  expansion to `/Users/sac/<repo>/docs/jira/v<release>/receipts/name.md`
  (newest release dir containing the file): WARN with the corrected path, exit 1.
- **FAIL-NOT-FOUND** — shorthand cite resolves nowhere: typed FAIL, exit 1.
- Same-repo bare `receipts/x.md` (no shorthand token on the line) is
  local-relative and intentionally not linted.

Repo map: cgap→cgap, lsp-max→lsp-max, ggen→ggen, arcus→arcus,
marketplace→ggen-marketplace. Exit codes: 0 clean / 1 findings / 2 fail-closed
(no scan roots, unreadable file arg) / 3 VACUOUS self-check.

## Gates (all run 2026-11-03, exact subject)

| Gate | Command | Result |
|---|---|---|
| Syntax | `bash -n scripts/receipt_lint.sh` | SYNTAX-OK |
| Anti-vacuity court | `bash scripts/receipt_lint.sh --self-check` | PASS: known-good→0, known-shorthand→1 (WARN-SHORTHAND with corrected path), missing-cite→1 (FAIL-NOT-FOUND); exit 0 |
| Known-good full-path corpus | `bash scripts/receipt_lint.sh /Users/sac/arcus/docs/jira/v26.11.3/cbmqs-standard.md` | CLEAN, exit 0 |
| Known bare-shorthand corpus (exit-1 witness) | `bash scripts/receipt_lint.sh /Users/sac/arcus/docs/jira/v26.11.3/cscb-wave-index.md /Users/sac/arcus/docs/jira/v26.11.3/bare-metal-transit-ledger.md` | 56 WARN-SHORTHAND rows (42 wave-index + 14 transit-ledger), every expansion resolved to a real on-disk file, exit 1 |
| Default scan | `bash scripts/receipt_lint.sh` (cgap docs/jira/*/receipts + closure) | 1 WARN (`receipts/rx-4-profile-guard.md:74` shorthand self-cite, resolves to itself), 0 FAIL, exit 1 |

## Broken cites found

None. Zero FAIL-NOT-FOUND rows across the ledgers and the default scan, so no
`CITE-FIX` correction lines were appended (nothing met the FAIL class; the
task's fix clause is vacuously satisfied, stated loudly rather than silently).

## Defects found and fixed during construction (in the lint itself)

1. Double-join: cgap's `root` already contains the repo dir; joining `cgap`
   again produced `/Users/sac/cgap/cgap/docs/jira` (silently degraded shorthand
   cites to the FAIL class). Fixed via `sac_base()` parent derivation.
2. Missing `.md` in the expansion join (`receipts/<name>` without extension),
   which would have made every shorthand cite FAIL even with correct paths.
   Both were caught by the fixture court refusing to flip the WARN arm — the
   court that cannot flip is FAIL, so it was treated as a lint defect, not a
   fixture defect.

## Standing

ALIVE — lint executed this session on the exact landed subject and the real
cross-repo corpora, all five gates observed with exits recorded above.
Anti-vacuity: the exit-1 arms are witnessed on both fixtures and on the real
bare-shorthand corpus (56-row wave), so the gate is non-vacuous on real data,
not only on constructed fixtures.

See Also: `scripts/receipt_lint.py` (claim-sentence linter, W2-3/RC-09),
`docs/jira/v26.11.3/receipts/ab-6-header.md`, AF-41 S6.
