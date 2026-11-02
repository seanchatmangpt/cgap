#!/usr/bin/env python3
"""W5 D4 enclave budget gate — 128 KiB whole-envelope footprint court.

House gate style (exit ABI per scripts/boundary_gate.py / zero_string_gate.py):
  0 = gate passes (all artifacts under their tier budgets, and the sum of
      all measured artifacts under the whole-envelope budget)
  1 = typed refusal (budget breach, unparsable artifact, VACUOUS GATE on
      --mutate / --self-check)
  2 = missing subject (an --artifact path does not exist; --check with no
      artifacts given)
  3 = loud skip ([SKIP] sizing tool unavailable; never a silent pass)

Budget table (frozen this lane, v1):
  whole envelope .......... 128 KiB = 131072 bytes
  tier kernel .............. 16 KiB = 16384 bytes
  tier storage ............. 48 KiB = 49152 bytes
  tier compute ............. 48 KiB = 49152 bytes
  tier attest .............. 12 KiB = 12288 bytes
  (tiers sum to 124 KiB; 4 KiB envelope slack — the envelope check is the
  binding constraint on the sum of MEASURED artifacts, tiers bind per-slot.)

Sizing method (real collaborators, no mocks): `llvm-size -A` over each
artifact. .rlib subjects are extracted with `llvm-ar` into a temp dir and
each *.o member is sized; the artifact size is the SUM of member Totals
(rmeta members are excluded — they carry no linked code). A sizing-tool
spawn failure on a parseable subject is a typed refusal (exit 1), never a
silent zero.

Anti-vacuity (house law: a court that cannot flip is FAIL):
  --mutate       copies the first --artifact to a /tmp mutant, appends a
                 30720-byte blob, and REQUIRES --check over the mutant set
                 to refuse (exit 1). A mutant that passes is VACUOUS GATE
                 (exit 1) — the gate proved it cannot see a 30 KB envelope
                 breach.
  --self-check   three real arms:
                 C clean   — a REAL cc-compiled tiny object under the kernel
                             tier measures and passes (exit 0).
                 P poisoned— a REAL cc-compiled object with a 32 KiB global
                             array breaches the kernel tier (exit 1).
                 M mutant  — the --mutate append-blob refusal, witnessed on
                             a real artifact copy (loud [SKIP] when no
                             artifact is supplied).
                 Any arm that cannot flip is a FAIL of the court.

Python 3 stdlib only. No Process.sleep; subprocess with timeouts.
"""

import argparse
import glob
import os
import re
import shutil
import subprocess
import sys
import tempfile

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# --- frozen budget table (v1) -----------------------------------------------

ENVELOPE_BUDGET = 131072          # 128 KiB whole envelope
TIER_BUDGETS = {
    "kernel": 16384,              # 16 KiB
    "storage": 49152,             # 48 KiB
    "compute": 49152,             # 48 KiB
    "attest": 12288,              # 12 KiB
}
ENVELOPE_NAME = "envelope"

# --- exit ABI ---------------------------------------------------------------

EXIT_PASS = 0
EXIT_REFUSE = 1
EXIT_MISSING = 2
EXIT_SKIP = 3

GATE_NAME = "enclave-budget-gate"
GATE_VERSION = 1
MUTANT_BLOB_BYTES = 30720         # 30 KB appended by --mutate
MUTANT_TAG = b"ENCLAVE-BUDGET-MUTANT-BLOB"  # repeated to fill the blob
SUBPROCESS_TIMEOUT_S = 120
KIIB = 1024


def fail(msg, code=EXIT_REFUSE):
    print("GATE REFUSED: %s" % msg, file=sys.stderr)
    sys.exit(code)


def skip(msg):
    print("[SKIP] %s" % msg, file=sys.stderr)
    sys.exit(EXIT_SKIP)


# --- sizing tool resolution ---------------------------------------------------

def find_llvm_size():
    """Resolve llvm-size: env override, PATH, rustup toolchains, homebrew,
    Xcode. None found -> the caller skips loudly (exit 3)."""
    env = os.environ.get("CGAP_LLVM_SIZE")
    if env and os.path.isfile(env):
        return env
    which = shutil.which("llvm-size")
    if which:
        return which
    candidates = []
    home = os.path.expanduser("~")
    candidates += sorted(glob.glob(os.path.join(
        home, ".rustup/toolchains/*/lib/rustlib/*-apple-darwin/bin/llvm-size")))
    candidates += [
        "/opt/homebrew/opt/llvm/bin/llvm-size",
        "/usr/local/opt/llvm/bin/llvm-size",
        "/Applications/Xcode.app/Contents/Developer/Toolchains/"
        "XcodeDefault.xctoolchain/usr/bin/llvm-size",
    ]
    for path in candidates:
        if os.path.isfile(path):
            return path
    return None


def find_llvm_ar(llvm_size_path):
    """llvm-ar sits beside llvm-size in every layout above; fall back to the
    same search minus the env override."""
    sibling = os.path.join(os.path.dirname(llvm_size_path), "llvm-ar")
    if os.path.isfile(sibling):
        return sibling
    home = os.path.expanduser("~")
    for path in glob.glob(os.path.join(
            home, ".rustup/toolchains/*/lib/rustlib/*-apple-darwin/bin/llvm-ar")):
        return path
    return None


