#!/usr/bin/env python3
"""ffi_surface_gate.py — cdylib export surface vs. C-header prototype 1:1 court.

Closes the RED-03 header gap: every exported `cgap_*` text symbol in the
built cdylib MUST have exactly one prototype in include/cgap.h, and every
prototype must be exported. Fail-closed law: exit 0 pass / 1 gate fail /
2 subject missing / 3 loud skip.

Usage:
  python3 scripts/ffi_surface_gate.py [--dylib PATH] [--header PATH]
  python3 scripts/ffi_surface_gate.py --self-check

Defaults: dylib /Users/sac/cgap/target/release/libcgap_ffi.dylib,
header /Users/sac/cgap/include/cgap.h. --build first builds a FRESH
release dylib into a /tmp lease target dir (cargo --manifest-path
crates/cgap-ffi/Cargo.toml) and audits that, never a stale artifact.
"""
import hashlib
import os
import re
import shutil
import subprocess
import sys

REPO = "/Users/sac/cgap"
DEFAULT_DYLIB = os.path.join(REPO, "target/release/libcgap_ffi.dylib")
DEFAULT_HEADER = os.path.join(REPO, "include/cgap.h")

PASS, FAIL, MISSING, SKIP = 0, 1, 2, 3


def fail(msg):
    print("GATE-FAIL: " + msg)
    return FAIL


def header_prototypes(header_path):
    """cgap_* names declared as functions in the header.

    A name counts iff immediately followed by '(' — struct typedefs
    (cgap_frame_view), enum constants, and offsetof(cgap_frame_view, ...)
    never carry a cgap_ name + '(' pair, so the extraction is not fooled.
    """
    with open(header_path, "r", encoding="utf-8") as fh:
        text = fh.read()
    # Strip C comments first: prose like "cgap_version() remains" must not
    # count as a prototype (witnessed: this exact comment masked Mutation 2).
    text = re.sub(r"/\*.*?\*/", " ", text, flags=re.S)
    text = re.sub(r"//[^\n]*", " ", text)
    hits = re.findall(r"\b(cgap_[A-Za-z0-9_]*\s*)\(", text)
    return {h.strip() for h in hits}


def dylib_symbols(dylib_path):
    out = subprocess.run(
        ["nm", "-gU", dylib_path],
        capture_output=True, text=True, check=False,
    )
    if out.returncode != 0:
        raise RuntimeError("nm failed on %s: %s" % (dylib_path, out.stderr))
    syms = set()
    for line in out.stdout.splitlines():
        if " T " not in line:
            continue
        name = line.rsplit(" ", 1)[-1]
        if name.startswith("_"):
            name = name[1:]  # macOS C-ABI underscore
        if name.startswith("cgap_"):
            syms.add(name)
    return syms


def sha256(path):
    with open(path, "rb") as fh:
        return hashlib.sha256(fh.read()).hexdigest()


def run_gate(dylib, header):
    """Returns gate exit code comparing one dylib against one header."""
    if not os.path.isfile(dylib):
        print("[GATE] subject missing: no dylib at %s" % dylib)
        return MISSING
    if not os.path.isfile(header):
        print("[GATE] subject missing: no header at %s" % header)
        return MISSING
    try:
        syms = dylib_symbols(dylib)
    except RuntimeError as exc:
        print("[GATE] loud skip: %s" % exc)
        return SKIP
    if not syms:
        print("[GATE] loud skip: zero cgap_* exported text symbols in %s" % dylib)
        return SKIP
    protos = header_prototypes(header)
    missing = sorted(syms - protos)
    extra = sorted(protos - syms)
    if missing or extra:
        for name in missing:
            print("GATE-FAIL: exported but no C prototype: %s" % name)
        for name in extra:
            print("GATE-FAIL: prototyped but not exported: %s" % name)
        return FAIL
    print("[GATE] PASS: %d exported cgap_* symbols == %d header prototypes (1:1)"
          % (len(syms), len(protos)))
    return PASS


SBX = "/tmp/fix07-sandbox"


def sandbox_copy(src, name):
    os.makedirs(SBX, exist_ok=True)
    dst = os.path.join(SBX, name)
    shutil.copyfile(src, dst)
    return dst


def expect(code, want, label):
    ok = "ok" if code == want else "MISMATCH"
    print("[SELF-CHECK] %s -> exit %d (want %d) %s" % (label, code, want, ok))
    return code == want


