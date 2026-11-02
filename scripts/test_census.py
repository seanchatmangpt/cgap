#!/usr/bin/env python3
"""DX-8 per-target test census (lane dx-8).

Aggregating `cargo test -p <crate>` conflates compile-aborts with real
failures. This tool enumerates test targets via `cargo metadata --no-deps`,
runs each target individually under a per-lane CARGO_TARGET_DIR, and
classifies each as COMPILE-ABORT (nonzero rc, no test executed — broken
file:line extracted from error output), PASS (n passed / i ignored),
FAIL (verbatim failure blocks captured) or TIMEOUT.

Exit codes: 1 only on real FAILs (TIMEOUT counts as not-passed);
compile-aborts print LOUD-PENDING lines and exit 0 unless --strict.
"""
import argparse
import json
import os
import re
import subprocess
import sys

RED = "\033[31m"
GRN = "\033[32m"
YLW = "\033[33m"
RST = "\033[0m"


def run(cmd, cwd, timeout, env):
    """Run a command, merged output, bounded deadline (no sleeps)."""
    try:
        p = subprocess.run(
            cmd, cwd=cwd, timeout=timeout, env=env,
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
        )
        return p.returncode, p.stdout
    except subprocess.TimeoutExpired as e:
        out = e.stdout if isinstance(e.stdout, str) else (e.stdout or b"").decode("utf-8", "replace")
        return 124, (out or "") + "\n[TIMEOUT after %ss]" % timeout


def enumerate_targets(root, crate_filter):
    """Return [(crate, kind_flag, target_name)] for test-runnable targets."""
    meta = json.loads(subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=root, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
        text=True).stdout)
    wanted = set()
    if crate_filter:
        wanted = {c.strip() for c in crate_filter.split(",") if c.strip()}
    out = []
    for pkg in meta["packages"]:
        if wanted and pkg["name"] not in wanted:
            continue
        for t in pkg["targets"]:
            kinds = t["kind"]
            if "test" in kinds:
                out.append((pkg["name"], "--test", t["name"]))
            elif "lib" in kinds:
                out.append((pkg["name"], "--lib", t["name"]))
    return out


ERR_FILE_RE = re.compile(r"^\s*-->\s+(\S+):(\d+):\d+", re.M)
RESULT_RE = re.compile(
    r"test result: (\w+)\.\s+(\d+) passed; (\d+) failed; (\d+) ignored")


def classify(out, rc):
    """Classify one target run -> (verdict, detail)."""
    results = RESULT_RE.findall(out)
    if rc == 124:
        return "TIMEOUT", "killed at deadline"
    if results:
        passed = sum(int(r[1]) for r in results)
        failed = sum(int(r[2]) for r in results)
        ignored = sum(int(r[3]) for r in results)
        if failed > 0:
            blocks = failure_blocks(out)
            return "FAIL", "%d passed; %d failed; %d ignored%s" % (
                passed, failed, ignored, blocks)
        return "PASS", "%d passed; %d ignored" % (passed, ignored)
    if rc != 0:
        errs = sorted({">%s:%s" % (m.group(1), m.group(2))
                       for m in ERR_FILE_RE.finditer(out)})
        first_err = ""
        m = re.search(r"^error(\[[^\]]+\])?: (.*)$", out, re.M)
        if m:
            first_err = m.group(0).strip()
        loc = errs[0] if errs else "location unresolved"
        return "COMPILE-ABORT", "%s %s" % (loc, first_err)
    return "COMPILE-ABORT", "rc=0 but no test result line (suspicious)"


def failure_blocks(out, cap=4000):
    """Verbatim failure block capture, bounded."""
    idx = out.find("\nfailures:\n")
    if idx < 0:
        return ""
    block = out[idx:]
    if len(block) > cap:
        block = block[:cap] + "\n...[truncated %d chars]" % (len(block) - cap)
    return "\n" + block.rstrip()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--crate", default="",
                    help="comma-separated crate filter (default: all)")
    ap.add_argument("--timeout", type=int, default=900,
                    help="per-target deadline seconds (default 900)")
    ap.add_argument("--strict", action="store_true",
                    help="COMPILE-ABORT also exits 1")
    ap.add_argument("--output", default="",
                    help="write the markdown table to this file too")
    ap.add_argument("--root", default=os.path.dirname(os.path.dirname(
        os.path.abspath(__file__))),
        help="workspace root (default: repo root; courts point at /tmp)")
    args = ap.parse_args()

    targets = enumerate_targets(args.root, args.crate)
    if not targets:
        print("census: no test targets matched --crate=%r" % args.crate)
        return 2
    env = dict(os.environ)
    env.setdefault("CARGO_TARGET_DIR", "/tmp/dx8-target")
    env.setdefault("CARGO_NET_OFFLINE", "true")
    env.setdefault("RUST_BACKTRACE", "0")

    rows = []
    for crate, flag, name in targets:
        rc, out = run(["cargo", "test", "-q", "-p", crate, flag, name],
                      args.root, args.timeout, env)
        verdict, detail = classify(out, rc)
        rows.append((crate, name, verdict, detail, out))
        tag = {"PASS": GRN, "FAIL": RED}.get(verdict, YLW)
        label = "PASS" if verdict == "PASS" else (
            "FAIL" if verdict == "FAIL" else "LOUD-PENDING[%s]" % verdict)
        print("%-14s %-40s %s%s%s  %s" % (crate, name, tag, label, RST,
              detail.splitlines()[0] if detail else ""))

    n_pass = sum(1 for r in rows if r[2] == "PASS")
    n_fail = sum(1 for r in rows if r[2] == "FAIL")
    n_abort = sum(1 for r in rows if r[2] == "COMPILE-ABORT")
    n_time = sum(1 for r in rows if r[2] == "TIMEOUT")
    print("\nCENSUS: %d targets | PASS %d | FAIL %d | COMPILE-ABORT %d "
          "| TIMEOUT %d" % (len(rows), n_pass, n_fail, n_abort, n_time))
    for r in rows:
        if r[2] == "FAIL":
            print("-" * 60)
            print("FAIL BLOCK: %s :: %s" % (r[0], r[1]))
            print(r[4].rstrip())

    write_table(args.output, rows, (n_pass, n_fail, n_abort, n_time))
    if n_fail or n_time:
        return 1
    if args.strict and n_abort:
        print("strict: COMPILE-ABORT present -> exit 1")
        return 1
    return 0


def write_table(path, rows, totals):
    if not path:
        return
    lines = [
        "# test census (%s)" % __import__("datetime").datetime.now().isoformat(
            "T", "seconds"),
        "",
        "| crate | target | verdict | detail |",
        "|---|---|---|---|",
    ]
    for crate, name, verdict, detail, _ in rows:
        d = detail.splitlines()[0].replace("|", "\\|") if detail else ""
        lines.append("| %s | %s | %s | %s |" % (crate, name, verdict, d))
    lines += ["", "CENSUS totals: PASS %d FAIL %d COMPILE-ABORT %d TIMEOUT %d"
              % tuple(totals), ""]
    with open(path, "w") as f:
        f.write("\n".join(lines))
    print("table written: %s" % path)


if __name__ == "__main__":
    sys.exit(main())
