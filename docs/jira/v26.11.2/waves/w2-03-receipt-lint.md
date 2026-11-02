# w2-03-receipt-lint

Lane W2-3 (lane-runner). Repo /Users/sac/cgap, no git commands, shared
tree untouched except the two files this lane owns.

## Delivered

`scripts/receipt_lint.py` (stdlib-only): claim-sentence receipt linter
per dispatch. Flags `--root`, `--dir` (repeatable), `--strict`, `--json`,
`--self-check`. Claim heuristic: declarative line containing landed |
removed | verified | green | passes | closed | fixed | deleted | stops |
pruned; must carry, within PROXIMITY=3 lines, a validated `waves/*.md`
ref (file exists; cited `:N` inside file range) or exact
`[TARGET-v26.11.2]`. Exit classes: 0 pass/census, 1 strict findings or
VACUOUS self-check, 2 fail-closed (zero docs / unreadable / exception),
3 loud skip (`[SKIP] <reason>` printed for excluded/missing dirs).

## Commands (this session, /Users/sac/cgap)

- `python3 scripts/receipt_lint.py --self-check` → exit 0
  (PASS; claim-less fixture flagged waves/w9-99-bad.md:3, receipted and
  [TARGET] fixtures clean). Non-vacuity: bad fixture is the named
  mutation; self-check failing to flag it exits 1 VACUOUS GATE.
- `python3 scripts/receipt_lint.py --root . --strict` → exit 1
  (findings present; this is the W8 census input).
- `python3 scripts/receipt_lint.py --root . --json` → exit 0;
  unreceipted_claims = 143, files_scanned = 58.
- excluded dir `--dir docs/jira/v26.11.2/patent` → `[SKIP]` + exit 3.
- missing dir `/tmp/empty-nope` → `[SKIP]` + exit 3; truly empty waves
  dir → exit 2 fail-closed.

## Census (docs/jira/v26.11.2/, 2026-11-01)

UNRECEIPTED-CLAIM total: 143 over 58 scanned docs (fixes deferred to
W8 per dispatch — census is the deliverable). Worst offenders:

- 10 docs/jira/v26.11.2/03-ost-design-around-register.md
- 9 docs/jira/v26.11.2/07-court-corpus.md
- 9 docs/jira/v26.11.2/01-release-gates.md
- 7 docs/jira/v26.11.2/waves/v2-07-census.md
- 7 docs/jira/v26.11.2/OVERNIGHT-LOOP.md
- 7 docs/jira/v26.11.2/05-wave-plan.md

Standing: ALIVE (all commands + exits above run this session against
the exact on-disk subject `scripts/receipt_lint.py`).

## Divergences

- Spec `w1-05-doctor-extension-spec.md` 4.2 describes a per-receipt
  rules (a)-(e) lint; the W2-3 dispatch defines the claim-sentence
  census lint actually built here. Dispatch governs; the 4.2 rules lint
  is a superset candidate for a later wave if the doctor chain wants
  per-receipt checks too.
- Two runs early in the session reported 136 findings vs 143 at close;
  the tree is concurrently edited by other lanes. 143 is the
  deterministic re-read count at close (twice).
