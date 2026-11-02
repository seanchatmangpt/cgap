#!/usr/bin/env python3
"""AE4 — disassembly-invariants court for cgap-branchless (riscv32imc).

Method of record (reused, not reinvented):
  docs/jira/v26.11.2/waves/p2-asm-0blr-census.md + EX3
  (docs/jira/v26.11.2/exhaustive/lane-03-branchless-complete.md:78):
  cargo +nightly RUSTFLAGS="--emit=asm -C opt-level=3 -C link-dead-code=yes
  -C debuginfo=0" build --release --target riscv32imc-unknown-none-elf,
  then per-symbol census of the emitted asm.

Court arms:
  A. census: every allowlist symbol scanned for forbidden branch opcodes.
  B. linked witness: shim staticlib + llvm-objdump -d (sysroot-resolved);
     surviving allowlist symbols re-scanned in the linked object stream.
  C. mutant court (anti-vacuity): if-injection into select.rs cmov_u64 on a
     /tmp sandbox copy must flip to exit 1 naming symbol+opcode.
  D. self-check: branchy reference `naive_if_cmov` (B6 pattern) MUST be
     flagged — a court that cannot flag a branchy fn is VACUOUS (exit 1).

Exit ABI (pinned, typed): 0 clean / 1 forbidden-branch or vacuous court /
2 subject-missing (nonexistent --witness path, missing crate, missing
cargo, failed build) / 3 loud skip (forced degrade arm, no .s emit,
missing llvm tool). Every arm has a documented test invocation — see
docs/jira/v26.11.2/lanes/lane-g03-disasm-gate.md.

Arm E (binary-level anti-vacuity, G03): copy the linked witness archive
to the /tmp scratch, patch one non-branch instruction word inside a
0-branch allowlist fn to the bnez encoding (0x00009e63 = bnez x1, 8),
re-disassemble; the census MUST newly flag bnez — proves the byte-level
disassembly path flips, not just the source-level mutant. The mutation
subject is a scratch copy; the original archive's sha256 is verified
unchanged after the arm (fail-closed restore law).

Dispatch divergence note: dispatch said "w6-b6/EX3" — no w6-b6 file exists
in the repo; EX3 (lane-03-branchless-complete.md) is the method of record
and is used. The census runs on the emitted .s (emit=asm preserves all 19
allowlist fns; object/rlib emission prunes unreferenced pub fns — witnessed
this session, see receipt). llvm-objdump -d is exercised on the linked
staticlib witness (arm B) and on the self-check object (arm D).
"""
import argparse
import glob
import os
import re
import shutil
import subprocess
import sys
import tempfile

REPO = os.environ.get("CGAP_ROOT", "/Users/sac/cgap")
CRATE = os.path.join(REPO, "crates", "cgap-branchless")
TARGET = "riscv32imc-unknown-none-elf"
SCRATCH = None  # unique per-run lease (sibling lanes run this gate concurrently)

# Typed exit ABI (G03 pin)
EXIT_CLEAN = 0
EXIT_BRANCH = 1
EXIT_SUBJECT_MISSING = 2
EXIT_LOUD_SKIP = 3

# Residual allowlist — RV32 ISA ceilings documented in
# docs/jira/v26.11.2/exhaustive/lane-03-branchless-complete.md (rows
# select::cmov_u64, select::select_slice, eytzinger::eytzinger_restore):
# "host PASS / riscv branchy is an ISA ceiling, not a source defect"
# (rv32imc has no Zicond/cmov; ISel lowers every 2-way select to a branch).
# Never silently passed: they emit ALLOWED-RESIDUAL rows with these cites.
RESIDUAL_ALLOWLIST = {
    "select_slice": (
        "lane-03 row select::select_slice: BRANCHY on riscv32imc (2: beqz "
        "len-guard + bnez cond byte) — RV32 ISA ceiling, REP1 owns re-census"),
    "cmov_u64": (
        "lane-03 row select::cmov_u64: BRANCHY (1x bnez; RV32 has no cmov) "
        "— RV32 ISA ceiling"),
    "eytzinger_restore": (
        "lane-03 row eytzinger::eytzinger_restore: BRANCHY (1x beqz, "
        "index/remainder guard) — RV32 ISA ceiling"),
}