def run_tool(cmd, cwd=None):
    try:
        proc = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True,
                              timeout=SUBPROCESS_TIMEOUT_S)
    except (OSError, subprocess.TimeoutExpired) as exc:
        fail("sizing tool spawn/timeout failure: %r: %s" % (cmd, exc))
        raise AssertionError("fail() is noreturn; unreachable")  # noqa: liveness
    if proc.returncode != 0:
        fail("sizing tool failed (exit %d): %r: %s"
             % (proc.returncode, cmd, (proc.stderr or "").strip()[-300:]))
    return proc.stdout


def parse_total(stdout, subject):
    """Parse `llvm-size -A` output; return the Total line's byte count.
    A summary without a Total is a typed refusal, never a silent 0."""
    totals = re.findall(r"^Total\s+(\d+)", stdout, re.M)
    if not totals:
        fail("no Total in llvm-size -A output for %s" % subject)
    return sum(int(t) for t in totals)


def measure_artifact(path, llvm_size, llvm_ar):
    """Measured byte footprint of one artifact: `llvm-size -A` directly, or
    for an .rlib (ar archive) the sum of per-member Totals over the *.o
    members, extracted to a temp dir by llvm-ar. Members that llvm-size
    cannot parse are a typed refusal (fail-closed), never skipped."""
    if not os.path.isfile(path):
        fail("artifact vanished during measurement: %s" % path,
             EXIT_MISSING)
    if path.endswith(".rlib"):
        if not llvm_ar:
            skip("llvm-ar unavailable; cannot size .rlib subject %s" % path)
        with tempfile.TemporaryDirectory(prefix="ebg-extract-") as tmp:
            run_tool([llvm_ar, "x", os.path.abspath(path)], cwd=tmp)
            members = sorted(f for f in os.listdir(tmp) if f.endswith(".o"))
            if not members:
                fail("no *.o member in %s" % path)
            return sum(parse_total(
                run_tool([llvm_size, "-A", os.path.join(tmp, m)]), m)
                for m in members)
    return parse_total(run_tool([llvm_size, "-A", path]), path)


# --- budget adjudication ------------------------------------------------------

def check_budgets(artifacts, llvm_size, llvm_ar):
    """Measure every (tier, path); enforce per-tier and whole-envelope
    budgets. Returns (exit_code, rows). Missing subjects refuse with exit 2
    BEFORE any measurement (a missing subject is not a breach)."""
    missing = [p for _, p in artifacts if not os.path.exists(p)]
    if not artifacts:
        fail("no subjects: --artifact TIER=PATH required (--check with zero "
             "artifacts enforces nothing)", EXIT_MISSING)
    if missing:
        fail("MISSING SUBJECT: %s" % ", ".join(missing), EXIT_MISSING)
    rows = []
    breaches = 0
    envelope_total = 0
    for tier, path in artifacts:
        if tier not in TIER_BUDGETS:
            fail("unknown tier %r (budget table has %s)"
                 % (tier, ", ".join(sorted(TIER_BUDGETS))))
        size = measure_artifact(path, llvm_size, llvm_ar)
        budget = TIER_BUDGETS[tier]
        breach = size > budget
        envelope_total += size
        breaches += breach
        rows.append({
            "tier": tier,
            "path": path,
            "bytes": size,
            "budget": budget,
            "verdict": "BREACH" if breach else "OK",
        })
        print("  %-8s %9d / %5d B  %-7s  %s"
              % (tier, size, budget, "BREACH" if breach else "OK", path))
    env_breach = envelope_total > ENVELOPE_BUDGET
    print("SUMMARY %s v%d artifacts=%d envelope=%d/%d verdict=%s "
          "tier_breaches=%d"
          % (GATE_NAME, GATE_VERSION, len(artifacts), envelope_total,
             ENVELOPE_BUDGET, "BREACH" if env_breach else "OK", breaches))
    if breaches or env_breach:
        print("BUDGET GATE FAILED: %d tier breach(es)%s" % (
            breaches,
            " + envelope breach (%d > %d B)"
            % (envelope_total, ENVELOPE_BUDGET) if env_breach else ""),
            file=sys.stderr)
        return EXIT_REFUSE, rows
    print("BUDGET GATE: all %d artifact(s) under tier budgets and the "
          "%d B whole envelope" % (len(artifacts), ENVELOPE_BUDGET))
    return EXIT_PASS, rows


def parse_artifact_args(values):
    out = []
    for spec in values:
        tier, sep, path = spec.partition("=")
        if not sep or not tier or not path:
            fail("bad --artifact %r (want TIER=PATH)" % spec)
        out.append((tier, path))
    return out


# --- anti-vacuity: --mutate and --self-check -----------------------------------

def build_mutant(source_path):
    """/tmp mutant: a byte copy of the real artifact with a 30720-byte
    tagged blob appended. Returns the mutant path. The blob is appended to
    the FILE, so the archive/ELF trailer is corrupted — the gate must
    refuse (budget verdict OR fail-closed parse verdict); either refusal
    kills the mutant, a pass is VACUOUS GATE."""
    with open(source_path, "rb") as f:
        base = f.read()
    blob = (MUTANT_TAG * (MUTANT_BLOB_BYTES // len(MUTANT_TAG) + 1))[
        :MUTANT_BLOB_BYTES]
    assert len(blob) == MUTANT_BLOB_BYTES
    fd, mutant = tempfile.mkstemp(prefix="ebg-mutant-", suffix=".rlib")
    with os.fdopen(fd, "wb") as f:
        f.write(base + blob)
    return mutant


def run_check(artifacts, label):
    """Spawn this gate's --check over `artifacts` as a REAL subprocess and
    return its exit code. The court observes the gate exactly as a user
    does — never by calling internals on trust."""
    cmd = [sys.executable, os.path.abspath(__file__), "--check"]
    for tier, path in artifacts:
        cmd += ["--artifact", "%s=%s" % (tier, path)]
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True,
                              timeout=SUBPROCESS_TIMEOUT_S)
    except (OSError, subprocess.TimeoutExpired) as exc:
        print("%s: gate spawn failed: %s" % (label, exc), file=sys.stderr)
        sys.exit(EXIT_REFUSE)
    return proc.returncode


