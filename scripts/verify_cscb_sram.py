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
}
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


def fail(msg, code=EXIT_OVERFLOW):
    print("GATE REFUSED: %s" % msg, file=sys.stderr)
    sys.exit(code)


def skip(msg):
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
    ap.add_argument("--mutate", choices=["sram-bloat"], default=None,
                    help="anti-vacuity: same caller + static [u8;100000] "
                         "MUST overflow (exit 1); a pass is VACUOUS GATE")
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
    sys.exit(EXIT_PASS)


if __name__ == "__main__":
    main()
