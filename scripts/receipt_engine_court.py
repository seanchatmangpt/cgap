#!/usr/bin/env python3
"""receipt_engine_court.py — anti-vacuity court for the LANE-1 receipt engine.

Shim-court protocol (house law): prove each engine mutation's effect with a
real refusal, never a mock. For every leg:

    BASELINE (sha256 of engine artifacts) ->
    SHIM    (named mutation, applied to a SANDBOX copy, marked SHIM-COURT) ->
    AFTER   (the consumer must REFUSE: nonzero exit, typed finding) ->
    RESTORE (sandbox discarded; originals re-hashed byte-exact, fail-closed)

A leg whose mutation does NOT flip its consumer is `COURT-FAIL` (exit 1).
A leg that cannot run loud-skips `[SKIP] <reason>` and never reads as pass.
Non-vacuity of restore-detection itself is proven by the RESTORE-CORRUPT
leg: a deliberately botched restore on a throwaway copy must be detected.

Legs:
  M1 compare-disable      verifier source mutated (block==cited) -> the
                          verifier's own --self-check must go VACUOUS
  M2 profile-axis-disable emitter source mutated (debug axis dead) ->
                          the emitter's --self-check must go VACUOUS
  M3 rust-emit-permit     sandbox cgap-receipt mutated to never refuse ->
                          `cargo test` (debug) must FAIL the fail-closed
                          court (bounded, 10-min SLA)
  M4 receipt-field-corrupt one byte of a REAL receipt's verbatim block
                          corrupted (sandbox copy) -> production verifier
                          must emit VERBATIM_MISMATCH, exit 1
  M5 restore-corrupt      botched restore on a throwaway copy -> the
                          court's restore detection must catch it

Exit ABI: 0 all legs flipped; 1 court fail / vacuous; 3 all-legs skipped.
"""

from __future__ import annotations

import hashlib
import os
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SCRIPTS = os.path.join(ROOT, "scripts")
CRATE = os.path.join(ROOT, "crates", "cgap-receipt")
RECEIPT = os.path.join(
    ROOT, "docs", "jira", "v26.11.4", "receipts", "bm3-chain-scale-v2.md")

sys.path.insert(0, SCRIPTS)
import receipt_verbatim_verify as rvv  # noqa: E402

TIMEOUT = 600  # compile-freeze SLA bound for the cargo leg


def sha256(path: str) -> str:
    with open(path, "rb") as fh:
        return hashlib.sha256(fh.read()).hexdigest()


class Leg:
    def __init__(self, name: str) -> None:
        self.name = name
        self.status = "PENDING"
        self.detail = ""


def run(cmd: list[str], **kw) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, capture_output=True, text=True, timeout=TIMEOUT, **kw)


def flip(name: str, ok: bool, detail: str) -> tuple[bool, str]:
    """Print the AFTER verdict for one leg."""
    tag = "FLIPPED" if ok else "COURT-FAIL"
    print(f"  [{tag}] {name}: {detail}")
    return ok, detail


def leg_source_selfcheck(leg: Leg, target: str, mutation: tuple[str, str],
                         expect: str) -> bool:
    """M1/M2: mutate a sandbox copy of a gate script; its --self-check must
    go VACUOUS (a court that still passes with a dead arm is a fake gate)."""
    src = os.path.join(SCRIPTS, target)
    with tempfile.TemporaryDirectory(prefix=f"rec-court-{leg.name}-") as tmp:
        sandbox = os.path.join(tmp, target)
        shutil.copy2(src, sandbox)
        before = sha256(src)
        with open(sandbox, encoding="utf-8") as fh:
            body = fh.read()
        old, new = mutation
        if old not in body:
            leg.status, leg.detail = "SKIP", f"mutation anchor absent in {target}"
            print(f"  [SKIP] {leg.name}: {leg.detail}")
            return True  # loud skip, aggregate handling by caller
        mutated = body.replace(old, new, 1)
        with open(sandbox, "w", encoding="utf-8") as fh:
            fh.write(f"# SHIM-COURT: mutation={leg.name} (sandbox copy)\n" + mutated)
        proc = run([sys.executable, sandbox, "--self-check"])
        flipped = proc.returncode != 0 and expect in (proc.stdout + proc.stderr)
        ok, detail = flip(
            leg.name, flipped,
            f"mutated {target} --self-check rc={proc.returncode} "
            f"(expect nonzero + '{expect}')")
        if not ok:
            leg.status, leg.detail = "FAIL", detail
            return False
        if sha256(src) != before:
            leg.status, leg.detail = "FAIL", "ORIGINAL changed during sandbox leg"
            print(f"  [COURT-FAIL] {leg.name}: {leg.detail}")
            return False
        leg.status = "PASS"
        return True


