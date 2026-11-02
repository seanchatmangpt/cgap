# DXD-06 plan — scripts/ gate branch-coverage recovery (floor 90%)

Date: 2026-11-02 · Repo: /Users/sac/cgap (main) · Status: PLAN (RED floor)
Prereq receipts read: lane-c06-scripts-gate-coverage.md (C06 method +
law), dflss/06-control-plan.md GATE-09 (OCAP law), lane-g16 (deps
self-check arm), lane-g17 (zero-string receipt).
Coverage floor reproduced FRESH this session (C06 method, combine +
`coverage report -m`, rcfile tests/gates/.coveragerc):

| Script | C06 (as-of) | FRESH | Verdict |
|---|---|---|---|
| boundary_gate.py | 24.4% | 55.0% | RED |
| zero_string_gate.py | 40.8% | 75.9% | RED |
| zero_datafusion_gate.py | 52.6% | 52.6% | RED (no lift) |
| receipt_lint.py | 64.9% | 82.9% | RED |
| floor check | exit 1 (4 red) | exit 1 (4 red) | unchanged |

`python3 tests/gates/enforce_coverage_floor.py` re-run on the fresh DB:
exit 1, "FLOOR: FAIL (floor=90%) — 4 red", both silent-pass probes
FIRED (exit 1 / exit 2). 30/30 gate-exit tests pass (305 s wall).

ERRATA (must record): `lanes/lane-gap-06-coverage.md` DOES NOT EXIST on
disk. The GAP-06 tests landed (30 `test_gap6_*` cases now inside
tests/gates/test_gate_exits.py) and they ARE the lift (see table), but
no lane receipt was written. Per repo law (source-of-truth-check), any
lane that ships without a receipt is drift — the GAP-06 executor must
backfill a receipt or the lift is unwitnessed.

## Fresh measurement method (repeatable)

coverage 7.13.5 (py3.13). Per C06: rcfile tests/gates/.coveragerc,
`coverage run --branch -p`, one fragment per invocation, combine,
`coverage report -m`. Fragments from: the 30-test suite
(`coverage run -m pytest tests/gates/test_gate_exits.py`) PLUS direct
instrumented runs of every mode the suite only exercises as a
SUBPROCESS (subprocess exits are invisible to the parent tracer — this
is why C06 saw 24.4/40.8; direct runs are load-bearing):

- zero_string_gate.py: clean, --self-check, --register, --courts,
  --dep-direction, --profile audit (exit 2, fail-closed witnessed)
- boundary_gate.py: --self-check-deps, no-mode (exit 1),
  --mutate --targets missing-target.rs --shim-inert (exit 2),
  --mutate --targets ../escape.toml (exit 1)
All exits witnessed live and listed in the probe transcript; the direct
runs are the delta between C06's 24.4% and this 55.0%.

## The math (branch units = stmts + branch arcs, coverage.py units)

| Script | Stmts | Branch arcs | Units | Now | 90% needs | Gap (units) |
|---|---|---|---|---|---|---|
| boundary_gate | 1042 | 414 | 1456 | 55.0% (~801) | 1310 | ~509 |
| zero_string | 1233 | 596 | 1829 | 75.9% (~1389) | 1646 | ~257 |
| zero_datafusion | 210 | 92 | 302 | 52.6% (~159) | 272 | ~113 |
| receipt_lint | 395 | 172 | 567 | 82.9% (~470) | 510 | ~40 |

Observed yield per test class (this fresh run): a mode-level probe test
covers 60-140 units; a fixture error-arm test 5-20; a pure-function
table test 15-40. Estimate to 90%: boundary ~30-36 tests, zero_string
~12-16, datafusion ~8-12, receipt_lint ~4-6. Total ~55-70 new tests.

## Ranked recovery order (most branch-% per test first)

### boundary_gate.py (worst, 509-unit gap)

Ranked by branch density per test, pure/fixture tests before any cargo
dependence:

| Rank | Range | What | Units | Test class | Est/ test |
|---|---|---|---|---|---|
| 1 | 497-572 | mutate_line mutators | ~110 | pure: in-process mutate_line() | 30-40 |
| 2 | 625-643 | classify_run table | ~25 | pure: stdout fixtures | 20 |
| 3 | 1883-1900 | parse_targets error arms | ~18 | pure: bad spec/escape/missing | 10 |
| 4 | 934-1002 | load_a07_register arms | ~55 | /tmp fixture register per arm | 8-12 |
| 5 | 1053-1079 | a07 scan + summary arms | ~35 | /tmp fixture stdout per arm | 8 |
| 6 | 1130-1182 | crate-dirs/runtime-deps arms | ~25 | /tmp manifest fixtures | 8-10 |
| 7 | 648-723 | ledger/report/verify/lock | ~60 | tmp_path + real files | 10 |
| 8 | 1213-1295 | deps/strings error arms | ~50 | fixture tree (bad manifest) | 10 |
| 9 | 1334-1374 | deps-strings/self-check SKIP | ~10 | missing-manifest root | 5 |
| 10 | 802-858 | shim_court arms | ~35 | needs campaign seam (FLAG-1) | 6-10 |
| 12 | 1759-1877 | mode_interference_court | ~90 | real cargo build | 50-80, slow test |
| 12 | 1759-1877 | mode_interference_court | ~90 | real cargo build | 50-80, ONE slow test |

Batches (projected):
- B1 ranks 1-3 (pure, ~8 tests, no repo writes): +170-210 units -> 55
  -> ~69-70%
