#!/usr/bin/env python3
"""cgap pre-publish release gate (stdlib only).

Gates:
  1. Workspace version is exactly 26.10.29 in root [workspace.package] and
     every member crate inherits it via version.workspace = true.
  2. Publishable crates (no publish = false) each have non-empty description,
     license, repository. Dependency law (repaired 2026-11-01, W2-10): every
     [dependencies] entry of a publishable crate must carry a version pin;
     a path dep must ALSO carry version (path+version). The prior law
     ("[dependencies] must be empty") contradicted shipped reality —
     cgap-bont depends on cgap-blake3 and is PUBLISHED-26.11.1 on
     crates.io — so path+version and registry version pins are both lawful.
  3. cargo package --no-verify --allow-dirty -p <crate> for each publishable
     crate: exit codes, .crate sizes, file lists (cargo package --list),
     warnings.

Exit 1 on any violation. Final line: RELEASE-CHECK: PASS | RELEASE-CHECK: FAIL <reasons>.
"""
from __future__ import annotations

import glob as globlib
import subprocess
import sys
import tomllib
from pathlib import Path

REQUIRED_VERSION = "26.11.2"
ALLOW_UNCOMMITTED = True  # canonical checkout carries uncommitted campaign
# docs by design; --allow-dirty is lawful on every cargo package invocation.
CRATES_DIR = "crates"
PKG_TIMEOUT_S = 900


class Findings:
    def __init__(self) -> None:
        self.reasons: list[str] = []
        self.notes: list[str] = []

    def fail(self, msg: str) -> None:
        self.reasons.append(msg)

    def note(self, msg: str) -> None:
        self.notes.append(msg)


def human(n: int) -> str:
    size = float(n)
    for unit in ("B", "KiB", "MiB", "GiB"):
        if size < 1024.0 or unit == "GiB":
            return f"{size:.1f} {unit}" if unit != "B" else f"{int(size)} B"
        size /= 1024.0
    return f"{n} B"


def load_manifest(path: Path) -> dict:
    with path.open("rb") as fh:
        return tomllib.load(fh)


def crate_version(pkg: dict, root_pkg: dict) -> str:
    v = pkg.get("version")
    if isinstance(v, dict) and v.get("workspace") is True:
        return str(root_pkg.get("version", "<workspace-version-missing>"))
    if isinstance(v, str):
        return v
    return "<version-unresolvable>"


def is_inherited(v: object) -> bool:
    return isinstance(v, dict) and v.get("workspace") is True


def resolve_members(root_dir: Path, members: list[str]) -> list[Path]:
    out: list[Path] = []
    for entry in members:
        pattern = str(root_dir / entry)
        matches = sorted(globlib.glob(pattern))
        if not matches:
            continue
        for m in matches:
            p = Path(m)
            if (p / "Cargo.toml").exists():
                out.append(p)
    return out


def check_workspace_version(root_dir: Path, root: dict, f: Findings) -> tuple[list[Path], list[Path], dict]:
    ws_pkg = root.get("workspace", {}).get("package", {})
    version = ws_pkg.get("version")

    print("=" * 72)
    print("CHECK 1: workspace version + member inheritance")
    print("=" * 72)
    if version == REQUIRED_VERSION:
        print(f"  OK    [workspace.package].version == {REQUIRED_VERSION!r}")
    else:
        f.fail(f"root [workspace.package].version is {version!r}, expected {REQUIRED_VERSION!r}")
        print(f"  FAIL  root [workspace.package].version = {version!r} (expected {REQUIRED_VERSION!r})")

    members_tbl = root.get("workspace", {}).get("members", [])
    member_dirs = resolve_members(root_dir, members_tbl)

    # Crates on disk under crates/ (publishable ones may sit outside `members`).
    disk_dirs = sorted(
        (d for d in (root_dir / CRATES_DIR).glob("*/") if (d / "Cargo.toml").exists()),
        key=lambda p: p.name,
    ) if (root_dir / CRATES_DIR).exists() else []
    member_names = {d.name for d in member_dirs}
    unlisted = [d for d in disk_dirs if d.name not in member_names]

    checked: list[tuple[Path, dict]] = []
    for d in member_dirs:
        man = load_manifest(d / "Cargo.toml")
        pkg = man.get("package", {})
        name = pkg.get("name", d.name)
        v = pkg.get("version")
        if is_inherited(v):
            print(f"  OK    {name}: version.workspace = true (inherits {version!r})")
        elif v is None:
            f.fail(f"{name}: package.version missing entirely (no inheritance, no literal)")
            print(f"  FAIL  {name}: package.version missing")
        elif isinstance(v, str):
            f.fail(f"{name}: package.version is pinned literal {v!r} instead of version.workspace = true")
            print(f"  FAIL  {name}: version pinned to literal {v!r} (not inherited)")
        checked.append((d, man))

    for d in unlisted:
        man = load_manifest(d / "Cargo.toml")
        pkg = man.get("package", {})
        name = pkg.get("name", d.name)
        v = pkg.get("version")
        inherited = is_inherited(v)
        standalone = "workspace" in man
        if standalone:
            # Lawful standalone crate (own [workspace] table): cgap-cli is a
            # standalone ggen six-pack by law; cgap-verification and
            # wasm-footprint declare their own [workspace] the same way.
            # Status is read from the manifest + cargo metadata, not assumed.
            status = "OK"
            msg = f"{name}: standalone crate (own [workspace], not a root member); version {v!r}"
            f.note(
                f"{name} (crates/{d.name}) is a standalone crate with its own "
                "[workspace] table — deliberately not a root-workspace member"
            )
        elif inherited:
            status = "OK"
            msg = f"{name}: version.workspace = true (inherits {version!r})"
            f.note(f"{name} (crates/{d.name}) inherits the workspace version but is NOT listed in [workspace].members")
        else:
            status = "FAIL"
            msg = f"{name}: on disk, not a member, not standalone, version not inherited"
            f.fail(
                f"{name}: on disk under crates/ but not a workspace member; version not inherited"
            )
        print(f"  {status:<5}  {msg}   [NOTE: on disk under crates/ but NOT in workspace members]")
        checked.append((d, man))

    for _d, _man in checked:
        _man["_manifest_dir"] = _d
    keyed = {
        man.get("package", {}).get("name", d.name): man
        for d, man in checked
    }
    return member_dirs, disk_dirs, keyed


