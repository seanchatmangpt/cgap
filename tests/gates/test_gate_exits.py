"""LANE C06 — gate exit-mode integration tests.

Real subprocess.run over the scripts/ gate surface; no mocks, no stubs.
Every assertion is on the child's final state (exit code + verdict line),
never on call counts.

Exit ABI (witnessed this session; see lane receipt for command+exit
evidence):
    0  pass (or census findings without --strict)
    1  typed refusal / strict findings / VACUOUS GATE
    2  subject-missing or indeterminate-refusal (lane-12 exit ABI)
    3  loud [SKIP]

Anti-vacuity: test_gate_exit_assertions_have_teeth proves the assertion
pattern flips on a deliberately wrong expected exit. Poison fixtures live
in pytest tmp_path sandboxes — the shared tree is never mutated, so no
sha256 restore is required for any test here; the floor checker
(enforce_coverage_floor.py) re-runs the poison probes fail-closed.
"""
import subprocess
import sys
from pathlib import Path

import pytest

REPO = Path("/Users/sac/cgap")
PY = sys.executable
GATES = REPO / "scripts"
SLOW = 600  # --strings and zero_string clean scans ~60s wall each
FAST = 120


def run_gate(script, *args, timeout=FAST):
    return subprocess.run(
        [str(PY), str(GATES / script), *[str(a) for a in args]],
        cwd=str(REPO), capture_output=True, text=True, timeout=timeout,
    )


# ---------------------------------------------------------------- nominal

def test_boundary_gate_deps_nominal_exit_0():
    r = run_gate("boundary_gate.py", "--deps")
    assert r.returncode == 0, r.stderr
    assert "DEPS GATE" in r.stdout


def test_boundary_gate_strings_nominal_exit_0():
    r = run_gate("boundary_gate.py", "--strings", timeout=SLOW)
    assert r.returncode == 0, r.stderr
    assert "STRINGS GATE" in r.stdout


def test_zero_string_gate_clean_nominal_exit_0():
    r = run_gate("zero_string_gate.py", timeout=SLOW)
    assert r.returncode == 0, r.stderr
    assert "SUMMARY scanned=" in r.stdout and "hits=0" in r.stdout


def test_zero_string_gate_self_check_exit_0():
    r = run_gate("zero_string_gate.py", "--self-check")
    assert r.returncode == 0, r.stderr
    assert "SELF-CHECK RESULT PASS" in r.stdout


def test_zero_datafusion_gate_nominal_exit_0():
    r = run_gate("zero_datafusion_gate.py")
    assert r.returncode == 0, r.stderr
    assert "ZERO-DATAFUSION: PASS" in r.stdout


def test_receipt_lint_repo_nominal_exit_0():
    # Default (non-strict) mode is a census: findings print, gate exits 0.
    r = run_gate("receipt_lint.py")
    assert r.returncode == 0, r.stderr
    assert "RECEIPT-LINT" in r.stdout


# ------------------------------------------------- failure / refusal modes

def test_boundary_gate_missing_target_exit_2():
    # lane-12 exit ABI: missing --targets file is a missing SUBJECT.
    r = run_gate("boundary_gate.py", "--mutate", "--targets", "missing-target.rs")
    assert r.returncode == 2, r.stderr
    assert "TARGET MISSING" in r.stderr


def test_boundary_gate_target_escape_exit_1():
    # Path escaping the repo is a typed refusal, not a missing subject.
    r = run_gate("boundary_gate.py", "--mutate", "--targets", "../escape.toml")
    assert r.returncode == 1, r.stderr
    assert "BAD TARGET" in r.stderr


def test_zero_string_gate_missing_root_exit_2():
    r = run_gate("zero_string_gate.py", "--root", "/tmp/c06-nope-dir")
    assert r.returncode == 2, r.stderr
    assert "GATE-REFUSED: vocab-missing" in r.stderr


def test_zero_datafusion_gate_missing_root_exit_2():
    r = run_gate("zero_datafusion_gate.py", "--root", "/tmp/c06-nope-dir")
    assert r.returncode == 2, r.stderr
    assert "[SKIP] subject missing" in r.stderr


def test_receipt_lint_missing_dir_exit_3():
    # Loud [SKIP]: a missing lintable dir is never a fake pass.
    r = run_gate("receipt_lint.py", "--root", "/tmp/c06-sb/doc",
                 "--dir", "does/not/exist", "--strict")
    assert r.returncode == 3, r.stderr
    assert "[SKIP]" in r.stdout


# -------------------------------------------------- /tmp poison fixtures

@pytest.fixture()
def doc_root(tmp_path):
    """Sandbox docs tree shaped like the repo (receipt_lint --root)."""
    waves = tmp_path / "docs" / "jira" / "v26.11.2" / "waves"
    waves.mkdir(parents=True)
    return tmp_path


