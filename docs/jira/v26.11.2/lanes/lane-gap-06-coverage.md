# Lane GAP-06 receipt — gate-script branch coverage (rest of C06's gaps)

Date: 2026-11-02 · Repo: /Users/sac/cgap (main) · Standing: **PARTIAL**
(honest red on the 90% floor; large measured deltas, floor NOT lowered)

## Files written (ownership fence held)

- `tests/gates/test_gate_exits.py` (extended: 17 → 30 tests)
- `tests/gates/gate_fixture_helpers.py` (NEW — shared /tmp fixture helpers)
- `tests/gates/.coveragerc.gap06` (NEW — lane-private coverage rcfile)
- `docs/jira/v26.11.2/lanes/lane-gap-06-coverage.md` (this file)

Nothing under `scripts/` was edited. boundary_gate.py and
zero_string_gate.py were only measured. Gate scripts are sha-witnessed
unchanged by the lane diff (no scripts/ path appears in the lane diff;
coordinator can confirm via scoped `git diff --stat scripts/`).

## Coverage commands (all run this session)

C06's method mirrored, per-invocation parallel data files, combined once:

```
/opt/homebrew/bin/coverage run --branch -p \
  --rcfile=/Users/sac/cgap/tests/gates/.coveragerc.gap06 \
  scripts/<gate>.py <mode>
coverage combine --rcfile=... ; coverage report --rcfile=...
coverage json -q -o - --rcfile=...   # per-script branch percentages
```

26 gate invocations (11 boundary_gate modes, 8 zero_string_gate modes,
7 receipt_lint/zero_datafusion modes), plus the nominal-only baseline:

```
coverage run --branch --rcfile=tests/gates/.coveragerc scripts/boundary_gate.py --deps   # exit 0
coverage run --branch --rcfile=tests/gates/.coveragerc scripts/zero_string_gate.py        # exit 0
coverage report -m ...                       # top uncovered ranges read
```

Exit-mode matrix actually witnessed this session (command → exit):
boundary --deps 0, --strings 0, --self-check-deps 0, --cross-gate-court 0,
--interference-court 0, --shim-court-only 0, --deps --report 0, no-mode 1,
--only-op nosuchop 1, --targets missing-target.rs 2, ../escape.toml 1;
zero_string scan 0, --self-check 0, --register 0, --courts 0
(COURTS-SUMMARY total=13 failed=0), --dep-direction 0 (checked=90),
--dep-direction --profile audit 2 (crate-no-artifacts),
--dep-direction --root missing 1 (internal-error), scan --root missing 2
(vocab-missing); receipt_lint repo 0, missing-dir --strict 3, ghost 1,
census 0, --self-check 0; zero_datafusion repo 0, missing-root 2.

## Before/after (percent_covered from `coverage json` summary; the
companion pure-branch % is in parentheses)

| Script | C06 baseline (cited) | nominal-only fresh | GAP-06 after | delta vs C06 |
|---|---|---|---|---|
| scripts/boundary_gate.py | 24.4% | 12.6% | **55.0%** (branch 45.2%, 187/414) | +30.6 |
| scripts/zero_string_gate.py | 40.8% | 34.9% | **75.5%** (branch 65.9%, 393/596) | +34.7 |
| scripts/receipt_lint.py | 64.9% | — | **82.9%** (branch 79.1%) | +18.0 |
| scripts/zero_datafusion_gate.py | 52.6% | — | 52.6% (branch 42.4%) | 0 |

Raw combined report row: boundary 55.0%, receipt_lint 82.9%,
zero_datafusion 52.6%, zero_string 75.5%, TOTAL 26.3% (7598 stmts,
3088 branches). No victory lap: boundary_gate at 55.0% is still deep
red — the §6.1 mutation-campaign body (run_shim/plant_shim successors,
per-operator mutators, campaign loop) and the --mutate happy path remain
uncovered because exercising them means running the full cargo court
campaign (~minutes per mutant); that is follow-on lane work, not a floor
adjustment. zero_string's remaining gaps: artifact-scan body details,
T-court fixture builders, register consumers.

