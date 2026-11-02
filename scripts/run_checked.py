#!/usr/bin/env python3
"""run_checked.py -- pipe-safe command runner for lane receipts.

Bug class (witnessed 4+ times, 2026-11-02 session): ``$?`` after a shell
pipe is the LAST command's rc, so ``<cmd> | tail -1; echo $?`` reports
tail's 0 even when <cmd> failed. This tool runs the command directly via
subprocess (list form, shell=False), tees combined stdout+stderr to a log
file and this stdout, then prints a typed ``exit=<rc>`` line taken from
the command itself and exits with that same rc.

Usage:
  python3 scripts/run_checked.py --out /tmp/x.log --label NAME -- cmd args...
Options:
  --out PATH    log file (required-ish: defaults to /tmp/run_checked.log)
  --label NAME  header line in the log
  --timeout N   seconds (default 1800); on expiry: SIGKILL, typed
                TIMEOUT line, exit 124
"""
import argparse
import subprocess
import sys
import threading


def main() -> int:
    ap = argparse.ArgumentParser(add_help=True)
    ap.add_argument("--out", default="/tmp/run_checked.log")
    ap.add_argument("--label", default=None)
    ap.add_argument("--timeout", type=int, default=1800)
    ap.add_argument("cmd", nargs=argparse.REMAINDER)
    args = ap.parse_args()

    if args.cmd and args.cmd[0] == "--":
        args.cmd = args.cmd[1:]
    if not args.cmd:
        print("run_checked: no command given (use -- cmd args...)",
              file=sys.stderr)
        return 2

    header = "run_checked: label=%s timeout=%ds cmd=%r" % (
        args.label, args.timeout, args.cmd)
    lines = [header]

    try:
        proc = subprocess.Popen(
            args.cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            shell=False,
        )
    except OSError as exc:
        lines.append("run_checked: SPAWN-ERROR %s" % exc)
        lines.append("exit=127")
        _emit(args.out, lines)
        return 127

    def _pump() -> None:
        for raw in proc.stdout:
            sys.stdout.buffer.write(raw)
            sys.stdout.buffer.flush()
            lines.append(raw.decode("utf-8", "replace").rstrip("\n"))
        proc.stdout.close()

    t = threading.Thread(target=_pump)
    t.start()
    try:
        rc = proc.wait(timeout=args.timeout)
        timed_out = False
    except subprocess.TimeoutExpired:
        proc.kill()
        rc = proc.wait()
        timed_out = True
    t.join()

    if timed_out:
        # GNU timeout convention: 124 distinguishes expiry from the
        # child's own rc (a KILL would otherwise surface as -9/247).
        lines.append("run_checked: TIMEOUT after %ds" % args.timeout)
        rc = 124
    elif rc < 0:
        # Killed by signal N -> conventional 128+N so the shell sees a
        # stable positive rc instead of the two's-complement artifact.
        rc = 128 + (-rc)
    lines.append("exit=%d" % rc)
    _emit(args.out, lines)
    return rc


def _emit(out_path: str, lines: list) -> None:
    # Full transcript (header + captured output + typed tail) -> log file.
    blob = "\n".join(lines) + "\n"
    with open(out_path, "w", encoding="utf-8") as fh:
        fh.write(blob)
    # stdout already got the live stream; print only the typed lines here
    # so the rc-bearing tail is the last thing on stdout.
    typed = [ln for ln in lines if ln.startswith("run_checked:")
             or ln.startswith("exit=")]
    sys.stdout.write("\n".join(typed) + "\n")
    sys.stdout.flush()


if __name__ == "__main__":
    sys.exit(main())