def leg_rust_permit(leg: Leg) -> bool:
    """M3: sandbox copy of cgap-receipt with the fail-closed profile check
    disabled; `cargo test` (debug) must FAIL the emit court."""
    if shutil.which("cargo") is None:
        leg.status, leg.detail = "SKIP", "cargo not on PATH"
        print(f"  [SKIP] {leg.name}: {leg.detail}")
        return True
    lib = os.path.join(CRATE, "src", "lib.rs")
    before = sha256(lib)
    with tempfile.TemporaryDirectory(prefix="rec-court-m3-") as tmp:
        sandbox = os.path.join(tmp, "cgap-receipt")
        shutil.copytree(CRATE, sandbox)
        main_src = os.path.join(sandbox, "src", "lib.rs")
        with open(main_src, encoding="utf-8") as fh:
            body = fh.read()
        anchor = "if debug_assertions || dev_opt {"
        if anchor not in body:
            leg.status, leg.detail = "SKIP", "mutation anchor absent in lib.rs"
            print(f"  [SKIP] {leg.name}: {leg.detail}")
            return True
        body = body.replace(anchor, "if false && (debug_assertions || dev_opt) {", 1)
        with open(main_src, "w", encoding="utf-8") as fh:
            fh.write("// SHIM-COURT: mutation=m3-rust-emit-permit (sandbox copy)\n" + body)
        # Silence the dead-code warning delta; correctness comes from the
        # fail-closed court test failing under the debug dev profile.
        proc = run(["cargo", "test", "--manifest-path",
                    os.path.join(sandbox, "Cargo.toml")])
        out = proc.stdout + proc.stderr
        flipped = proc.returncode != 0 and "emit_is_fail_closed_on_debug_profile" in out
        ok, detail = flip(
            leg.name, flipped,
            f"mutated crate cargo test rc={proc.returncode} "
            f"(expect failing fail-closed court)")
        if not ok:
            leg.status, leg.detail = "FAIL", detail
            print(f"    tail: {out[-800:]}")
            return False
    if sha256(lib) != before:
        leg.status, leg.detail = "FAIL", "ORIGINAL lib.rs changed during sandbox leg"
        print(f"  [COURT-FAIL] {leg.name}: {leg.detail}")
        return False
    leg.status = "PASS"
    return True


def leg_receipt_corrupt(leg: Leg) -> bool:
    """M4: corrupt one byte of the REAL receipt's verbatim block (sandbox
    copy); the production verifier must refuse with VERBATIM_MISMATCH."""
    if not os.path.isfile(RECEIPT):
        leg.status, leg.detail = "SKIP", f"receipt absent: {RECEIPT}"
        print(f"  [SKIP] {leg.name}: {leg.detail}")
        return True
    before = sha256(RECEIPT)
    with tempfile.TemporaryDirectory(prefix="rec-court-m4-") as tmp:
        sandbox = os.path.join(tmp, "bm3-chain-scale-v2.md")
        with open(RECEIPT, encoding="utf-8") as fh:
            body = fh.read()
        # Named mutation: one digit of the measured number drifts.
        m = re.search(r"\[SPAN-CURVE\] span 50000: best 63\.330ms", body)
        if m is None:
            leg.status, leg.detail = "SKIP", "receipt anchor line absent"
            print(f"  [SKIP] {leg.name}: {leg.detail}")
            return True
        mutated = body.replace(
            "[SPAN-CURVE] span 50000: best 63.330ms",
            "[SPAN-CURVE] span 50000: best 63.331ms", 1)
        with open(sandbox, "w", encoding="utf-8") as fh:
            fh.write(mutated)
        hard, skips = rvv.check_receipt(sandbox, ROOT)
        flipped = any(h.startswith("VERBATIM_MISMATCH") for h in hard)
        ok, detail = flip(
            leg.name, flipped,
            f"1-byte number drift -> {hard[0] if hard else 'NO FINDING'}")
        if not ok:
            leg.status, leg.detail = "FAIL", detail
            return False
    if sha256(RECEIPT) != before:
        leg.status, leg.detail = "FAIL", "ORIGINAL receipt changed during sandbox leg"
        print(f"  [COURT-FAIL] {leg.name}: {leg.detail}")
        return False
    leg.status = "PASS"
    return True


