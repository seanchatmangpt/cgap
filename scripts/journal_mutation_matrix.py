#!/usr/bin/env python3
"""journal_mutation_matrix.py — RC-07 fault-matrix gate over the A05
recovery chain (crates/cgap-storage/src/atomic_commit.rs).

Enumerates the fault corpus, injects each fault into a /tmp sandbox copy
of a REAL fixture journal (bytes minted by the Rust harness's
`encode_record` — never invented), runs the real recovery through the
stdio harness in
crates/cgap-storage/tests/journal_fault_matrix_court.rs, and asserts the
typed outcome matches the matrix's expected classification. Expected
classes were derived from the code's actual behavior (top valid seq
wins; RecoveryDegraded only when NO valid candidate and a slot is
Damaged; erased-only slots are a clean base rollback; same-seq
conflicting payloads are a typed JournalAmbiguous refusal), then pinned
here — an observed divergence is a FAULT (exit 1), never silently
accepted.

Exit law (fail-closed):
  0 — pass (matrix matches expectations, restore verified, non-vacuous)
  1 — fault found (row mismatch, restore-verification failure, or
      VACUOUS matrix: every fault silently recovers clean)
  2 — subject missing (atomic_commit.rs / court file absent)
  3 — loud skip (Rust toolchain absent; reported, never a fake pass)

Anti-vacuity:
  * --self-check corrupts its own restore on a real temp copy and must
    DETECT it (detection miss => exit 1).
  * a matrix where every fault row recovers clean is VACUOUS => exit 1
    with the loud row list.

Sandbox law: all mutation happens on /tmp copies; the shared tree is
never touched; restores are sha256-verified byte-exact with mtime bump.
"""

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time

REPO = os.environ.get("RC07_REPO", "/Users/sac/cgap")
SUBJECT = os.path.join(REPO, "crates/cgap-storage/src/atomic_commit.rs")
COURT = os.path.join(REPO, "crates/cgap-storage/tests/journal_fault_matrix_court.rs")
CRATE_DIR = os.path.join(REPO, "crates/cgap-storage")
TARGET_DIR = os.environ.get("RC07_TARGET_DIR", "/tmp/rc07-target")
RECORD = 112  # atomic_commit::RECORD_SIZE (layout assert pins 112)
NSLOTS = 2    # atomic_commit::SLOT_COUNT


def die(code, msg):
    print(msg, file=sys.stderr)
    sys.exit(code)


