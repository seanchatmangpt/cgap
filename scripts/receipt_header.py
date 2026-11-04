#!/usr/bin/env python3
"""receipt_header.py — thin Python emitter for the forensic receipt header.

The Python-side twin of the `cgap-receipt` Rust crate (LANE-1): script gates
(census, verbatim verifier, courts) mint the same self-contained header
block the Rust benches emit. Field order and names are the CONTRACT shared
with crates/cgap-receipt/src/lib.rs `render()`:

    plane: <caller-supplied const>
    git-sha: <live `git rev-parse HEAD`>
    target-binary: <caller-supplied>
    build-profile: release | DISQUALIFIED_PROFILE (...)
    cores: <available parallelism>
    [alloc-signature: <caller-supplied>]
    load-avg: 1m 5m 15m   (host confession; UNKNOWN per axis when unreadable)
    utc: YYYY-MM-DDTHH:MM:SSZ

Fail-closed laws:
  * non-release profile -> DISQUALIFIED_PROFILE ledger entry + nonzero exit
    (never a warning). Detection mirrors the Rust axes: env OPT_LEVEL=="0"
    or CGAP_DEBUG_ASSERTIONS=1 -> disqualified; `--require-release` makes
    it an exit-1 refusal, default emits the disqualified ledger form.
  * git identity missing -> RECEIPT-IDENTITY-MISSING, nonzero exit.

Exit ABI: 0 emitted; 1 fail-closed refusal; 64 usage.

`--self-check` builds a poison (profile-disqualified + missing-sha) and a
clean fixture through the production functions; exits nonzero printing
`VACUOUS GATE` unless poison refuses and clean emits every contract field.
"""

from __future__ import annotations

import argparse
import datetime
import os
import re
import subprocess
import sys

DISQUALIFIED_PROFILE = "DISQUALIFIED_PROFILE"

# Field-name contract shared with the Rust render().
CONTRACT_FIELDS = (
    "plane",
    "git-sha",
    "target-binary",
    "build-profile",
    "cores",
    "load-avg",
    "utc",
)

SHA_RE = re.compile(r"^[0-9a-f]{40}$")

# Identity anchor: git is interrogated under the SCRIPT's repo root, never
# the process cwd — a gate run from a foreign cwd must not mint a foreign
# repo's SHA as identity (witnessed live against cgap, 2026-11-04).
REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

EXIT_OK = 0
EXIT_GATE_FAIL = 1
EXIT_USAGE = 64


def git_sha() -> str:
    """Live git identity; raises RuntimeError on any failure (fail-closed)."""
    try:
        out = subprocess.run(
            ["git", "-C", REPO_ROOT, "rev-parse", "HEAD"],
            capture_output=True, text=True, check=True, timeout=10,
        )
    except (OSError, subprocess.SubprocessError) as e:
        raise RuntimeError(f"git rev-parse HEAD failed: {e}") from e
    sha = out.stdout.strip()
    if not SHA_RE.match(sha):
        raise RuntimeError(f"git output not a 40-hex sha: {sha!r}")
    return sha


def load_avg() -> tuple[str, str, str]:
    """sysctl -n vm.loadavg (macOS) then /proc/loadavg (Linux); UNKNOWN axis."""
    def parse3(text: str):
        nums = [t for t in text.split() if re.fullmatch(r"[0-9]+\.[0-9]+|[0-9]+", t)]
        if len(nums) >= 3:
            return nums[0], nums[1], nums[2]
        return None

    try:
        out = subprocess.run(
            ["sysctl", "-n", "vm.loadavg"],
            capture_output=True, text=True, check=True, timeout=5,
        )
        got = parse3(out.stdout)
        if got:
            return got
    except (OSError, subprocess.SubprocessError):
        pass
    try:
        with open("/proc/loadavg", "r", encoding="utf-8") as fh:
            got = parse3(fh.read())
        if got:
            return got
    except OSError:
        pass
    return ("UNKNOWN", "UNKNOWN", "UNKNOWN")


def cores() -> str:
    n = os.cpu_count()
    if not n or n < 1:
        raise RuntimeError("core count unavailable")
    return str(n)


def build_profile(require_release: bool) -> str:
    """Mirror of the Rust profile axes (OPT_LEVEL / debug-assertions)."""
    opt = os.environ.get("OPT_LEVEL")
    debug_axis = os.environ.get("CGAP_DEBUG_ASSERTIONS") == "1"
    dev_opt = opt == "0"
    if debug_axis or dev_opt:
        reason = f"debug_assertions={str(debug_axis).lower()} opt-level={opt or 'unset'}"
        if require_release:
            print(
                f"LEDGER [{DISQUALIFIED_PROFILE}] non-release profile: {reason}",
                file=sys.stderr,
            )
            raise SystemExit(EXIT_GATE_FAIL)
        return f"{DISQUALIFIED_PROFILE} ({reason})"
    return "release"


