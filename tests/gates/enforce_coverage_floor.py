#!/usr/bin/env python3
"""LANE C06 — fail-closed 90% branch floor for the scripts/ gate surface.

Usage:
    python3 tests/gates/enforce_coverage_floor.py [--floor 90]

Exits:
    0  every tested gate >= floor AND silent-pass probes all fired
    1  coverage below floor (honest red), OR a poison probe stayed
       silent (exit 0 on corrupted input) — the suite fails
    2  coverage data missing / unreadable

Coverage data is combined from the parallel-mode runs recorded by
tests/gates/.coveragerc (data_file /tmp/c06-cov/.coverage.*).
"""
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path("/Users/sac/cgap")
VENV = "/tmp/c06-venv/bin"
DATA_GLOB = "/tmp/c06-cov/.coverage.*"
TESTED = {
    "scripts/boundary_gate.py",
    "scripts/zero_string_gate.py",
    "scripts/receipt_lint.py",
    "scripts/zero_datafusion_gate.py",
}
FLOOR = 90.0

# Fail-closed silent-pass probes: (label, argv, forbidden-stdout token).
# A gate that reports PASS on corrupted input means the probe's poison
# did not fire — floor check fails even if branch coverage is green.
PROBES = [
    ("receipt_lint ghost receipt",
     ["scripts/receipt_lint.py", "--root", "<SB>", "--dir",
      "docs/jira/v26.11.2/waves", "--strict"],
     "UNRECEIPTED-CLAIM"),
    ("zero_datafusion missing root",
     ["scripts/zero_datafusion_gate.py", "--root", "/tmp/c06-nope-dir"],
     "ZERO-DATAFUSION: SKIP 0"),
]


def sh(args):
    return subprocess.run(args, cwd=str(REPO), capture_output=True,
                          text=True, timeout=600)


def combine():
    files = sorted(Path("/tmp/c06-cov").glob(".coverage.*"))
    combined = Path("/tmp/c06-cov/.coverage")
    if not files:
        if combined.exists():
            return 0  # already combined this session
        print("FLOOR: no coverage data at %s" % DATA_GLOB)
        return 2
    rc = sh([f"{VENV}/coverage", "combine"])
    if rc.returncode != 0:
        # combine fails when a previous combine already produced
        # /tmp/c06-cov/.coverage — keep the existing combined db.
        if not Path("/tmp/c06-cov/.coverage").exists():
            print("FLOOR: combine failed: %s" % rc.stderr.strip())
            return 2
    return 0


def coverage_rows():
    rc = sh([f"{VENV}/coverage", "json", "-q", "-o", "-",
             "--rcfile=/Users/sac/cgap/tests/gates/.coveragerc"])
    if rc.returncode != 0:
        print("FLOOR: coverage json failed: %s" % rc.stderr.strip())
        return None
    data = json.loads(rc.stdout)
    return {path: meta["summary"] for path, meta in data["files"].items()}


def silent_pass_probes():
    """Poison probes must be loud. Any exit-0-on-corrupted-input = FAIL 1."""
    bad = []
    for label, argv, token in PROBES:
        sb = tempfile.mkdtemp(prefix="c06-floor-")
        root = sb
        waves = Path(root) / "docs/jira/v26.11.2/waves"
        waves.mkdir(parents=True)
        (waves / "w1-claim.md").write_text(
            "# claim\n\nThis landed. waves/ghost.md:1\n")
        args = [a.replace("<SB>", root) for a in argv]
        r = sh([sys.executable] + args)
        fired = token in r.stdout and r.returncode != 0
        if not fired:
            bad.append((label, r.returncode))
        print("  probe %-32s exit=%d %s"
              % (label, r.returncode, "FIRED" if fired else "SILENT-PASS"))
    return bad


def main():
    floor = float(sys.argv[sys.argv.index("--floor") + 1]) \
        if "--floor" in sys.argv else FLOOR
    rc = combine()
    if rc:
        return rc
    rows = coverage_rows()
    if rows is None:
        return 2
    gaps, reds = [], []
    for path, summ in sorted(rows.items()):
        if path not in TESTED:
            continue
        pct = summ.get("percent_covered", 0.0)
        cov_type = "branch" if summ.get("num_branches") else "line"
        print("  %-36s %s%% %s covered" % (path, round(pct, 1), cov_type))
        if pct < floor:
            reds.append((path, pct))
    # A tested gate with NO coverage row is a silent omission — red.
    covered_paths = set(rows)
    for t in sorted(TESTED):
        if t not in covered_paths and str(REPO / t) not in covered_paths:
            gaps.append(t)
    if gaps:
        print("FLOOR: no coverage data for %s" % gaps)
        reds.extend((g, 0.0) for g in gaps)
    bad = silent_pass_probes()
    if bad:
        print("FLOOR: SILENT-PASS detected (exit 0 on corrupted input): %s"
              % bad)
    if reds or bad:
        print("FLOOR: FAIL (floor=%.0f%%) — %d red script(s), %d silent-pass"
              % (floor, len(reds), len(bad)))
        for p, pct in reds:
            print("  RED %s %.1f%% < %.0f%%" % (p, pct, floor))
        return 1
    print("FLOOR: PASS (floor=%.0f%%)" % floor)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