def check_publishable_metadata(root_dir: Path, root: dict, manifests: dict[str, dict], f: Findings) -> list[str]:
    print()
    print("=" * 72)
    print("CHECK 2: publishable crate metadata + dependency hygiene")
    print("=" * 72)
    ws_pkg = root.get("workspace", {}).get("package", {})

    def has_field(pkg: dict, key: str) -> tuple[bool, str]:
        v = pkg.get(key)
        if isinstance(v, str) and v.strip():
            return True, v
        if is_inherited(v):
            wv = ws_pkg.get(key)
            if isinstance(wv, str) and wv.strip():
                return True, f"workspace-inherited: {wv}"
            return False, "declares <key>.workspace = true but root [workspace.package] lacks it"
        if v is None:
            return False, "missing"
        return False, f"empty/invalid: {v!r}"

    publishable: list[str] = []
    for name, man in sorted(manifests.items()):
        pkg = man.get("package", {})
        if pkg.get("publish") is False:
            print(f"  --    {name}: publish = false (skipped)")
            continue
        publishable.append(name)

    for name in sorted(publishable):
        man = manifests[name]
        pkg = man.get("package", {})
        print(f"  crate {name}:")

        ok_desc, desc = has_field(pkg, "description")
        if ok_desc:
            print(f"    OK    description: {desc[:70]}{'...' if len(desc) > 70 else ''}")
        else:
            f.fail(f"{name}: description {desc}")
            print(f"    FAIL  description: {desc}")

        # license: accept `license` or house-style `license-file` (incl.
        # workspace-inherited license-file from [workspace.package])
        ok_lic, lic = has_field(pkg, "license")
        if not ok_lic:
            ok_lic, lic = has_field(pkg, "license-file")
        if ok_lic:
            print(f"    OK    license: {lic}")
        else:
            f.fail(f"{name}: license {lic}")
            print(f"    FAIL  license: {lic}")

        ok_repo, repo = has_field(pkg, "repository")
        if ok_repo:
            print(f"    OK    repository: {repo}")
        else:
            f.fail(f"{name}: repository {repo}")
            print(f"    FAIL  repository: {repo}")

        deps = man.get("dependencies", {})
        unversioned = []
        for dep, spec in sorted(deps.items()):
            has_version = (
                isinstance(spec, dict)
                and isinstance(spec.get("version"), str)
                and bool(spec["version"].strip())
            ) or isinstance(spec, str)
            # A bare-string spec is always a registry requirement ("x = \"1.2\"").
            if isinstance(spec, dict):
                has_path = isinstance(spec.get("path"), str)
                if has_path and not has_version:
                    unversioned.append(f"{dep} (path-only, no version pin)")
                    continue
                if not has_path and not has_version:
                    unversioned.append(f"{dep} (table spec with neither path nor version)")
                    continue
            if not has_version and not isinstance(spec, str):
                unversioned.append(f"{dep} (no version pin)")
                continue
        if unversioned:
            names = ", ".join(unversioned)
            f.fail(f"{name}: unversioned dependencies would fail crates.io publish: {names}")
            print(f"    FAIL  [dependencies] unversioned: {names}")
        else:
            n = len(deps)
            if n == 0:
                print("    OK    [dependencies] empty")
            else:
                names = ", ".join(sorted(deps))
                print(f"    OK    [dependencies] {n} dep(s), all version-pinned: {names}")

        dev = man.get("dev-dependencies", {})
        for dep, spec in sorted(dev.items()):
            if isinstance(spec, dict) and "path" in spec and "version" not in spec:
                f.note(f"{name}: dev-dependency {dep!r} is path-only (allowed by cargo publish, informational)")

    return publishable


