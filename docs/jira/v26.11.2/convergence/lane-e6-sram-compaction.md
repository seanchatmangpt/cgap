# Lane E6 — enclave SRAM/binary compaction study

Date: 2026-11-02. Lane: read-only + /tmp A/B. Shared tree untouched except
this receipt. Build root `/tmp/e6-target` (lease, deleted at lane end).

## 1. Baseline (`--profile-deployment`, 4 subjects, exit 0)

Command: `python3 scripts/enclave_budget_gate.py --profile-deployment
--deploy-mutant --subject cgap-blake3=tier0 --subject cgap-gate=tier0
--subject cgap-p28=tier0 --subject cgap-parquet=tier2`

| subject     | tier  | linked .text+.rodata | budget  | verdict | linked/audit-rlib |
|-------------|-------|----------------------|---------|---------|-------------------|
| cgap-blake3 | tier0 | 35,440 B             | 49,152  | OK      | 31.1%             |
| cgap-gate   | tier0 | 606 B                | 49,152  | OK      | 57.8%             |
| cgap-p28    | tier0 | 886 B                | 49,152  | OK      | 9.6%              |
| cgap-parquet| tier2 | 2,252 B              | 24,576  | OK      | 4.0%              |

Envelope: 39,184 / 131,072 B (the 39.2 KB entry surface), verdict OK,
`tier_breaches=0`. Anti-vacuity deploy-mutant court held: mutant ELF
66,176 B > 49,152 B tier0 budget — breach flips (gate exit 0, run this
session).

## 2. Symbol census (llvm-nm --size-sort --print-size)

- cgap-blake3: dominant symbol `_RNvCsafoCUaRwdzP_11cgap_blake317
  hash_many_simd_x2` = 0x76ec = **30,444 B** (85.9% of the 35,440 B ELF).
  Next largest: compiler_builtins memcpy 0x186=390 B, _start 0x80=128 B,
  then OUTLINED_FUNCTION_* ≤ 0x9e=158 B and .rodata ≤ 0x48. Expectation
  ("~30 KB dominant") confirmed exactly.
- cgap-parquet: largest .text symbol `_start` = 0x4be = 1,214 B; crate
  symbols ≤ 0xa4 (RawVec finish_grow). Entry surface 2,252 B confirmed —
  the crate is functionally tiny; almost all of _start is the caller
  prologue (volatile-fill loop + bump allocator), not parquet code.
- cgap-gate (606 B) and cgap-p28 (886 B) have no symbol over 0x1d0=464 B.
  p28's largest crate symbol: from_bytes 0xea = 234 B.

## 3. A/B: #[inline] annotation on hash_many_simd_x2 (/tmp sandboxes)

Sandbox copies of `crates/cgap-blake3` at `/tmp/e6-blake3-{never,always}`,
manifest de-workspaced, single attribute added on `pub fn
hash_many_simd_x2`; caller crates `/tmp/e6-caller-{never,always}` with the
identical gate-generated main.rs, riscv32imc z/fat/1cgu/abort.

| arm             | text+rodata | hash_many_simd_x2 symbol |
|-----------------|-------------|--------------------------|
| baseline (none) | 35,440 B    | 0x76ec = 30,444 B        |
| inline(never)   | 35,440 B    | 0x76ec = 30,444 B        |
| inline(always)  | 35,440 B    | 0x76ec = 30,444 B        |

Delta: **0 B across all three arms.** Under fat LTO with a single caller,
LLVM already inlines `hash` → `hash_many_simd` → `hash_many_simd_x2` and
folds them into one symbol (the always-arm symbol is even renamed
`hash_many_simd` post-inline); there is one call site, so (never) changes
nothing and (always) changes nothing. The 30.4 KB is the riscv32imc
scalar/vector-lowering cost of the real 8-way chunk pipeline — code, not
table.

## 4. .bss/.data audit

`llvm-size -A` over all four ELFs: only cgap-parquet carries .bss, and it
is exactly **65,540 B** = 65,536 B `EBG_HEAP` (caller bump-heap static,
symbol `_RNvCsfZWKCbzSX4Q_23ebg_caller_cgap_parquet8EBG_HEAP` — proven
caller-side, defined in the gate-generated caller crate, not the crate) +
4 B residual. cgap-blake3/gate/p28 carry zero .bss/.data. No crate-side
static lookup table spills any tier budget; the gate charges only
.text/.rodata, correctly excluding caller scaffold.

## 5. Verdict

**Functionally floored.** The 39.2 KB entry surface is NOT reducible ≥5%
by annotation alone:

- blake3's 30,444 B symbol is 77.7% of the whole envelope and is
  invariant to #[inline] annotations (0 B delta measured). Any real
  reduction would require semantic change (e.g. dropping the x2 dual-group
  pipeline or a smaller compression profile), which is a functional
  tradeoff, not compaction.
- parquet (2,252 B) and gate (606 B) are already near-floor; p28 (886 B)
  likewise. Even zeroing gate+p28+parquet entirely would be a 7.5%
  envelope reduction, below the combined bar once the floor is blake3.
- .bss is caller-side and outside the charged footprint.

Follow-up landing is therefore not warranted: no winner to land (delta 0%).

## Receipt

- Commands + exits: baseline gate exit 0 (with deploy-mutant court pass);
  llvm-nm/llvm-size runs exit 0; A/B builds exit 0 (never rc 0, always rc 0).
- Subject: linked ELFs at `/tmp/bgp-target/caller-*/riscv32imc-unknown-none-elf/release/`.
- Shared tree: only this receipt file written. Sandbox crates under /tmp.
- Standing: ALIVE (all commands + exits run this session).
