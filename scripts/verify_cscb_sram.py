#!/usr/bin/env python3
"""C5 / lane B04 — physical SRAM budget inspector (csct coordinate).

Verifies that the csct/branchless kernel surface (BLAKE3 seal + SWAR
filter frame) fits the 128 KiB physical SRAM envelope when linked into a
minimal freestanding no_std caller, per the house footprint law
(cgap CLAUDE.md): a HOST-linked release binary will NOT fit 128 KiB —
footprint truth is the riscv32imc FREESTANDING witness built by the /tmp
no_std caller-crate method:

  #![no_std] + #![no_main] + panic_handler + black_box-ed inputs
  profile: opt-level=z / lto=fat / codegen-units=1 / panic=abort
  target:  riscv32imc-unknown-none-elf

Build recipe copied from the working precedent
`scripts/enclave_budget_gate.py` (--profile-deployment mode).

Exit ABI (house style, fail-closed):
  0 = PASS  (measured SRAM <= 131072 B hard bound)
  1 = OVERFLOW (measured > bound, unparsable size output, build blocked,
      or --mutate=sram-bloat failed to flip = VACUOUS GATE)
  2 = MISSING SUBJECT (crate dir absent or baseline build failed)
  3 = LOUD SKIP (llvm-size not found)

Anti-vacuity: --mutate=sram-bloat builds the SAME caller with an
injected `static [u8; 100_000]` and REQUIRES the measurement to exceed
131072 (exit 1). A mutant under the bound is a VACUOUS GATE (exit 1).

Python 3 stdlib only. No Process.sleep; subprocess with timeouts.
"""

import argparse
import glob
import os
import re
import shutil
import subprocess
import sys
from typing import NoReturn

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# --- frozen constants ---------------------------------------------------------

SRAM_BUDGET = 131072          # 128 KiB hard bound (gate, not advisory)
EMPIRICAL_BASELINE_BYTES = 39184   # report-only comparison line
BASELINE_CITE = ("v26.11.2 OVERNIGHT-LOOP empirical riscv32imc freestanding "
                 "witness (report-only; never a gate)")
MUTANT_STATIC_BYTES = 100_000
SUBPROCESS_TIMEOUT_S = 600

DEPENDENCY_CRATES = {
    "cgap-blake3": os.path.join(REPO, "crates", "cgap-blake3"),
    "cgap-branchless": os.path.join(REPO, "crates", "cgap-branchless"),
    "cgap-bn3": os.path.join(REPO, "crates", "cgap-bn3"),
}
# RED-12 closure (lane FIX-02, 2026-11-02): cgap-bn3 is a GATED subject.
# Its noalloc state tables are statically charged against the envelope;
# see BN3_SOURCE / bn3_capacity_table below.
BN3_SOURCE = os.path.join(REPO, "crates", "cgap-bn3", "src", "noalloc.rs")

# Ground-truth pins: host `size_of` values witnessed in
# docs/jira/v26.11.2/redteam/red-12-sram-overflow-exploit.md:61-65
# (cd /tmp/red12 && cargo run --release, exit 0). If the noalloc consts
# drift, the static table diverges from these pins and the gate REFUSES
# rather than charging a stale model.
BN3_PIN_MATCHES_BYTES = 2162696   # size_of::<NaMatches>()
BN3_PIN_ENGINE_BYTES = 4358288    # size_of::<NaEngine>()

TARGET = "riscv32imc-unknown-none-elf"
CALLER_ROOT = "/tmp/cscb-sram-caller"
TARGET_ROOT = "/tmp/cscb-sram-target"

EXIT_PASS = 0
EXIT_OVERFLOW = 1
EXIT_MISSING = 2
EXIT_SKIP = 3


LLVM_SIZE_SEARCH = [
    os.environ.get("CGAP_LLVM_SIZE", ""),
    shutil.which("llvm-size") or "",
]
for _p in (
    sorted(glob.glob(os.path.expanduser(
        "~/.rustup/toolchains/*/lib/rustlib/*-apple-darwin/bin/llvm-size")))
    + ["/opt/homebrew/opt/llvm/bin/llvm-size",
       "/usr/local/opt/llvm/bin/llvm-size",
       "/Applications/Xcode.app/Contents/Developer/Toolchains/"
       "XcodeDefault.xctoolchain/usr/bin/llvm-size"]
):
    LLVM_SIZE_SEARCH.append(_p)