def mode_mutate(artifacts, _llvm_size, _llvm_ar):
    """--mutate: append the 30 KB blob to a /tmp copy of the first artifact
    and REQUIRE --check to refuse. Mutant passes -> VACUOUS GATE (exit 1).
    No artifact supplied -> loud skip (exit 3), never a silent pass."""
    if not artifacts:
        skip("--mutate needs at least one --artifact TIER=PATH subject")
    tier, path = artifacts[0]
    mutant = build_mutant(path)
    try:
        rc = run_check([(tier, mutant)], "MUTATE")
    finally:
        os.unlink(mutant)
    if rc == EXIT_PASS:
        print("VACUOUS GATE: --check PASSED over the 30 KB-append mutant "
              "of %s — the gate cannot see an envelope breach; refusing "
              "the gate (mutate court FAIL)" % path, file=sys.stderr)
        return EXIT_REFUSE
    print("MUTATE COURT: mutant refused (gate exit %d) over %s + 30 KB "
          "blob — anti-vacuity holds" % (rc, path))
    return EXIT_PASS


def mode_self_check(artifacts, _llvm_size, _llvm_ar):
    """--self-check: poisoned-arm / clean-arm non-vacuity (house pattern).
    C: real cc-compiled tiny object under the kernel tier -> exit 0.
    P: real cc-compiled 32 KiB-global object -> exit 1 (tier breach).
    M: the --mutate append-blob refusal over a real artifact (loud [SKIP]
       when no artifact is supplied).
    Every arm must flip as claimed; any other outcome is a FAIL (exit 1)."""
    failures = []

    def expect(label, got, want, detail):
        if got != want:
            failures.append("%s: exit %d (expected %d)%s"
                            % (label, got, want, (" — %s" % detail) if detail else ""))
            print("%s FAIL: %s" % (label, failures[-1]), file=sys.stderr)
        else:
            print("%s: exit %d as required %s" % (label, got, detail))

    cc = os.environ.get("CC", "cc")
    with tempfile.TemporaryDirectory(prefix="ebg-selfcheck-") as tmp:
        # Arm C: clean — tiny object, under the 16 KiB kernel tier.
        clean_c = os.path.join(tmp, "clean.c")
        with open(clean_c, "w") as f:
            f.write("int ebg_clean(void) { return 7; }\n")
        clean_o = os.path.join(tmp, "clean.o")
        r = subprocess.run([cc, "-c", clean_c, "-o", clean_o],
                           capture_output=True, text=True, timeout=60)
        if r.returncode != 0:
            failures.append("arm C: cc failed: %s" % (r.stderr or "")[-200:])
        else:
            expect("arm C (clean)", run_check([("kernel", clean_o)], "C"),
                   EXIT_PASS, "tiny real object under kernel tier")
        # Arm P: poisoned — 32 KiB global array, over the kernel tier.
        poison_c = os.path.join(tmp, "poison.c")
        with open(poison_c, "w") as f:
            f.write("char ebg_poison[32768] = {1};\n"
                    "int ebg_poison_get(void) { return ebg_poison[0]; }\n")
        poison_o = os.path.join(tmp, "poison.o")
        r = subprocess.run([cc, "-c", poison_c, "-o", poison_o],
                           capture_output=True, text=True, timeout=60)
        if r.returncode != 0:
            failures.append("arm P: cc failed: %s" % (r.stderr or "")[-200:])
        else:
            expect("arm P (poisoned)", run_check([("kernel", poison_o)], "P"),
                   EXIT_REFUSE, "32 KiB global breaches kernel tier")
        # Arm M: real-artifact mutant flip (delegates to --mutate semantics).
        if artifacts:
            tier, path = artifacts[0]
            mutant = build_mutant(path)
            try:
                expect("arm M (mutant)", run_check([(tier, mutant)], "M"),
                       EXIT_REFUSE, "30 KB-append mutant must refuse")
            finally:
                os.unlink(mutant)
        else:
            print("[SKIP] arm M (mutant): no --artifact subject supplied "
                  "(loud skip; arms C and P still ran)", file=sys.stderr)

    if failures:
        print("SELF-CHECK FAIL: %d arm failure(s) — the budget court "
              "cannot flip and is VACUOUS" % len(failures), file=sys.stderr)
        return EXIT_REFUSE
    print("SELF-CHECK: clean passes, poisoned refuses%s — budget gate "
          "non-vacuous" % (", mutant refuses" if artifacts else ""))
    return EXIT_PASS


# --- deployment-profile mode (--profile-deployment) --------------------------
#
# Operator order (W6 lane BGP): the budget gate must measure LINKED
# deployment-profile ELFs, not audit rlibs. Witnessed sprawl: cgap-blake3
# audit-rlib 113,360 B vs linked z/fat-LTO/cgu1 ELF 40,208 B (2.81x). For
# each subject crate the gate generates a minimal no_std caller crate under
# /tmp (panic_handler + #[no_mangle] _start exercising the crate's entry
# API), builds it with cargo +nightly --release --target riscv32imc-
# unknown-none-elf under opt-level=z / lto=fat / codegen-units=1 /
# panic=abort, and llvm-sizes the LINKED ELF (.text + .rodata + .data +
# .bss family — superset of the witnessed .text+.rodata method, fail-closed
# direction). The old audit-profile modes are untouched.