- B2 ranks 4-9 (fixture arms, ~18 tests): +190-230 -> ~85%
- B3 ranks 10-12 (seam/cargo, 3 slow tests): +160-240 -> 90%+ crossing
  needs B3; B1+B2 alone plateau ~85-87%.
- B3 has two honest options; see "Refactor flags".

### zero_string_gate.py (257-unit gap)

| Rank | Range | What | Test class |
|---|---|---|---|
| 0 | (modes) | five zero_string modes banked by direct-run probes; rest is arc-level | (done) |
| 1 | 593-652 | printable_runs/select_sections tails | pure tables, tmp bytes fixtures |
| 2 | 987-995 | ar_members arcs | pure, ar fixture bytes |
| 3 | 1254-1263, 1346-1362 | cmd_scan error arms | fixture root per arm |
| 4 | 1397-1447 | cmd_dep_direction tails | reuse depdir fixture builder |
| 5 | 1505-1626, 1668-1895 | self_check/court tails + FAIL-row | court suite, raising thunk |
| 6 | 2065-2169 | emitter exemption arcs | existing T16 fixture, in-process |

### zero_datafusion_gate.py (113-unit gap, zero lift since C06)

314-line gate; gaps are --json/--verbose/P1/P2 deep paths (209-227,
260-303). All reachable with /tmp fixture workspaces; the only known
RED on the live tree is the vendor symlink-loop errata (C06 ERRATA 1)
which poisons the self-check sandbox copy — exclude loop or point
--root at a fixture copy. ~8-12 tests -> 90%+ without refactor.

### receipt_lint.py (40-unit gap)

Closest to green. Missing: --json/--self-check tails + similarity
validator arcs (316-380, 508-560). Existing doc_root fixtures cover the
pattern; ~4-6 tests -> 90%+. No refactor.

## Poison-twin law (per C06, applies to EVERY new test)

1. No assert-only courts: every new exit-mode test carries a twin
   probe asserting the FLIP (poisoned fixture must change the exit),
   in the style of test_gate_exit_assertions_have_teeth. A test whose
   poison arm still passes is a vacuous court and must fail the suite.
2. Poison fixtures are real files in /tmp workspaces (Chicago: no
   mocks); the repo tree is never the mutation target.
3. Every command in a test is hand-witnessed live (exit + verdict line)
   BEFORE it is pinned — the C06 table discipline.
4. Anti-vacuity spot checks inside the floor checker stay; new silent-
   pass probes: one per script (ghost receipt, missing root already
   exist; add ghost-register + ghost-workspace for boundary/
   zero_string arms in B2).

## Refactor flags (honest)

- FLAG-1 (hard): boundary --mutate campaign + shim/interference/
  cross-gate courts mutate the REAL tree / build real cargo fixtures.
  A campaign run is up to 24 mutants x cargo test; hours per run.
  CI-testable only via a court-command seam. TODAY the only options:
  (a) in-process import + module-attribute patch of COURT_ARGS —
  fast, no gate edit, but is attribute-patching "a mock"? It asserts
  real subprocess behavior on real files, so Chicago-compatible in
  letter; needs coordinator sign-off; or (a) refactor: a small gate
  change adding a sanctioned override (e.g. env CGAP_COURT_ARGS +
  --target-dir sandbox copy) so the campaign runs against a /tmp copy.
  Either unblocks ~250 units that B1+B2 cannot reach.
- FLAG-2: argparse/sys.exit dispatch arms (no-mode exit 1, unknown
  mode arms) ARE coverable cheaply — do NOT flag them untestable; the
  no-mode probe alone banks ~10 units (witnessed: exit 1).
- FLAG-3: subprocess invisibility is a METHOD rule, not a defect:
  floor runs must include direct instrumented runs per mode, or the
  numbers silently regress to C06's 24/41/53/65. Encode the probe
  list (above) in enforce_coverage_floor.py docs or a runner script.
  (Writing a runner script is lane work, not this plan.)

## Verdict (realistic)

- zero_string_gate, zero_datafusion_gate, receipt_lint: 90% is
  reachable WITHOUT any gate refactor (~25-34 tests total, B1'/B2',
  datafusion fixture arms). Verify with the same fresh-DB method.
- boundary_gate alone: B1+B2 plateaus at ~85-87% (~26 tests banked
  ~430-480 of 509 units); the last ~120-160 units live in
  campaign/court bodies that require either the FLAG-1(a) attribute-
  patch sanction or the FLAG-1(b) court-seam refactor. Therefore:
  the 90% floor is NOT reachable on all four scripts from tests alone
  unless FLAG-1 is resolved.
- OCAP ('raise coverage, never lower floor', dflss/06-control-plan.md
  GATE-09): the floor stays; NO lowering, NO rounding.
- Recommendation: a small sanctioned court-seam refactor lane (env
  override + --mutate --target-dir sandbox copy of the tree) is the
  cheaper, law-cleaner path than an OCAP operator exception receipt.
  If the coordinator refuses the refactor, the exception receipt must
  state: boundary_gate 85-87% test-only ceiling, gap units ~120-160
  in mode_cross_gate_court (1503-1731) / mode_interference_court
  (1759-1877) / campaign body (2059-2169), rationale hours-per-run,
  expiry one release.

### Critical Files for Implementation
- /Users/sac/cgap/scripts/boundary_gate.py
- /Users/sac/cgap/scripts/zero_string_gate.py
- /Users/sac/cgap/tests/gates/test_gate_exits.py
- /Users/sac/cgap/tests/gates/enforce_coverage_floor.py
- /Users/sac/cgap/tests/gates/.coveragerc