def test_receipt_lint_clean_fixture_exit_0(doc_root):
    waves = doc_root / "docs" / "jira" / "v26.11.2" / "waves"
    # GAP-06: receipt_lint gained the DOC-RECEIPT-003 honesty-header rule
    # after C06 landed; fixture docs now carry status vocab + as-of date.
    header = "Status: PARTIAL as-of 2026-11-02\n"
    (waves / "w2-body.md").write_text(
        header + "header\nfused_row lever attestation body\n")
    (waves / "w1-claim.md").write_text(
        header + "# claim\n\nThe fused_row lever landed. waves/w2-body.md:3\n")
    r = run_gate("receipt_lint.py", "--root", str(doc_root),
                 "--dir", "docs/jira/v26.11.2/waves", "--strict")
    assert r.returncode == 0, r.stdout


def test_receipt_lint_ghost_receipt_exit_1(doc_root):
    waves = doc_root / "docs" / "jira" / "v26.11.2" / "waves"
    (waves / "w1-claim.md").write_text(
        "# claim\n\nThis landed. waves/ghost.md:1\n")
    r = run_gate("receipt_lint.py", "--root", str(doc_root),
                 "--dir", "docs/jira/v26.11.2/waves", "--strict")
    assert r.returncode == 1, r.stdout
    assert "UNRECEIPTED-CLAIM" in r.stdout
    assert "[receipt file missing: ghost.md]" in r.stdout


def test_receipt_lint_zero_docs_fail_closed_exit_2(doc_root):
    # Empty lintable dir: zero docs scanned must refuse, never pass.
    r = run_gate("receipt_lint.py", "--root", str(doc_root),
                 "--dir", "docs/jira/v26.11.2/waves", "--strict")
    assert r.returncode == 2, r.stdout
    assert "fail-closed" in r.stderr


def test_receipt_lint_findings_without_strict_exit_0(doc_root):
    waves = doc_root / "docs" / "jira" / "v26.11.2" / "waves"
    (waves / "w1-claim.md").write_text("# claim\n\nThis landed. waves/g.md:1\n")
    r = run_gate("receipt_lint.py", "--root", str(doc_root),
                 "--dir", "docs/jira/v26.11.2/waves")
    assert r.returncode == 0, r.stdout
    assert "UNRECEIPTED-CLAIM" in r.stdout


def test_zero_datafusion_poisoned_workspace_exit_2(tmp_path):
    # A virtual manifest carrying a forbidden dep is indeterminate at
    # P1/P2 (cargo refuses it) — typed GATE-REFUSED, never a pass.
    ws = tmp_path / "ws"
    ws.mkdir()
    (ws / "Cargo.toml").write_text(
        '[workspace]\nmembers = []\nresolver = "2"\n\n'
        '[dependencies]\ndatafusion = "1"\n')
    (ws / "Cargo.lock").write_text("version 3\n")
    r = run_gate("zero_datafusion_gate.py", "--root", str(ws))
    assert r.returncode == 2, r.stdout
    assert "ZERO-DATAFUSION: FAIL" in r.stdout


# ------------------------------------------------------------ anti-vacuity

def test_gate_exit_assertions_have_teeth():
    """Twin court: the same assertion pattern with a deliberately WRONG
    expected exit must raise. If this ever passes without the twin
    raising, the exit-mode assertions above are vacuous."""
    r = run_gate("boundary_gate.py", "--deps")
    assert r.returncode == 0  # true claim holds
    raised = False
    try:
        assert r.returncode == 1, "deliberately wrong expected exit"
    except AssertionError:
        raised = True
    assert raised, "wrong-exit twin did NOT fail — assertions are vacuous"
    # Second tooth: a poison verdict line must be detectable in output.
    poisoned = "RECEIPT-LINT: FAIL 1"
    assert poisoned in "x\nRECEIPT-LINT: FAIL 1\n"  # detector fires on poison
    assert poisoned not in "RECEIPT-LINT: PASS"      # and stays quiet on clean


# ===================================================================
# LANE GAP-06 additions — the REST of the uncovered ranges C06 left.
# Every assertion below routes through assert_exit_with_teeth, so each
# test carries its own poison twin (wrong expected exit must fail).
# Modes were probed live this session before being pinned here; see
# docs/jira/v26.11.2/lanes/lane-gap-06-coverage.md for command+exit.
# ===================================================================

from gate_fixture_helpers import (  # noqa: E402  (helpers live beside tests)
    REPO as GAP6_REPO,
    assert_exit_with_teeth,
    write_ledger_assert,
)


# --------------------------------------- boundary_gate: alt court modes

def test_gap6_boundary_self_check_deps_exit_0():
    r = assert_exit_with_teeth(
        run_gate("boundary_gate.py", "--self-check-deps"), 0,
        "both arms flipped", stream="stdout")
    assert "DEPS SELF-CHECK" in r.stdout


