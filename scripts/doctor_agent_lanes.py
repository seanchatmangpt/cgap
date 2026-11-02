#!/usr/bin/env python3
"""DX-1: doctor checks answering per-lane recurring questions.

Exit classes (doctor.rs model): 0 all-GREEN, 1 any-WARN (informational),
2 any-RED (structural fault in the check itself).

Env overrides for courtability (default = real subjects):
  DX1_CGAP_ROOT, DX1_LSPMAX_ROOT, DX1_TMP_ROOT, DX1_CLAUDE_MD
"""
import glob
import json
import os
import re
import shutil
import subprocess
import sys
import time

CGAP_ROOT = os.environ.get("DX1_CGAP_ROOT", "/Users/sac/cgap")
LSPMAX_ROOT = os.environ.get("DX1_LSPMAX_ROOT", "/Users/sac/lsp-max")
TMP_ROOT = os.environ.get("DX1_TMP_ROOT", "/private/tmp")
CLAUDE_MD = os.environ.get("DX1_CLAUDE_MD", os.path.join(CGAP_ROOT, "CLAUDE.md"))
# TC-11 (tps-07 rank 2 / A2): dispatch-ledger silent-death watch roots.
LEDGER_PATH = os.environ.get(
    "DX1_LEDGER_PATH",
    os.path.join(CGAP_ROOT, "docs/jira/v26.11.2/lanes/dispatch-ledger.jsonl"))
LANES_DIR = os.environ.get(
    "DX1_LANES_DIR", os.path.join(CGAP_ROOT, "docs/jira/v26.11.2/lanes"))
LEDGER_STALE_MIN = float(os.environ.get("DX1_LEDGER_STALE_MIN", "45"))

RESULTS = []


def record(name, status, detail, remediation=None):
    RESULTS.append((name, status, detail, remediation))
    line = "{:22s} {:5s} {}".format(name, status, detail)
    if remediation:
        line += "\n  -> remediation: " + remediation
    print(line)
    return status


def newest_mtime(path):
    newest = 0.0
    for root, _dirs, files in os.walk(path):
        for f in files:
            try:
                m = os.path.getmtime(os.path.join(root, f))
            except OSError:
                continue
            newest = max(newest, m)
    return newest


def check_stale_server():
    """(a) branchless-lsp binary stale vs newest source mtime."""
    name = "stale-server"
    src = os.path.join(LSPMAX_ROOT, "crates/branchless-lsp/src")
    bins = [
        os.path.join(LSPMAX_ROOT, "target/release/branchless-lsp"),
        os.path.join(LSPMAX_ROOT, "target/debug/branchless-lsp"),
    ]
    if not os.path.isdir(src):
        return record(name, "RED", "source dir missing: " + src)
    newest_src = newest_mtime(src)
    if newest_src == 0.0:
        return record(name, "RED", "no source files under " + src)
    present = [b for b in bins if os.path.exists(b)]
    if not present:
        return record(name, "WARN", "no built binary", "cargo build --release -p branchless-lsp")
    newest_bin = max(os.path.getmtime(b) for b in present)
    if newest_bin < newest_src:
        age_min = (newest_src - newest_bin) / 60.0
        return record(
            name, "WARN",
            "binary stale by {:.0f} min (src {} > bin {})".format(age_min,
                time.strftime("%H:%M", time.localtime(newest_src)),
                time.strftime("%H:%M", time.localtime(newest_bin))),
            "cargo build --release -p branchless-lsp",
        )
    return record(name, "PASS", "binary newer than newest source")

LEASE_PATTERNS = [
    "b0*-target", "c0*-target", "exp0*-target", "fix0*-target",
    "git0*-target", "csct-*-target", "mv*-target", "g0*-target",
    "seed*-target", "lspa*-target", "lspa*", "dx*-target",
]


def dir_size(path):
    total = 0
    for root, _dirs, files in os.walk(path):
        for f in files:
            try:
                total += os.lstat(os.path.join(root, f)).st_size
            except OSError:
                continue
    return total