## Test suite + floor

- `/opt/homebrew/bin/pytest tests/gates/test_gate_exits.py -q` →
  **30 passed in 334.30s, exit 0** (was 17 tests at C06; +13).
- Every new test routes through `assert_exit_with_teeth`
  (tests/gates/gate_fixture_helpers.py:52), which runs a poison twin:
  the same observed exit asserted against a deliberately wrong expected
  exit MUST raise, or the helper fails the suite. Load-bearing-token
  twins added where a token could silently rot (BAD --only-op,
  DEP-DIR-EXEMPT vs DEP-DIR-VIOLATION, COURTS-SUMMARY failed= parse).
- `python3 tests/gates/enforce_coverage_floor.py` → **exit 1, honest
  red**: 4 red scripts (55.0 / 82.9 / 52.6 / 75.5 all < 90), both
  fail-closed silent-pass probes FIRED (ghost receipt exit=1, missing
  root exit=2). Floor NOT lowered. Receipt of record:
  /tmp/gap06-floor-final.log (exit 1 witnessed).

## Divergences (doc yields to reality; recorded, not silently absorbed)

1. **/tmp/c06-venv was gone.** C06's venv (cited in its receipt) no
   longer exists on this host; system coverage.py 7.13.5 +
   /opt/homebrew/bin/pytest were used instead. A `/tmp/c06-venv/bin`
   shim (symlinks to system coverage/python3) was rebuilt so
   enforce_coverage_floor.py's hardcoded VENV path runs; if another lane
   deleted that path mid-flight, the floor check exits 2 — the shim is
   in /tmp, not the tree.
2. **Shared coverage db race (witnessed).** C06's
   `tests/gates/.coveragerc` points data_file at /tmp/c06-cov/.coverage;
   a concurrent writer combined/erased that db mid-lane (combined db
   replaced with 0.0% data, then deleted). GAP-06 switched to a
   lane-leased rcfile `.coveragerc.gap06` (data_file
   /tmp/gap06-cov/.coverage) and seeded the shared db for the floor
   run. Follow-on lanes should use their own data_file; the floor
   enforcer's hardcoded /tmp/c06-cov path is a shared-race surface the
   owning lane may want to fix.
3. **Pre-existing C06 test rot, repaired in-place (disclosed).**
   `test_receipt_lint_clean_fixture_exit_0` (a C06-authored test inside
   my owned file) failed this session: receipt_lint.py gained the
   DOC-RECEIPT-003 honesty-header rule after C06 landed, and the fixture
   docs lacked status-vocab + as-of date; the repaired claim also cites
   w2-body.md:3 now (header shift). ERRATA to C06's receipt
   (lane-c06-scripts-gate-coverage.md, the 17-passed claim): as of
   GAP-06 the suite is 30 tests / 30 passed; the clean-fixture test
   needed the honesty-header lines to hold.
4. **COVERAGE_PROCESS_START subprocess coverage did not engage** (no
   coverage .pth hook on the system python), so the suite-under-coverage
   route C06 described is not reproducible on this host as-is; the
   per-invocation matrix (this receipt) is the working method.
5. **coverage combine consumes parallel files** — a second `combine`
   after new partial runs REPLACES the previous combined db instead of
   merging (witnessed: first matrix's data lost). Combine exactly once
   per batch.

## Anti-vacuity summary

- Poison twins on every new assertion (teeth helper; a twin that stops
  raising fails the suite via assert).
- Load-bearing-token twins for refusal/emitter tokens.
- Floor probes: both FIRED (exit 1 / exit 2 on corrupted inputs).
- Shared tree untouched by any test or coverage run; poison fixtures
  live in pytest tmp_path and mkdtemp sandboxes; gate scripts never
  mutated.