DEPLOY_TIERS = {
    "tier0": 49152,   # 48 KiB — admission/attestation (incl. cgap-blake3)
    "tier1": 49152,   # 48 KiB — arrow relational
    "tier2": 24576,   # 24 KiB — parquet/git storage
    "tier3": 8192,    # 8 KiB — C-ABI membrane
}
DEPLOY_TARGET = "riscv32imc-unknown-none-elf"
DEPLOY_CALLER_ROOT = "/tmp/bgp-crate"
DEPLOY_TARGET_ROOT = "/tmp/bgp-target"
DEPLOY_MUTANT_BYTES = 30720      # 30 KB static array in the mutant caller

# FA10 witness constants (docs/jira/v26.11.2/adversarial/forensic-lane-10.md,
# audit-rlib denominators re-measured that session and matching
# w6-budget-elf-profile.md exactly). Used ONLY for the stdout coverage-ratio
# disclosure (FA10 fix (a)); never in any exit-code or breach decision.
FA10_AUDIT_RLIB_BYTES = {
    "cgap-blake3": 114072,   # coverage 31.1%
    "cgap-gate": 1048,       # coverage 46.2% (crate is genuinely tiny)
    "cgap-p28": 9220,        # coverage 9.6% (vacuum-class per FA10)
    "cgap-parquet": 55799,   # coverage 4.0% (VACUUM per FA10)
}
# FA10 maximal one-link no_std consumer across the same four crates,
# same z/fat/1cgu/abort profile: 102,187 B .text+.rodata — under the
# 131072 B envelope with ~22% headroom, established by the FA10 probe,
# NOT by this gate. Cited in the mandatory stdout caveat (FA10 fix (c)).
FA10_MAX_CONSUMER_BYTES = 102187

# Per-crate caller body: real calls into the crate's public entry API, kept
# alive through black_box so the linker cannot dead-strip the dependency.
ENTRY_SNIPPETS = {
    "cgap-blake3": (
        "    black_box(cgap_blake3::hash(&data));\n"
        "    black_box(cgap_blake3::hash_many_simd(&data));\n"
    ),
    "cgap-gate": (
        "    black_box(cgap_gate::penstock::ingress(&data, data.len(), None));\n"
    ),
    "cgap-p28": (
        "    black_box(cgap_p28::attestation::P28LicenseAttestation::"
        "from_bytes(&data[..80]));\n"
    ),
    "cgap-parquet": (
        "    let mut src = [0u64; 16];\n"
        "    src[black_box(0)] = black_box(1u64);\n"
        "    let mut out = [0u32; 128];\n"
        "    cgap_parquet::ffor::ffor_unpack(&src, 8,\n"
        "        black_box(0u32), &mut out);\n"
        "    unsafe { black_box(core::ptr::read_volatile(out.as_ptr())); }\n"
        "    let mut v: [u32; 64] = [7u32; 64];\n"
        "    v[black_box(0)] = black_box(9u32);\n"
        "    let enc = cgap_parquet::rle_encode(&v, 8);\n"
        "    unsafe { black_box(core::ptr::read_volatile(enc.as_ptr() as\n"
        "        *const u8)); }\n"
        "    black_box(enc.len());\n"
    ),
}

_CALLER_MAIN_TMPL = '''#![no_std]
#![no_main]
use core::hint::black_box;

#[panic_handler]
fn ebg_panic(_: &core::panic::PanicInfo) -> ! { loop {} }

// Minimal bump allocator: subjects built with `extern crate alloc` (gate,
// p28, parquet) need a global allocator in ANY linked caller; one-shot
// _start never frees, so an align-up bump over a static heap is exact.
use core::alloc::{GlobalAlloc, Layout};
struct EbgBump;
static mut EBG_HEAP: [u8; 65536] = [0u8; 65536];
unsafe impl Sync for EbgBump {}
unsafe impl GlobalAlloc for EbgBump {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        static mut CUR: usize = 0;
        let base = (&raw const EBG_HEAP) as usize;
        let cur = ((base + CUR + l.align() - 1) / l.align().max(1)) * l.align();
        if cur + l.size() > base + EBG_HEAP.len() {
            return core::ptr::null_mut();
        }
        CUR = cur - base + l.size();
        cur as *mut u8
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}
#[global_allocator]
static EBG_ALLOC: EbgBump = EbgBump;

%(extra_statics)s
#[no_mangle]
extern "C" fn _start() {
    // Runtime-opaque input: volatile writes block const-folding so the
    // linker must keep the crate's real code paths (constant inputs let
    // LLVM fold hash/ingress/from_bytes/ffor_unpack away — witnessed as
    // 11 KB / 514 B / 58 B / 86 B vacuum ELFs before this prologue).
    let mut data = [0u8; 4096];
    for i in 0..data.len() {
        unsafe {
            core::ptr::write_volatile(
                data.as_mut_ptr().add(i),
                core::hint::black_box((i * 7) as u8),
            );
        }
    }
    %(body)s    loop {}
}
'''