def check_orphan_leases():
    """(b) orphaned /tmp build-lease dirs, count + bytes."""
    name = "orphan-leases"
    dirs = []
    for pat in LEASE_PATTERNS:
        for d in sorted(glob.glob(os.path.join(TMP_ROOT, pat))):
            if os.path.isdir(d) and d not in dirs:
                dirs.append(d)
    if not dirs:
        return record(name, "PASS", "no lease dirs in " + TMP_ROOT)
    total = sum(dir_size(d) for d in dirs)
    return record(
        name, "WARN",
        "{} lease dir(s), {:.1f} MB: {}".format(len(dirs), total / 1e6,
            ", ".join(os.path.basename(d) for d in dirs[:12])),
        "python3 scripts/lease_janitor.py (DX-3)",
    )


def check_workspace_freshness():
    """(c) cargo metadata member count vs CLAUDE.md recorded count."""
    name = "workspace-count"
    try:
        out = subprocess.run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1"],
            cwd=CGAP_ROOT, capture_output=True, text=True, timeout=60,
        ).stdout
    except Exception as e:  # noqa: BLE001
        return record(name, "RED", "cargo metadata failed: {}".format(e))
    import json
    try:
        n = len(json.loads(out).get("workspace_members", []))
    except ValueError:
        return record(name, "RED", "cargo metadata returned invalid JSON")
    try:
        doc = open(CLAUDE_MD).read()
    except OSError as e:
        return record(name, "RED", "CLAUDE.md unreadable: {}".format(e))
    m = re.search(r"(\d+)-crate root", doc)
    if not m:
        return record(name, "RED", "no '<N>-crate root' claim in CLAUDE.md")
    recorded = int(m.group(1))
    if n != recorded:
        return record(name, "WARN",
            "cargo metadata members={} vs CLAUDE.md claims {}".format(n, recorded),
            "update CLAUDE.md '<N>-crate root' line (MV-8 owns CLAUDE.md)")
    return record(name, "PASS", "members={} matches CLAUDE.md".format(n))


def check_dirty_tree():
    """(d) dirty-tree census by top-level dir (read-only git)."""
    name = "dirty-tree"
    out = subprocess.run(
        ["git", "status", "--porcelain"],
        cwd=CGAP_ROOT, capture_output=True, text=True,
    ).stdout
    counts = {}
    for line in out.splitlines():
        p = line[3:].strip().strip('"')
        top = p.split("/")[0] if p else "?"
        counts[top] = counts.get(top, 0) + 1
    if not counts:
        return record(name, "PASS", "clean tree")
    summary = ", ".join("{}:{}".format(k, v)
                        for k, v in sorted(counts.items(), key=lambda kv: -kv[1])[:15])
    return record(name, "WARN", "{} path(s) dirty: {}".format(len(out.splitlines()), summary),
                  "expected mid-wave; coordinator integrates")


def check_c01_toolchain():
    """c01 (ported from doctor.rs:186): cargo + rustc respond.

    Missing-but-expected = WARN (cull-02 DXD-02 degrade law), never RED.
    Poison arm: DX1_POISON_C01=1 simulates a broken toolchain -> WARN."""
    name = "c01-toolchain"
    if os.environ.get("DX1_POISON_C01") == "1":
        return record(name, "WARN",
                      "poison arm: toolchain broken (DX1_POISON_C01=1)")
    for tool in ("cargo", "rustc"):
        try:
            p = subprocess.run([tool, "--version"], capture_output=True,
                               text=True, timeout=30)
        except (OSError, subprocess.TimeoutExpired):
            return record(name, "WARN", tool + " not available",
                          "install rustup / rust toolchain")
        if p.returncode != 0:
            return record(name, "WARN",
                          tool + " --version exit {}".format(p.returncode),
                          "reinstall toolchain")
    cargo_v = subprocess.run(["cargo", "--version"], capture_output=True,
                             text=True, timeout=30).stdout.strip()
    return record(name, "PASS", cargo_v)


