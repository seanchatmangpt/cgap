#!/usr/bin/env python3
"""Lane XC-02 — fail-closed rollup battery over the four house gates.

Invokes every gate as a subprocess (arg-list, no shell), captures output,
and rolls up fail-closed: battery exits 0 ONLY if every invoked gate
exits 0. Any gate exit 1/2/3 -> battery exit 1 with the failing rows.
A MISSING gate script -> battery exit 2 with the named row (loud).

Exit ABI:
  0 = every invoked gate passed          ("GATE BATTERY: n/n PASS")
  1 = at least one gate failed           (failing rows listed)
  2 = a gate script is missing           (named row)
  3 = loud SKIP (only inside --with-self-check when a gate exposes no
      self-check mode — recorded as a SKIP row, never a silent pass)

Modes:
  (default)         run each gate's normal gate pass
  --with-self-check ALSO run each gate's self-check / mutate-court mode,
                    as discovered from each script's argparse (read, not
                    guessed). A gate with no self-check mode gets a loud
                    SKIP row (exit 3 contribution is a battery FAIL per
                    the fail-closed law).
  --prove-refusal   anti-vacuity for the battery itself: inject one real
                    violation against a /tmp sandbox copy of a gate
                    subject and assert the battery observes exit 1
                    (fails). A battery that cannot flip is VACUOUS ->
                    exit 1.

Python 3 stdlib only. No Process.sleep; subprocess with timeouts.
"""

import argparse
import hashlib
import os
import re
import subprocess
import sys
import tempfile

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SCRIPTS = os.path.join(REPO, "scripts")

GATE_TIMEOUT_S = 1800  # per-gate subprocess deadline (stated, bounded)

# --- gate registry --------------------------------------------------------------
# name -> (script path, gate-pass argv suffix, self-check argv suffix).
# boundary_gate.py requires an explicit mode (bare = EXIT_REFUSE exit 1);
# canonical gate pass is --deps (witnessed lane-g16 / red-02 receipts).
# Self-check modes were read from each script's argparse on 2026-11-02:
#   boundary_gate.py  : --self-check-deps (§6.4 dep-gate self-check)
#   zero_string_gate.py: --self-check (byte-exactness fixture suite)
#   verify_cscb_sram.py: --mutate bn3-pin (sandboxed pin-drift court)
#   ffi_surface_gate.py: --self-check
GATES = {
    "boundary": (
        os.path.join(SCRIPTS, "boundary_gate.py"),
        ["--deps"],
        ["--self-check-deps"],
    ),
    "zero-string": (
        os.path.join(SCRIPTS, "zero_string_gate.py"),
        [],
        ["--self-check"],
    ),
    "cscb-sram": (
        os.path.join(SCRIPTS, "verify_cscb_sram.py"),
        [],
        ["--mutate", "bn3-pin"],
    ),
    "ffi-surface": (
        os.path.join(SCRIPTS, "ffi_surface_gate.py"),
        [],
        ["--self-check"],
    ),
}


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 16), b""):
            h.update(chunk)
    return h.hexdigest()


def run_gate(name, argv, timeout=GATE_TIMEOUT_S):
    """Run one gate via arg-list subprocess (no shell). Returns a row dict
    (name, argv, exit, ok, tail). Never raises on gate failure."""
    row = {"name": name, "argv": argv, "exit": None, "ok": False,
           "tail": "(no output captured)", "skipped": False}
    try:
        proc = subprocess.run(
            [sys.executable] + argv,
            capture_output=True, timeout=timeout)
    except subprocess.TimeoutExpired as exc:
        row["exit"] = 124  # timeout, house run_checked convention
        row["tail"] = "TIMEOUT after %ds" % timeout
        return row
    except OSError as exc:
        row["exit"] = 127  # spawn failure, house run_checked convention
        row["tail"] = "SPAWN FAIL: %s" % exc
        return row
    raw = (proc.stdout or b"") + (proc.stderr or b"")
    # ADV-05-A fix: invalid-UTF-8 child output must not crash the battery.
    # Decode leniently; a strict-decode failure marks the row as binary.
    try:
        out = raw.decode("utf-8")
        binary_note = ""
    except UnicodeDecodeError:
        out = raw.decode("utf-8", errors="replace")
        n_repl = out.count("�")
        binary_note = "(binary output, %d replacement chars) " % n_repl
    lines = [ln for ln in out.splitlines() if ln.strip()]
    row["exit"] = proc.returncode
    row["ok"] = (proc.returncode == 0)
    if lines:
        row["tail"] = binary_note + lines[-1][:300]
    elif binary_note:
        row["tail"] = binary_note.strip()
    return row


def format_row(row, mode_label=""):
    tag = "[SKIP]" if row.get("skipped") else ("PASS" if row["ok"] else "FAIL")
    return "%-14s %-6s exit=%-4s %s %s" % (
        row["name"], tag, row["exit"], mode_label, row["tail"])


def battery(rows):
    """Rollup. Returns battery exit code (0/1/2). Fail-closed."""
    missing = [r for r in rows if r["exit"] == 2]
    failed = [r for r in rows if not r["ok"]]
    if missing:
        print("GATE BATTERY: MISSING GATE SCRIPT(S) — fail-closed exit 2")
        for r in missing:
            print("  " + format_row(r))
        return 2
    if failed:
        print("GATE BATTERY: %d/%d PASS — FAILING GATES:" % (
            len(rows) - len(failed), len(rows)))
        for r in failed:
            print("  " + format_row(r))
        return 1
    print("GATE BATTERY: %d/%d PASS" % (len(rows), len(rows)))
    return 0