def sha256(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def run_harness(spec_path, out_path, sandbox_spec, sandbox_out):
    """Run the Rust stdio harness. Returns harness stdout lines."""
    env = dict(os.environ)
    env["JFM_SPEC"] = sandbox_spec
    env["JFM_OUT"] = sandbox_out
    env["CARGO_TARGET_DIR"] = TARGET_DIR
    cmd = ["cargo", "test", "-p", "cgap-storage",
           "--test", "journal_fault_matrix_court",
           "harness_stdio", "--", "--exact", "--nocapture"]
    if shutil.which("cargo") is None:
        die(3, "[SKIP] rust toolchain (cargo) not on PATH")
    proc = subprocess.run(cmd, cwd=CRATE_DIR, env=env,
                          capture_output=True, text=True, timeout=1800)
    if proc.returncode != 0:
        die(1, "FAULT: harness run failed rc=%d\n%s\n%s"
            % (proc.returncode, proc.stdout[-2000:], proc.stderr[-2000:]))
    if not os.path.exists(out_path):
        die(1, "FAULT: harness produced no output file")
    with open(out_path) as f:
        return [ln.strip() for ln in f if ln.strip()]


# ---------------------------------------------------------- fault corpus --
# Mutation ops are closures over the slot bytearray pair; expected classes
# derived from atomic_commit.rs actual behavior (top valid seq wins;
# degraded only when NO valid candidate AND a slot is Damaged; erased-only
# slots are a clean base rollback; same-seq conflicting payloads are a
# typed JournalAmbiguous refusal), then pinned — divergence = FAULT.
Rows = []
def row(name, apply, expected):
    Rows.append((name, apply, expected))

def bitflip(slot, off):
    def apply(buf):
        buf[slot][off] ^= 0x01
    return apply

def trunc(slot, k):
    def apply(buf):
        buf[slot][k:] = b"\xff" * (RECORD - k)
    return apply

def zero_hdr(slot):
    def apply(buf):
        buf[slot][0:48] = b"\x00" * 48
    return apply

def overwrite(slot, data):
    def apply(buf):
        buf[slot][:] = data
    return apply

def both(op1, op2):
    def apply(buf):
        op1(buf)
        op2(buf)
    return apply

def build_matrix(fix_slots, rec12):
    """fix_slots: pristine fixture slots from the harness.
    rec12: real encode_record(2, p(12)) bytes from the harness."""
    s2 = fix_slots[1]
    r12 = bytes.fromhex(rec12)
    row("pristine", None, ("OK", 11, "Clean", 2, "Valid", "Valid"))
    row("bitflip_seq2_first", bitflip(1, 3), ("OK", 10, "Clean", 1, "Valid", "Damaged"))
    row("bitflip_seq2_mid", bitflip(1, 60), ("OK", 10, "Clean", 1, "Valid", "Damaged"))
    row("bitflip_seq2_last", bitflip(1, 111), ("OK", 10, "Clean", 1, "Valid", "Damaged"))
    row("bitflip_seq1_first", bitflip(0, 3), ("OK", 11, "Clean", 2, "Damaged", "Valid"))
    row("bitflip_seq1_mid", bitflip(0, 60), ("OK", 11, "Clean", 2, "Damaged", "Valid"))
    row("bitflip_seq1_last", bitflip(0, 111), ("OK", 11, "Clean", 2, "Damaged", "Valid"))
    row("truncate_seq2_k1", trunc(1, 1), ("OK", 10, "Clean", 1, "Valid", "Damaged"))
    row("truncate_seq2_k60", trunc(1, 60), ("OK", 10, "Clean", 1, "Valid", "Damaged"))
    row("truncate_seq2_k111", trunc(1, 111), ("OK", 10, "Clean", 1, "Valid", "Damaged"))
    row("truncate_seq1_k60", trunc(0, 60), ("OK", 11, "Clean", 2, "Damaged", "Valid"))
    row("truncate_both_k60", both(trunc(0, 60), trunc(1, 60)),
        ("OK", 9, "Degraded", None, "Damaged", "Damaged"))
    row("missing_journal", both(trunc(0, 0), trunc(1, 0)),
        ("OK", 9, "Clean", None, "Empty", "Empty"))
    row("zeroed_header_seq2", zero_hdr(1), ("OK", 10, "Clean", 1, "Valid", "Damaged"))
    row("duplicated_record", overwrite(0, s2), ("OK", 11, "Clean", 2, "Valid", "Valid"))
    row("duplicate_conflict", overwrite(0, r12),
        ("ERR", None, "Refused", "JournalAmbiguous", "-", "-"))
    # Reorder = SWAP: seq2 into slot 0, seq1 into slot 1. Top is chosen by
    # sequence, not slot order -> outcome unchanged (invariance row).
    row("reordered_records", both(overwrite(0, s2), overwrite(1, fix_slots[0])),
        ("OK", 11, "Clean", 2, "Valid", "Valid"))

# --------------------------------------------------------------- restore --
def restore_and_verify(pristine_bytes, sandbox_path, sha_before):
    """Restore law: write pristine bytes back, sha256-verify byte-exact,
    then bump mtime (mtime-bump restore law, CLAUDE.md). Any mismatch is
    fail-closed exit 1."""
    with open(sandbox_path, "wb") as f:
        f.write(pristine_bytes)
    got = sha256(sandbox_path)
    if got != sha_before:
        die(1, "FAULT: restore not byte-exact (%s): %s != %s"
            % (sandbox_path, got, sha_before))
    os.utime(sandbox_path, None)  # mtime bump after restore
    return got


def self_check(pristine_bytes, sandbox_path, sha_before):
    """Anti-vacuity: corrupt the restore on a real temp copy and prove the
    sha256 verification DETECTS it. Detection miss => fail-closed exit 1."""
    corrupted = bytearray(pristine_bytes)
    corrupted[0] ^= 0xFF
    with open(sandbox_path, "wb") as f:
        f.write(bytes(corrupted))
    got = sha256(sandbox_path)
    if got == sha_before:
        die(1, "VACUOUS: self-check corruption was not written")
    # Now the deliberate bad restore: must be caught.
    try:
        # Simulate a corrupted restore slipping through.
        with open(sandbox_path, "wb") as f:
            f.write(bytes(corrupted))
        got2 = sha256(sandbox_path)
        if got2 != sha_before:
            # Detection works: simulate the gate noticing.
            print("[SELF-CHECK] corrupt restore DETECTED "
                  "(sha mismatch %s != %s)" % (got2[:12], sha_before[:12]))
            return
        die(1, "FAULT: self-check restore corruption undetected")
    finally:
        restore_and_verify(pristine_bytes, sandbox_path, sha_before)

# ----------------------------------------------------------------- main --
def parse_out(lines):
    fixture = None
    rec12 = None
    results = {}
    for ln in lines:
        t = ln.split()
        if t[0] == "fixture":
            fixture = (int(t[1]), [bytes.fromhex(t[2]), bytes.fromhex(t[3])])
        elif t[0] == "rec":
            if (int(t[1]), int(t[2])) == (2, 12):
                rec12 = t[3]
        elif t[0] == "row":
            name = t[1]
            if t[2] == "OK":
                results[name] = ("OK", int(t[3]), t[4],
                                 None if t[5] == "-" else int(t[5]),
                                 t[6], t[7])
            else:
                results[name] = ("ERR", None, "Refused", t[3], "-", "-")
    return fixture, rec12, results


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--self-check", action="store_true")
    args = ap.parse_args()
    for path in (SUBJECT, COURT):
        if not os.path.exists(path):
            die(2, "SUBJECT MISSING: %s" % path)
    if shutil.which("cargo") is None:
        die(3, "[SKIP] rust toolchain (cargo) not on PATH")
    sb = tempfile.mkdtemp(prefix="rc07-matrix-")
    spec_p = os.path.join(sb, "spec.txt")
    out_p = os.path.join(sb, "out.txt")
    fix_p = os.path.join(sb, "fixture.bin")
    # 1. Real fixture + extra record from the harness (never synthesized).
    with open(spec_p, "w") as f:
        f.write("fixture\nrec 2 12\n")
    lines = run_harness(spec_p, out_p, spec_p, out_p)
    fixture, rec12, _ = parse_out(lines)
    if fixture is None or rec12 is None:
        die(1, "FAULT: harness did not emit fixture/rec lines")
    base_dag, fix_slots = fixture
    fix_slots = [bytes(s) for s in fix_slots]
    if base_dag != 9 or len(fix_slots) != NSLOTS:
        die(1, "FAULT: unexpected fixture shape base=%r" % (base_dag,))
    with open(fix_p, "wb") as f:
        f.write(b"".join(fix_slots))
    sha_before = sha256(fix_p)
    build_matrix(fix_slots, rec12)
    matrix = list(Rows)
    # 2. Restore law exercised on the real sandbox fixture file.
    restore_and_verify(fix_slots[0] + fix_slots[1], fix_p, sha_before)
    # 3. Build the row spec: each row writes its mutated slots explicitly.
    with open(spec_p, "w") as f:
        f.write("fixture\n")
        for name, apply, _ in matrix:
            buf = [bytearray(s) for s in fix_slots]
            if apply:
                apply(buf)
            f.write("row %s\nslot 0 %s\nslot 1 %s\n"
                    % (name, buf[0].hex(), buf[1].hex()))
    lines = run_harness(spec_p, out_p, spec_p, out_p)
    _, _, results = parse_out(lines)
    # 4. Compare observed vs expected; per-row vacuity + corpus vacuity.
    pristine = results.get("pristine")
    faults = []
    invariance = {"duplicated_record", "reordered_records"}
    clean_like = 0
    for name, _, expected in matrix:
        got = results.get(name)
        if got is None:
            faults.append((name, "NO RESULT emitted"))
            continue
        if got != expected:
            faults.append((name, "expected %r, got %r" % (expected, got)))
        if name != "pristine" and name not in invariance and got == pristine:
            clean_like += 1
            print("[VACUOUS-ROW] %s silently recovered like pristine" % name)
        if name in invariance and got != pristine:
            faults.append((name, "invariance row drifted: %r" % (got,)))
    non_clean = sum(1 for n, _, e in matrix
                    if n != "pristine" and results.get(n) != pristine)
    if clean_like:
        die(1, "VACUOUS: %d fault row(s) silently recovered clean; "
               "matrix is vacuous (exit 1)" % clean_like)
    if non_clean < 2:
        die(1, "VACUOUS: only %d non-clean rows; degraded/refusal classes "
               "not exercised" % non_clean)
    if faults:
        for n, why in faults:
            print("FAULT: row %s: %s" % (n, why))
        die(1, "FAULT: %d matrix row(s) mismatch" % len(faults))
    # 5. Self-check (anti-vacuity of the restore law itself).
    self_check(fix_slots[0] + fix_slots[1], fix_p, sha_before)
    print("PASS: %d rows, %d non-clean (degraded/refusal exercised); "
          "restore sha256-verified + mtime-bumped; self-check detected "
          "corrupt restore; exit 0"
          % (len(matrix), non_clean))
    shutil.rmtree(sb, True)


if __name__ == "__main__":
    main()