LLVM_SIZE = next((p for p in LLVM_SIZE_SEARCH if p and os.path.isfile(p)),
                 None)
LLVM_OBJDUMP = None
if LLVM_SIZE:
    _sib = os.path.join(os.path.dirname(LLVM_SIZE), "llvm-objdump")
    if os.path.isfile(_sib):
        LLVM_OBJDUMP = _sib


def fail(msg, code=EXIT_OVERFLOW) -> NoReturn:
    # NoReturn annotation (lane RC-10): sys.exit is NoReturn, so type
    # checkers can narrow `proc` after a fail() in an except block.
    # Behavior unchanged — sys.exit raises SystemExit exactly as before.
    print("GATE REFUSED: %s" % msg, file=sys.stderr)
    sys.exit(code)


def skip(msg) -> NoReturn:
    print("[SKIP] %s" % msg, file=sys.stderr)
    sys.exit(EXIT_SKIP)


def run_tool(cmd, cwd=None):
    try:
        proc = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True,
                              timeout=SUBPROCESS_TIMEOUT_S)
    except (OSError, subprocess.TimeoutExpired) as exc:
        fail("tool spawn/timeout failure: %r: %s" % (cmd, exc))
    if proc.returncode != 0:
        fail("tool failed (exit %d): %r: %s"
             % (proc.returncode, cmd, (proc.stderr or "").strip()[-300:]))
    return proc.stdout


# SRAM charge = .text + .rodata + .data + .bss families (lane B04 spec:
# flash-resident .text/.rodata plus RAM .data/.bss — the full physical
# budget, broader than the enclave gate's flash-only .text/.rodata).
SRAM_SECTIONS = (".text", ".rodata", ".data", ".bss")


def size_elf_sram(path):
    """Sum .text/.rodata/.data/.bss family sections from `llvm-size -A`
    over the LINKED ELF. No matching section = typed refusal, never a
    silent 0."""
    out = run_tool([LLVM_SIZE, "-A", path])
    total = 0
    matched = False
    for m in re.finditer(r"^(\.\S+)\s+(\d+)(?:\s+\d+)?\s*$", out, re.M):
        name, nbytes = m.group(1), int(m.group(2))
        if name.startswith(SRAM_SECTIONS):
            total += nbytes
            matched = True
    if not matched:
        fail("no .text/.rodata/.data/.bss section in llvm-size -A for %s"
             % path)
    return total


# --- cgap-bn3 static capacity/charge table (RED-12 closure) ---------------------

_NA_CONST_RE = re.compile(
    r"^pub\s+const\s+NA_MAX_(CONSTS|VARS|ATOMS|MATCHES|ROWS)\s*:\s*usize"
    r"\s*=\s*(\d+);", re.M)


def parse_na_consts(path=BN3_SOURCE):
    """Statically read the NA_MAX_* capacity consts from noalloc.rs.
    Sizes are parsed from the source of truth, never hardcoded here.
    Missing file/consts = typed gate failure (fail-closed)."""
    if not os.path.isfile(path):
        fail("MISSING SUBJECT: cgap-bn3 source %s" % path, EXIT_MISSING)
    src = open(path, encoding="utf-8").read()
    found = {m.group(1): int(m.group(2)) for m in _NA_CONST_RE.finditer(src)}
    need = {"CONSTS", "VARS", "ATOMS", "MATCHES", "ROWS"}
    missing = need - set(found)
    if missing:
        fail("bn3 capacity table unparsable: NA_MAX_%s not found in %s"
             % (",".join(sorted(missing)), path))
    return found