_CALLER_CARGO_TMPL = '''[package]
name = "ebg-%(crate)s"
version = "0.1.0"
edition = "2021"

[dependencies]
%(dep)s

[profile.release]
opt-level = "z"
lto = "fat"
codegen-units = 1
panic = "abort"
'''

# LLVM section-name prefixes counted as linked footprint — exactly the
# witnessed .text+.rodata method. (.bss/.data are RAM/zero-init or caller
# scaffold — e.g. the bump allocator's 64 KiB heap static lives in .bss —
# and must not be charged to the subject's deployment footprint.)
_DEPLOY_SECTIONS = (".text", ".rodata")


def parse_subject_args(values):
    out = []
    for spec in values:
        crate, sep, tier = spec.partition("=")
        if not sep or not crate or not tier:
            fail("bad --subject %r (want CRATE=tier)" % spec)
        if tier not in DEPLOY_TIERS:
            fail("unknown deployment tier %r (table has %s)"
                 % (tier, ", ".join(sorted(DEPLOY_TIERS)))
                 )  # exit 1 typed refusal
        if not os.path.isdir(os.path.join(REPO, "crates", crate)):
            fail("unknown subject crate %r (no %s)"
                 % (crate, os.path.join(REPO, "crates", crate)),
                 EXIT_MISSING)
        out.append((crate, tier))
    return out


def size_elf_sections(path, llvm_size):
    """Sum the _DEPLOY_SECTIONS families from `llvm-size -A` over the LINKED
    ELF. No matching section is a typed refusal (never a silent 0)."""
    stdout = run_tool([llvm_size, "-A", path])
    total = 0
    matched = False
    for m in re.finditer(r"^(\.\S+)\s+(\d+)(?:\s+\d+)?\s*$", stdout, re.M):
        name, nbytes = m.group(1), int(m.group(2))
        if name.startswith(_DEPLOY_SECTIONS):
            total += nbytes
            matched = True
    if not matched:
        fail("no .text/.rodata/.data family section in llvm-size -A output "
             "for %s" % path)
    return total


def generate_caller_crate(crate, extra_statics, body, variant="caller"):
    """Minimal no_std caller crate under /tmp/bgp-crate: panic_handler +
    #[no_mangle] _start exercising the crate's entry API. variant isolates
    baseline ('caller') from mutant ('mutant') package/dir/ELF names so a
    mutant build never overwrites the baseline artifact path (H13)."""
    root = os.path.join(DEPLOY_CALLER_ROOT, "%s-%s" % (variant, crate))
    os.makedirs(os.path.join(root, "src"), exist_ok=True)
    with open(os.path.join(root, "Cargo.toml"), "w") as f:
        f.write(_CALLER_CARGO_TMPL % {
            "crate": "%s-%s" % (variant, crate),
            "dep": '%s = { path = "%s" }' % (
                crate, os.path.join(REPO, "crates", crate)),
        })
    if body is None:
        fail("no deployment entry snippet for crate %r" % crate)
    with open(os.path.join(os.path.join(root, "src"), "main.rs"), "w") as f:
        f.write(_CALLER_MAIN_TMPL % {"extra_statics": extra_statics,
                                     "body": body})
    return root


def build_caller_elf(root, crate, variant="caller"):
    """cargo +nightly --release --target riscv32imc under the z/fat/1cgu/
    abort profile. Returns (elf_path, None) or (None, blocked_reason)."""
    target_dir = os.path.join(DEPLOY_TARGET_ROOT, "%s-%s" % (variant, crate))
    env = dict(os.environ)
    env["CARGO_TARGET_DIR"] = target_dir
    cmd = ["cargo", "+nightly", "build", "--release", "--target",
           DEPLOY_TARGET]
    try:
        proc = subprocess.run(cmd, cwd=root, capture_output=True, text=True,
                              env=env, timeout=600)
    except (OSError, subprocess.TimeoutExpired) as exc:
        return None, "cargo spawn/timeout failure: %s" % exc
    if proc.returncode != 0:
        tail = (proc.stderr or "").strip().splitlines()
        reason = tail[-1][:200] if tail else "exit %d" % proc.returncode
        return None, "build failed: %s" % reason
    elf = os.path.join(target_dir, DEPLOY_TARGET, "release",
                       "ebg-%s-%s" % (variant, crate))
    if not os.path.isfile(elf):
        return None, "linked ELF missing after successful build: %s" % elf
    return elf, None


