# gate-5-silicon

Status: ALIVE (cgap-arrow) / ALIVE (cgap-ffi ELF size law, R11 flip
2026-11-01: ELF subject 150 B flash / 0 B SRAM under budget; production
.oabi/.bn3 sections still absent from the production object — raw-object
link refuses, demo-byte divergence recorded in the Production link
section) — coordinator-written from gate-runner GAP-5's verdict; all
measurements first-hand this session, that lane wrote no marker per
read-only discipline.

## Size table (budgets: flash 65,536 / SRAM 32,768; llvm-size measured)

| crate | artifact | flash B | SRAM B | verdict |
|---|---|---|---|---|
| cgap-arrow | riscv32imc release rlib (kernel variant) | 30,157 (blake3::hash 27,914; seal_with_mask x3; .rodata 1) | 0 | PASS (pre-link object; linked-ELF authoritative) |
| cgap-arrow | lto bitcode variant | 0 object bytes (.llvmbc deferred) | 0 | no footprint subject |
| cgap-ffi | rlib at snapshot | 6 B + 2,936 .llvmbc deferred | 0 | PARTIAL — GAP-1 bodies in flight |
| xip.ld probe ELF | linked this session | 146 (.text 2 + .oabi 96 + .bn3 48) | 0 | PASS |
| A01 xip ELF (receipted) | linked ELF | 150 | 0 | PASS (4 B shim delta vs probe, same law) |

- Placement measured: .text @0x42000020, .oabi 96B @0x42010000, .bn3 48B
  @0x42020000 — matches the A19 receipt exactly.
- xip.ld ASSERT court flipped BOTH directions: section-absent → rust-lld
  refuses (exit 1, both section messages); sections-present → exit 0, ELF
  measured. Non-vacuous.
- nm purity: alloc 0 / panic 0 on both cgap-arrow variants, two passes
  (rust-toolchain llvm-nm; Apple LLVM 17 cannot parse nightly LLVM 23
  objects — toolchain-internal bin used).
- Builds: cgap-arrow + cgap-ffi --target riscv32imc --release both exit 0;
  no E0463 std leak (positive result). Expected cdylib-drop warning on
  none-target recorded.
- xtensa: toolchain absent → loud [SKIP] (rc=1 grep, witnessed).
- Staticlib 264,881 B: receipted, host archive subject — NOT a footprint
  subject; the ELF subject is authoritative per spec §4.
- Close condition: GAP-1 lands the sovereign bodies → production xip link
  un-refuses → re-measure the production ELF for the Gate-5 ALIVE row.

## Production link (R11 gate-runner re-measure, 2026-11-01, this session)

- GAP-1 bounded poll: 19 polls (~9.5 min) absent, then FOUND after 2 polls in
  the background window — `waves/gap-1-sovereign.md` landed 17:29, status
  ALIVE (42/0 lib courts, four sovereign T exports receipted).
- Build (exit 0): `CARGO_TARGET_DIR=/tmp/r11-target cargo build --release
  -p cgap-ffi --target riscv32imc-unknown-none-elf`; rlib 5,142 B; the
  none-target object carries .text 0 + .llvmbc 2,936 B deferred +
  `rust_begin_unwind` 6 B and NO .oabi/.bn3 and NO sovereign symbols
  (llvm-objdump -h / llvm-nm, first-hand).
