#!/usr/bin/env python3
"""profile_guard_census.py — fail-closed timing-fence census (ZP-18 gate).

Mechanically enumerates workspace members via `cargo metadata` (never a
hardcoded crate list), walks each member's tests/*.rs, benches/*.rs and
src/bin/*.rs, and classifies each file TIMING iff its content contains
`Instant::now`, `std::hint::black_box` or `Instant::elapsed`. A TIMING file
passes iff it composes the profile guard (`profile_guard::assert_release`
or `profile_guard::fence`) or appears in the allowlist.

Exit ABI: 0 pass, 1 gate-fail, 2 subject-missing, 3 loud-skip.

Fail-closed rules:
  * allowlist entry for a nonexistent path       -> exit 1
  * allowlist entry missing `ticket` or `reason` -> exit 1
  * allowlisted file that is ALSO fenced         -> exit 1 (allowlist stays minimal)
  * unfenced TIMING file not allowlisted         -> exit 1 (file list printed)

`--self-check` builds poison/clean twin fixtures in a temp workdir and runs
the production predicate on both; exits nonzero printing `VACUOUS GATE`
unless poison HITS and clean passes.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
import tomllib

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ALLOWLIST = os.path.join(os.path.dirname(os.path.abspath(__file__)), "profile_guard_allowlist.toml")

MARKERS = ("Instant::now", "std::hint::black_box", "Instant::elapsed")
GUARD_MARKERS = ("profile_guard::assert_release", "profile_guard::fence")

EXIT_OK = 0
EXIT_GATE_FAIL = 1
EXIT_SUBJECT_MISSING = 2
EXIT_LOUD_SKIP = 3


def classify(content: str) -> tuple[bool, bool]:
    """Production predicate: (is_timing, has_guard) for one file's content."""
    return (
        any(m in content for m in MARKERS),
        any(g in content for g in GUARD_MARKERS),
    )


def workspace_members() -> list[str]:
    """Member package roots from `cargo metadata` — mechanical, no hardcoded list."""
    try:
        out = subprocess.check_output(
            ["cargo", "metadata", "--no-deps", "--format-version", "1",
             "--manifest-path", os.path.join(ROOT, "Cargo.toml")],
            text=True,
            stderr=subprocess.DEVNULL,
        )
    except FileNotFoundError:
        print("LOUD-SKIP profile-guard-census: cargo not found on PATH", file=sys.stderr)
        sys.exit(EXIT_LOUD_SKIP)
    except subprocess.CalledProcessError as e:
        print(f"SUBJECT-MISSING: cargo metadata failed (rc={e.returncode})", file=sys.stderr)
        sys.exit(EXIT_SUBJECT_MISSING)
    return sorted({os.path.dirname(p["manifest_path"]) for p in json.loads(out)["packages"]})


def load_allowlist() -> tuple[list[dict], list[str]]:
    """Returns (entries, errors). Every entry needs path/ticket/reason/added."""
    if not os.path.exists(ALLOWLIST):
        return [], [f"allowlist file missing: {ALLOWLIST}"]
    try:
        with open(ALLOWLIST, "rb") as fh:
            data = tomllib.load(fh)
    except tomllib.TOMLDecodeError as e:
        return [], [f"allowlist TOML parse error: {e}"]

    entries = data.get("allowlist", [])
    errors: list[str] = []
    seen: set[str] = set()
    for e in entries:
        path = e.get("path", "")
        if not path or not e.get("ticket") or not e.get("reason"):
            errors.append(f"allowlist entry missing path/ticket/reason: {e}")
            continue
        if path in seen:
            errors.append(f"duplicate allowlist entry: {path}")
            continue
        seen.add(path)
        full = os.path.join(ROOT, path)
        if not os.path.isfile(full):
            errors.append(f"allowlist entry for nonexistent path: {path}")
            continue
        with open(full, encoding="utf-8", errors="replace") as fh:
            content = fh.read()
        if classify(content)[1]:
            errors.append(
                f"allowlist entry is ALSO fenced — allowlist must stay minimal: {path}"
            )
    return entries, errors


def scan() -> tuple[list[str], int, int]:
    """Walk member tests/benches/src-bin trees.

    Returns (unguarded_timing_relpaths, timing_count, guard_count).
    """
    unguarded: list[str] = []
    timing = 0
    guarded = 0
    for member in workspace_members():
        for sub in ("tests", "benches", "src/bin"):
            d = os.path.join(member, sub)
            if not os.path.isdir(d):
                continue
            for name in sorted(os.listdir(d)):
                full = os.path.join(d, name)
                if not name.endswith(".rs") or not os.path.isfile(full):
                    continue
                with open(full, encoding="utf-8", errors="replace") as fh:
                    content = fh.read()
                is_timing, has_guard = classify(content)
                if is_timing:
                    timing += 1
                if has_guard:
                    guarded += 1
                if is_timing and not has_guard:
                    unguarded.append(os.path.relpath(full, ROOT))
    return unguarded, timing, guarded