def check_c03_git():
    """c03 (ported from doctor.rs:244): git responds in CGAP_ROOT.

    Poison arm: DX1_POISON_C03=1 -> WARN."""
    name = "c03-git"
    if os.environ.get("DX1_POISON_C03") == "1":
        return record(name, "WARN", "poison arm: git unavailable (DX1_POISON_C03=1)",
                      "install git")
    try:
        p = subprocess.run(["git", "-C", CGAP_ROOT, "rev-parse", "--git-dir"],
                           capture_output=True, text=True, timeout=30)
    except (OSError, subprocess.TimeoutExpired):
        return record(name, "WARN", "git not available", "install git")
    if p.returncode != 0:
        return record(name, "WARN", "git rev-parse exit {}".format(p.returncode),
                      "check " + CGAP_ROOT + " is a git checkout")
    return record(name, "PASS", "git responds in " + CGAP_ROOT)


def check_c04_b3sum():
    """c04 (ported from doctor.rs:252): b3sum on PATH (BLAKE3 cross-check
    oracle). Poison arms: DX1_B3SUM override to a nonexistent path, or a
    stripped PATH environment -> WARN (never RED)."""
    name = "c04-b3sum"
    b3sum = os.environ.get("DX1_B3SUM", "b3sum")
    if os.path.isabs(b3sum) and not os.path.exists(b3sum):
        return record(name, "WARN", "b3sum override not found: " + b3sum,
                      "cargo install b3sum / brew install b3sum")
    which = shutil.which(b3sum)
    if which is None:
        return record(name, "WARN", "b3sum not on PATH",
                      "cargo install b3sum / brew install b3sum")
    try:
        p = subprocess.run([which, "--version"], capture_output=True,
                           text=True, timeout=30)
    except (OSError, subprocess.TimeoutExpired):
        return record(name, "WARN", "b3sum --version failed", "reinstall b3sum")
    return record(name, "PASS", p.stdout.strip() or which)


def check_c05_pyshacl():
    """c05 (ported from doctor.rs:261): pyshacl importable via python3.
    Poison arms: DX1_PYTHON3 override to a nonexistent interpreter, or a
    stripped PATH environment -> WARN (never RED)."""
    name = "c05-pyshacl"
    python = os.environ.get("DX1_PYTHON3", "python3")
    if not _spawn_ok(python):
        return record(name, "WARN", "python3 not available: " + python,
                      "install python3; python3 -m pip install pyshacl")
    try:
        p = subprocess.run([python, "-c", "import pyshacl"],
                           capture_output=True, text=True, timeout=60)
    except (OSError, subprocess.TimeoutExpired):
        return record(name, "WARN", "python3 spawn failed", "install python3")
    if p.returncode != 0:
        tail = (p.stderr or "").strip().splitlines()
        tail = tail[-1].strip() if tail else ""
        return record(name, "WARN",
                      "import pyshacl failed (exit {}){}".format(p.returncode,
                                                                 ": " + tail if tail else ""),
                      "python3 -m pip install pyshacl")
    return record(name, "PASS", "import pyshacl ok")


def check_c06_workspace_check(full=False):
    """c06 (ported from doctor.rs:285): `cargo check --workspace` exit.

    FLAG-GATED per cull-02 open question 2 (heaviest check vs DXD-02's
    ~5s budget): runs only under `--full`; absent flag -> loud SKIP line,
    never silent. WARN-degrade: nonzero exit = WARN, never RED (DXD-02
    degrade law). Poison arm: DX1_POISON_C06=1 -> WARN."""
    name = "c06-workspace-check"
    if not full:
        return record(name, "SKIP",
                      "cargo check --workspace gated behind --full "
                      "(cull-02 runtime budget)")
    if os.environ.get("DX1_POISON_C06") == "1":
        return record(name, "WARN", "poison arm: check broken (DX1_POISON_C06=1)")
    try:
        p = subprocess.run(["cargo", "check", "--workspace"],
                           cwd=CGAP_ROOT, capture_output=True, text=True,
                           timeout=600)
    except (OSError, subprocess.TimeoutExpired) as e:
        return record(name, "WARN", "cargo check failed to run: {}".format(e),
                      "cargo check --workspace 2>&1 | head -50")
    if p.returncode != 0:
        tail = [l.strip() for l in (p.stderr or "").splitlines()
                if l.strip()][-3:]
        return record(name, "WARN",
                      "cargo check exit {}: {}".format(p.returncode,
                                                       " | ".join(tail)),
                      "cargo check --workspace 2>&1 | head -50")
    warns = sum(1 for l in (p.stderr or "").splitlines()
                if l.strip().startswith("warning:"))
    if warns:
        return record(name, "WARN", "exit 0 with {} warning(s)".format(warns),
                      "cargo check --workspace 2>&1 | grep -A3 warning")
    return record(name, "PASS", "cargo check --workspace exit 0, 0 warnings")


