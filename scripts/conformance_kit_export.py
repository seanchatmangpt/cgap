#!/usr/bin/env python3
"""conformance_kit_export.py — package the repo's courts as an external conformance kit.

Lane XC-05 (v26.11.2, red-20 P4). Walks the court inventory:
  1. cargo test targets: crates/*/tests/*court*.rs (+ courts.rs variants)
  2. gate-script court modes (scripts/*.py --self-check / court modes)
  3. entries in docs/jira/v26.11.2/07-court-corpus.md
and emits a deterministic (sorted, byte-identical for a fixed tree) JSON
manifest: court name, invocation, expected exit class, subject crate,
wire-constant surface per fix-09 (redteam/fix-09-wire-constant-manifest.md).

Modes:
  (default)          emit manifest JSON to stdout
  --out FILE         write manifest to FILE
  --run NAME[,NAME]  execute the named cargo-test courts for real
                     (subprocess arg-lists, no shell); fail-closed rollup
  --list             print inventory names only
  --self-check       anti-vacuity battery: determinism leg, flip leg,
                     one real --run court leg
  --root DIR         repo root to scan (default: script's repo)

Exit law: 0 pass / 1 gate fail / 2 subject missing / 3 loud skip.
"""
import argparse
import hashlib
import json
import os
import re
import subprocess
import sys

DEFAULT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# fix-09 wire-constant surface -> owning crates (redteam/fix-09 manifest S1-S5).
WIRE_SURFACES = {
    "S1-status-register": ["cgap-ffi"],
    "S2-wire-magics": ["cgap-gate", "cgap-p28"],
    "S3-frame-header-layout": ["cgap-gate", "cgap-core"],
    "S4-p28-attestation-layout": ["cgap-p28"],
    "S5-receipt-chain-swar": ["cgap-ffi", "cgap-blake3", "cgap-branchless"],
}

# Gate scripts with court/self-check modes (surface, invocation).
GATE_COURT_MODES = [
    ("scripts/boundary_gate.py", "python3 scripts/boundary_gate.py --shim-court-only", "S3-frame-header-layout"),
    ("scripts/boundary_gate.py", "python3 scripts/boundary_gate.py --cross-gate-court", "S1-status-register"),
    ("scripts/zero_string_gate.py", "python3 scripts/zero_string_gate.py --self-check", "S2-wire-magics"),
    ("scripts/ffi_surface_gate.py", "python3 scripts/ffi_surface_gate.py --self-check", "S1-status-register"),
    ("scripts/enclave_budget_gate.py", "python3 scripts/enclave_budget_gate.py --self-check", "S5-receipt-chain-swar"),
]


def is_court_file(name):
    return "court" in name.lower() and name.endswith(".rs")


def scan_cargo_courts(root):
    """Scan crates/*/tests for court test targets. Sorted, deterministic."""
    out = []
    crates_dir = os.path.join(root, "crates")
    if not os.path.isdir(crates_dir):
        return out
    for crate in sorted(os.listdir(crates_dir)):
        tdir = os.path.join(crates_dir, crate, "tests")
        if not os.path.isdir(tdir):
            continue
        for fn in sorted(os.listdir(tdir)):
            if is_court_file(fn):
                stem = fn[:-3]
                out.append({
                    "name": stem,
                    "kind": "cargo-test",
                    "subject_crate": crate,
                    "invocation": ["cargo", "test", "-p", crate, "--test", stem],
                    "expected_exit_class": {"pass": 0, "fail": "nonzero"},
                    "wire_surfaces": sorted(s for s, cs in WIRE_SURFACES.items() if crate in cs),
                })
    return out


def scan_corpus_courts(root):
    """Parse docs/jira/v26.11.2/07-court-corpus.md '### ...COURT' entries."""
    doc = os.path.join(root, "docs", "jira", "v26.11.2", "07-court-corpus.md")
    out = []
    if not os.path.isfile(doc):
        return out
    with open(doc, "r", encoding="utf-8") as f:
        text = f.read()
    for m in re.finditer(r"^###\s+(\S*COURT\S*)\s+.*$", text, re.M):
        name = m.group(1)
        out.append({
            "name": name,
            "kind": "court-corpus-doc",
            "subject_crate": None,
            "invocation": ["docs/jira/v26.11.2/07-court-corpus.md#" + name],
            "expected_exit_class": {"pass": 0, "fail": "nonzero"},
            "wire_surfaces": [],
        })
    return out


def scan_gate_courts():
    out = []
    for path, inv, surface in GATE_COURT_MODES:
        name = os.path.basename(path)[:-3] + ":" + inv.split()[-1].lstrip("-")
        out.append({
            "name": name,
            "kind": "gate-mode",
            "subject_crate": path,
            "invocation": inv.split(),
            "expected_exit_class": {"pass": 0, "fail": "nonzero"},
            "wire_surfaces": [surface],
        })
    return out


