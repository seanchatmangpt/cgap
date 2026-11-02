#!/usr/bin/env python3
"""gate_cargo_test.py -- mandatory-execution test gate (lane AE3).

Structural law: parse every ``test result:`` line from `cargo test` stdout.
  exit 0 : every crate met its quota, every target executed >=1 test
  exit 1 : quota deficit, failed tests, or compile-abort (0 tests + errors)
  exit 2 : subject missing (crate not a workspace member / cargo missing)
  exit 3 : VACUOUS_TEST_EVASION -- a target ran with passed==0 and
           filtered_out==0 (tests evaded, not filtered)

No Process.sleep; cargo exit codes are the observable.
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

RESULT_RE = re.compile(
    r"^test result: \S+\. (\d+) passed; (\d+) failed; (\d+) ignored; "
    r"(\d+) measured; (\d+) filtered out",
    re.MULTILINE,
)
ERROR_RE = re.compile(r"^\s*error(\[E\d+\])?:", re.MULTILINE)
SEGMENT_RE = re.compile(r"^\s+(Running|Doc-tests) ", re.MULTILINE)

# A doctest section legitimately reports `0 passed; 0 filtered out` when the
# crate carries no doctests. Evasion law fires on unit/integration targets
# only; a Doc-tests segment with 0 tests is not evasion.

# ~80% of measured counts (see lane receipt). cgap-parquet measured count is
# UNMEASURABLE this session (shared-tree compile abort, see receipt); the
# dispatch floor 50 stands until a measured count replaces it.
DEFAULT_QUOTAS: dict[str, int] = {
    "cgap-branchless": 28,  # measured 35
    "cgap-parquet": 50,  # dispatch floor; measured count UNMEASURABLE
    "cgap-arrow": 153,  # measured 192
    "cgap": 12,  # measured 15
}


def run_cargo(crate: str, jobs: int | None) -> tuple[int, str]:
    cmd = ["cargo", "test", "-p", crate]
    if jobs:
        cmd += ["-j", str(jobs)]
    try:
        proc = subprocess.run(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            timeout=3600,
        )
    except FileNotFoundError:
        print("[FATAL] cargo not found on PATH", file=sys.stderr)
        raise SystemExit(2)
    except subprocess.TimeoutExpired:
        print(f"[FATAL] cargo test -p {crate} exceeded 3600s", file=sys.stderr)
        raise SystemExit(1)
    return proc.returncode, proc.stdout


def parse_results(crate: str, out: str, cargo_rc: int) -> tuple[dict, list[str]]:
    """Parse all `test result:` lines; classify the target set."""
    findings: list[str] = []
    total_passed = 0
    vacuous_targets = 0
    executed_targets = 0
    # Split into per-target segments at Running/Doc-tests headers so a
    # 0-test doctest section is never misread as test evasion.
    segments = SEGMENT_RE.split(out)
    # SEGMENT_RE.split interleaves prose and (kind, header); walk pairs.
    seg_kinds = [
        (segments[i], segments[i + 1]) for i in range(1, len(segments) - 1, 2)
    ]
    for kind, body in seg_kinds:
        for m in RESULT_RE.finditer(body):
            executed_targets += 1
            passed, failed, _ign, _meas, filtered = (
                int(g) for g in m.groups()
            )
            total_passed += passed
            if (
                kind == "Running"
                and passed == 0
                and filtered == 0
                and failed == 0
            ):
                vacuous_targets += 1
                findings.append(
                    f"VACUOUS_TARGET [{crate}]: passed==0 filtered_out==0 "
                    f"(0 executed tests)"
                )
    if executed_targets == 0 or (total_passed == 0 and cargo_rc != 0):
        if ERROR_RE.search(out) or cargo_rc not in (0,):
            findings.append(
                f"COMPILE_ABORT [{crate}]: 0 tests executed, cargo rc="
                f"{cargo_rc} -- counted as FAILURE, never a pass"
            )
    return {
        "crate": crate,
        "cargo_rc": cargo_rc,
        "targets": executed_targets,
        "passed": total_passed,
        "failed_targets": len(re.findall(r"test result: FAILED", out)),
        "vacuous_targets": vacuous_targets,
        "errors": findings,
    }, findings


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--crates",
        default=",".join(DEFAULT_QUOTAS),
        help="comma-separated crate list",
    )
    ap.add_argument(
        "--quota",
        action="append",
        default=[],
        metavar="CRATE=N",
        help="override quota, repeatable",
    )
    ap.add_argument(
        "--jobs", type=int, default=None, help="cargo -j parallelism"
    )
    ap.add_argument(
        "--json", action="store_true", help="emit machine-readable report"
    )
    args = ap.parse_args()

    quotas = dict(DEFAULT_QUOTAS)
    for spec in args.quota:
        crate, _, n = spec.partition("=")
        if not n.isdigit():
            print(f"[FATAL] bad quota spec {spec!r}", file=sys.stderr)
            return 2
        quotas[crate] = int(n)

    crate_list = [c.strip() for c in args.crates.split(",") if c.strip()]
    for c in crate_list:
        if c not in quotas:
            quotas[c] = 1

    report: list[dict] = []
    exit_code = 0
    for crate in crate_list:
        rc, out = run_cargo(crate, args.jobs)
        summary, findings = parse_results(crate, out, rc)
        deficit = quotas[crate] - summary["passed"]
        if deficit > 0:
            findings.append(
                f"QUOTA_DEFICIT [{crate}]: passed {summary['passed']} < "
                f"quota {quotas[crate]} (deficit {deficit})"
            )
            summary["quota_deficit"] = deficit
        if summary["failed_targets"]:
            findings.append(
                f"FAILED_TESTS [{crate}]: {summary['failed_targets']} "
                f"target(s) with failing tests"
            )
        if any("VACUOUS_TARGET" in f for f in findings):
            exit_code = max(exit_code, 3)
        elif any(
            "QUOTA_DEFICIT" in f or "COMPILE_ABORT" in f or "FAILED_TESTS" in f
            for f in findings
        ):
            exit_code = max(exit_code, 1)
        report.append(summary)
        for f in findings:
            print(f"[GATE] {f}", file=sys.stderr)

    if args.json:
        print(
            json.dumps(
                {
                    "exit": exit_code,
                    "quotas": quotas,
                    "crates": report,
                },
                indent=2,
            )
        )
    if exit_code == 0:
        print(f"[GATE] PASS: {len(report)} crates met quota, 0 vacuous")
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