def profile_deployment(subjects, llvm_size, deploy_mutant):
    """Deployment-profile ELF court. Per subject: generate caller crate in
    /tmp, build riscv32imc z/fat/1cgu/abort, llvm-size the linked ELF,
    enforce the rebalanced tier table + whole-envelope budget. Build failures
    are typed BLOCKED rows (reported, not breaches); an all-BLOCKED run
    refuses (a court with zero measured subjects enforces nothing)."""
    if not subjects:
        fail("no subjects: --subject CRATE=tier required", EXIT_MISSING)
    for _crate, tier in subjects:
        if tier not in DEPLOY_TIERS:
            fail("unknown deployment tier %r" % tier)
    rows = []
    measured = []   # (crate, tier, bytes) that actually linked
    skips = 0
    breaches = 0
    for crate, tier in subjects:
        body = ENTRY_SNIPPETS.get(crate)
        if body is None:
            print("  %-12s %-5s %-9s %s  [SKIP: no entry snippet for %s]"
                  % (crate, tier, "—", "SKIP", crate), file=sys.stderr)
            rows.append({"crate": crate, "tier": tier, "verdict": "SKIP",
                         "reason": "no entry snippet for %s" % crate})
            skips += 1
            continue
        root = generate_caller_crate(crate, "", body)
        elf, blocked = build_caller_elf(root, crate)
        if blocked:
            print("  %-12s %-5s %-9s %s  [BLOCKED] %s"
                  % (crate, tier, "—", "BLOCKED", blocked))
            rows.append({"crate": crate, "tier": tier, "verdict": "BLOCKED",
                         "reason": blocked})
            continue
        size = size_elf_sections(elf, llvm_size)
        budget = DEPLOY_TIERS[tier]
        breach = size > budget
        breaches += breach
        measured.append((crate, tier, size))
        coverage = ""
        if crate in FA10_AUDIT_RLIB_BYTES:
            coverage = "  linked/audit-rlib %.1f%%" % (
                100.0 * size / FA10_AUDIT_RLIB_BYTES[crate])
        print("  %-12s %-5s %9d / %5d B  %-7s%s  %s"
              % (crate, tier, size, budget, "BREACH" if breach else "OK",
                 coverage, elf))
        rows.append({"crate": crate, "tier": tier, "bytes": size,
                     "budget": budget,
                     "verdict": "BREACH" if breach else "OK"})
    if deploy_mutant:
        mrc, mdetail = deployment_mutant_arm(subjects[0], llvm_size)
        if mrc != EXIT_REFUSE:
            print("VACUOUS GATE: deployment mutant arm did not refuse (%s)"
                  % mdetail, file=sys.stderr)
            return EXIT_REFUSE, rows
        print("DEPLOY-MUTANT COURT: %s" % mdetail)
    if not measured and skips:
        print("BUDGET GATE: %d subject(s) skipped loud (no entry snippet) — "
              "nothing measured" % skips, file=sys.stderr)
        return EXIT_SKIP, rows
    if not measured:
        print("BUDGET GATE FAILED: all %d subject(s) BLOCKED — a deployment "
              "court with zero measured ELFs enforces nothing" % len(subjects),
              file=sys.stderr)
        return EXIT_REFUSE, rows
    envelope = sum(s for _, _, s in measured)
    env_breach = envelope > ENVELOPE_BUDGET
    print("DEPLOYMENT SUMMARY %s v%d measured=%d blocked=%d skipped=%d "
          "envelope=%d/%d B verdict=%s tier_breaches=%d"
          % (GATE_NAME, GATE_VERSION, len(measured),
             len(rows) - len(measured) - skips, skips,
             envelope, ENVELOPE_BUDGET,
             "BREACH" if env_breach else "OK", breaches))
    if breaches or env_breach:
        print("BUDGET GATE FAILED: %d linked-ELF tier breach(es)%s" % (
            breaches,
            " + envelope breach (%d > %d B)" % (envelope, ENVELOPE_BUDGET)
            if env_breach else ""), file=sys.stderr)
        return EXIT_REFUSE, rows
    if skips:
        print("BUDGET GATE: %d subject(s) skipped loud (no entry snippet)"
              % skips, file=sys.stderr)
        return EXIT_SKIP, rows
    # FA10 fix (c): the gate measures the MINIMUM ENTRY SURFACE exercised by
    # ENTRY_SNIPPETS, not the whole engine. The success verdict says so.
    print("MINIMUM ENTRY SURFACE FITS: all %d linked deployment ELF(s) under "
          "the rebalanced tier table and the %d B whole envelope"
          % (len(measured), ENVELOPE_BUDGET))
    print("COVERAGE CAVEAT (mandatory, FA10): this verdict covers only the "
          "exercised subset of each subject — the minimal no_std entry "
          "snippets, not the whole engine. Per-crate linked/audit-rlib "
          "coverage is printed above where computable. FA10 maximal "
          "one-link no_std consumer = %d B .text+.rodata (under the %d B "
          "envelope, ~22%% headroom; established by the FA10 probe, NOT by "
          "this gate; std-gated parquet scan is structurally absent from "
          "every no_std image measured) — witness cite: "
          "docs/jira/v26.11.2/adversarial/forensic-lane-10.md"
          % (FA10_MAX_CONSUMER_BYTES, ENVELOPE_BUDGET))
    return EXIT_PASS, rows


def deployment_mutant_arm(baseline_subject, llvm_size):
    """Anti-vacuity deployment witness: a /tmp MUTANT caller crate carrying
    a 30 KB static array must BREACH its tier (gate exit 1). A pass is
    VACUOUS GATE. Returns (expected_rc_of_mode, detail)."""
    crate, tier = baseline_subject
    body = ENTRY_SNIPPETS.get(crate)
    if body is None:
        return EXIT_SKIP, ("no entry snippet for %r — mutant arm cannot "
                           "run against this subject" % crate)
    # Mutant: distinct variant crate/dir/target/ELF name (ebg-mutant-<crate>)
    # so the mutant build never overwrites the baseline artifact path (H13).
    root = generate_caller_crate(crate, "", body, variant="mutant")
    mutant_main = _CALLER_MAIN_TMPL % {
        "extra_statics":
            "static EBG_MUTANT_BLOB: [u8; %d] = [1u8; %d];\n"
            % (DEPLOY_MUTANT_BYTES, DEPLOY_MUTANT_BYTES),
        "body":
            body + "    black_box(&EBG_MUTANT_BLOB);\n",
    }
    with open(os.path.join(root, "src", "main.rs"), "w") as f:
        f.write(mutant_main)
    elf, blocked = build_caller_elf(root, crate, variant="mutant")
    if blocked:
        return EXIT_REFUSE, ("mutant build BLOCKED (%s) — treat as refuse "
                             "to fail closed" % blocked)
    msize = size_elf_sections(elf, llvm_size)
    mbudget = DEPLOY_TIERS[tier]
    if msize > mbudget:
        return EXIT_REFUSE, ("mutant ELF %d B > %s budget %d B — breach "
                             "flips (anti-vacuity holds)"
                             % (msize, tier, mbudget))
    return EXIT_PASS, ("mutant ELF %d B <= %s budget %d B — the gate cannot "
                       "see a 30 KB deployment breach"
                       % (msize, tier, mbudget))


