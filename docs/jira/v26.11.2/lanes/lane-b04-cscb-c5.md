# Lane B04 — csct C5 coordinate: physical SRAM budget inspector

Date: 2026-11-02. Standing: **ALIVE** (every claim below re-run this
session; commands + exits cited inline).

## Files written (lane-owned, disjoint)

- `/Users/sac/cgap/scripts/verify_cscb_sram.py` (327 lines, python3
  stdlib only; sha256 prefix `6b06c2f73b12b573`, 13,585 B on disk)

## What was built

`verify_cscb_sram.py` — physical SRAM gate for the csct/branchless
kernel surface, exit ABI 0 pass / 1 overflow / 2 subject-missing / 3
loud-skip. Builds the /tmp no_std caller crate (#![no_std], #![no_main],
panic_handler, bump allocator, black_box-ed prologue) around a real
kernel surface — `cgap_blake3::hash` + `hash_many_simd` (seal) and
`cgap_branchless::swar::swar_range_mask` + `swar_lt` (SWAR filter frame)
— targeting riscv32imc-unknown-none-elf under opt-level=z / lto=fat /
codegen-units=1 / panic=abort, then measures the LINKED ELF with
`llvm-size -A` over the `.text+.rodata+.data+.bss` section families and
gates ≤ 131,072 B. Recipe copied from the working precedent
`scripts/enclave_budget_gate.py` (--profile-deployment).

## Commands + exits (this session)

| command | exit |
|---|---|
| `python3 scripts/verify_cscb_sram.py` (final baseline) | 0 |
| `python3 scripts/verify_cscb_sram.py --mutate=sram-bloat` (final court) | 0 |
| earlier mutation-court runs that failed to flip (36,048 B == baseline, dead-stripped static) | 1 (VACUOUS GATE correctly refused) |

## Measured sizes (linked ELFs, .text+.rodata+.data+.bss)

- Baseline: **36,048 B** ≤ 131,072 B hard bound — PASS, headroom 95,024 B.
- Empirical-baseline comparison (report-only, NOT gated): 39,184 B
  baseline → 36,048 B measured, delta −3,136 B. The 39,184 B figure is
  the OVERNIGHT-LOOP empirical riscv32imc freestanding witness; treated
  as hearsay-adjacent, cited not gated.
- sram-bloat mutant: **136,068 B** > 131,072 B — overflow flips,
  anti-vacuity holds.
- First mutant attempt was a witnessed VACUOUS PASS: the static
  `[u8;100_000]` was unreferenced and dead-stripped (36,048 B ==
  baseline). Fixed with a volatile read of the static in `_start`; the
  court now flips by exactly the +100,048 B the mutation injects.
- Stack: max observed `addi sp,sp,-N` leaf frame = 2,048 B (hex-encoding
  fixed after a first pass that parsed 0 — recorded UNVERIFIED as a
  worst-case chain bound; no runtime watermark claimed).

## Toolchain + target spec (recorded)

- rustc/cargo 1.101.0-nightly (12433c307 / cad07cf8e, 2026-10-23/24),
  per repo `rust-toolchain.toml` (floating nightly pin).
- Target `riscv32imc-unknown-none-elf` (rustup: installed).
- llvm-size/llvm-objdump: `/opt/homebrew/opt/llvm/bin/`.
- Build roots `/tmp/cscb-sram-{caller,target}` (lane leases, deleted at
  lane end).

## Divergences + honest standing

1. First mutant flip failure was real and disclosed: an unreferenced
   static is dead-stripped at -Oz — the injected-SRAM court needs a
   volatile touch, not just a declaration. That VACUOUS PASS was refused
   (exit 1) by the gate itself before the fix.
2. Stack bound is UNVERIFIED as a chain bound — static objdump scan
   only, honest output line in the gate, no fabrication.
3. Measured surface is the MINIMUM ENTRY SURFACE exercised by the
   caller (enclave-gate coverage caveat applies verbatim); the 128 KiB
   verdict does not cover the whole engine.
4. 39,184 B baseline comparison is report-only per dispatch; the gate
   is the 131,072 B hard bound.
