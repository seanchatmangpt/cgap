#!/usr/bin/env python3
"""zero_datafusion_gate.py — three-probe zero-datafusion consensus gate.

Probes (all three must agree; disagreement = typed refusal, exit 2):
  P1 metadata: `cargo metadata` JSON — no package/dependency named
     *datafusion* anywhere in the resolved graph.
  P2 tree:     `cargo tree --workspace --edges all` text scan.
  P3 literal:  substring scan of root Cargo.toml, every member Cargo.toml,
     and Cargo.lock.

Latent opt-in channel: every probe also folds in the root
[workspace.dependencies] keys — an unused ws-dependency entry is a
one-line opt-in for any member and must be flagged.

Frozen interface: docs/jira/v26.11.2/waves/w1-05-doctor-extension-spec.md
§4.1. Citation: docs/jira/v26.11.1/waves/cgap-datafusion-removal.md.

Exits: 0 clean / 1 VIOLATION (file+dep-path printed) / 2 GATE-REFUSED
(indeterminate or probe disagreement) / 3 VACUOUS (self-check poison
survived or clean twin failed).
Final stdout line: `ZERO-DATAFUSION: PASS` | `ZERO-DATAFUSION: FAIL n`.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

EXIT_CLEAN = 0
EXIT_VIOLATION = 1
EXIT_REFUSED = 2
EXIT_VACUOUS = 3
NEEDLE = "datafusion"
CLEAN, HIT, ERROR = "CLEAN", "HIT", "ERROR"


def ws_dep_names(root_toml: Path) -> set[str]:
    """Root [workspace.dependencies] keys (latent opt-in channel)."""
    try:
        data = tomllib.loads(root_toml.read_text(encoding="utf-8"))
        return set(data.get("workspace", {}).get("dependencies", {}).keys())
    except Exception:
        txt = root_toml.read_text(encoding="utf-8")
        m = re.search(r"\[workspace\.dependencies\](.*?)(?:\n\[|\Z)", txt, re.S)
        if not m:
            return set()
        return set(re.findall(r"^([A-Za-z0-9_\-]+)\s*=", m.group(1), re.M))


def member_manifests(root_dir: Path) -> list[Path]:
    """Member Cargo.toml paths from the root manifest's explicit members."""
    root_toml = root_dir / "Cargo.toml"
    out = [root_toml]
    try:
        data = tomllib.loads(root_toml.read_text(encoding="utf-8"))
        members = data.get("workspace", {}).get("members", [])
    except Exception:
        return out
    for m in members:
        p = (root_dir / m).resolve()
        cand = p / "Cargo.toml"
        if cand.is_file():
            out.append(cand)
    return out


def probe_metadata(root_dir: Path) -> tuple[str, list[dict]]:
    """P1: cargo metadata JSON + root ws-deps. ERROR on cargo failure."""
    hits: list[dict] = []
    try:
        proc = subprocess.run(
            ["cargo", "metadata", "--format-version", "1",
             "--manifest-path", str(root_dir / "Cargo.toml")],
            capture_output=True, text=True, timeout=180, cwd=str(root_dir))
    except (OSError, subprocess.TimeoutExpired) as exc:
        return ERROR, [{"path": "<cargo metadata>", "line": 0,
                        "snippet": f"spawn/timeout: {exc}"}]
    if proc.returncode != 0:
        return ERROR, [{"path": "<cargo metadata>", "line": 0,
                        "snippet": (proc.stderr or "").strip()[-300:]}]
    data = json.loads(proc.stdout)
    for pkg in data.get("packages", []):
        name = pkg.get("name", "")
        if NEEDLE in name.lower():
            hits.append({"path": f"cargo-metadata:pkg:{name}", "line": 0,
                         "snippet": name})
        for dep in pkg.get("dependencies", []):
            dname = dep.get("name", "")
            if NEEDLE in dname.lower():
                dep_path = f"{name} -> {dname} ({dep.get('kind') or 'normal'})"
                hits.append({"path": f"cargo-metadata:dep:{dep_path}",
                             "line": 0, "snippet": dep_path})
    for wname in ws_dep_names(root_dir / "Cargo.toml"):
        if NEEDLE in wname.lower():
            hits.append({"path": "cargo-metadata:workspace-dependencies",
                         "line": 0, "snippet": wname})
    return (HIT if hits else CLEAN), hits