def bn3_capacity_table(consts):
    """Static byte model of the noalloc state tables
    (crates/cgap-bn3/src/noalloc.rs:99-161):

      NaMatches = len:usize
                + rows: [[u32; NA_MAX_ATOMS]; NA_MAX_MATCHES]
                + vars: [[u64; NA_MAX_VARS]; NA_MAX_MATCHES]
      NaEngine  = cur/nxt double buffers (2x the rows+vars planes)
                + bound: [bool; NA_MAX_VARS]
                + radix_rows: [u32; NA_MAX_ROWS]

    Capacities are topology-INDEPENDENT consts: the buffers are sized
    for the worst topology at construction (red-12 W4), so the charge is
    the full-const model — an 8-atom cyclic join within every bound
    still charges this. Cross-checked against the red-12 host size_of
    pins; divergence = typed refusal, never a stale charge."""
    matches_bytes = 8 + 4 * consts["ATOMS"] * consts["MATCHES"] \
                      + 8 * consts["VARS"] * consts["MATCHES"]
    engine_bytes = (2 * (4 * consts["ATOMS"] * consts["MATCHES"]
                         + 8 * consts["VARS"] * consts["MATCHES"])
                    + 2 * 8  # cur_len + nxt_len: usize
                    + consts["VARS"] + 4 * consts["ROWS"])
    if matches_bytes != BN3_PIN_MATCHES_BYTES:
        fail("bn3 capacity table DIVERGED from red-12 size_of pin: "
             "static NaMatches %d B != pinned %d B (consts drifted?)"
             % (matches_bytes, BN3_PIN_MATCHES_BYTES))
    if engine_bytes != BN3_PIN_ENGINE_BYTES:
        fail("bn3 capacity table DIVERGED from red-12 size_of pin: "
             "static NaEngine %d B != pinned %d B (consts drifted?)"
             % (engine_bytes, BN3_PIN_ENGINE_BYTES))
    return {"matches": matches_bytes, "engine": engine_bytes,
            "total": matches_bytes + engine_bytes}


def bn3_enclave_verdict(charge):
    """Typed verdict on admitting the bn3 noalloc state tables into the
    128 KiB enclave. Over ceiling = loud typed gate failure (exit 1),
    never a silent pass — the red-12 surface gap is closed."""
    if charge["total"] > SRAM_BUDGET:
        print("BN3_ENCLAVE_REFUSED: static noalloc state-table charge "
              "%d B (NaMatches %d B + NaEngine %d B) > %d B hard bound "
              "(%.2fx ceiling; red-12 topology). Admitting "
              "NaEngine+NaMatches into the SRAM enclave is a typed gate "
              "failure." % (charge["total"], charge["matches"],
                            charge["engine"], SRAM_BUDGET,
                            charge["total"] / SRAM_BUDGET), file=sys.stderr)
        return False
    return True


_CALLER_MAIN = '''#![no_std]
#![no_main]
use core::hint::black_box;

#[panic_handler]
fn cscb_panic(_: &core::panic::PanicInfo) -> ! { loop {} }

use core::alloc::{GlobalAlloc, Layout};
struct CscbBump;
static mut CSCB_HEAP: [u8; 16384] = [0u8; 16384];
unsafe impl Sync for CscbBump {}
unsafe impl GlobalAlloc for CscbBump {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        static mut CUR: usize = 0;
        let base = (&raw const CSCB_HEAP) as usize;
        let cur = ((base + CUR + l.align() - 1) / l.align().max(1)) * l.align();
        if cur + l.size() > base + CSCB_HEAP.len() { return core::ptr::null_mut(); }
        CUR = cur - base + l.size();
        cur as *mut u8
    }
    unsafe fn dealloc(&self, _p: *mut u8, _l: Layout) {}
}
#[global_allocator]
static CSCB_ALLOC: CscbBump = CscbBump;

%(extra_statics)s
#[no_mangle]
extern "C" fn _start() -> ! {
    // Runtime-opaque prologue: volatile writes block const-folding so the
    // linker must keep the real kernel paths (constant inputs fold to
    // vacuum ELFs — house footprint law).
    let mut data = [0u8; 4096];
    for i in 0..data.len() {
        unsafe {
            core::ptr::write_volatile(data.as_mut_ptr().add(i),
                                      black_box((i * 7) as u8));
        }
    }
    // SEAL: real BLAKE3 over the runtime-opaque buffer.
    black_box(cgap_blake3::hash(&data));
    black_box(cgap_blake3::hash_many_simd(&data));
    // FILTER FRAME: real SWAR masks over runtime-opaque words.
    let mut words = [0u64; 64];
    for (i, w) in words.iter_mut().enumerate() {
        *w = u64::from_le_bytes([
            black_box((i * 3) as u8), black_box((i * 5) as u8),
            black_box((i * 7) as u8), black_box((i * 11) as u8),
            black_box((i * 13) as u8), black_box((i * 17) as u8),
            black_box((i * 19) as u8), black_box((i * 23) as u8),
        ]);
    }
    let mut acc: u64 = 0;
    for w in words.iter() {
        acc ^= cgap_branchless::swar::swar_range_mask(*w & 0x00ff00ff00ff00ff,
                                                      black_box(3),
                                                      black_box(200));
        acc = acc.rotate_left(1) ^ cgap_branchless::swar::swar_lt(*w,
                                                                 acc);
    }
    unsafe { black_box(core::ptr::read_volatile(&acc)); }%(touch)s
    loop {}
}
'''