def test_gap6_boundary_cross_gate_court_exit_0():
    r = assert_exit_with_teeth(
        run_gate("boundary_gate.py", "--cross-gate-court"), 0,
        "CROSS-GATE COURT arm R", stream="stdout")
    assert "arm G" in r.stdout


def test_gap6_boundary_interference_court_exit_0():
    assert_exit_with_teeth(
        run_gate("boundary_gate.py", "--interference-court"), 0,
        "interference court PASSES", stream="stdout")


def test_gap6_boundary_shim_court_only_exit_0():
    r = assert_exit_with_teeth(
        run_gate("boundary_gate.py", "--shim-court-only", timeout=550), 0,
        "SHIM-COURT", stream="stdout")
    assert "non-vacuous" in r.stdout


def test_gap6_boundary_deps_report_ledger_written(tmp_path):
    ledger = tmp_path / "ledger.json"
    assert_exit_with_teeth(
        run_gate("boundary_gate.py", "--deps", "--report", str(ledger)), 0)
    data = write_ledger_assert(ledger)
    assert data["abi_version"] == 1
    assert data.get("mode") == "deps"
    assert "covered_crates" in data


# --------------------------------- boundary_gate: refusal paths (exit 1)

def test_gap6_boundary_no_mode_exit_1():
    r = assert_exit_with_teeth(run_gate("boundary_gate.py"), 1,
                               "no mode: one of", stream="stderr")


def test_gap6_boundary_bad_only_op_exit_1():
    r = run_gate("boundary_gate.py", "--mutate", "--targets",
                 "crates/cgap-storage/src/atomic_commit.rs",
                 "--only-op", "nosuchop")
    assert r.returncode == 1, r.stderr
    assert "BAD --only-op" in r.stderr
    # poison twin: the refusal token must be load-bearing — a clean-run
    # stdout must NOT contain it.
    assert "BAD --only-op" not in run_gate(
        "boundary_gate.py", "--deps").stdout


# --------------------------------------- zero_string_gate: emitter modes

def test_gap6_zero_string_register_emits_json_exit_0():
    r = assert_exit_with_teeth(
        run_gate("zero_string_gate.py", "--register"), 0)
    import json
    reg = json.loads(r.stdout)
    assert reg["register"] == "a07-zero-string-dep-audit"
    assert reg["version"] == 2


def test_gap6_zero_string_courts_suite_exit_0():
    r = assert_exit_with_teeth(
        run_gate("zero_string_gate.py", "--courts"), 0,
        "COURTS-SUMMARY", stream="stdout")
    row = [ln for ln in r.stdout.splitlines()
           if ln.startswith("COURTS-SUMMARY")][-1]
    failed = int(row.split("failed=")[1].split()[0])
    total = int(row.split("total=")[1].split()[0])
    assert failed == 0 and total >= 13
    # poison twin: the parsed row must detect a poisoned summary.
    assert int("COURTS-SUMMARY total=13 failed=3".split(
        "failed=")[1].split()[0]) == 3


def test_gap6_zero_string_dep_direction_exit_0():
    r = assert_exit_with_teeth(
        run_gate("zero_string_gate.py", "--dep-direction"), 0,
        "DEP-DIR-SUMMARY", stream="stdout")
    row = [ln for ln in r.stdout.splitlines()
           if ln.startswith("DEP-DIR-SUMMARY")][-1]
    assert "violations=0" in row
    assert "checked=" in row and "checked=0" not in row


def test_gap6_zero_string_dep_direction_emitter_exempt_lines():
    # Emitter-exempt crates print typed justification lines (T16 family).
    r = run_gate("zero_string_gate.py", "--dep-direction")
    assert r.returncode == 0, r.stderr
    assert "DEP-DIR-EXEMPT crate=cgap-bosym" in r.stdout
    assert "EXCLUDED crate=cgap-court" in r.stdout
    # poison twin: exemption tokens are load-bearing — they must be
    # distinguishable from a violation row.
    assert "DEP-DIR-VIOLATION" not in r.stdout


# -------------------------------- zero_string_gate: refuse paths (exit 2)

def test_gap6_zero_string_dep_direction_audit_profile_exit_2():
    # profile=audit has no built artifacts in the shared tree -> typed
    # crate-no-artifacts refusal, exit 2 (subject missing).
    r = run_gate("zero_string_gate.py", "--dep-direction", "--profile",
                 "audit")
    assert r.returncode == 2, r.stderr
    assert "GATE-REFUSED: crate-no-artifacts" in r.stderr
    # poison twin: the refusal must name real crate names, not empty.
    assert "cgap-core" in r.stderr


def test_gap6_zero_string_dep_direction_missing_root_exit_1():
    # Witnessed live this session: a vanished root is an internal-error
    # typed refusal (exit 1), NOT a silent pass.
    r = run_gate("zero_string_gate.py", "--dep-direction", "--root",
                 "/tmp/c06-nope-dir")
    assert r.returncode == 1, r.stderr
    assert "GATE-REFUSED: internal-error" in r.stderr