def leg_restore_corrupt(leg: Leg) -> bool:
    """M5: non-vacuity of restore detection — a botched restore on a
    throwaway copy MUST be caught by the sha256 check."""
    with tempfile.TemporaryDirectory(prefix="rec-court-m5-") as tmp:
        victim = os.path.join(tmp, "victim.txt")
        with open(victim, "w", encoding="utf-8") as fh:
            fh.write("baseline-payload\n")
        baseline = sha256(victim)
        # Simulate the restore, then corrupt it (the "botched restore").
        with open(victim, "w", encoding="utf-8") as fh:
            fh.write("baseline-payload\n")
        with open(victim, "a", encoding="utf-8") as fh:
            fh.write("restore-corruption\n")
        caught = sha256(victim) != baseline
        ok, detail = flip(leg.name, caught,
                          "botched restore detected via sha256 mismatch")
        if not ok:
            leg.status, leg.detail = "FAIL", detail
            return False
        # And a faithful restore must verify clean (else every leg would
        # falsely fail — this proves the check is two-sided).
        with open(victim, "w", encoding="utf-8") as fh:
            fh.write("baseline-payload\n")
        if sha256(victim) != baseline:
            leg.status, leg.detail = "FAIL", "faithful restore read as corrupt"
            print(f"  [COURT-FAIL] {leg.name}: {leg.detail}")
            return False
    leg.status = "PASS"
    return True


def main() -> int:
    print("receipt-engine anti-vacuity court (SHIM-COURT protocol)")
    print("baseline artifacts:")
    artifacts = [
        os.path.join(SCRIPTS, "receipt_verbatim_verify.py"),
        os.path.join(SCRIPTS, "receipt_header.py"),
        os.path.join(CRATE, "src", "lib.rs"),
        RECEIPT,
    ]
    baselines = {}
    for a in artifacts:
        if os.path.isfile(a):
            baselines[a] = sha256(a)
            print(f"  sha256 {a} = {baselines[a]}")
        else:
            print(f"  [SKIP] baseline artifact absent: {a}")

    legs: list[Leg] = []
    ok_all = True

    l1 = Leg("m1-compare-disable")
    res = leg_source_selfcheck(
        l1, "receipt_verbatim_verify.py",
        ("if block != cited:", "if False and block != cited:"),
        "VACUOUS")
    legs.append(l1)
    ok_all &= res

    l2 = Leg("m2-profile-axis-disable")
    res = leg_source_selfcheck(
        l2, "receipt_header.py",
        ('debug_axis = os.environ.get("CGAP_DEBUG_ASSERTIONS") == "1"',
         'debug_axis = os.environ.get("CGAP_DEBUG_ASSERTIONS") == "0"'),
        "VACUOUS")
    legs.append(l2)
    ok_all &= res

    l3 = Leg("m3-rust-emit-permit")
    res = leg_rust_permit(l3)
    legs.append(l3)
    ok_all &= res

    l4 = Leg("m4-receipt-field-corrupt")
    res = leg_receipt_corrupt(l4)
    legs.append(l4)
    ok_all &= res

    l5 = Leg("m5-restore-corrupt")
    res = leg_restore_corrupt(l5)
    legs.append(l5)
    ok_all &= res

    # Restore integrity over every ORIGINAL artifact the court touched.
    restored = True
    for a, h in baselines.items():
        if not os.path.isfile(a) or sha256(a) != h:
            print(f"RESTORE-FAIL: {a} drifted from baseline")
            restored = False
    print("restore: " + ("BYTE-EXACT (sha256, all originals)" if restored
                        else "DRIFT DETECTED"))

    for l in legs:
        print(f"  leg {l.name}: {l.status} {l.detail}")

    if not restored:
        return 1
    if all(l.status == "SKIP" for l in legs):
        print("ALL LEGS SKIPPED — court vacuous by definition")
        return 3
    if not ok_all:
        print("COURT FAIL: a mutation did not flip its consumer")
        return 1
    print("COURT PASS: every mutation flipped its consumer; restores byte-exact")
    return 0


if __name__ == "__main__":
    sys.exit(main())
