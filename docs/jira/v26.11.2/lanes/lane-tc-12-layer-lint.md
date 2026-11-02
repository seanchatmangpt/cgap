# lane-tc-12-layer-lint

Standing: ALIVE (2026-11-02). Lane TC-12, repo /Users/sac/cgap, branch main.

## Contract

Extend scripts/receipt_lint.py with LAYER-QUALIFIER-001 (tps-07 rank 3
(C1) + the witnessed drift: tps-01-reality-ledger.md refuted the
manual's unqualified 5.70 ns/frame figure; AX-5 policed
"containerized execution" by hand). Plus this receipt.

## Diff

scripts/receipt_lint.py (only my rule; TC-09's DOC-RECEIPT class and
DX-4's RECEIPT-STRUCT class untouched):

- LAYER_UNIT_RE: ns/frame|ns/call|ns/op|us/batch after a number.
- LAYER_TOKEN_RE: bare|kernel|membrane|container\w*|in-process
  (container\w* so "container" and "containerized" both qualify —
  divergence noted below).
- layer_qualifier_scan(): CLAIM ROWS ONLY (markdown table rows, outside
  code fences) in DOC-RECEIPT-scoped docs (audit/ + waves/). A latency
  unit token with no layer token within PROXIMITY (±3 lines) = typed
  finding LAYER-QUALIFIER-001.
- Wired into main() next to doc_receipt_scan (audit/waves scope) and
  into self_check() as layer_flip_check().

## Courts (all run this session, real /tmp fixtures)

- FLIP A (fires): row `| x | 6.04 ns/frame |` →
  LAYER-QUALIFIER-001 fires. VACUOUS-GATE-guarded.
- FLIP B (silent): `| y | 6.04 ns/frame (membrane, containerized) |`
  → no finding.
- FLIP C (self-compliance): tps-01-reality-ledger.md's own corrected
  rows → zero findings (both in the /tmp fixture copy and in the real
  census run below). tps-01 qualifies via "container" rows in its
  ±3 window.
- --self-check: rc 0 (strict + doc-flip + layer-flip all green).
- --strict typed exit: rc 1 citing
  DOC-RECEIPT-001/002/003,LAYER-QUALIFIER-001,UNRECEIPTED-CLAIM; zero
  KeyError/Traceback (grep -c = 0).
- GAP-05/TC-09 rules unaffected: their doc-flip arm passes inside the
  same --self-check run (001/002/003 each fire and 001 clears).

## Census delta (real run, waves + audit)

LAYER-QUALIFIER-001: 6 findings (all honest: unqualified latency rows
in audit/audit-ledger.md:25, audit/cgapprd-prd-v2.md:261,
waves/w3-l1-ingress.md:63,64,65,67). Pre-existing counts unchanged by
me (re-read at final verification): DOC-RECEIPT-001 2626 / 002 139 /
003 170 across 249 docs; verdict FAIL is the pre-existing DOC/claim
backlog, not a TC-12 regression. exit 0 (non-strict census). Note:
first census read 248 docs / 2590/137/170 — the tree moved under this
lane mid-flight (concurrent lanes landing docs); the 6 LAYER findings
were stable across both reads.

## Fences (must-not-regress, run)

- Receipt-struct rules (DX-4): self-check strict arm passes (exit-cite
  flip witnessed in output).
- DOC-RECEIPT rules (TC-09/GAP-05): doc-flip arm passes; census counts
  for 001/002/003 identical to a run with my rule (my scan is additive,
  separate function; no shared state mutated).
- tps-01 self-compliance (the founding defect must not re-fire on its
  own corrected ledger): silent, FLIP C.

## Divergences

1. TC-09's receipt file (lane-tc-09*) was NOT on disk when I edited;
   their DOC-RECEIPT rules WERE present on disk (633-line coherent
   file; --self-check rc 0 pre-my-edit — see Commands block for the
   witnessed exit). I added my rule after theirs as directed.
   Disclose-in-both: I touched no TC-09 lines.
2. Layer token set extended with container\w* (matches "container" as
   well as "containerized") — required by the self-compliance court:
   tps-01's own rows say "container", not "containerized".
3. Scope narrowed to markdown table claim rows ("claim row" per
   contract): prose quotes-of-refuted-figures in tps-01 §2 stay silent
   by design.
4. Pre-existing write-integrity hook reports BRACE_PAREN_DELTA -1 vs
   HEAD on this file — traceable to TC-09's in-flight LINECITE_RE
   character class `[\w.)]` (literal `)`), not my diff; ast.parse OK.
5. TC-09's GAP-A/B flip battery arrived in doc_flip_check mid-flight
   with a failing arm ("GAP-A flattened CTQ row did not fire 001"),
   transiently blocking --self-check rc 0. TC-09 (owner) repaired it
   within a 180s bounded poll; no cross-lane unblock applied. Final
   --self-check rc 0 witnessed after their repair (Commands block).

## Commands

```
python3 scripts/receipt_lint.py --self-check            # rc 0
python3 scripts/receipt_lint.py \
  --dir docs/jira/v26.11.2/waves \
  --dir docs/jira/v26.11.2/audit --json                 # exit 0, 6 LAYER findings
python3 scripts/receipt_lint.py \
  --dir docs/jira/v26.11.2/waves \
  --dir docs/jira/v26.11.2/audit --strict               # exit 1 typed, no KeyError
```

The above census run is receipted by
audit/tps-01-reality-ledger.md (the founding drift ledger this rule
polices; its corrected rows lint silent under LAYER-QUALIFIER-001).

## See Also

audit/tps-01-reality-ledger.md · scripts/receipt_lint.py ·
docs/jira/v26.11.2/lanes/ (TC-09 receipt pending)