def run_battery(gates, with_self_check):
    """Run the full gate set; returns rows. A gate missing on disk is a
    MISSING row (exit 2), loud, never skipped silently."""
    rows = []
    for name in gates:
        path, pass_argv, selfcheck_argv = GATES[name]
        if not os.path.isfile(path):
            rows.append({"name": name, "argv": [path], "exit": 2,
                         "ok": False, "skipped": False,
                         "tail": "MISSING gate script: %s" % path})
            continue
        rows.append(run_gate(name, [path] + pass_argv))
        if with_self_check:
            if selfcheck_argv is None:
                rows.append({"name": name, "argv": [path] + (selfcheck_argv or []),
                             "exit": 3, "ok": False, "skipped": True,
                             "tail": "[SKIP] no self-check mode in argparse"})
            else:
                sc = run_gate(name + ":self-check", [path] + selfcheck_argv)
                sc["name"] = name + ":self-check"
                rows.append(sc)
    return rows


# --- --prove-refusal (battery anti-vacuity) --------------------------------------
# Inject one real violation against a /tmp sandbox copy of the cgap-bn3
# noalloc source (never the shared tree), feed it to verify_cscb_sram.py
# --subject bn3 --bn3-source <sandbox>, and require the battery to see a
# gate exit 1 (battery FAIL path flips). Restore law: the shared tree is
# never touched — sha256 witnessed identical before/after; the sandbox
# copy lives in tempfile.mkdtemp and is removed. mtime of the shared
# source is preserved automatically (no write occurs).
NA_MATCHES_RE = re.compile(r"pub const NA_MAX_MATCHES: usize = 2048;")
BN3_SOURCE = os.path.join(REPO, "crates", "cgap-bn3", "src", "noalloc.rs")


def prove_refusal():
    sram = GATES["cscb-sram"][0]
    if not os.path.isfile(sram):
        print("PROVE-REFUSAL: gate script missing: %s" % sram)
        return 2
    if not os.path.isfile(BN3_SOURCE):
        print("PROVE-REFUSAL: subject missing: %s" % BN3_SOURCE)
        return 2
    before_sha = sha256_file(BN3_SOURCE)
    sandbox = tempfile.mkdtemp(prefix="gate_battery_prove.")
    mutated = os.path.join(sandbox, "noalloc.rs")
    src = open(BN3_SOURCE, encoding="utf-8").read()
    mutated_src, n = NA_MATCHES_RE.subn(
        "pub const NA_MAX_MATCHES: usize = 1024;", src)
    if n != 1:
        print("PROVE-REFUSAL: mutation injection failed (subn=%d) — "
              "VACUOUS COURT" % n)
        return 1
    with open(mutated, "w", encoding="utf-8") as f:
        f.write(mutated_src)
    print("PROVE-REFUSAL: injected NA_MAX_MATCHES 2048->1024 in %s" % mutated)
    row = run_gate("cscb-sram:mutated", [
        sram, "--subject", "bn3", "--bn3-source", mutated])
    print(format_row(row, "(mutated sandbox subject)"))
    after_sha = sha256_file(BN3_SOURCE)
    if after_sha != before_sha:
        print("PROVE-REFUSAL: RESTORE FAILURE — shared tree sha256 drifted "
              "(%s -> %s)" % (before_sha[:16], after_sha[:16]))
        return 1
    battery_rc = battery([row])  # witness the FAIL rollup row for the log
    if row["exit"] == 1 and battery_rc == 1:
        print("PROVE-REFUSAL COURT: HOLDS (exit 0) — battery failure path "
              "FLIPPED: mutated gate exit 1 observed and rolled up as FAIL; "
              "shared tree sha256-exact (%s), mtime untouched (no write "
              "occurred). A battery that cannot flip is VACUOUS."
              % before_sha[:16])
        return 0
    print("PROVE-REFUSAL: VACUOUS BATTERY (exit 1) — mutated gate exit was "
          "%s, battery rollup was %s; the battery cannot see a real "
          "violation" % (row["exit"], battery_rc))
    return 1


def main():
    ap = argparse.ArgumentParser(
        description="fail-closed rollup battery over the four house gates")
    ap.add_argument("--with-self-check", action="store_true",
                    help="also run each gate's self-check/mutate-court mode")
    ap.add_argument("--prove-refusal", action="store_true",
                    help="anti-vacuity: inject a sandboxed violation, "
                         "require battery FAIL (a pass is VACUOUS)")
    ap.add_argument("--only", action="append", default=[],
                    help="run only the named gate(s) (repeatable)")
    ap.add_argument("--timeout", type=int, default=GATE_TIMEOUT_S,
                    help="per-gate subprocess deadline in seconds "
                         "(default %d)" % GATE_TIMEOUT_S)
    args = ap.parse_args()
    gates = args.only or list(GATES)
    unknown = [g for g in gates if g not in GATES]
    if unknown:
        print("UNKNOWN gate name(s): %s (known: %s)"
              % (", ".join(unknown), ", ".join(sorted(GATES))))
        return 2
    if args.prove_refusal:
        return prove_refusal()
    print("GATE BATTERY: gates=%s self-check=%s timeout=%ds"
          % (",".join(gates), args.with_self_check, args.timeout))
    rows = run_battery(gates, args.with_self_check)
    for r in rows:
        print(format_row(r, "(self-check)" if r["name"].endswith(":self-check")
                         else ""))
    return battery(rows)


if __name__ == "__main__":
    sys.exit(main())