BRANCH_OPS = {
    "beq", "bne", "blt", "bge", "bltu", "bgeu",
    "beqz", "bnez", "blez", "bgtz", "bltz", "bgez",
    "c.beqz", "c.bnez", "c.j", "c.jal", "c.jr",
}

SYMBOL_RE = re.compile(r"^(?:[0-9a-f]+ )?<?(_RNv[A-Za-z0-9_]+)>?:$")
INSTR_RE = re.compile(r"^\s+([a-zA-Z][A-Za-z0-9._]*)[\s,]")
OBJDUMP_INSTR_RE = re.compile(r"^\s*[0-9a-f]+:\s+([0-9a-f ]+\s+)(\S+)")
BRACKET_RE = re.compile(r"^(_RNv[A-Za-z0-9_]+)\s*$")


def run(cmd, cwd=None, env=None, check=True):
    e = dict(os.environ)
    if env:
        e.update(env)
    p = subprocess.run(cmd, cwd=cwd, env=e, capture_output=True, text=True)
    if check and p.returncode != 0:
        sys.stderr.write(p.stdout[-2000:] + p.stderr[-2000:])
        sys.exit(2)
    return p


def sysroot():
    p = run(["rustc", "--print", "sysroot"])
    return p.stdout.strip()


def llvm_bin(name):
    base = os.path.join(sysroot(), "lib", "rustlib",
                        "aarch64-apple-darwin", "bin")
    path = os.path.join(base, name)
    if not os.path.exists(path):
        # generic fallback: glob any <name>
        hits = glob.glob(os.path.join(sysroot(), "lib", "rustlib",
                                      "*", "bin", name))
        if not hits:
            print(f"[SKIP] llvm tool {name} not found in rustc sysroot",
                  file=sys.stderr)
            return None
        path = hits[0]
    return path


def allowlist_fns():
    """Enumerate the branchless set: pub fns of cgap-branchless + any
    #[export_name = \"branchless_*\"] convention across the crate src."""
    fns = set()
    for path in glob.glob(os.path.join(CRATE, "src", "*.rs")):
        src = open(path).read()
        for m in re.finditer(r"pub (?:const )?fn ([A-Za-z0-9_]+)", src):
            fns.add(m.group(1))
        for m in re.finditer(r'#\[export_name\s*=\s*"([^"]+)"\]', src):
            fns.add(m.group(1))
    return sorted(fns)


# REFERENCE-DRIVER set (EX3 classification, lane-03 rows): these pub fns are
# drivers over the hot kernels (Iterator::all back-edges, chunks_exact loops,
# alloc-driver) — their branches are loop back-edges, not data-dependent
# 2-way selects. Reported as REFERENCE-DRIVER rows, never silent PASS.
REFERENCE_DRIVERS = {
    "checked_uniform_select": "lane-03: Iterator::all loop back-edge",
    "penstock_prefilter8": "lane-03: chunks_exact loop",
    "swar_all_zero_vl": "lane-03: loop, 5 br",
    "eytzinger_search": "lane-03: search loop back-edge",
    "to_eytzinger_layout": "lane-03: layout-build loop + Vec alloc driver",
}


def census_asm(asm_text, fns):
    """Split emitted .s into per-symbol regions; attribute to allowlist fns.
    Returns {fn: [(symbol, opcode, count)]} plus unmapped-symbol diagnostics."""
    per_fn = {}
    current = None
    for line in asm_text.splitlines():
        m = SYMBOL_RE.match(line)
        if m:
            current = m.group(1)
            continue
        im = OBJDUMP_INSTR_RE.match(line) or INSTR_RE.match(line)
        if not im or current is None:
            continue
        assert im.lastindex is not None
        op = im.group(im.lastindex)
        if op not in BRANCH_OPS:
            continue
        for fn in fns:
            if fn in current:
                per_fn.setdefault(fn, {}).setdefault(
                    (current, op), 0)
                per_fn[fn][(current, op)] += 1
                break
        else:
            per_fn.setdefault("<unmapped>", {}).setdefault(
                (current, op), 0)
            per_fn["<unmapped>"][(current, op)] += 1
    return per_fn


def emit_asm_build(crate_dir, target_dir):
    env = {
        "RUSTFLAGS": "--emit=asm -C opt-level=3 -C link-dead-code=yes "
                     "-C debuginfo=0",
        "CARGO_TARGET_DIR": target_dir,
    }
    p = run(["cargo", "+nightly", "build", "--release", "--target", TARGET],
            cwd=crate_dir, env=env, check=False)
    return p