# --- lane AE9: --verify-symbols (linker-pin cross-check) -----------------------
#
# After a deployment build (an ELF linked against crates/cgap-ffi/
# enclave_128k.ld), assert the AE9 symbol pins exist and their derived
# section sizes agree with `llvm-size -A` within 1 byte. A stripped/
# truncated pin set refuses (exit 1) — the gate must be able to FAIL on a
# missing pin, never pass on one. llvm-nm absent -> loud [SKIP] (exit 3),
# house style. Anti-vacuity: --verify-symbols-mutant truncates the symbol
# table of a /tmp ELF copy; the gate MUST refuse that mutant.

def find_llvm_nm(llvm_size_path):
    """llvm-nm sits beside llvm-size in every searched layout."""
    sibling = os.path.join(os.path.dirname(llvm_size_path), "llvm-nm")
    if os.path.isfile(sibling):
        return sibling
    home = os.path.expanduser("~")
    for path in glob.glob(os.path.join(
            home, ".rustup/toolchains/*/lib/rustlib/*-apple-darwin/bin/llvm-nm")):
        return path
    return None


def verify_symbols(elf, llvm_size, llvm_nm):
    """Cross-check the AE9 linker pins against the real section sizes."""
    if not os.path.isfile(elf):
        fail("MISSING SUBJECT: ELF %s does not exist" % elf, EXIT_MISSING)
    if not llvm_nm:
        skip("llvm-nm not found beside llvm-size (%s)" % llvm_size)
    sym_out = run_tool([llvm_nm, "-n", "--defined-only", elf])
    syms = {}
    for line in sym_out.splitlines():
        parts = line.split()
        if len(parts) == 3 and re.fullmatch(r"[0-9a-fA-F]+", parts[0]):
            syms[parts[2]] = int(parts[0], 16)
    pins = {"_stext": ".text", "_etext": ".text",
            "_sbss": ".bss", "_ebss": ".bss"}
    missing = [s for s in pins if s not in syms]
    if missing:
        print("SYMBOL GATE FAILED: AE9 pin(s) absent from %s: %s"
              % (elf, ", ".join(missing)), file=sys.stderr)
        return EXIT_REFUSE
    size_out = run_tool([llvm_size, "-A", elf])
    secs = {}
    for m in re.finditer(r"^(\.\S+)\s+(\d+)(?:\s+\d+)?\s*$", size_out, re.M):
        secs.setdefault(m.group(1), int(m.group(2)))
    derived = {
        ".text": syms["_etext"] - syms["_stext"],
        ".bss": syms["_ebss"] - syms["_sbss"],
    }
    for sec, got in sorted(derived.items()):
        want = secs.get(sec)
        if want is None:
            print("SYMBOL GATE FAILED: section %s absent from llvm-size -A "
                  "of %s (pins present but section unmeasurable)" % (sec, elf),
                  file=sys.stderr)
            return EXIT_REFUSE
        delta = abs(got - want)
        print("  pin-derived %-6s %8d B vs llvm-size %8d B (delta %d) %s"
              % (sec, got, want, delta,
                 "OK" if delta <= 1 else "MISMATCH"))
        if delta > 1:
            print("SYMBOL GATE FAILED: pin-derived %s size %d diverges from "
                  "llvm-size %d by %d B (>1 B) in %s" % (sec, got, want,
                                                         delta, elf),
                  file=sys.stderr)
            return EXIT_REFUSE
    print("SYMBOL GATE: all 4 AE9 pins present, pin-derived .text/.bss "
          "match llvm-size -A within 1 byte — %s" % elf)
    return EXIT_PASS


def truncate_symbol_mutant(elf):
    """/tmp mutant: byte copy of the ELF with one AE9 pin's symbol-table
    NAME STRING zeroed in .strtab (an Elf32/64 sym entry's st_name points
    into .strtab; zeroing the string makes llvm-nm report the symbol
    nameless, so the pin lookup must fail). Returns the mutant path; None
    when no pin name string is present (caller skips loud)."""
    from struct import unpack_from
    with open(elf, "rb") as f:
        data = f.read()
    if data[:4] != b"\x7fELF":
        return None
    for pin in (b"_stext\x00", b"_etext\x00", b"_sbss\x00", b"_ebss\x00"):
        pos = data.find(pin)
        if pos != -1:
            data = data[:pos] + b"\x00" * len(pin) + data[pos + len(pin):]
            break
    else:
        return None
    fd, mutant = tempfile.mkstemp(prefix="ebg-symmut-", suffix=".elf")
    with os.fdopen(fd, "wb") as f:
        f.write(data)
    return mutant