_CALLER_CARGO = '''[package]
name = "cscb-sram-%(variant)s"
version = "0.1.0"
edition = "2021"

[dependencies]
cgap-blake3 = { path = %(blake3_path)r }
cgap-branchless = { path = %(branchless_path)r }

[profile.release]
opt-level = "z"
lto = "fat"
codegen-units = 1
panic = "abort"

[workspace]
'''


def build_caller(variant, extra_statics, touch=""):
    """Generate + build the no_std caller crate in /tmp (isolated variant
    names so mutant builds never overwrite the baseline ELF — H13).
    Returns (elf_path, None) or (None, blocked_reason)."""
    root = os.path.join(CALLER_ROOT, variant)
    os.makedirs(os.path.join(root, "src"), exist_ok=True)
    ctx = {
        "variant": variant,
        "blake3_path": DEPENDENCY_CRATES["cgap-blake3"],
        "branchless_path": DEPENDENCY_CRATES["cgap-branchless"],
    }
    with open(os.path.join(root, "Cargo.toml"), "w") as f:
        f.write(_CALLER_CARGO % ctx)
    with open(os.path.join(root, "src", "main.rs"), "w") as f:
        f.write(_CALLER_MAIN % {"extra_statics": extra_statics,
                            "touch": touch})
    env = dict(os.environ)
    env["CARGO_TARGET_DIR"] = os.path.join(TARGET_ROOT, variant)
    cmd = ["cargo", "+nightly", "build", "--release", "--target", TARGET]
    try:
        proc = subprocess.run(cmd, cwd=root, capture_output=True, text=True,
                              env=env, timeout=SUBPROCESS_TIMEOUT_S)
    except (OSError, subprocess.TimeoutExpired) as exc:
        return None, "cargo spawn/timeout failure: %s" % exc
    if proc.returncode != 0:
        tail = (proc.stderr or "").strip().splitlines()
        reason = tail[-1][:300] if tail else "exit %d" % proc.returncode
        return None, "build failed: %s" % reason
    elf = os.path.join(TARGET_ROOT, variant, TARGET, "release",
                       "cscb-sram-%s" % variant)
    if not os.path.isfile(elf):
        return None, "linked ELF missing after successful build: %s" % elf
    return elf, None


def stack_estimate(elf):
    """Static-analysis-only stack estimate: sum the largest immediate
    frame adjustments (`addi sp, sp, -N`) per function via llvm-objdump.
    Honest output: the max leaf-frame bound, or UNVERIFIED when the tool
    is absent or nothing parsable is found. NEVER fabricated."""
    if not LLVM_OBJDUMP:
        return None, "llvm-objdump not found beside llvm-size"
    out = run_tool([LLVM_OBJDUMP, "-d", "--no-show-raw-insn", elf])
    frames = [int(m.group(1), 0) for m in
              re.finditer(r"addi\s+sp,\s*sp,\s*-(0x[0-9a-fA-F]+|\d+)",
                          out)]
    frames = [f for f in frames if f > 0]
    if not frames:
        return None, ("no nonzero `addi sp,sp,-N` frame sizes parsable "
                      "(riscv leaf prologues may use a different encoding)")
    return max(frames), ("max observed `addi sp,sp,-N` leaf-frame size "
                         "over %d sp-adjusting prologues" % len(frames))