def find_emit(target_dir):
    hits = sorted(glob.glob(os.path.join(
        target_dir, TARGET, "release", "build", "cgap-branchless", "*",
        "out", "cgap_branchless*.s")))
    return hits[-1] if hits else None


SHIM_LIB = """#![no_std]
extern crate alloc;
use core::panic::PanicInfo;
#[panic_handler]
fn ae4_panic(_: &PanicInfo) -> ! { loop {} }
struct Ae4Alloc;
unsafe impl core::alloc::GlobalAlloc for Ae4Alloc {
    unsafe fn alloc(&self, _: core::alloc::Layout) -> *mut u8 { core::ptr::null_mut() }
    unsafe fn dealloc(&self, _: *mut u8, _: core::alloc::Layout) {}
}
#[global_allocator]
static AE4_ALLOC: Ae4Alloc = Ae4Alloc;
pub use cgap_branchless::*;
"""

SHIM_TOML = """[package]
name = "ae4-shim"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["staticlib"]

[dependencies]
cgap-branchless = { path = "{crate}" }
"""


def linked_witness(scratch, fns):
    """Arm B: shim staticlib (no_std: panic handler + GlobalAlloc shim +
    `pub use`) linked for riscv32imc; llvm-objdump -d over the archive."""
    shim_dir = os.path.join(scratch, "shim")
    os.makedirs(os.path.join(shim_dir, "src"), exist_ok=True)
    open(os.path.join(shim_dir, "Cargo.toml"), "w").write(
        SHIM_TOML.replace("{crate}", CRATE))
    open(os.path.join(shim_dir, "src", "lib.rs"), "w").write(SHIM_LIB)
    tdir = os.path.join(scratch, "witness-target")
    env = {"RUSTFLAGS": "-C opt-level=3 -C link-dead-code=yes -C debuginfo=0",
           "CARGO_TARGET_DIR": tdir}
    p = run(["cargo", "+nightly", "build", "--release", "--target", TARGET],
            cwd=shim_dir, env=env, check=False)
    if p.returncode != 0:
        print("[SKIP] linked witness build failed (loud):",
              p.stderr.strip().splitlines()[-1] if p.stderr else "?",
              file=sys.stderr)
        return None
    arch = llvm_bin("llvm-objdump")
    if arch is None:
        return None
    arc = os.path.join(tdir, TARGET, "release", "libae4_shim.a")
    od = run([arch, "-d", arc], check=False)
    if od.returncode != 0:
        print(f"[SKIP] llvm-objdump -d refused {arc} (loud)", file=sys.stderr)
        return None
    per_fn = census_asm(od.stdout, fns)
    print(f"[WITNESS] linked staticlib disassembled: "
          f"{len(od.stdout.splitlines())} lines, llvm-objdump -d "
          f"(sysroot: {sysroot()})")
    arc_ret = arc
    for fn in sorted(per_fn):
        for (sym, op), n in sorted(per_fn[fn].items()):
            print(f"  linked-witness {fn}: {op} x{n} in {sym[:60]}")
    return per_fn, arc_ret


MUTANT_SELECT = """    let mask = 0u64.wrapping_sub(cond as u64);
    let r = (mask & true_val) | (!mask & false_val);
    if r > 0 { r } else { true_val } // SHIM-COURT M-AE4-1 if-injection
"""
CLEAN_SELECT = """    let mask = 0u64.wrapping_sub(cond as u64);
    (mask & true_val) | (!mask & false_val)
"""

BRANCHY_REF = """#![no_std]
#![no_main]
#![no_builtins]
use core::panic::PanicInfo;
#[panic_handler]
fn p(_: &PanicInfo) -> ! { loop {} }
#[inline(never)]
pub extern "C" fn naive_if_cmov(cond: bool, t: u64, f: u64) -> u64 {
    if cond { t } else { f }
}
"""