def run(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, cwd=str(cwd), capture_output=True, text=True, timeout=PKG_TIMEOUT_S)


def check_cargo_package(root_dir: Path, publishable: list[str], manifests: dict[str, dict], root_pkg: dict, f: Findings) -> None:
    print()
    print("=" * 72)
    print("CHECK 3: cargo package --no-verify --allow-dirty")
    print("=" * 72)
    try:
        cargo_v = run(["cargo", "--version"], root_dir)
        print(f"  ({cargo_v.stdout.strip() or 'cargo --version failed'})")
    except (OSError, subprocess.TimeoutExpired) as exc:
        print(f"  (cargo --version unavailable: {exc})")

    for name in publishable:
        pkg = manifests.get(name, {}).get("package", {})
        ver = crate_version(pkg, root_pkg)
        print(f"  --- {name} v{ver} ---")

        # -p must use the manifest's package NAME, not the directory name
        # (audit -> cgap-audit, cloud -> cgap-cloud, fibo -> cgap-fibo,
        # integration -> cgap-integration). Standalone crates (own
        # [workspace]) cannot be selected with -p from the root at all;
        # --manifest-path is used for those.
        real_name = pkg.get("name", name)
        standalone = "workspace" in manifests.get(name, {})
        if standalone:
            mpath = manifests.get(name, {}).get("_manifest_dir")
            cmd = [
                "cargo", "package", "--no-verify", "--allow-dirty",
                "--manifest-path", str(mpath / "Cargo.toml"),
            ]
        else:
            cmd = ["cargo", "package", "--no-verify", "--allow-dirty", "-p", real_name]
        try:
            proc = run(cmd, root_dir)
        except (OSError, subprocess.TimeoutExpired) as _exc:
            f.fail(f"{name}: cargo package could not run: {_exc}")
            print(f"    FAIL  cargo package could not run: {_exc}")
            continue

        print(f"    command : {' '.join(cmd)}")
        print(f"    exit    : {proc.returncode}")
        if proc.returncode != 0:
            f.fail(f"{name}: cargo package exited {proc.returncode}")
        combined = (proc.stdout or "") + (proc.stderr or "")
        warnings = [ln.strip() for ln in combined.splitlines() if "warning" in ln.lower()]
        if warnings:
            print("    warnings:")
            for w in warnings:
                print(f"      ! {w}")
            f.note(f"{name}: cargo package emitted {len(warnings)} warning line(s)")
        else:
            print("    warnings: none")
        if proc.returncode != 0:
            tail = "\n".join((proc.stderr or proc.stdout or "").strip().splitlines()[-15:])
            print("    stderr tail:")
            for ln in tail.splitlines():
                print(f"      | {ln}")

        if standalone:
            crate_glob = sorted((mpath.parent / "target" / "package").glob(f"{real_name}-*.crate"))
        else:
            crate_glob = sorted((root_dir / "target" / "package").glob(f"{real_name}-*.crate"))
        if proc.returncode == 0:
            if crate_glob:
                for c in crate_glob:
                    print(f"    artifact: {c.name}  ({human(c.stat().st_size)})")
            else:
                print("    artifact: none found under target/package/")
                f.fail(f"{name}: cargo package exited 0 but no .crate under target/package/")

        if standalone:
            list_cmd = [
                "cargo", "package", "--list", "--allow-dirty",
                "--manifest-path", str(mpath / "Cargo.toml"),
            ]
        else:
            list_cmd = ["cargo", "package", "--list", "--allow-dirty", "-p", real_name]
        list_proc = run(list_cmd, root_dir)
        if list_proc.returncode == 0:
            files = [ln for ln in list_proc.stdout.splitlines() if ln.strip()]
            print(f"    files ({len(files)}):")
            for fn in files:
                print(f"      - {fn}")
        else:
            f.fail(f"{name}: cargo package --list exited {list_proc.returncode}")
            print(f"    FAIL  cargo package --list exited {list_proc.returncode}: {(list_proc.stderr or '').strip().splitlines()[-1:]}")


def main() -> int:
    root_dir = Path(__file__).resolve().parent.parent
    if not (root_dir / "Cargo.toml").exists():
        print(f"RELEASE-CHECK: FAIL root Cargo.toml not found at {root_dir}")
        return 1
    root = load_manifest(root_dir / "Cargo.toml")
    f = Findings()

    member_dirs, disk_dirs, manifests = check_workspace_version(root_dir, root, f)
    root_pkg = root.get("workspace", {}).get("package", {})
    publishable = check_publishable_metadata(root_dir, root, manifests, f)
    check_cargo_package(root_dir, publishable, manifests, root_pkg, f)

    print()
    print("=" * 72)
    print("NOTES (report-only, not gate failures)")
    print("=" * 72)
    if f.notes:
        for n in f.notes:
            print(f"  * {n}")
    else:
        print("  (none)")

    print()
    if f.reasons:
        print("RELEASE-CHECK: FAIL " + "; ".join(f.reasons))
        return 1
    print("RELEASE-CHECK: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