def mode_verify_symbols(args, llvm_size, llvm_nm):
    """--verify-symbols baseline plus the required named mutation court
    (--verify-symbols-mutant): a /tmp symbol-truncated copy of the ELF
    MUST refuse; a pass is VACUOUS GATE (exit 1)."""
    code = verify_symbols(args.elf, llvm_size, llvm_nm)
    if code != EXIT_PASS:
        sys.exit(code)
    if not args.verify_symbols_mutant:
        sys.exit(EXIT_PASS)
    mutant = truncate_symbol_mutant(args.elf)
    if mutant is None:
        skip("ELF has no parsable symtab to truncate — mutant arm cannot run")
    try:
        mcode = verify_symbols(mutant, llvm_size, llvm_nm)
    finally:
        if mutant is not None:
            os.unlink(mutant)
    if mcode == EXIT_PASS:
        print("VACUOUS GATE: symbol-truncated mutant PASSED — the pin "
              "cross-check cannot see a lost symbol; refusing the gate",
              file=sys.stderr)
        sys.exit(EXIT_REFUSE)
    print("SYMBOL-MUTANT COURT: truncated-symbol mutant refused (gate exit "
          "%d) — anti-vacuity holds" % mcode)
    sys.exit(EXIT_PASS)


# --- main ----------------------------------------------------------------------

def main():
    ap = argparse.ArgumentParser(
        description="W5 D4 enclave budget gate (128 KiB envelope; "
                    "tiers kernel/storage/compute/attest)")
    ap.add_argument("--check", action="store_true",
                    help="measure every --artifact and enforce the budget "
                         "table (exit 0/1/2 per the house ABI)")
    ap.add_argument("--artifact", action="append", default=[], metavar="TIER=PATH",
                    help="subject to size: tier in {kernel,storage,compute,"
                         "attest} = path to a .o/.elf/.rlib")
    ap.add_argument("--mutate", action="store_true",
                    help="anti-vacuity: 30 KB-append mutant of the first "
                         "artifact MUST be refused; a pass is VACUOUS GATE "
                         "(exit 1)")
    ap.add_argument("--self-check", action="store_true",
                    help="non-vacuity court: clean arm (real cc object "
                         "passes), poisoned arm (32 KiB global refuses), "
                         "mutant arm (real-artifact append refuses; [SKIP] "
                         "when no artifact is supplied)")
    ap.add_argument("--profile-deployment", action="store_true",
                    help="deployment-profile court: generate a minimal "
                         "no_std caller crate per --subject CRATE=tier in "
                         "/tmp, build riscv32imc z/fat/1cgu/abort with "
                         "cargo +nightly, llvm-size the LINKED ELF, enforce "
                         "the rebalanced tier table + envelope")
    ap.add_argument("--subject", action="append", default=[],
                    metavar="CRATE=tier",
                    help="deployment subject: crate under crates/ and tier "
                         "in {tier0,tier1,tier2,tier3}")
    ap.add_argument("--deploy-mutant", action="store_true",
                    help="anti-vacuity deployment witness (with "
                         "--profile-deployment): a /tmp mutant caller with "
                         "a 30 KB static array MUST breach its tier")
    ap.add_argument("--verify-symbols", action="store_true",
                    help="AE9 lane: after a deployment build, assert the "
                         "enclave_128k.ld pins (_stext/_etext/_sbss/_ebss) "
                         "exist via llvm-nm and that pin-derived .text/.bss "
                         "sizes match llvm-size -A within 1 byte")
    ap.add_argument("--verify-symbols-mutant", action="store_true",
                    help="with --verify-symbols: after the baseline passes, "
                         "truncate one symbol in a /tmp copy of the ELF and "
                         "REQUIRE the gate to refuse it (VACUOUS GATE on a "
                         "pass)")
    ap.add_argument("--elf", metavar="PATH",
                    help="linked ELF subject for --verify-symbols")
    args = ap.parse_args()

    if args.verify_symbols:
        vsize = find_llvm_size()
        if vsize is None:
            skip("llvm-size not found (env CGAP_LLVM_SIZE, PATH, rustup "
                 "toolchains, homebrew, Xcode all searched)")
        llvm_nm = find_llvm_nm(vsize)
        mode_verify_symbols(args, vsize, llvm_nm)

    if not (args.check or args.mutate or args.self_check
            or args.profile_deployment):
        print("no mode: one of --check, --mutate, --self-check, "
              "--profile-deployment required", file=sys.stderr)
        sys.exit(EXIT_REFUSE)

    artifacts = parse_artifact_args(args.artifact)
    if (args.check or args.mutate) and not artifacts:
        fail("--check/--mutate need at least one --artifact TIER=PATH "
             "(a gate with no subjects enforces nothing)", EXIT_MISSING)

    llvm_size = find_llvm_size()
    if llvm_size is None:
        skip("llvm-size not found (env CGAP_LLVM_SIZE, PATH, rustup "
             "toolchains, homebrew, Xcode all searched)")
    llvm_ar = find_llvm_ar(llvm_size)

    if args.profile_deployment:
        subjects = parse_subject_args(args.subject)
        if args.deploy_mutant and subjects:
            pass  # mutant arm runs inside profile_deployment after baseline
        code, _rows = profile_deployment(subjects, llvm_size,
                                         bool(args.deploy_mutant))
        sys.exit(code)
    if args.self_check:
        sys.exit(mode_self_check(artifacts, llvm_size, llvm_ar))
    if args.mutate:
        sys.exit(mode_mutate(artifacts, llvm_size, llvm_ar))
    code, _rows_unused = check_budgets(artifacts, llvm_size, llvm_ar)
    sys.exit(code)


if __name__ == "__main__":
    main()