def probe_tree(root_dir: Path, target_dir: Path | None) -> tuple[str, list[dict]]:
    """P2: `cargo tree --workspace --edges all` text scan + root ws-deps."""
    hits: list[dict] = []
    cmd = ["cargo", "tree", "--workspace", "--edges", "all",
           "--manifest-path", str(root_dir / "Cargo.toml")]
    if target_dir is not None:
        cmd += ["--target-dir", str(target_dir)]
    env = dict(os.environ, CARGO_NET_OFFLINE="true")
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True,
                              timeout=300, cwd=str(root_dir), env=env)
    except (OSError, subprocess.TimeoutExpired) as exc:
        return ERROR, [{"path": "<cargo tree>", "line": 0,
                        "snippet": f"spawn/timeout: {exc}"}]
    if proc.returncode != 0:
        return ERROR, [{"path": "<cargo tree>", "line": 0,
                        "snippet": (proc.stderr or "").strip()[-300:]}]
    for i, line in enumerate(proc.stdout.splitlines(), 1):
        if NEEDLE in line.lower():
            hits.append({"path": "<cargo tree stdout>", "line": i,
                         "snippet": line.strip()[:160]})
    for wname in ws_dep_names(root_dir / "Cargo.toml"):
        if NEEDLE in wname.lower():
            hits.append({"path": "cargo-tree:workspace-dependencies",
                         "line": 0, "snippet": wname})
    return (HIT if hits else CLEAN), hits


def probe_literal(root_dir: Path) -> tuple[str, list[dict]]:
    """P3: literal substring scan of root/member Cargo.tomls + Cargo.lock."""
    hits: list[dict] = []
    files = member_manifests(root_dir) + [root_dir / "Cargo.lock"]
    for f in files:
        if not f.is_file():
            if f.name == "Cargo.lock":
                return ERROR, [{"path": "<cargo lock>", "line": 0,
                                "snippet": "Cargo.lock missing"}]
            continue
        try:
            lines = f.read_text(encoding="utf-8", errors="replace").splitlines()
        except OSError as exc:
            return ERROR, [{"path": str(f), "line": 0,
                            "snippet": f"unreadable: {exc}"}]
        for i, line in enumerate(lines, 1):
            if NEEDLE in line.lower():
                hits.append({"path": str(f), "line": i,
                             "snippet": line.strip()[:160]})
    return (HIT if hits else CLEAN), hits


def run_probes(root_dir: Path,
               target_dir: Path | None) -> tuple[dict, list[dict]]:
    """Run P1..P3; return per-probe status map and union of violations."""
    probes: dict[str, str] = {}
    violations: list[dict] = []
    for label, fn in (("metadata", probe_metadata),
                      ("tree", lambda r: probe_tree(r, target_dir)),
                      ("literal", probe_literal)):
        status, hits = fn(root_dir)
        probes[label] = status
        violations.extend(hits)
    return probes, violations


def classify(probes: dict[str, str]) -> int:
    """Consensus: all CLEAN -> 0; any ERROR -> 2; all HIT -> 1;
    mixed HIT/CLEAN (probe disagreement) -> 2."""
    states = set(probes.values())
    if states == {CLEAN}:
        return EXIT_CLEAN
    if ERROR in states:
        return EXIT_REFUSED
    if states == {HIT}:
        return EXIT_VIOLATION
    return EXIT_REFUSED


POISON_BLOCK = """
# SELF-CHECK POISON (zero_datafusion_gate): synthetic marker, not a real
# dependency. Present ONLY inside /tmp sandbox copies under --self-check.
[workspace.dependencies]
datafusion = "1"
"""


def copy_subject(root_dir: Path, sandbox: Path) -> None:
    """Copy the scan corpus into the sandbox (ignore .git/target/docs)."""
    ignore = shutil.ignore_patterns(".git", "target", "docs", "*.lock")
    shutil.copytree(root_dir, sandbox, dirs_exist_ok=True, ignore=ignore)
    for name in ("Cargo.toml", "Cargo.lock"):
        src = root_dir / name
        if src.is_file():
            shutil.copy2(src, sandbox / name)