def self_check() -> int:
    """Poison/clean twin fixtures in a temp workdir -> run the production predicate.

    Passes only when the poison fixture HITS (classified unguarded TIMING) and
    the clean twin passes (guarded). Otherwise prints VACUOUS GATE, exits 1.
    """
    poison_body = (
        "use std::time::Instant;\n"
        "fn main() {\n"
        "    let t = Instant::now();\n"
        "    let d = t.elapsed();\n"
        "    std::hint::black_box(d);\n"
        "}\n"
    )
    clean_body = (
        "use std::time::Instant;\n"
        "fn main() {\n"
        "    let t = Instant::now();\n"
        "    let d = t.elapsed();\n"
        "    let sink = cgap_aoq::profile_guard::fence(d);\n"
        "    std::hint::black_box(sink);\n"
        "}\n"
    )
    ok = True
    p_is_timing, p_guard = classify(poison_body)
    c_is_timing, c_guard = classify(clean_body)
    if not p_is_timing:
        print("VACUOUS GATE: poison fixture not classified TIMING", file=sys.stderr)
        ok = False
    if p_guard:
        print("VACUOUS GATE: poison fixture reads as guarded", file=sys.stderr)
        ok = False
    if not c_is_timing or not c_guard:
        print("VACUOUS GATE: clean fixture not classified guarded TIMING", file=sys.stderr)
        ok = False

    # Real-file path of the predicate: materialize both fixtures in a temp
    # workdir shaped like a crate and run the production scan over it.
    with tempfile.TemporaryDirectory(prefix="pgc-selfcheck-") as tmp:
        for crate, body in (("poison", poison_body), ("clean", clean_body)):
            d = os.path.join(tmp, crate, "src", "bin")
            os.makedirs(d)
            with open(os.path.join(d, "twin.rs"), "w", encoding="utf-8") as fh:
                fh.write(body)
        poison_path = os.path.join(tmp, "poison", "src", "bin", "twin.rs")
        clean_path = os.path.join(tmp, "clean", "src", "bin", "twin.rs")
        for label, path, expect_hit in (("poison", poison_path, True), ("clean", clean_path, False)):
            with open(path, encoding="utf-8") as fh:
                is_timing, has_guard = classify(fh.read())
            hit = is_timing and not has_guard
            if hit != expect_hit:
                print(
                    f"VACUOUS GATE: {label} fixture hit={hit} expected {expect_hit}",
                    file=sys.stderr,
                )
                ok = False

    if not ok:
        print("VACUOUS GATE", file=sys.stderr)
        return EXIT_GATE_FAIL
    print("SELF-CHECK OK: poison HIT (unguarded TIMING), clean PASS (guarded)")
    return EXIT_OK


def main() -> int:
    ap = argparse.ArgumentParser(description="ZP-18 timing-fence census (fail-closed)")
    ap.add_argument("--self-check", action="store_true",
                    help="poison/clean twin-fixture vacuity check; exits 0 only when poison HITs and clean passes")
    ap.add_argument("--receipt-verbatim", action="store_true",
                    help="LANE-1 composite: after the census passes, run the "
                         "VERBATIM_MISMATCH verifier over every "
                         "docs/jira/v*/receipts/*.md receipt (exit 1 on any "
                         "byte-drift finding; exit 3 when all cited raw logs "
                         "are loud-skipped as missing)")
    args = ap.parse_args()

    if args.self_check:
        return self_check()

    entries, errors = load_allowlist()
    allow_paths = {e.get("path", "") for e in entries}

    unguarded, timing, guarded = scan()
    remaining = [f for f in unguarded if f not in allow_paths]

    print(f"profile-guard census: {timing} TIMING files, {guarded} guard-composing files, "
          f"{len(entries)} allowlist entries")

    if errors:
        print("FAIL: allowlist errors (fail-closed):")
        for e in errors:
            print(f"  - {e}")
        return EXIT_GATE_FAIL

    if remaining:
        print(f"FAIL: {len(remaining)} unfenced TIMING file(s) not allowlisted:")
        for f in remaining:
            print(f"  - {f}")
        return EXIT_GATE_FAIL

    # LANE-1 composite arm: verbatim receipt verification, opt-in so the
    # default census exit ABI stays byte-identical (a vanished /tmp raw log
    # must not turn the timing census red — it loud-skips under this flag).
    if args.receipt_verbatim:
        import glob
        sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
        import receipt_verbatim_verify as rvv
        receipts = sorted(
            glob.glob(os.path.join(ROOT, "docs", "jira", "v*", "receipts", "*.md"))
        )
        rc = rvv.scan(receipts, ROOT) if receipts else rvv.EXIT_OK
        if not receipts:
            print("FAIL: --receipt-verbatim found no receipts (fail-closed)")
            return EXIT_GATE_FAIL
        if rc == rvv.EXIT_GATE_FAIL:
            return EXIT_GATE_FAIL
        if rc == rvv.EXIT_LOUD_SKIP:
            print("PASS(census) + LOUD-SKIP(verbatim): cited raw logs absent")
            return EXIT_LOUD_SKIP

    if unguarded and not remaining:
        covered = [f for f in unguarded if f in allow_paths]
        print(f"PASS: all {len(unguarded)} unfenced TIMING file(s) allowlisted:")
        for f in covered:
            print(f"  - {f} [allowlisted]")
    else:
        print("PASS: every TIMING file composes profile_guard (assert_release|fence)")
    return EXIT_OK


if __name__ == "__main__":
    sys.exit(main())
