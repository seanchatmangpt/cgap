# Lane C06 receipt — scripts/ gate coverage + exit-mode integration tests

Date: 2026-11-02 · Repo: /Users/sac/cgap (main) · Status: PARTIAL (honest red on floor)

## Files written (ownership fence held)

- `tests/gates/.coveragerc` (sha256 e8a512f506b19b20)
- `tests/gates/test_gate_exits.py` (sha256 d078b72a4e116b14)
- `tests/gates/enforce_coverage_floor.py` (sha256 6e100472b6435fdc)
- `docs/jira/v26.11.2/lanes/lane-c06-scripts-gate-coverage.md` (this file)

Nothing under `scripts/` was edited. boundary_gate.py and
zero_string_gate.py were only measured (coverage.py instruments the
interpreter, not the file) and are sha256-witnessed at
45254e376a4343f2 / 968ac884cdf2eeef as of this run.
`scripts/verify_disassembly_invariants.py` (G03 in flight): never
instrumented, never edited; skipped for measurement this session per the
read-only fence and the retry budget — recorded as loud skip, not a pass.

## Commands + exits (all run this session)

| Command | Exit |
|---|---|
| `python3 scripts/boundary_gate.py --deps` | 0 |
| `python3 scripts/boundary_gate.py --strings` | 0 (64s wall) |
| `python3 scripts/zero_string_gate.py` (clean) | 0 (~60s wall) |
| `python3 scripts/zero_string_gate.py --self-check` | 0 (poison fired=True) |
| `python3 scripts/boundary_gate.py --mutate --targets missing-target.rs` | 2 (`TARGET MISSING`) |
| `python3 scripts/boundary_gate.py --mutate --targets ../escape.toml` | 1 (`BAD TARGET`) |
| `python3 scripts/zero_string_gate.py --root /tmp/c06-nope-dir` | 2 (`vocab-missing`) |
| `python3 scripts/zero_datafusion_gate.py` | 0 |
| `python3 scripts/zero_datafusion_gate.py --root /tmp/c06-nope-dir` | 2 (`[SKIP] subject missing` on stderr) |
| `python3 scripts/zero_datafusion_gate.py --self-check` | 1 — PRE-EXISTING defect, see ERRATA below |
| `/tmp/c06-venv/bin/pytest tests/gates/test_gate_exits.py -q` | 0 — 17 passed in 194.77s |
| `python3 tests/gates/enforce_coverage_floor.py` | 1 — honest red, see below |

Coverage method: `coverage run --branch -p` (parallel mode, rcfile
`tests/gates/.coveragerc`), 11 data files combined with
`coverage combine`, `coverage report -m` per script. Interpreter-level
instrumentation; no gate file was modified.

## Per-script branch coverage (measured, combined db)

| Script | Branch coverage |
|---|---|
| scripts/boundary_gate.py | 24.4% |
| scripts/receipt_lint.py | 64.9% |
| scripts/zero_datafusion_gate.py | 52.6% |
| scripts/zero_string_gate.py | 40.8% |

## The 90% floor — HONEST RED (exit 1 recorded)

`enforce_coverage_floor.py --floor 90` exits 1 with four red scripts.
The floor was NOT lowered. Gap list (coverage report -m line ranges are
in the combined report; headline gaps):

- boundary_gate.py: the entire §6.1 mutation campaign engine
  (lines 1406-1634, 1662-1780) and most per-operator mutators are
  unexercised by --deps/--strings/exit-mode probes.
- zero_string_gate.py: --courts, --dep-direction, --register modes and
  the artifact-scan body (lines 1381-1450, 1599-1627, 1695-1723,
  1855-1916, 2060-2170) unexercised.
- receipt_lint.py: --json, --self-check and the content-similarity
  receipt validator (156-188) unexercised by the suite (self-check was
  covered by the ad hoc coverage run only).
- zero_datafusion_gate.py: --json, --verbose, P1/P2 deep paths.

Closing the floor requires invoking --mutate/--courts/--register under
coverage — follow-on lane work, not a floor adjustment.

## Anti-vacuity

- `test_gate_exit_assertions_have_teeth`: the same assert pattern with a
  deliberately wrong expected exit raises inside the court; the court
  fails the suite if the twin ever stops raising. Worked example cited
  in the test body (boundary --deps: true claim `== 0` holds, twin
  `== 1` raises AssertionError).
- Poison fixtures (ghost receipt, poisoned virtual Cargo workspace,
  missing subjects) all produce nonzero exits; two silent-pass probes
  run inside the floor check and both FIRED (exit 1 / exit 2).
- Every pytest exit-mode test was first witnessed against the real
  gates by hand (table above) before being pinned.

## ERRATA

1. `scripts/zero_datafusion_gate.py --self-check` exits 1 this session:
   `shutil.Error` copying the repo tree — the sandbox copy walks a
   self-recursive symlink loop under
   `vendor/nanoarrow/python/subprojects/arrow-nanoarrow/...` (each level
   re-nests `python/subprojects/arrow-nanoarrow`). Out of C06 ownership
   (file is unmodified-committed; the loop is repo content). Recorded
   with exit=1; owner should exclude the symlink loop from its sandbox
   copy.
2. Corrected dispatch: `scripts/verify_cpk.py` does not exist; the Cpk
   rig is `crates/bench/src/bin/cpk_study.rs` (Rust, G09 scope) — out of
   C06 scope, not measured.
3. `receipt_lint.py --strict` on the live repo exits 1 (witnessed) —
   real UNRECEIPTED-CLAIM findings exist in docs/jira/v26.11.2. This is
   the gate working, not a test defect; non-strict census mode is the
   nominal 0 path.
4. First `enforce_coverage_floor.py` run exited 2 (glob ran after
   `coverage combine` had consumed the fragments) and a second bug
   double-counted gaps via absolute/relative path mismatch — both fixed
   in this lane before the recorded run; the recorded floor exit 1 is
   from the fixed checker (sha256 6e100472b6435fdc).

## Fences (must-not-regress surfaces adjacent to this diff)

- scripts/ files untouched: sha256s recorded above; any drift is another
  lane's edit, disclosed here as cross-lane surface.
- Existing gates still exit 0 in nominal modes after this lane ran
  (table above) — no regression observed.
- No git commands run; /tmp build/test leases used: /tmp/c06-venv,
  /tmp/c06-cov, /tmp/c06-sb (deleted at lane end below).

## Standing

PARTIAL. Suite ALIVE (17/17, exit 0, this session). Floor checker ALIVE
and honestly red (exit 1) — 90% branch floor NOT met on any of the four
tested gate scripts; no silent-pass. verify_disassembly_invariants.py
measurement: loud [SKIP] (G03 fence).