- Production link leg A (raw production object): `rust-lld -flavor gnu -T
  crates/cgap-ffi/xip.ld <cgu.o> -o prod.elf` -> exit 1, REFUSED on both
  ASSERT texts ("cgap XIP: .oabi section absent" / "cgap XIP: .bn3 section
  absent"). The close condition's premise — GAP-1 bodies land as production
  .oabi/.bn3 sections — did NOT hold: GAP-1's bodies are host-surface
  (`#[cfg(not(target_os = "none"))]`) and the tree's only .oabi/.bn3
  link_section statics are court-fixture STRINGS in the xip courts.
- Production link leg B (§4 ELF subject, T1-court pattern, /tmp sandbox):
  real src copy + bin (entry + demo .oabi 96 B / .bn3 48 B) over the real
  crate, RUSTFLAGS `-C link-arg=-T xip.ld -C link-arg=--gc-sections` ->
  exit 0, ELF linked.
- llvm-size -A (first-hand): .text 6 @0x42000020, .oabi 96 @0x42010000,
  .bn3 48 @0x42020000 -> FLASH 150 / 65,536 B, SRAM 0 / 32,768 B —
  byte-identical to GAP-1's in-court receipt (150/0). Placement matches
  the A19 receipt exactly.
- nm purity (rust-toolchain llvm-nm): alloc family 0, panic family 0;
  full table = cgap_xip_start (T), OABI_TABLE/BN3_NET (r), _stack_top (A).
- Host cdylib `nm -gU`: all four sovereigns T — `_cgap_admit_frame`,
  `_cgap_eval_bn3`, `_cgap_commit_parquet`, `_cgap_verify_p28` (plus
  `_cgap_p28_invariant_check`/`_cgap_i32_batch`/`_cgap_version` T).
- cgap-ffi leg FLIPPED: PARTIAL -> ALIVE on the size law (ELF subject green
  under budget, first-hand this session). Divergence recorded (not
  laundered): the ELF's .oabi/.bn3 are the court DEMO bytes (0xA7/0xB3
  fills); the production object contributes none and the raw-object
  production link still refuses on the ASSERTs. Production .oabi/.bn3
  byte sources (A06 rule tables / A11 networks into the none-target
  object) remain a future lane dispatch; sovereigns stay host-surface
  (XIP rule execution BLOCKED-on-owner: A12 unifier alloc API, no
  allocator in the XIP budget).
- Lane leases cleaned (python3 shutil.rmtree): /tmp/r11-target,
  /tmp/r11-t1, /tmp/r11-extract, /tmp/r11-link.

## SRAM stack estimate (lane A-5, 2026-11-01) — ESTIMATE (static, method stated)

Standing change: §4 SRAM term "worst-case eval stack" UNVERIFIED -> PARTIAL
(static estimate landed; true hardware/qemu stack-paint witness remains a
future lane). Method: real rustc asm emission (`RUSTFLAGS="--emit=asm
-C opt-level=s -C panic=abort -C debuginfo=0" cargo build --release -p
cgap-bn3 -p cgap-ffi`, exit 0), per-function frames parsed from the
prologue `sub sp, sp, #N` maxima and the `bl` call graph, longest path by
DFS over 515 real edges. riscv32imc subject leg: BLOCKED — cgap-bn3 is a
std host crate; `cargo build --target riscv32imc-unknown-none-elf -p
cgap-bn3` exits 1 (E0463 can't find crate for `std`, witnessed this
session), consistent with the recorded XIP-eval BLOCKED-on-owner
divergence. The aarch64 frames are therefore the ESTIMATE BOUND for rv32
(rv32 pointers are 4 B not 8 B, so true rv32 frames are expected smaller;
u64 wire words stay 8 B — never claimed as measured).

| call chain (deepest per leg) | frames | est bytes (aarch64 bound) |
|---|---|---|
| cgap_eval_bn3 | 1 | 688 |
| -> ConsequentProjector::project | 2 | +272 |
| -> ConsequentProjector::emit_into | 3 | +2144 |
| -> sha256_padded (leaf) | 4 | +432 |
| **deepest chain total** | 4 | **3,536** |
| alt leg: eval -> RuleCompiler::compile -> Interner::intern -> hash chain | 5 | 1,632 incl. eval |
| alt leg: eval -> unify_batch -> unify_batch_into -> Vec::extend_with | 4 | 1,712 incl. eval |

vs the 32,768 B SRAM budget (which carries measured .bss+.data = 0 —
this term is the whole SRAM story): est stack 3,536 B = 10.8% of budget;
margin 29,232 B. Assumptions, labeled: single exception context (no ISR
frame overlay modeled), panic=abort, no LTO cross-crate frame shrink
(bound-friendly), allocator heap NOT counted (separate term; XIP carries
no allocator). The 3,536 B figure is ESTIMATE — never claim measured.