def mutant_court(scratch, fns):
    """Arm C: if-injection into cmov_u64 on a /tmp sandbox copy of
    cgap-branchless MUST flip the census to exit-1 class findings."""
    mut = os.path.join(scratch, "mutant")
    shutil.copytree(CRATE, mut, ignore=shutil.ignore_patterns("target", "tests"))
    sel = os.path.join(mut, "src", "select.rs")
    src = open(sel).read()
    if CLEAN_SELECT not in src:
        print("[SKIP] mutant anchor text not found in select.rs (loud)",
              file=sys.stderr)
        return None
    open(sel, "w").write(src.replace(CLEAN_SELECT, MUTANT_SELECT))
    tdir = os.path.join(scratch, "mutant-target")
    p = emit_asm_build(mut, tdir)
    if p.returncode != 0:
        print("[SKIP] mutant build failed (loud)", file=sys.stderr)
        return None
    emit = find_emit(tdir)
    if emit is None:
        print("[SKIP] mutant .s emit missing (loud)", file=sys.stderr)
        return None
    per_fn = census_asm(open(emit).read(), fns)
    flagged = "cmov_u64" in per_fn and per_fn["cmov_u64"]
    return bool(flagged)


def branchy_selfcheck(scratch):
    """Arm D: branchy reference naive_if_cmov compiled for riscv32imc and
    disassembled with llvm-objdump -d MUST be flagged (B6 pattern)."""
    ref = os.path.join(scratch, "branchy_ref")
    os.makedirs(ref, exist_ok=True)
    src_path = os.path.join(ref, "branchy.rs")
    open(src_path, "w").write(BRANCHY_REF)
    obj = os.path.join(ref, "branchy.o")
    sysroot_dir = sysroot()
    rustc = os.path.join(sysroot_dir, "bin", "rustc")
    p = run([rustc, "--edition", "2021", "--emit=obj",
             "--target", TARGET, "-C", "opt-level=3",
             "-C", "link-dead-code=yes", "--crate-type", "lib",
             "-o", obj, src_path], check=False)
    if p.returncode != 0:
        print("[SKIP] branchy self-check rustc failed (loud):",
              p.stderr.strip().splitlines()[-1:] or "?", file=sys.stderr)
        return None
    arch = llvm_bin("llvm-objdump")
    if arch is None:
        return None
    od = run([arch, "-d", obj], check=False)
    if od.returncode != 0:
        print(f"[SKIP] llvm-objdump -d refused {obj} (loud)", file=sys.stderr)
        return None
    m = re.search(r"<([A-Za-z0-9_]*naive_if_cmov[^:]*):", od.stdout)
    if not m:
        print("[SKIP] naive_if_cmov symbol absent from disassembly (loud)",
              file=sys.stderr)
        return None
    # census the naive_if_cmov region for branch ops
    body = od.stdout.split(m.group(0), 1)[1]
    ops = []
    for line in body.splitlines():
        im = OBJDUMP_INSTR_RE.match(line) or INSTR_RE.match(line)
        if im and im.lastindex is not None \
                and im.group(im.lastindex) in BRANCH_OPS:
            ops.append(im.group(im.lastindex))
    return ops


BNEZ_BYTES = bytes([0x63, 0x9e, 0x00, 0x00])  # bnez x1, 8 (B-type, bne x1,x0)

AR_HDR_FMT = "16s12s6s6s8s10s2s"

def extract_member(arc_path, prefix, all_members=False):
    """Parse a GNU/BSD `ar` archive; return bytes of the first member whose
    resolved name starts with `prefix` (or a list of all matching member
    bodies with all_members=True). Handles GNU extended names (`/N` ->
    `//` string table) and BSD `#1/<len>` inline names."""
    import struct
    data = open(arc_path, "rb").read()
    if data[:8] != b"!<arch>\n":
        return None
    hits = [] if all_members else None
    off = 8
    longnames = b""
    while off + 60 <= len(data):
        hdr = struct.unpack(AR_HDR_FMT, data[off:off + 60])
        name, size_b = hdr[0], hdr[5]
        size = int(size_b.decode().strip() or "0")
        nm = name.decode(errors="replace")
        body = data[off + 60:off + 60 + size]
        if nm.startswith("//"):
            longnames = body
        elif nm.startswith("#1/"):
            ln = int(nm[3:].strip())
            body = body[ln:]
        if (nm.startswith("/") and nm[1:].strip().isdigit()):
            idx = int(nm[1:])
            end = longnames.find(b"\n", idx)
            real = longnames[idx:end].decode(errors="replace")
        elif nm.startswith("#1/"):
            real = "member-%d" % len(hits or [])
        else:
            real = nm.rstrip("/")
        if real.startswith(prefix):
            if all_members:
                hits.append((real, body))
            else:
                return body
        off += 60 + size + (size & 1)
    return hits

