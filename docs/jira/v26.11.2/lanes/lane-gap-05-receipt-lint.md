# Lane GAP-05 — receipt_lint strict-crash fix + DOC-RECEIPT rule class

Standing: ALIVE (all commands below ran this session, exits shown).
Subject: `scripts/receipt_lint.py` (sole owned file). Register of truth
read first: `lanes/lane-dx-04-receipt-lint.md`.

## Census baseline (recorded at lane START, not drain)

Concurrent lanes are landing docs; baseline frozen before any edit:

```
$ python3 scripts/receipt_lint.py --json
189 docs, 439 claims, all STRUCT counts 0        # exit 0
$ python3 scripts/receipt_lint.py --dir docs/jira/v26.11.2/waves \
    --dir docs/jira/v26.11.2/lanes --dir docs/jira/v26.11.2/audit --json
290 docs, 728 claims, STRUCT-001 62, STRUCT-002 8  # exit 0
```

## (a) --strict crash: root cause + fix

Witnessed in `lanes/lane-dflss-03-pfmea.md` (DF-3). Reproduced this
session before any edit:

```
$ python3 scripts/receipt_lint.py --dir docs/jira/v26.11.2/waves \
    --dir docs/jira/v26.11.2/lanes --strict
  File ".../scripts/receipt_lint.py", line 436, in main
    rules = sorted({f["rule"] for f in findings} | ...)
KeyError: 'rule'
```

Root cause: `lint_doc()` claim findings carried
`file/line/snippet/reason` but NO `"rule"` key; the strict arm at :436
joins `f["rule"]` over ALL findings. Any run with >=1 claim finding
panicked. Fix: claim findings now carry
`"rule": "UNRECEIPTED-CLAIM"`, and the strict arm cites rule ids
directly (no remap). Post-fix, same command: typed STRICT-FAIL, exit 1,
no panic — cites `DOC-RECEIPT-001,DOC-RECEIPT-002,DOC-RECEIPT-003,
RECEIPT-STRUCT-001,RECEIPT-STRUCT-002,UNRECEIPTED-CLAIM`.

## (b) DOC-RECEIPT rule class (doctrine-ship-plan.md section C)

Scope: `audit/` + `waves/` only; `lanes/` keeps RECEIPT-STRUCT-*
(enforced by `doc_receipt_scope()`). All findings typed with `rule`:

| Rule | Fires when | Mechanical |
|---|---|---|
| DOC-RECEIPT-001 | quantitative claim (number+unit, Nx, %, count) in a shipped narrative with no resolvable receipt ref within PROXIMITY=3 lines; `[TARGET-v26.11.2]` still exempts; version strings, dates, and line-citation artifacts (`file.md:27-28`, `rs:309`) are stripped before matching | no |
| DOC-RECEIPT-002 | cited `waves/` or `lanes/` path does not resolve (path-only check, no content heuristics) | YES |
| DOC-RECEIPT-003 | no status-vocabulary token (ALIVE/PARTIAL/BLOCKED/UNVERIFIED/REFUSED) OR no as-of date within the first 30 lines | no |

## (c) Anti-vacuity flips (/tmp, shared tree untouched)

In-process arm: `python3 scripts/receipt_lint.py --self-check`
rc 0 — includes the new DOC-FLIP battery (001 no-ref fires + clears on
ref; 002 nonexistent path fires; 003 header stripped fires) plus the
DX-4 strict arm unchanged. External CLI transcript (fresh /tmp dir):

```
$ ... --root /tmp/gap05_flip_* --dir waves --strict
BASELINE ("The kernel is 3.4x faster.", no ref)
  DOC-RECEIPT-001 waves/w-doc.md [quantitative claim, ...]   exit 1
FIX (append "See waves/w-ref.md:5 for the measurement.")
  RECEIPT-LINT: STRICT-PASS                                  exit 0
MUT-002 ("See waves/w9-nope-missing.md for proof.")
  DOC-RECEIPT-002 [... does not resolve]  STRICT-FAIL        exit 1
MUT-003 (honesty header stripped)
  DOC-RECEIPT-003 [missing honesty-header]  STRICT-FAIL      exit 1
```

## (d) Fresh census after (re-read from disk, this session)

```
$ ... waves + lanes + audit --json
293 docs, 739 claim findings
DOC-RECEIPT-001 2324 / 002 81 / 003 145
RECEIPT-STRUCT-001 64 / 002 8 / 003..005 0
$ python3 scripts/receipt_lint.py --json   (default waves scope)
189 docs, 439 claims   # unchanged vs START baseline — fence held
```

Count rose as predicted (docs 290->293 concurrent landings; claims
728->739; 001 is a new rule over shipped narratives, recall-first:
each finding is a candidate requiring human re-derivation, precision
imperfect on identifier-adjacent numbers — it is the non-mechanical
class by design).

## Fences, divergences, disclosure

- Fence: default claim-lint census. Re-ran post-edit: 439/189 exact —
  semantics unchanged for the pre-existing gate.
- Fence: DX-4 strict arm + --fix move path. `--self-check` rc 0 covers
  both; dry-run move planning still prints PLAN-MOVE lines.
- Divergence: the PostToolUse write-integrity hook flagged
  BRACE_PAREN_DELTA on an intermediate edit state; on-disk bytes were
  re-verified with `ast.parse` (syntax OK) and the full battery ran
  green. Hook measured mid-edit, not final state.
- No git commands run. Build root: none (stdlib-only script, no cargo).