def check_c07_scripts():
    """c07 (ported from doctor.rs:337): gate scripts exist + executable.
    DX1_CGAP_ROOT-relative. WARN-degrade: missing OR not-executable ->
    WARN (cull-02: v26.10.31-frozen 10-name list migrates WARN-degrade).
    Poison arm: DX1_CGAP_ROOT pointed at a poisoned /tmp root missing the
    scripts -> WARN."""
    name = "c07-scripts"
    scripts = [
        "verify_v26.10.31.sh", "audit_branchless.sh", "cfg_homology.sh",
        "validate_ontology.sh", "mint_receipt.sh", "perf_guard.sh",
        "dod.sh", "zero_datafusion_gate.py", "receipt_lint.py",
        "registry_sweep.py",
    ]
    missing, notx = [], []
    for s in scripts:
        p = os.path.join(CGAP_ROOT, "scripts", s)
        if not os.path.isfile(p):
            missing.append(s)
        elif not os.access(p, os.X_OK):
            notx.append(s)
    if not missing and not notx:
        return record(name, "PASS", "10/10 gate scripts executable")
    detail = []
    if missing:
        detail.append("missing: " + ", ".join(missing))
    if notx:
        detail.append("not executable: " + ", ".join(notx))
    return record(name, "WARN", "; ".join(detail),
                  "chmod +x scripts/<name> / check DX1_CGAP_ROOT")


def check_c08_receipts():
    """c08 (ported from doctor.rs:395): receipt count + staleness vs
    newest crates/ mtime. DX1_CGAP_ROOT-relative; receipt dir is the
    v26.10.31 milestone (frozen path, WARN-degrade). Code newer than the
    newest receipt -> WARN. Poison arm: DX1_CGAP_ROOT without receipts
    dir -> WARN."""
    name = "c08-receipts"
    rdir = os.path.join(CGAP_ROOT, "docs/jira/v26.10.31/receipts")
    if not os.path.isdir(rdir):
        return record(name, "WARN", "receipts dir unreadable: " + rdir,
                      "expected docs/jira/v26.10.31/receipts/")
    count = 0
    newest_r = 0.0
    for f in os.listdir(rdir):
        p = os.path.join(rdir, f)
        if f.endswith(".md") and os.path.isfile(p):
            count += 1
            newest_r = max(newest_r, os.path.getmtime(p))
    crates_newest = newest_mtime(os.path.join(CGAP_ROOT, "crates"))
    if crates_newest > newest_r:
        return record(name, "WARN",
                      "{} receipts; newest crates/ mtime newer than newest "
                      "receipt by {:.0f}s".format(count,
                                                  crates_newest - newest_r),
                      "mint a fresh receipt (scripts/mint_receipt.sh)")
    return record(name, "PASS",
                  "{} receipts; not stale".format(count))


def check_c09_lsl():
    """c09 (ported from doctor.rs:449): CTQ-06 LSL ratification state in
    crates/bench/src/bin/ctq_report.rs. DX1_CGAP_ROOT-relative,
    WARN-degrade: missing file / missing const / None -> WARN.
    Poison arm: DX1_CGAP_ROOT without ctq_report.rs -> WARN."""
    name = "c09-lsl-ratified"
    p = os.path.join(CGAP_ROOT, "crates/bench/src/bin/ctq_report.rs")
    try:
        src = open(p).read()
    except OSError as e:
        return record(name, "WARN", "ctq_report.rs unreadable: {}".format(e),
                      "expected crates/bench/src/bin/ctq_report.rs")
    line = next((l for l in src.splitlines()
                 if "const LSL_RATIFIED: Option<f64> =" in l), None)
    if line is None:
        return record(name, "WARN", "LSL_RATIFIED const not found",
                      "inspect crates/bench/src/bin/ctq_report.rs")
    rhs = line.split("=", 1)[1].strip().rstrip(";").strip()
    if rhs.startswith("Some"):
        return record(name, "PASS", "RATIFIED ({})".format(rhs))
    return record(name, "WARN",
                  "CTQ-06 ratification pending ({})".format(rhs),
                  "59-ratification-package.md")


