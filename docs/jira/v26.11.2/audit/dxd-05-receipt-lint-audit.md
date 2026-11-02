# dxd-05-receipt-lint-audit

DXD-05 receipt-linter audit vs the Go-sealed interop receipt (2026-11-02).
Linter scripts/receipt_lint.py; interop shape
courts/interop-chicago/cmd/seal_receipt/{main.go, receipt-template.md}.

## 1. Fresh runs (all modes)

| mode | rc | findings |
|---|---|---|
| default | 0 | FAIL, 189 docs, 439 UNRECEIPTED-CLAIM, 0 struct |
| --json | 0 | same payload, verdict FAIL |
| --strict | 1 | STRICT-FAIL citing DOC-RECEIPT-001/002/003 + UNRECEIPTED-CLAIM |
| DOC-RECEIPT scope | 0 | 214 docs, 501 claims |

Struct counts: DOC-RECEIPT-001 2332 (recall-first, expected),
002 83, 003 145; RECEIPT-STRUCT-001..005 all 0.

## 2. GAP-05 strict-arm crash fix HOLDS

lint_doc() stamps rule=UNRECEIPTED-CLAIM on claim findings
(receipt_lint.py:114-120); --strict exits 1 typed, no KeyError;
--self-check rc=0 (strict_check + DOC-FLIP 001/002/003 all fire,
001 clears on receipt). Confirmed holding.

## 3. Per-rule behavior vs the interop receipt shape

| rule | fires on interop receipt? | verdict |
|---|---|---|
| UNRECEIPTED-CLAIM | no (table rows skipped; no claim verbs) | correct |
| DOC-RECEIPT-001 | yes — CTQ table rows + sales bullets | MISFIRE (row IS the receipt) |
| DOC-RECEIPT-002 | no | GAP — [receipt: <path>] cites invisible to REF_RE |
| DOC-RECEIPT-003 | yes, always | MISFIRE — interop honesty header is sealed-by/DIVERGENCE-NAMED |
| RECEIPT-STRUCT-* | no | correct |

Empirical regex probes (run, not guessed): transcript-root sha256 line
correctly silent; DIVERGENCE-NAMED silent; sealed-by version-stripped
silent; ROADMAP-PENDING deferral line MISFIRES as 001.

## 4. Rule gaps + fixture proposals

- GAP A: 001 doesn't recognize [receipt: <path>] or inline CTQ-table
  self-receipting. Fixture: emitted template w/ one real CTQ row;
  expected 001 silent on the row + resolved-cite bullets.
- GAP B: 002 can't validate receipt: paths (scoped waves/+lanes only)
  — dangling interop cites silently unchecked (false NEGATIVE).
  Fixture: cite receipts/w9-nope-missing.md; expected 002 fires.
- GAP C: 003 honesty vocabulary is campaign-local; sealed-by +
  DIVERGENCE-NAMED + as-of date should pass (or scope per class).
- GAP D: no ROADMAP-PENDING exemption; quantified deferrals misfire.
- GAP E (informational): no digest rule exists — any future digest
  rule must accept the named sha256 divergence (template :25), not
  demand BLAKE3.

## 5. Net assessment

No crash-class risk; exit codes behave as documented (0 census,
1 strict typed, 2 fail-closed, 3 loud skip). Four misfire/silent-gap
findings, all DOC-RECEIPT-class interacting with the interop shape;
UNRECEIPTED-CLAIM and RECEIPT-STRUCT-* clean against that shape.

## See Also

audit/doctrine-ship-plan.md · lanes/lane-gap-05-receipt-lint.md ·
courts/interop-chicago/cmd/seal_receipt/main.go
