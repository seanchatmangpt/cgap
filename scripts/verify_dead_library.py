#!/usr/bin/env python3
"""verify_dead_library.py — BR12 consume-then-census law as a gate (lane AE5).

Law: every `pub fn` in cgap-branchless must be consumed by a PRODUCTION
consumer (cgap-gate / cgap-parquet / cgap-p28 / cgap-ffi) — linked-binary
truth per the FA10 method, not rlib presence and not source grep. Unconsumed
symbol => exit 1 `DEAD_LIBRARY_SHOWROOM: <symbol> unconsumed in production`,
unless the symbol is a justified exception (scripts/dead_library_exceptions.json).

Why object census, not rlib name-presence: an rlib's lib.rmeta embeds the
full source path/doc strings; a name in rmeta is not a code reference. The
gate parses the rlib archive, keeps ONLY the rcgu object members, and counts
mangled-name tokens (Rust legacy mangling keeps identifiers verbatim:
`_ZN18cgap_branchless8penstock19penstock_prefilter8E...` contains the plain
identifier) inside those object bytes. A counted token in a consumer object
is a real symbol reference emitted by production codegen.

Must-consume set (regression to unconsumed = gate FAIL, not exception):
  penstock_prefilter8, swar_all_zero_vl
  (witnessed consumed via cgap-gate penstock, REP5 swap).

Divergence note (recorded in lane receipt
docs/jira/v26.11.2/countermeasures/lane-05-dead-library-gate.md): the lane
dispatch asserted radix_bucket_12 is consumed ("B5 precedent"); the
as-of-now census shows ZERO production refs outside cgap-branchless, so it
ships as a justified exception, not a must-consume.

Anti-vacuity (--self-check, mutations on /tmp copies only, shared tree
never written):
  MUT-A (stub mutant): enumerate a /tmp copy of cgap-branchless with
        `pub fn unconsumed_stub()` appended — gate must exit 1 naming it.
  MUT-B (negative control): cgap-parquet's rlib objects must yield ZERO
        branchless tokens (parquet does not depend on branchless). If the
        scanner matches there, it is over-matching rmeta-like strings =>
        VACUOUS => exit 1. Proves the census is not trivially all-pass.

Exit ABI: 0 pass / 1 gate fail / 2 subject missing / 3 loud skip.
stdlib only. No git commands. No Process.sleep (cargo subprocess is sync).
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
BRANCHLESS = REPO / "crates" / "cgap-branchless"
CONSUMERS = ["cgap-gate", "cgap-parquet", "cgap-p28", "cgap-ffi"]
# BR12 must-consume set: regression to unconsumed is a FAIL, never an exception.
MUST_CONSUME = {"penstock_prefilter8", "swar_all_zero_vl"}
EXCEPTIONS_FILE = REPO / "scripts" / "dead_library_exceptions.json"
DEFAULT_LEASE = "/tmp/ae5-deadlib-target"
FN_RE = re.compile(r"\bpub\s+(?:const\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)")

EXIT_OK, EXIT_FAIL, EXIT_MISSING, EXIT_SKIP = 0, 1, 2, 3


def fail(msg: str) -> int:
    print(f"DEAD_LIBRARY_FAIL: {msg}")
    return EXIT_FAIL


def enumerate_pub_fns(src_root: Path) -> list:
    """All pub fns declared in the branchless crate sources."""
    fns = set()
    for path in sorted(src_root.glob("src/*.rs")):
        fns |= set(FN_RE.findall(path.read_text()))
    return sorted(fns)


def build_consumers(target_dir: Path) -> int:
    """Release-build the consumer crates into the /tmp lease. Sync subprocess."""
    os.environ["CARGO_TARGET_DIR"] = str(target_dir)
    argv = ["cargo", "build", "--release", "--quiet"]
    for c in CONSUMERS:
        argv += ["-p", c]
    proc = subprocess.run(argv, cwd=REPO, capture_output=True, text=True)
    if proc.returncode != 0:
        sys.stderr.write(proc.stdout + proc.stderr)
    return proc.returncode


def consumer_object_blob(target_dir: Path, crate: str) -> bytes:
    """Concatenated rcgu object bytes of the crate's release rlib.

    rmeta members are excluded (they embed source strings and would
    false-positive the token census). Archive layout per rlib ar format.
    """
    hits = sorted(target_dir.glob(f"**/lib{crate.replace('-', '_')}*.rlib"))
    if not hits:
        return b""
    data = hits[0].read_bytes()
    if not data.startswith(b"!<arch>\n"):
        return b""
    blob = b""
    off = 8
    while off + 60 <= len(data):
        size = int(data[off + 48:off + 58].decode(errors="replace").strip() or 0)
        name = data[off:off + 16].split(b"\x00")[0].decode(errors="replace").strip()
        body = data[off + 60:off + 60 + size]
        if "rmeta" not in name:
            blob += body
        off += 60 + size + (size % 2)
    return blob


def census_tokens(blob: bytes, fns: list) -> dict:
    """Count `<len(name)><name>` mangled tokens per pub fn in object bytes."""
    return {f: blob.count(str(len(f)).encode() + f.encode()) for f in fns}


def load_exceptions() -> dict:
    if not EXCEPTIONS_FILE.exists():
        return {}
    return json.loads(EXCEPTIONS_FILE.read_text())


def run_gate(enumerate_root: Path | None = None, quiet: bool = False) -> int:
    """Core census. enumerate_root overrides the branchless source root
    (self-check mutant uses a /tmp sandbox copy)."""
    root = enumerate_root or BRANCHLESS
    if not (root / "Cargo.toml").exists():
        print(f"DEAD_LIBRARY_SUBJECT_MISSING: {root}")
        return EXIT_MISSING
    fns = enumerate_pub_fns(root)
    if not fns:
        print("DEAD_LIBRARY_LOUD_SKIP: zero pub fns enumerated")
        return EXIT_SKIP
    target_dir = Path(os.environ.get("DEADLIB_TARGET", DEFAULT_LEASE))
    if enumerate_root is None:
        if build_consumers(target_dir) != 0:
            print(f"DEAD_LIBRARY_SUBJECT_MISSING: consumer release build failed")
            return EXIT_MISSING
    seen = {}
    scanned = []
    for crate in CONSUMERS:
        blob = consumer_object_blob(target_dir, crate)
        if not blob:
            continue
        scanned.append(crate)
        for fn, n in census_tokens(blob, fns).items():
            seen[fn] = seen.get(fn, 0) + n
    if not scanned:
        print("DEAD_LIBRARY_LOUD_SKIP: no consumer rlibs found under "
              f"{target_dir}; nothing scanned")
        return EXIT_SKIP
    unconsumed = [f for f in fns if seen.get(f, 0) == 0]
    exceptions = load_exceptions()
    exc_symbols = {k for k in exceptions if not k.startswith("_")}
    stale = sorted(exc_symbols - set(fns))
    showrooms = sorted(f for f in unconsumed if f not in exc_symbols)
    must_miss = sorted(f for f in MUST_CONSUME if seen.get(f, 0) == 0)
    missing_artifacts = [c for c in CONSUMERS if c not in scanned]
    if not quiet:
        print(f"census subject : cgap-branchless ({len(fns)} pub fns)")
        print(f"consumers      : {', '.join(scanned)}"
              + (f" (missing: {', '.join(missing_artifacts)})" if missing_artifacts else ""))
        for f in fns:
            status = "CONSUMED" if seen.get(f, 0) else (
                "EXCEPTION-JUSTIFIED" if f in exc_symbols else "UNCONSUMED")
            print(f"  {f:32s} refs={seen.get(f, 0)}  {status}")
    if stale:
        return fail(f"stale exceptions entries name no enumerated symbol: {stale}")
    if showrooms:
        for f in showrooms:
            print(f"DEAD_LIBRARY_SHOWROOM: {f} unconsumed in production")
        return EXIT_FAIL
    if must_miss:
        return fail(f"must-consume regression (was consumed via gate penstock, "
                    f"now zero production refs): {must_miss}")
    if missing_artifacts and not quiet:
        print(f"note: consumer rlibs absent for {missing_artifacts} "
              "(no dependency path to cgap-branchless; census covers "
              "the crates that link it)")
    if not quiet:
        print("PASS: consume-then-census law satisfied "
              f"({len(fns) - len(unconsumed)} consumed, "
              f"{len(unconsumed)} justified exceptions, "
              f"must-consume set intact: {sorted(MUST_CONSUME)})")
    return EXIT_OK


def self_check() -> int:
    """Anti-vacuity court. Mutations live ONLY in /tmp; shared tree untouched."""
    import shutil
    import tempfile
    rc = 0
    # MUT-A: stub mutant in a /tmp sandbox copy of cgap-branchless.
    tmp = Path(tempfile.mkdtemp(prefix="ae5-deadlib-mut-"))
    sandbox = tmp / "cgap-branchless"
    shutil.copytree(BRANCHLESS, sandbox)
    lib = sandbox / "src" / "lib.rs"
    lib.write_text(lib.read_text() + "\n#[no_mangle]\n"
                   "pub fn unconsumed_stub() -> u64 { 0 }\n")
    import io, contextlib
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        rc_a = run_gate(enumerate_root=sandbox, quiet=True)
    out = buf.getvalue()
    print("[self-check MUT-A] stub mutant (expect exit 1 naming unconsumed_stub)")
    if rc_a != EXIT_FAIL or "unconsumed_stub" not in out:
        print(f"[self-check MUT-A] VACUOUS GATE: stub mutant did not fire "
              f"(exit={rc_a}, named={'unconsumed_stub' in out})")
        rc = EXIT_FAIL
    else:
        print("[self-check MUT-A] FIRED: gate names unconsumed_stub, exit 1")
    # MUT-B: negative control — parquet objects must census zero branchless
    # tokens; scanner matching there = over-match = vacuous.
    target_dir = Path(os.environ.get("DEADLIB_TARGET", DEFAULT_LEASE))
    pblob = consumer_object_blob(target_dir, "cgap-parquet")
    if not pblob:
        print("[self-check MUT-B] [SKIP] parquet rlib absent — cannot prove "
              "non-vacuity (loud skip, gate FAIL)")
        rc = EXIT_FAIL
    else:
        tok = census_tokens(pblob, enumerate_pub_fns(BRANCHLESS))
        if sum(tok.values()) != 0:
            print(f"[self-check MUT-B] VACUOUS GATE: scanner matched branchless "
                  f"tokens in cgap-parquet objects (no dependency exists): {tok}")
            rc = EXIT_FAIL
        else:
            print("[self-check MUT-B] FIRED-proof: parquet objects census 0 "
                  "branchless tokens — scanner is selective, not all-pass")
    shutil.rmtree(tmp, ignore_errors=True)
    return rc


def main() -> int:
    args = sys.argv[1:]
    if "--self-check" in args:
        return self_check()
    if "--help" in args or "-h" in args:
        print(__doc__)
        return EXIT_OK
    return run_gate()


if __name__ == "__main__":
    sys.exit(main())