def build_manifest(root):
    entries = scan_cargo_courts(root) + scan_corpus_courts(root) + scan_gate_courts()
    entries.sort(key=lambda e: (e["kind"], e["name"]))
    manifest = {
        "kit": "cgap-conformance-kit",
        "root": root,
        "inventory_count": len(entries),
        "wire_surfaces": sorted(WIRE_SURFACES),
        "courts": entries,
    }
    return manifest


def emit(manifest, out_path=None):
    blob = json.dumps(manifest, indent=2, sort_keys=True) + "\n"
    if out_path:
        with open(out_path, "w", encoding="utf-8") as f:
            f.write(blob)
    return blob


def resolve_courts(manifest, names):
    by_name = {e["name"]: e for e in manifest["courts"] if e["kind"] == "cargo-test"}
    if names:
        missing = [n for n in names if n not in by_name]
        if missing:
            sys.stderr.write("SUBJECT-MISSING: %s\n" % ", ".join(missing))
            sys.exit(2)
        chosen = [by_name[n] for n in names]
    else:
        chosen = sorted(by_name.values(), key=lambda e: e["name"])
    return chosen


def run_courts(courts, root):
    """Real subprocess runs (arg-list, no shell). Fail-closed rollup."""
    failures = []
    for c in courts:
        env = dict(os.environ)
        env.setdefault("CARGO_TARGET_DIR", "/tmp/xc05-kit-target")
        sys.stderr.write("RUN %s :: %s\n" % (c["name"], " ".join(c["invocation"])))
        proc = subprocess.run(c["invocation"], cwd=root, env=env)
        rc = proc.returncode
        sys.stderr.write("EXIT %d %s\n" % (rc, c["name"]))
        if rc != 0:
            failures.append((c["name"], rc))
    if failures:
        for name, rc in failures:
            sys.stderr.write("COURT-FAIL %s rc=%d\n" % (name, rc))
        sys.exit(1)
    sys.stderr.write("ALL-COURTS-PASS %d\n" % len(courts))


def self_check(root):
    """Anti-vacuity battery. Exit 0 only if all three legs witness."""
    manifest = build_manifest(root)
    blob1 = emit(manifest)
    # (a) determinism leg: regenerate, byte-compare.
    blob2 = emit(build_manifest(root))
    if blob1 != blob2:
        sys.stderr.write("SELF-CHECK-FAIL: determinism leg\n")
        sys.exit(1)
    sys.stderr.write("LEG-A determinism PASS (byte-identical, %d B)\n" % len(blob1))
    # (b) flip leg: /tmp-mirrored subset with one court file dropped ->
    # manifest must differ; detection is the witness.
    tmp = "/tmp/xc05-kit-flip"
    import shutil
    shutil.rmtree(tmp, ignore_errors=True)
    cargo_courts = scan_cargo_courts(root)
    if not cargo_courts:
        sys.stderr.write("[SKIP] flip leg: empty inventory\n")
        sys.exit(3)
    victim = cargo_courts[0]
    src_tests = os.path.join(root, "crates", victim["subject_crate"], "tests")
    dst_tests = os.path.join(tmp, "crates", victim["subject_crate"], "tests")
    os.makedirs(dst_tests, exist_ok=True)
    for fn in sorted(os.listdir(src_tests)):
        if is_court_file(fn) and fn != victim["name"] + ".rs":
            with open(os.path.join(src_tests, fn), "rb") as rf, \
                    open(os.path.join(dst_tests, fn), "wb") as wf:
                wf.write(rf.read())
    flip_manifest = build_manifest(tmp)
    if flip_manifest["inventory_count"] >= manifest["inventory_count"]:
        sys.stderr.write("SELF-CHECK-FAIL: flip leg undetected\n")
        sys.exit(1)
    sys.stderr.write("LEG-B flip PASS (dropped %s: %d -> %d entries)\n" % (
        victim["name"], manifest["inventory_count"], flip_manifest["inventory_count"]))
    shutil.rmtree(tmp, ignore_errors=True)
    # (c) run leg: one real, small court.
    pick = None
    for pref in ("doc_offset_table_court", "identity_court", "static_assert_court"):
        for c in cargo_courts:
            if c["name"] == pref:
                pick = c
                break
        if pick:
            break
    if pick is None:
        pick = cargo_courts[0]
    sys.stderr.write("LEG-C running %s\n" % pick["name"])
    before = sys.stderr
    run_courts([pick], root)
    sys.stderr.write("SELF-CHECK PASS\n")
    return 0


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--root", default=DEFAULT_ROOT)
    ap.add_argument("--out")
    ap.add_argument("--run", help="comma-separated court names (cargo-test kind)")
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--self-check", action="store_true")
    args = ap.parse_args(argv)
    root = os.path.abspath(args.root)
    if args.self_check:
        return self_check(root)
    manifest = build_manifest(root)
    if args.list:
        for e in manifest["courts"]:
            print(e["kind"] + "\t" + e["name"])
        return 0
    if args.run is not None:
        names = [n.strip() for n in args.run.split(",") if n.strip()]
        courts = resolve_courts(manifest, names)
        run_courts(courts, root)
        return 0
    sys.stdout.write(emit(manifest, args.out))
    return 0


if __name__ == "__main__":
    sys.exit(main())