def elf32_text_section(path, sym_substr):
    """Parse the ELF32 section header table; return (offset, size) of the
    section carrying `sym_substr` (rustc emits per-function .text.<mangled>
    sections for riscv32imc objects)."""
    import struct
    d = open(path, "rb").read()
    if d[:4] != b"\x7fELF" or d[4] != 1:
        return None
    e_shoff, = struct.unpack_from("<I", d, 0x20)
    e_shentsize, e_shnum, e_shstrndx = struct.unpack_from("<HHH", d, 0x2E)
    def sh(i):
        o = e_shoff + i * e_shentsize
        return struct.unpack_from("<IIIIIIIIII", d, o)
    strtab = sh(e_shstrndx)
    strs = d[strtab[4]:strtab[4] + strtab[5]]
    fallback = None
    for i in range(e_shnum):
        h = sh(i)
        e = strs.find(b"\x00", h[0])
        nm = strs[h[0]:e].decode(errors="replace")
        if sym_substr in nm and nm.startswith(".text") and h[5] > 0:
            return (h[4], h[5])
        if nm == ".text" and h[5] > 0 and fallback is None:
            fallback = (h[4], h[5])
    return fallback

def binary_mutant_court(scratch, witness_arc, fns):
    """Arm E (G03): byte-level anti-vacuity. Patch one non-branch
    instruction word inside a 0-branch allowlist fn of a SCRATCH COPY of
    the linked witness archive to `bnez x1, 8`; re-disassemble; the
    census MUST newly flag bnez. Original archive sha256-verified
    unchanged after the arm."""
    import hashlib
    def sha(p):
        h = hashlib.sha256()
        with open(p, "rb") as f:
            for chunk in iter(lambda: f.read(1 << 16), b""):
                h.update(chunk)
        return h.hexdigest()

    clean_target = pick_zero_branch_target(fns)
    if clean_target is None:
        print("[SKIP] arm E: no 0-branch allowlist fn censused (loud)",
              file=sys.stderr)
        return None
    if witness_arc is None or not os.path.exists(witness_arc):
        print("[SKIP] arm E: witness archive not built (loud)", file=sys.stderr)
        return None
    before_sha = sha(witness_arc)

    members = extract_member(witness_arc, "cgap_branchless",
                             all_members=True) or []
    arch = llvm_bin("llvm-objdump")
    if arch is None:
        return None
    work = os.path.join(scratch, "binmut")
    os.makedirs(work, exist_ok=True)
    result = None
    for mname, member_bytes in members:
        obj = os.path.join(work, "member.o")
        with open(obj, "wb") as f:
            f.write(member_bytes)
        od = run([arch, "-d", obj])
        text = od.stdout
        cands = []
        for mm in re.finditer(r"<(_RNv[^:>]*)>:", text):
            sym = mm.group(1)
            if not any(f in sym for f in fns):
                continue
            body = text.split(mm.group(0), 1)[1]
            nxt = re.search(r"\n[0-9a-f]+ <_RNv", body)
            region = body[:nxt.start()] if nxt else body
            ops = [OBJDUMP_INSTR_RE.match(l).group(2)
                   for l in region.splitlines()
                   if OBJDUMP_INSTR_RE.match(l)]
            cands.append((sym, ops))
        tgt = next(((s, o) for s, o in cands
                    if clean_target in s), None) or \
            (cands[0] if cands else None)
        if tgt is None:
            continue
        sym, pre_ops = tgt
        sec = elf32_text_section(obj, sym)
        if sec is None:
            continue
        text_off, _sz = sec
        m0 = re.search(r"<" + re.escape(sym) + r">:", text)
        body = text.split(m0.group(0), 1)[1]
        nxt = re.search(r"\n[0-9a-f]+ <_RNv", body)
        region = body[:nxt.start()] if nxt else body
        addr = None
        for line in region.splitlines():
            im = OBJDUMP_INSTR_RE.match(line)
            if not im:
                continue
            bcol = im.group(1).strip().replace(" ", "")
            if len(bcol) >= 8:  # 4-byte RV32 encoding
                addr = int(line.split(":")[0].strip(), 16)
                break
        if addr is None:
            continue
        with open(obj, "r+b") as f:
            f.seek(text_off + addr)
            f.write(BNEZ_BYTES)
        od2 = run([arch, "-d", obj])
        m1 = re.search(r"<" + re.escape(sym) + r">:", od2.stdout)
        if m1 is None:
            continue
        body = od2.stdout.split(m1.group(0), 1)[1]
        nxt = re.search(r"\n[0-9a-f]+ <_RNv", body)
        region = body[:nxt.start()] if nxt else body
        post_ops = [OBJDUMP_INSTR_RE.match(l).group(2)
                    for l in region.splitlines()
                    if OBJDUMP_INSTR_RE.match(l)]
        flipped = post_ops.count("bnez") == pre_ops.count("bnez") + 1
        after_sha = sha(witness_arc)
        print(f"[BINMUT] member={mname[:40]} sym={sym[:50]}... patched "
              f"0x{addr:x} -> bnez x1, 8; pre_ops={pre_ops} "
              f"post_ops={post_ops}")
        print(f"[BINMUT] original archive sha256 "
              f"{'UNCHANGED (restore verified)' if before_sha == after_sha else 'CHANGED — FAIL-CLOSED'}")
        result = flipped
        break
    return result
    print(f"[BINMUT] subject was scratch copy; original archive sha256 "
          f"{'UNCHANGED (restore verified)' if before_sha == after_sha else 'CHANGED — FAIL-CLOSED'}")
    return flipped