def utc_now() -> str:
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def emit_header(
    plane: str,
    target_binary: str,
    alloc_signature: str | None = None,
    require_release: bool = False,
    sha: str | None = None,
) -> str:
    """Production emitter. `sha` override exists ONLY for tests/courts."""
    profile = build_profile(require_release)
    if sha is None:
        sha = git_sha()
    lines = [
        f"plane: {plane}",
        f"git-sha: {sha}",
        f"target-binary: {target_binary}",
        f"build-profile: {profile}",
        f"cores: {cores()}",
    ]
    if alloc_signature:
        lines.append(f"alloc-signature: {alloc_signature}")
    la = load_avg()
    lines.append(f"load-avg: {la[0]} {la[1]} {la[2]}")
    lines.append(f"utc: {utc_now()}")
    return "\n".join(lines) + "\n"


def check_contract(text: str) -> list[str]:
    """Every contract field present, in order, exactly once."""
    errors: list[str] = []
    seen: list[str] = []
    for line in text.splitlines():
        if ":" in line:
            seen.append(line.split(":", 1)[0])
    expected = [f for f in CONTRACT_FIELDS if f != "alloc-signature"]
    idx = 0
    for f in expected:
        while idx < len(seen) and seen[idx] != f:
            idx += 1
        if idx >= len(seen):
            errors.append(f"missing contract field: {f}")
        idx += 1
    if len(seen) != len(set(seen)):
        errors.append("duplicate header field")
    return errors


def self_check() -> int:
    """Poison/clean twins through the production functions."""
    ok = True
    # Clean leg: full contract surface with a synthetic (valid-shape) sha.
    clean = emit_header("selfcheck-plane", "selfcheck-bin", "allocs=2/1k",
                        sha="a" * 40)
    errs = check_contract(clean)
    if errs:
        print(f"VACUOUS GATE: clean fixture failed contract: {errs}", file=sys.stderr)
        ok = False
    if "DISQUALIFIED_PROFILE" in clean:
        print("VACUOUS GATE: clean fixture read as disqualified", file=sys.stderr)
        ok = False

    # Poison leg 1: profile-disqualified fixture must carry the ledger token.
    os.environ["CGAP_DEBUG_ASSERTIONS"] = "1"
    try:
        poison = emit_header("selfcheck-plane", "selfcheck-bin", sha="a" * 40)
    finally:
        os.environ.pop("CGAP_DEBUG_ASSERTIONS", None)
    if DISQUALIFIED_PROFILE not in poison:
        print("VACUOUS GATE: disqualified fixture lacked ledger token", file=sys.stderr)
        ok = False
    # require_release arm must actually refuse (exit 1).
    os.environ["CGAP_DEBUG_ASSERTIONS"] = "1"
    try:
        try:
            emit_header("selfcheck-plane", "selfcheck-bin", sha="a" * 40,
                        require_release=True)
            print("VACUOUS GATE: require_release did not refuse", file=sys.stderr)
            ok = False
        except SystemExit as e:
            if e.code != EXIT_GATE_FAIL:
                print(f"VACUOUS GATE: refusal exit {e.code} != {EXIT_GATE_FAIL}",
                      file=sys.stderr)
                ok = False
    finally:
        os.environ.pop("CGAP_DEBUG_ASSERTIONS", None)

    # Poison leg 2: missing git identity must raise (fail-closed), not emit.
    # Force the failure lawfully: point GIT_DIR at a nonexistent repo.
    os.environ["GIT_DIR"] = "/rec-court-selfcheck-nonexistent.git"
    try:
        emit_header("selfcheck-plane", "selfcheck-bin", sha=None)
        print("VACUOUS GATE: missing git identity did not refuse", file=sys.stderr)
        ok = False
    except RuntimeError:
        pass
    finally:
        os.environ.pop("GIT_DIR", None)

    if not ok:
        print("VACUOUS GATE", file=sys.stderr)
        return EXIT_GATE_FAIL
    print("SELF-CHECK OK: poison refused (profile+identity), clean emitted full contract")
    return EXIT_OK


def main() -> int:
    ap = argparse.ArgumentParser(description="forensic receipt header emitter (LANE-1)")
    ap.add_argument("--plane", default=None, help="caller-supplied plane const")
    ap.add_argument("--target", default=None, help="target binary/test name")
    ap.add_argument("--alloc-signature", default=None)
    ap.add_argument("--require-release", action="store_true",
                    help="non-release profile -> ledger line on stderr + exit 1")
    ap.add_argument("--self-check", action="store_true")
    args = ap.parse_args()

    if args.self_check:
        return self_check()

    if not args.plane or not args.target:
        ap.error("--plane and --target are required (unless --self-check)")

    try:
        text = emit_header(args.plane, args.target, args.alloc_signature,
                           require_release=args.require_release)
    except SystemExit as e:
        return int(e.code or 0)
    except RuntimeError as e:
        print(f"RECEIPT-IDENTITY-MISSING: {e}", file=sys.stderr)
        return EXIT_GATE_FAIL
    sys.stdout.write(text)
    return EXIT_OK


if __name__ == "__main__":
    sys.exit(main())
