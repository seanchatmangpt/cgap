# lane-tc-09-lint-precision

Lane TC-09 — receipt-lint precision: DXD-05 GAPs A-D over
`scripts/receipt_lint.py` + `scripts/receipt_lint_fixtures/`.
Spec: `docs/jira/v26.11.2/audit/dxd-05-receipt-lint-audit.md` (section 4).

## W1 Files written

- `/Users/sac/cgap/scripts/receipt_lint.py` (edits; sha256 `d7ceaad23457da64…`, 41305 B on disk at close)
- `/Users/sac/cgap/scripts/receipt_lint_fixtures/waves/fixture-gap-a.md` (270 B)
- `/Users/sac/cgap/scripts/receipt_lint_fixtures/waves/fixture-gap-b.md` (114 B)
- `/Users/sac/cgap/scripts/receipt_lint_fixtures/waves/fixture-gap-c.md` (234 B)
- `/Users/sac/cgap/scripts/receipt_lint_fixtures/waves/fixture-gap-d.md` (85 B)
- `/Users/sac/cgap/scripts/receipt_lint_fixtures/receipts/w9-gap-a-receipt.md` (127 B)

## W2 Diff (what the rules now do)

- GAP A (DOC-RECEIPT-001 misfire): `|`-prefixed CTQ-table rows are skipped
  as self-receipting; a RESOLVED `[receipt: <path>]` cite in the proximity
  window receipts the claim (new `RECEIPT_CITE_RE`).
- GAP B (002 false negative): `receipt:` cites are mechanically validated —
  dangling `receipt:` path fires DOC-RECEIPT-002. `resolve_any_ref` gained
  a root-relative fallback (`<root>/<name>`, the `receipts/` sibling case).
- GAP C (003 vocabulary): honesty header accepts `sealed-by` +
  `DIVERGENCE-NAMED` (+ date, already required) as equivalent to the
  campaign standing vocab.
- GAP D: a line carrying `ROADMAP-PENDING` is exempt from 001 (quantified
  deferrals are declared future work).
- New court `gap_flip_check()`, wired into `--self-check` after
  `doc_flip_check()`/`layer_flip_check()`.

## W3 Commands + exits (this session, exact subject)

- `python3 scripts/receipt_lint.py --self-check` → rc 0; tail:
  `RECEIPT-LINT GAP-FLIP: PASS (A ctq-row+receipt-cite silent then fires
  001; B dangling cite fires 002 then clears; C sealed-by silent then
  fires 003; D deferral silent then fires 001)`.
- `python3 scripts/receipt_lint.py --strict` → rc 1, stderr
  `STRICT-FAIL citing DOC-RECEIPT-001,DOC-RECEIPT-002,DOC-RECEIPT-003,LAYER-QUALIFIER-001,UNRECEIPTED-CLAIM`
  — typed, NO KeyError (GAP-05 fix holds).
- `python3 scripts/receipt_lint.py --json` → rc 0.

## W4 Courts (flip transcripts)

All in /tmp sandboxes (`receipt_lint_gapflip_<gap>_`), shared tree never
mutated. Each gap runs in its own root so fixtures cannot contaminate
each other's baselines.

- A: fixture silent as shipped (STRICT-PASS, rc 0) → mutation A1 (flatten
  CTQ row + drop receipt-cite bullet) fires `DOC-RECEIPT-001` rc 1 →
  mutation A2 (swap cite to `w9-nope-missing.md`) fires `DOC-RECEIPT-001`
  rc 1. An unresolved receipt cite no longer receipts a claim.
- B: fixture fires `DOC-RECEIPT-002` rc 1 as shipped → creating
  `receipts/w9-nope-missing.md` in the sandbox → rc 0, 002 not cited.
- C: fixture 003-silent → stripping sealed-by/DIVERGENCE-NAMED lines →
  `DOC-RECEIPT-003` fires (via `doc_receipt_scan` on the sandbox copy).
- D: fixture silent (rc 0) → drop `ROADMAP-PENDING` token →
  `DOC-RECEIPT-001` fires rc 1 → restore → rc 0.
- Pre-existing arms also green in the same run: SELF-CHECK, STRICT
  (RECEIPT-STRUCT-002 flip), DOC-FLIP, LAYER-FLIP (TC-12's arm).

## W5 Census delta (honest, exact-subject run this session)

`--json` over the real tree: verdict FAIL, 189 docs, 441 unreceipted
claims, DOC-RECEIPT-001 **1470** (audit baseline 2332), 002 **28**
(was 83), 003 **132** (was 145), RECEIPT-STRUCT-001..005 all 0.
Delta is the intended precision fix; claims 439→441 is doc-set drift
(TC-12 landed wave docs since the audit), not a regression — the claim
linter itself is untouched by this lane.

## W6 Fences (stated + run)

- GAP-05 strict arm: typed exit 1, no KeyError — held (W3 run 2).
- `--self-check` rc 0 — held (W3 run 1), including TC-12's LAYER-FLIP.
- No `Process.sleep` anywhere; courts poll via subprocess rc + rule-id
  membership only.
- Write-integrity hook fired `BRACE_PAREN_DELTA` on every edit — false
  positive: `LINECITE_RE`'s character class `[\w.)]` (receipt_lint.py,
  LINECITE_RE definition) contains an unmatched `)` inside a regex
  string; `ast.parse` OK, linter runs green. Recorded, not "fixed"
  (the regex is load-bearing).

## W7 Standing

ALIVE for everything above (each claim re-ran this session with cited
rc); census claims are as-of 2026-11-02 on the tree state at this lane's
close and will drift as other lanes land. GAP E remains informational
(no digest rule exists; noted in the audit, not in TC-09 scope).

## Divergences

1. The dispatch named `audit/dxd-05-receipt-lint-audit.md`; the file
   actually lives at `docs/jira/v26.11.2/audit/dxd-05-receipt-lint-audit.md`.
2. TC-12's LAYER-QUALIFIER-001 + `layer_flip_check` landed concurrently in
   `scripts/receipt_lint.py` (shared file, disjoint rules); this lane
   wired its own `gap_flip_check` around TC-12's self-check wiring without
   touching the layer rule. No cross-lane unblock was applied in either
   direction.
3. Fixture C originally carried "template :25" (space before colon);
   `LINECITE_RE` did not strip it and 001 fired. Fixed the fixture text to
   "template:25" — the fixture now also documents that line-cite stripping
   is no-space-bound.
