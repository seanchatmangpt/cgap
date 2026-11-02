# Lane DX-4 — receipt_lint typed strict mode + mechanical --fix

Status: ALIVE (all commands below run this session, exits shown).

## (a) Baseline classification of the 445 findings

Command: `python3 scripts/receipt_lint.py --json` (default waves scope,
v26.11.2). Exit 0 (census, no --strict).

| Category | Count | Class |
|---|---|---|
| no waves/ ref in proximity | 428 | needs per-claim receipt re-derivation — NOT mechanical |
| receipt file missing (cross-campaign ref: w3-l5-fused, cgap-datafusion-removal, a02-finish-receipt, p5-integration, w9-99-bad, ...) | 17 | NOT mechanical — v26.11.1/v26.11.2 cross-campaign citations; resolve_ref only searches v26.11.2 + docs/jira root |
| (of the 428) proximity window has exit citation nearby | ~205 | borderline — claim sentence lacks a ref but evidence exists ±8 lines |
| (of the 428) numeric claim (possible stale count) | 51 | irreducible: stale counts need re-derivation, not sed |
| (of the 428) bare prose claim, no evidence anywhere near | 170 | needs real receipting work |
| (of the 428) fence present but no valid ref | 19 | needs ref fixing |

445 total on 190 docs; full waves+lanes census this session: 632 claim
findings on 251 docs (618 on 245 earlier in the session — concurrent
lanes are landing docs live; the delta is other lanes' writes, not mine).

## (b) --strict typed rules (implemented)

Legal classes, defined explicitly:

- **class LANE** — lane receipt of record for one lane's contract. MUST
  live in a `lanes/` dir named `lane-<lane-id>-<slug>.md`; MUST contain
  >=1 fenced code block and >=1 exit citation.
- **class WAVE** — wave-level docs under `waves/` (any name NOT starting
  `lane-`). Free-form; governed only by the existing claim-sentence lint.

Rule ids (RECEIPT-STRUCT-*):

| Rule | Meaning | Mechanical? |
|---|---|---|
| RECEIPT-STRUCT-001 | lane receipt missing fenced code block | no |
| RECEIPT-STRUCT-002 | lane receipt missing exit citation | no |
| RECEIPT-STRUCT-003 | lane-named doc outside lanes/ | YES (move) |
| RECEIPT-STRUCT-004 | wave doc inside lanes/ | YES (move) |
| RECEIPT-STRUCT-005 | lane receipt name not lane-<id>-<slug>.md | no (rename touches refs) |

## (c) --fix mechanical class, dry-run default

- Dry-run printed exactly 2 planned moves (witnessed above in the
  transcript, `PLAN-MOVE` lines, rc 0).
- `--fix` applied for real this session:

```
$ python3 scripts/receipt_lint.py --dir docs/jira/v26.11.2/waves \
    --dir docs/jira/v26.11.2/lanes --fix
MOVED docs/jira/v26.11.2/waves/lane-g08-defect-register-close.md \
    -> docs/jira/v26.11.2/lanes/lane-g08-defect-register-close.md
MOVED docs/jira/v26.11.2/waves/lane-git-03-ingress-ring-receipt.md \
    -> docs/jira/v26.11.2/lanes/lane-git-03-ingress-ring-receipt.md
RECEIPT-LINT: 2 move(s) applied
exit 0
```

## Delta before/after --fix

| Metric | before | after |
|---|---|---|
| RECEIPT-STRUCT-003 (misplaced lane receipts) | 2 | 0 |
| RECEIPT-STRUCT-001 | 48 | 50 |
| RECEIPT-STRUCT-002 | 5 | 6 |
| claim findings (waves+lanes) | 632 | 632 |

The 001/002 rise is reclassification, not regression: the two moved
receipts themselves lack fence/exit and are now counted in lanes/.
Net mechanical class is empty: STRUCT-003/004 both zero after fix.

## (d) Irreducible class — honest report

Stale-looking counts (51 in the 445 baseline) cannot be fixed by sed or
a script: a count claim is only refreshed by re-running the thing it
counts. The 428 "no ref in proximity" findings likewise need a human/lane
to either produce a real receipt ref or retarget [TARGET-v26.11.2].
Zero of the 445 are safely auto-fixable; the only mechanical class in the
tree was path drift, and it is now drained.

## Anti-vacuity witness (both directions, /tmp, shared tree untouched)

```
$ python3 scripts/receipt_lint.py --root /tmp/dx4-strict \
    --dir docs/jira/v26.11.2/lanes --strict
RECEIPT-STRUCT-001 docs/jira/v26.11.2/lanes/lane-dx9-zeroexit.md [...]
RECEIPT-STRUCT-002 docs/jira/v26.11.2/lanes/lane-dx9-zeroexit.md [...]
RECEIPT-LINT: STRICT-FAIL citing RECEIPT-STRUCT-001,RECEIPT-STRUCT-002
exit 1        # zero-exit-citation planted receipt FAILS
# after appending a fence + exit citation:
RECEIPT-LINT: STRICT-PASS
exit 0        # compliant receipt rc 0
```

In-process arm rc 0: `python3 scripts/receipt_lint.py --self-check`
(existing claim-flip arm + new strict arm + dry-run move plan).

## Fences / standing / divergences

- Fences that could regress: the existing claim-lint census. Re-ran after
  edits: default scope still 445/190 exactly (unchanged semantics).
- Standing: ALIVE for lint extension + 2 mechanical moves + both-way
  strict court. PARTIAL for the doc corpus: 632 claim + 50/6 struct
  findings remain and are NOT fixable mechanically.
- Divergence: inbound citations in `docs/jira/v26.11.2/audit/
  release-blockers.md` and `audit/audit-ledger.md` still cite the two
  moved receipts at their old `waves/` paths. Ref rewrites are not
  mechanical sed (they change meaning); left to the audit lane owner.
- No git commands run; build root: none (stdlib-only script, no cargo).