def _spawn_ok(python):
    """Spawn-probe used by the c05 poison arm (nonexistent interpreter)."""
    try:
        return subprocess.run([python, "-c", "pass"], capture_output=True,
                              text=True, timeout=15).returncode == 0
    except (OSError, subprocess.TimeoutExpired):
        return False


def check_c10_dispatch_ledger():
    """c10 (TC-11, tps-07 rank 2 / A2): dispatch-ledger silent-death watch.

    A ledger row older than LEDGER_STALE_MIN (default 45) with NO matching
    lanes/lane-<id>*.md receipt = WARN 'silent death'. Row + receipt = PASS.
    A <lane-id>.alive heartbeat (contract-block law) older than the same N
    = WARN. Fresh in-flight rows (no receipt yet, within N) = PASS.
    Courts root the check at /tmp sandboxes via DX1_LEDGER_PATH /
    DX1_LANES_DIR / DX1_LEDGER_STALE_MIN; poison arm DX1_POISON_C10=1."""
    name = "c10-dispatch-ledger"
    if os.environ.get("DX1_POISON_C10") == "1":
        return record(name, "WARN", "poison arm: ledger watch (DX1_POISON_C10=1)")
    if not os.path.isfile(LEDGER_PATH):
        return record(name, "WARN", "no dispatch ledger: " + LEDGER_PATH,
                      "seed docs/jira/v26.11.2/lanes/dispatch-ledger.jsonl (TC-11)")
    try:
        raw_rows = open(LEDGER_PATH).read().splitlines()
    except OSError as e:
        return record(name, "RED", "ledger unreadable: {}".format(e))
    now = time.time()
    silent, stale_hb, receipted, fresh = [], [], [], []
    for line in raw_rows:
        if not line.strip():
            continue
        try:
            row = json.loads(line)
        except ValueError:
            return record(name, "RED",
                          "invalid JSON row: " + line[:60], "fix ledger row")
        lane = str(row.get("lane", ""))
        age_min = (now - float(row.get("ts", 0))) / 60.0
        has_receipt = bool(glob.glob(os.path.join(LANES_DIR, "lane-%s*.md" % lane)))
        hb = os.path.join(LANES_DIR, lane + ".alive")
        hb_min = ((now - os.path.getmtime(hb)) / 60.0
                  if os.path.exists(hb) else None)
        if has_receipt:
            receipted.append(lane)
        elif age_min > LEDGER_STALE_MIN:
            silent.append("%s (row %.0f min old, no receipt)" % (lane, age_min))
        else:
            fresh.append(lane)
        if hb_min is not None and hb_min > LEDGER_STALE_MIN:
            stale_hb.append("%s (heartbeat %.0f min old)" % (lane, hb_min))
    if silent:
        return record(name, "WARN", "SILENT DEATH: " + "; ".join(silent[:8]),
                      "dispatch a replacement lane or mint the receipt")
    if stale_hb:
        return record(name, "WARN", "stale heartbeat: " + "; ".join(stale_hb[:8]),
                      "lane process touched .alive; check it is alive")
    detail = "{} receipted, {} in-flight(fresh)".format(len(receipted), len(fresh))
    return record(name, "PASS", detail)


def main(argv=None):
    argv = sys.argv[1:] if argv is None else argv
    full = "--full" in argv
    check_stale_server()
    check_orphan_leases()
    check_workspace_freshness()
    check_dirty_tree()
    check_c01_toolchain()
    check_c03_git()
    check_c04_b3sum()
    check_c05_pyshacl()
    check_c06_workspace_check(full=full)
    check_c07_scripts()
    check_c08_receipts()
    check_c09_lsl()
    # TC-11 appends LAST; when TC-02's ports check lands it must slot
    # BEFORE this line (disclosed per same-checkout-fanout law).
    check_c10_dispatch_ledger()
    codes = {"PASS": 0, "WARN": 1, "SKIP": 0, "RED": 2}
    return max(codes[s] for _n, s, _d, _r in RESULTS)


if __name__ == "__main__":
    sys.exit(main())
