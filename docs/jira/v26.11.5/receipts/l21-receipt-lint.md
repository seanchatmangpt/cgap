# L21 — Receipt Standing Lint Port to cgap

Standing: PASS

## Subject

Port of ggen `scripts/receipt_lint.py` (Gate 3, receipt-standing-law; commits
069d13bc3/32b83765e) to cgap as `scripts/receipt_standing_lint.py` + `just
receipt-standing-lint` gate, plus the D108-pattern backfill census over
`docs/jira/v*/receipts/`.

## Census (from real lint output, not memory)

Pre-backfill run: `python3 scripts/receipt_standing_lint.py` →
`166 files, 29 parse OK, 137 FAIL` (exit 1), standing-map hash
`a18ba9a0fceb2904`.

Failure split: 112 files with no parseable standing key line; 25 files with a
key line whose value was out-of-vocabulary (`'**BLOCKED**'`, `'ALIVE:'`,
`'NON-CONFORMANT'`, `'cargo'`, `'bash'`, …).

Backfill per D108 (quote only vocabulary already present in the file; else
UNVERIFIED; one Standing line per file; placed after the H1):

- 112 files: `Standing: <token>` inserted after the H1, token = an exact
  vocabulary word already present in the file body (e.g. ALIVE, PASS,
  BLOCKED). 3 of these had no vocabulary token anywhere → `Standing:
  UNVERIFIED`.
- 25 files: the EXISTING key line's value normalized in place to the exact
  vocabulary token already present in the line or file (no second line
  inserted) — 8 of these had no vocabulary token at all → value set to
  UNVERIFIED.

Post-backfill run: `166 files, 166 parse OK, 0 FAIL` (exit 0), standing-map
hash `3dec79a5aef7cf5a` (hash changed because UNVERIFIED was added to the
vocabulary; see divergence note).

Standing distribution (from docs/jira/receipt-standing-map.json):
ALIVE 103 · PARTIAL 11 · PARTIAL_ALIVE 10 · BLOCKED 7 · UNVERIFIED 8 ·
PASS 5 · UNSUPPORTED 5 · CONFORMANT 3 · VERIFIED 3 · REFUSED 3 · CLOSED 2 ·
UNKNOWN 2 · Exit-code-1/7/11 3 · FAIL 1.

## Files

- `scripts/receipt_standing_lint.py` (new) — port. Named
  `receipt_standing_lint.py`, NOT `receipt_lint.py`: cgap already owns
  `scripts/receipt_lint.py` as a different tool (claim-sentence receipt
  linter, lane W2-3, wired into `just doctor`'s gate chain); clobbering it
  would break the doctor chain.
- `justfile` — new `receipt-standing-lint` recipe (verified: `just
  receipt-standing-lint` → exit 0, PASS).
- `docs/jira/receipt-standing-map.json` (new) — file→standing map + hash.
- 137 receipt files under `docs/jira/v26.10.31|v26.11.3|v26.11.4/receipts/`
  backfilled/repaired. No crates/ or tests/ files touched.

## Documented divergence

`UNVERIFIED` added to the Gate-3 vocabulary. The receipt-standing-law's closed
vocabulary carries UNVERIFIED (the lawful fallback for backfills with no
present vocabulary); ggen's Gate-3 map omitted it, which made its own fallback
unlintable. cgap's map hash is `3dec79a5aef7cf5a` and differs from ggen's.

## Commands / exits

- `python3 scripts/receipt_standing_lint.py` → exit 1 (137 FAIL, census run)
- `python3 /tmp/l21_backfill.py` (one-off, not committed) over the 137 FAIL
  files → pass 1: repaired=8, backfilled=112, unverified-inserted=3; pass 2
  over the 17 initially-skipped files: remaining=17 normalized. Final tally
  per git: 137 files changed, 137 insertions, 25 deletions.
- `python3 scripts/receipt_lint.py` (cgap's pre-existing claim-linter) — not
  run; not in scope.
- `just receipt-standing-lint` → exit 0, `PASS (0 missing)`