def self_check():
    """Non-vacuity battery. Every mutation happens on a /tmp sandbox copy;
    restores are verified sha256-exact and any tampering is detected."""
    os.makedirs(SBX, exist_ok=True)
    results = []

    dylib, rc = build_fresh_dylib()
    if dylib is None:
        return rc
    baseline = run_gate(dylib, DEFAULT_HEADER)
    results.append(expect(baseline, PASS, "baseline (unmutated subject)"))

    # Mutation 1: inject a fake prototype (extra in header, absent in dylib).
    h1 = sandbox_copy(DEFAULT_HEADER, "fake_proto.h")
    with open(h1, "a") as fh:
        fh.write("CgapStatus cgap_fake_probe(void);\n")
    results.append(expect(run_gate(dylib, h1), FAIL,
                          "mutation: fake prototype injected"))
    shutil.copyfile(DEFAULT_HEADER, h1)  # the actual restore
    print("[SHIM-COURT] restore fake_proto.h sha256 %s vs %s"
          % (sha256(h1), sha256(DEFAULT_HEADER)))
    results.append(expect(
        PASS if sha256(h1) == sha256(DEFAULT_HEADER) else FAIL,
        PASS, "restore verify (fake_proto.h byte-exact)"))

    # Mutation 2: drop one prototype (cgap_version) from a sandbox header.
    h2 = sandbox_copy(DEFAULT_HEADER, "drop_proto.h")
    lines = [ln for ln in open(h2) if not ln.startswith("const char* cgap_version")]
    with open(h2, "w") as fh:
        fh.writelines(lines)
    results.append(expect(run_gate(dylib, h2), FAIL,
                          "mutation: cgap_version prototype dropped"))
    shutil.copyfile(DEFAULT_HEADER, h2)  # the actual restore
    print("[SHIM-COURT] restore drop_proto.h sha256 %s vs %s"
          % (sha256(h2), sha256(DEFAULT_HEADER)))
    results.append(expect(
        PASS if sha256(h2) == sha256(DEFAULT_HEADER) else FAIL,
        PASS, "restore verify (drop_proto.h byte-exact)"))

    # Mutation 3: foreign shim dylib in /tmp (missing the sovereign symbols).
    shim_src = os.path.join(SBX, "shim.c")
    with open(shim_src, "w") as fh:
        fh.write("int cgap_version(void){return 0;}\n")
    shim_dylib = os.path.join(SBX, "libshim.dylib")
    cc = subprocess.run(["clang", "-dynamiclib", "-o", shim_dylib, shim_src],
                        capture_output=True, text=True, check=False)
    if cc.returncode != 0:
        print("[SKIP] cannot build shim dylib: %s" % cc.stderr.strip())
        results.append(False)
    else:
        results.append(expect(run_gate(shim_dylib, DEFAULT_HEADER), FAIL,
                              "mutation: hostile 1-symbol shim dylib"))

    # Restore-detection (anti-vacuity of the restore itself): restore h1 with
    # CORRUPTED bytes — the sha256 check must catch it, not wave it through.
    with open(h1, "r") as fh:
        corrupt = fh.read() + "\n/* corrupted restore */\n"
    with open(h1, "w") as fh:
        fh.write(corrupt)
    detected = sha256(h1) != sha256(DEFAULT_HEADER)
    print("[SHIM-COURT] corrupted restore detected: %s" % detected)
    results.append(expect(PASS if detected else FAIL, PASS,
                          "corrupted restore is detected (fail-closed)"))
    # Clean-room re-restore, byte-exact.
    shutil.copyfile(DEFAULT_HEADER, h1)
    results.append(expect(
        PASS if sha256(h1) == sha256(DEFAULT_HEADER) else FAIL,
        PASS, "final re-restore byte-exact"))
    return PASS if all(results) else FAIL


def build_fresh_dylib():
    """Build a fresh release cdylib into a /tmp lease target (XC-01).

    The truth source for the census is the dylib built THIS session, not a
    possibly-stale artifact under repo target/. Lease law: the target dir
    is /tmp/xc01-target and is deleted by the caller at lane end.
    """
    target = "/tmp/xc01-target"
    out = subprocess.run(
        ["cargo", "build", "--release",
         "--manifest-path", os.path.join(REPO, "crates/cgap-ffi/Cargo.toml")],
        capture_output=True, text=True, check=False,
        env=dict(os.environ, CARGO_TARGET_DIR=target),
    )
    if out.returncode != 0:
        print("[GATE] loud skip: fresh build failed rc=%d: %s"
              % (out.returncode, out.stderr[-2000:]))
        return None, SKIP
    dylib = os.path.join(target, "release/libcgap_ffi.dylib")
    if not os.path.isfile(dylib):
        print("[GATE] loud skip: build ok but no dylib at %s" % dylib)
        return None, SKIP
    print("[GATE] fresh dylib built: %s" % dylib)
    return dylib, PASS


def main():
    args = sys.argv[1:]
    if "--self-check" in args:
        return self_check()
    dylib = None
    header = DEFAULT_HEADER
    if "--dylib" in args:
        dylib = args[args.index("--dylib") + 1]
    else:
        # Truth-based default: audit the dylib built THIS session, never a
        # possibly-stale repo-target artifact (XC-01).
        dylib, rc = build_fresh_dylib()
        if dylib is None:
            return rc
    if "--header" in args:
        header = args[args.index("--header") + 1]
    return run_gate(dylib, header)


if __name__ == "__main__":
    sys.exit(main())