def toolchain_record():
    rc = run_tool(["cargo", "+nightly", "--version"])
    host = run_tool(["rustc", "+nightly", "--version"])
    tgt = "installed" if os.path.isdir(os.path.expanduser(
        "~/.rustup/toolchains")) and subprocess.run(
            ["rustup", "target", "list", "--installed"],
            capture_output=True, text=True).stdout.find(TARGET) >= 0 \
        else "NOT INSTALLED"
    return rc.strip(), host.strip(), tgt


# --- main -----------------------------------------------------------------------

def main():
    ap = argparse.ArgumentParser(
        description="lane B04 physical SRAM budget inspector (128 KiB)")
    ap.add_argument("--mutate", choices=["sram-bloat", "bn3-pin"],
                    default=None,
                    help="anti-vacuity: sram-bloat = same caller + static "
                         "[u8;100000] MUST overflow (exit 1); bn3-pin = "
                         "sandboxed NA_MAX_MATCHES drift MUST trip the "
                         "red-12 size_of pin check (exit 1). A pass is "
                         "VACUOUS GATE.")
    ap.add_argument("--subject", choices=["baseline", "bn3"],
                    default="baseline",
                    help="baseline = blake3+branchless linked-ELF witness; "
                         "bn3 = static cgap-bn3 noalloc state-table charge "
                         "against the envelope (typed refusal over ceiling)")
    ap.add_argument("--bn3-source", default=BN3_SOURCE,
                    help=argparse.SUPPRESS)  # court seam: sandboxed source
    args = ap.parse_args()

    if LLVM_SIZE is None:
        skip("llvm-size not found (CGAP_LLVM_SIZE, PATH, rustup toolchains, "
             "homebrew, Xcode all searched)")
    for crate, path in sorted(DEPENDENCY_CRATES.items()):
        if not os.path.isdir(path):
            fail("MISSING SUBJECT: crate dir %s (%s)" % (path, crate),
                 EXIT_MISSING)
    cargo_ver, rustc_ver, tgt_state = toolchain_record()
    print("toolchain: %s | %s | target %s: %s"
          % (cargo_ver, rustc_ver, TARGET, tgt_state))

    consts = parse_na_consts(args.bn3_source)
    charge = bn3_capacity_table(consts)
    print("bn3 capacity table (static, from %s): NA_MAX_ATOMS=%d "
          "NA_MAX_MATCHES=%d NA_MAX_VARS=%d NA_MAX_ROWS=%d -> "
          "NaMatches %d B + NaEngine %d B = %d B"
          % (args.bn3_source, consts["ATOMS"], consts["MATCHES"],
             consts["VARS"], consts["ROWS"], charge["matches"],
             charge["engine"], charge["total"]))

    if args.subject == "bn3":
        # RED-12 closure: bn3 as an enclave subject is charged statically.
        ok = bn3_enclave_verdict(charge)
        if ok:
            print("BN3 subject: %d B <= %d B — in ceiling (headroom %d B)"
                  % (charge["total"], SRAM_BUDGET,
                     SRAM_BUDGET - charge["total"]))
        sys.exit(EXIT_PASS if ok else EXIT_OVERFLOW)

    if args.mutate == "bn3-pin":
        # Anti-vacuity for the TABLE: a sandboxed capacity drift must
        # trip the red-12 size_of pin cross-check (exit 1, DIVERGED).
        # Run against a /tmp copy with NA_MAX_MATCHES halved.
        root = "/tmp/cscb-sram-bn3-pin"
        os.makedirs(root, exist_ok=True)
        mutated = os.path.join(root, "noalloc.rs")
        src = open(BN3_SOURCE, encoding="utf-8").read()
        mutated_src, n = re.subn(r"pub const NA_MAX_MATCHES: usize = 2048;",
                                 "pub const NA_MAX_MATCHES: usize = 1024;",
                                 src)
        if n != 1:
            fail("bn3-pin court could not inject the NA_MAX_MATCHES "
                 "mutation (subn=%d) — VACUOUS COURT" % n)
        open(mutated, "w", encoding="utf-8").write(mutated_src)
        probe = subprocess.run(
            [sys.executable, os.path.abspath(__file__),
             "--subject", "bn3", "--bn3-source", mutated],
            capture_output=True, text=True)
        if probe.returncode == 0:
            print("VACUOUS GATE: bn3-pin mutant PASSED — the capacity "
                  "table cannot see a 2x NA_MAX_MATCHES drift",
                  file=sys.stderr)
            sys.exit(EXIT_OVERFLOW)
        print("MUTATE COURT: bn3-pin mutant refused (exit %d): %s"
              % (probe.returncode,
                 (probe.stderr or probe.stdout).strip().splitlines()[-1]))
        sys.exit(EXIT_PASS)

    if args.mutate == "sram-bloat":
        blob = "static CSCB_MUTANT_SRAM: [u8; %d] = [7u8; %d];\n" % (
            MUTANT_STATIC_BYTES, MUTANT_STATIC_BYTES)
        touch = ("\n    unsafe { black_box(core::ptr::read_volatile("
                 "CSCB_MUTANT_SRAM.as_ptr())); }")
        elf, blocked = build_caller("mutant", blob, touch)
        if blocked:
            fail("mutant build BLOCKED (%s) — fail-closed, not a pass"
                 % blocked)
        size = size_elf_sram(elf)
        if size <= SRAM_BUDGET:
            print("VACUOUS GATE: sram-bloat mutant measured %d B <= %d B — "
                  "the gate cannot see 100 KB of injected SRAM"
                  % (size, SRAM_BUDGET), file=sys.stderr)
            sys.exit(EXIT_OVERFLOW)
        print("MUTATE COURT: sram-bloat mutant measured %d B > %d B — "
              "overflow flips (anti-vacuity holds); ELF: %s"
              % (size, SRAM_BUDGET, elf))
        sys.exit(EXIT_PASS)

    elf, blocked = build_caller("caller", "")
    if blocked:
        fail("MISSING SUBJECT: baseline build failed (%s)" % blocked,
             EXIT_MISSING)
    size = size_elf_sram(elf)
    delta = size - EMPIRICAL_BASELINE_BYTES
    print("measured SRAM (%s): %d B  [.text+.rodata+.data+.bss families]"
          % (TARGET, size))
    print("empirical-baseline comparison (report-only, NOT gated): "
          "baseline %d B -> measured %d B (delta %+d B); cite: %s"
          % (EMPIRICAL_BASELINE_BYTES, size, delta, BASELINE_CITE))
    frame, why = stack_estimate(elf)
    if frame is None:
        print("stack estimate: UNVERIFIED (%s) — no runtime watermark "
              "claimed" % why)
    else:
        print("stack estimate (static analysis only): max leaf frame %d B "
              "(%s); UNVERIFIED as a worst-case chain bound" % (frame, why))
    if size > SRAM_BUDGET:
        print("SRAM GATE FAILED: %d B > %d B hard bound"
              % (size, SRAM_BUDGET), file=sys.stderr)
        sys.exit(EXIT_OVERFLOW)
    print("SRAM GATE: %d B <= %d B — csct/branchless kernel surface fits "
          "the physical envelope (minimum entry surface; headroom %d B)"
          % (size, SRAM_BUDGET, SRAM_BUDGET - size))
    print("bn3 enclave surface (report-only here, gate under --subject "
          "bn3): static state-table charge %d B — in ceiling: %s"
          % (charge["total"], "yes" if charge["total"] <= SRAM_BUDGET
             else "NO — admission REFUSED (typed gate failure, exit 1)"))
    sys.exit(EXIT_PASS)


if __name__ == "__main__":
    main()