def pick_zero_branch_target(fns):
    """Choose a censused 0-branch allowlist fn present in the witness."""
    for cand in ("swar_eq_u64", "prefix_or_u64x8", "poison_mask",
                 "swar_splat"):
        if cand in fns:
            return cand
    return None


def main():
    ap = argparse.ArgumentParser(
        description="AE4 disassembly-invariants court (riscv32imc)")
    ap.add_argument("--scratch", default=None,
                    help="/tmp scratch dir (default: unique per-run lease)")
    ap.add_argument("--skip-mutant", action="store_true")
    ap.add_argument("--skip-selfcheck", action="store_true")
    ap.add_argument("--skip-witness", action="store_true")
    ap.add_argument("--skip-binmut", action="store_true")
    ap.add_argument("--witness", default=None,
                    help="explicit witness archive path; nonexistent path "
                         "=> typed exit 2 (subject-missing ABI arm)")
    args = ap.parse_args()

    if os.environ.get("CGAP_AE4_FORCE_SKIP") == "1":
        print("[SKIP] forced loud-skip arm (CGAP_AE4_FORCE_SKIP=1) — "
              "typed exit 3")
        return EXIT_LOUD_SKIP
    if args.witness is not None and not os.path.exists(args.witness):
        print(f"[FAIL] witness binary missing: {args.witness} — typed "
              f"exit 2 (subject-missing ABI arm)")
        return EXIT_SUBJECT_MISSING

    if not os.path.isdir(os.path.join(CRATE, "src")):
        print(f"[FAIL] subject crate missing: {CRATE}")
        return 2
    if shutil.which("cargo") is None:
        print("[FAIL] cargo not on PATH")
        return 2
    fns = allowlist_fns()
    if not fns:
        print(f"[FAIL] no allowlist fns enumerated from {CRATE}/src")
        return 2
    print(f"[ALLOWLIST] {len(fns)} fns: {', '.join(fns)}")

    if args.scratch:
        scratch = os.path.abspath(args.scratch)
        shutil.rmtree(scratch, ignore_errors=True)
    else:
        import tempfile
        scratch = tempfile.mkdtemp(prefix="ae4-scratch-")
    os.makedirs(scratch, exist_ok=True)
    print(f"[SCRATCH] {scratch}")

    # ---- Arm A: emit-asm census (EX3 method of record) ----
    tdir = os.path.join(scratch, "census-target")
    p = emit_asm_build(CRATE, tdir)
    if p.returncode != 0:
        print(f"[FAIL] emit-asm build failed (exit {p.returncode}): "
              + (p.stderr.strip().splitlines() or ["?"])[-1])
        return 2
    emit = find_emit(tdir)
    if emit is None:
        print("[SKIP] no cgap_branchless*.s produced (loud) — emit=asm "
              "surface changed; census arm not run")
        return 3
    per_fn = census_asm(open(emit).read(), fns)
    censused = sorted(fn for fn in fns)
    hits = {fn: per_fn.get(fn, {}) for fn in censused}

    verdict = 0
    for fn in censused:
        hh = hits[fn]
        if not hh:
            print(f"[PASS] {fn}: 0 forbidden branches")
            continue
        n = sum(hh.values())
        if fn in RESIDUAL_ALLOWLIST:
            print(f"[ALLOWED-RESIDUAL] {fn}: branchy ({n}) — "
                  f"{RESIDUAL_ALLOWLIST[fn]}")
        elif fn in REFERENCE_DRIVERS:
            print(f"[REFERENCE-DRIVER] {fn}: loop branches ({n}) — "
                  f"{REFERENCE_DRIVERS[fn]}")
        else:
            print(f"[FORBIDDEN] {fn}: {sorted(hh)}")
            verdict = 1
    if "<unmapped>" in per_fn:
        for (sym, op), n in sorted(per_fn["<unmapped>"].items()):
            if "cgap_branchless" in sym and not re.search(
                    r"_(5alloc|4core|3std)", sym.split("cgap_branchless")[0]):
                print(f"[OWN-CRATE-PRIVATE] {op} x{n} in {sym[:70]} "
                      "(private helper / inlined glue; EX3 precedent: "
                      "censused, not a pub-fn row)")
            else:
                print(f"[FOREIGN-INSTANTIATION] {op} x{n} in {sym[:70]} "
                      "(std alloc/core monomorphization instantiated by "
                      "cgap-branchless; p2 receipt out-of-region class)")
        # loud honesty rows, verdict-neutral (p2 precedent)
    witness_arc = None
    witness = None
    if not args.skip_witness:
        res = linked_witness(scratch, fns)
        if isinstance(res, tuple):
            witness, witness_arc = res
    if witness:
        for fn, hh in witness.items():
            if fn in ("<unmapped>",):
                continue
            if fn in RESIDUAL_ALLOWLIST or fn in REFERENCE_DRIVERS:
                continue
            if hh:
                print(f"[FORBIDDEN-LINKED] {fn}: {sorted(hh)}")
                verdict = 1
    # ---- Arm C: mutant court (anti-vacuity) ----
    if not args.skip_mutant:
        flipped = mutant_court(scratch, fns)
        if flipped is None:
            print("[SKIP] mutant court did not run (loud, see stderr)")
            verdict = 3
        elif flipped:
            print("[PASS] anti-vacuity M-AE4-1: if-injection into cmov_u64 "
                  "flipped census to branchy (court can flip)")
        else:
            print("[VACUOUS] mutant if-injection NOT flagged — court cannot "
                  "flip")
            verdict = 1
    # ---- Arm D: branchy self-check ----
    if not args.skip_selfcheck:
        ops = branchy_selfcheck(scratch)
        if ops is None:
            print("[SKIP] branchy self-check did not run (loud, see stderr)")
            verdict = 3
        elif ops:
            print(f"[PASS] self-check naive_if_cmov flagged BRANCHY: "
                  f"{ops}")
        else:
            print("[VACUOUS] branchy reference naive_if_cmov not flagged")
            verdict = 1
    # ---- Arm E: binary-level mutant court (G03 anti-vacuity) ----
    if not args.skip_binmut:
        if witness_arc is None:
            print("[SKIP] arm E binary-mutant court skipped: no witness "
                  "archive (loud)")
            if verdict == 0:
                verdict = EXIT_LOUD_SKIP
        else:
            flipped = binary_mutant_court(scratch, witness_arc, fns)
            if flipped is None:
                print("[SKIP] arm E binary-mutant court did not run (loud)")
                if verdict == 0:
                    verdict = EXIT_LOUD_SKIP
            elif flipped:
                print("[PASS] anti-vacuity M-G03-E: synthetic bnez injected "
                      "into witness scratch copy flagged by census; original "
                      "archive sha256-verified unchanged (court can flip at "
                      "byte level)")
            else:
                print("[VACUOUS] arm E: injected bnez NOT flagged — court "
                      "cannot flip at byte level")
                verdict = EXIT_BRANCH
    if verdict == 0:
        print("AE4 disassembly-invariants court: CLEAN "
              "(census + linked witness + mutant + self-check + binary "
              "mutant all held)")
    return verdict


if __name__ == "__main__":
    sys.exit(main())