def self_check(root_dir: Path, target_dir: Path | None) -> tuple[bool, bool, str]:
    """Two arms on /tmp sandbox copies of the scan corpus.

    clean arm: unmutated copy must classify CLEAN (exit 0).
    poison arm: inject dummy `datafusion = "1"` ws-dep into the copied
    root Cargo.toml; production predicate MUST flag it (exit 1).
    Returns (clean_ok, poison_killed, note)."""
    note = ""
    clean_ok = poison_killed = False
    sandbox = Path(tempfile.mkdtemp(prefix="zdf-selfcheck-"))
    try:
        copy_subject(root_dir, sandbox)
        probes, _ = run_probes(sandbox, target_dir)
        clean_ok = classify(probes) == EXIT_CLEAN
        if not clean_ok:
            note = f"clean twin not green: {probes}"
        root_copy = sandbox / "Cargo.toml"
        txt = root_copy.read_text(encoding="utf-8")
        root_copy.write_text(txt + POISON_BLOCK, encoding="utf-8")
        probes, _ = run_probes(sandbox, target_dir)
        poison_killed = classify(probes) == EXIT_VIOLATION
        if not poison_killed:
            note += f" poison probes={probes}"
    finally:
        shutil.rmtree(sandbox, ignore_errors=True)
    return clean_ok, poison_killed, note


def main() -> int:
    root_default = Path(__file__).resolve().parent.parent
    ap = argparse.ArgumentParser(description="zero-datafusion consensus gate")
    ap.add_argument("--root", default=str(root_default))
    ap.add_argument("--target-dir", default=None)
    ap.add_argument("--self-check", action="store_true")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--verbose", action="store_true")
    args = ap.parse_args()
    root_dir = Path(args.root).resolve()
    target_dir = Path(args.target_dir).resolve() if args.target_dir else None
    receipt: dict = {"gate": "zero_datafusion", "verdict": None,
                     "violations": [], "probes": {},
                     "self_check": {"ran": False, "poison_killed": False,
                                    "clean_twin_green": False},
                     "seed": None}

    root_toml = root_dir / "Cargo.toml"
    if not root_toml.is_file():
        receipt["verdict"] = "SKIP"
        print(f"[SKIP] subject missing: no Cargo.toml at {root_dir}",
              file=sys.stderr)
        receipt["violations"] = [{"path": str(root_toml), "line": 0,
                                  "snippet": "Cargo.toml missing"}]
        if args.json:
            print(json.dumps(receipt, indent=2))
        print("ZERO-DATAFUSION: SKIP 0")
        return EXIT_REFUSED

    if args.self_check:
        receipt["self_check"]["ran"] = True
        clean_ok, poison_killed, note = self_check(root_dir, target_dir)
        receipt["self_check"]["clean_twin_green"] = clean_ok
        receipt["self_check"]["poison_killed"] = poison_killed
        if not (clean_ok and poison_killed):
            receipt["verdict"] = "SKIP"
            print(f"VACUOUS GATE: self-check arms failed ({note})",
                  file=sys.stderr)
            if args.json:
                print(json.dumps(receipt, indent=2))
            print("ZERO-DATAFUSION: SKIP 1")
            return EXIT_VACUOUS
        receipt["verdict"] = "PASS"
        if args.verbose:
            print("self-check: clean twin green; poison killed (exit 1)")
        if args.json:
            print(json.dumps(receipt, indent=2))
        print("ZERO-DATAFUSION: PASS")
        return EXIT_CLEAN

    probes, violations = run_probes(root_dir, target_dir)
    receipt["probes"] = probes
    receipt["violations"] = violations
    rc = classify(probes)
    if rc == EXIT_CLEAN:
        receipt["verdict"] = "PASS"
    elif rc == EXIT_VIOLATION:
        receipt["verdict"] = "FAIL"
        print("VIOLATION: datafusion reference found —", file=sys.stderr)
        for v in violations:
            print(f"  {v['path']}:{v['line']}: {v['snippet']}",
                  file=sys.stderr)
    else:
        receipt["verdict"] = "SKIP"
        print("GATE-REFUSED: probes indeterminate or disagree —",
              file=sys.stderr)
        for label, st in probes.items():
            print(f"  {label}: {st}", file=sys.stderr)
        for v in violations:
            print(f"  {v['path']}:{v['line']}: {v['snippet']}",
                  file=sys.stderr)
    if args.verbose:
        for v in violations:
            print(f"  {v['path']}:{v['line']}: {v['snippet']}")
    if args.json:
        print(json.dumps(receipt, indent=2))
    if rc == EXIT_CLEAN:
        print("ZERO-DATAFUSION: PASS")
    else:
        print(f"ZERO-DATAFUSION: FAIL {len(violations)}")
    return rc


if __name__ == "__main__":
    sys.exit(main())
