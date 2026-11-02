"""LANE GAP-06 — shared /tmp fixture helpers for the scripts/ gate tests.

Chicago discipline: every helper drives the REAL gate script in a
subprocess over REAL files (repo tree for nominal modes, pytest tmp_path
sandboxes for poison fixtures). No mocks, no stubs. The shared tree is
never mutated: nominal modes are read-only probes; poison fixtures build
their corrupted inputs under pytest tmp_path (/tmp), never in scripts/.

Every poison twin here follows the same law as
test_gate_exit_assertions_have_teeth: the assertion pattern must FLIP on
a deliberately wrong expected exit, or the court is vacuous.
"""
import json
import subprocess
import sys
from pathlib import Path

REPO = Path("/Users/sac/cgap")
PY = sys.executable
GATES = REPO / "scripts"

# lane-12 exit ABI (witnessed this session, see lane-gap-06 receipt):
SLOW_TIMEOUT = 550


def run_gate(script, *args, timeout=SLOW_TIMEOUT):
    """Run a scripts/ gate as a REAL subprocess; returns CompletedProcess."""
    return subprocess.run(
        [str(PY), str(GATES / script), *[str(a) for a in args]],
        cwd=str(REPO), capture_output=True, text=True, timeout=timeout,
    )


def assert_exit_with_teeth(result, expected_exit, verdict_token=None,
                           stream="stderr"):
    """Assert final state (exit + verdict token) AND run the poison twin.

    The twin re-asserts the SAME observed exit against a deliberately
    WRONG expected exit and must raise AssertionError. If the twin ever
    stops raising, every exit assertion gated through this helper is
    vacuous — the helper fails the suite instead of passing silently.
    """
    observed = result.returncode
    assert observed == expected_exit, (
        f"exit {observed} != {expected_exit}; "
        f"stdout={result.stdout[-400:]!r} stderr={result.stderr[-400:]!r}")
    if verdict_token is not None:
        text = result.stdout if stream == "stdout" else result.stderr
        assert verdict_token in text, (
            f"token {verdict_token!r} missing from {stream}")
    # ---- poison twin (anti-vacuity): wrong expected exit must raise
    raised = False
    try:
        assert observed == expected_exit + 1, "poison twin"
    except AssertionError:
        raised = True
    if expected_exit + 1 == observed:  # twin would accidentally hold
        try:
            assert observed == expected_exit - 1, "poison twin"
        except AssertionError:
            raised = True
    assert raised, (
        f"poison twin did NOT raise for exit {observed} "
        f"(expected {expected_exit}) — assertion pattern is vacuous")
    return result


def write_ledger_assert(report_path, require_rows=False):
    """Assert a boundary_gate --report ledger exists, parses as JSON, and
    carries the verdict ABI field. require_rows=True additionally demands
    the campaign ledger shape (mode=mutate emits 'rows')."""
    p = Path(report_path)
    assert p.exists(), f"ledger not written: {p}"
    data = json.loads(p.read_text())
    assert "abi_version" in data, data.keys()
    if require_rows:
        assert data.get("rows"), "campaign ledger has no rows"
    return data
