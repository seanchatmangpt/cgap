# AU2 — Branchless algorithmic complexity (Phase 1: measured facts)

Date 2026-11-02. Repo /Users/sac/cgap @ main. Scope: cgap-branchless (8 modules),
cgap-gate penstock primitives, cgap-core isomorphism/dma_ring. E9/E10 LSP machinery is a
separate repo — excluded per lane charter.
## 1. Inventory — cgap-branchless (crates/cgap-branchless/src/, 2,240 LOC)
`#![no_std]` + `#![forbid(unsafe_code)]` (lib.rs:15-16). 28 public fns measured.
Families: SWAR, cmov/select, eytzinger, radix, transpose, Kogge-Stone scan networks,
multiplicative fixed-point mix. No floating-point anywhere.
| fn (file:line) | LOC | family |
|---|---|---|
| arithmetic::saturated_subs_p28 / poison_mask (arithmetic.rs:21,39) | 17/10 | SWAR mask; mask fixed-point |
| select::cmov_u64 / select_slice / checked_uniform_select / pred_inc (select.rs:44,59,80,96) | 14/20/15/8 | cmov family |
| swar::swar_lt / eq_u64 / has_zero_byte / range_mask / splat (swar.rs:33,44,53,60,69) | 10/8/6/8/4 | SWAR |
| eytzinger::next / restore / search / to_layout (eytzinger.rs:43,55,71,102) | 11/15/30/35 | eytzinger (van Emde Boas) |
| radix::radix_bucket_12 (radix.rs:45) | 18 | 12-bit radix bucket fold |
| transpose::transpose_8x8 / transpose_64x64 (transpose.rs:46,59) | 12/30 | bit-matrix dswap transpose |
| scan::prefix_or / and / xor_u64x8 (scan.rs:20,45,70) | 24/24/24 | Kogge-Stone network |
| scan::prefix_sum_u32x8 / segmented_prefix_sum_u32x8 (scan.rs:94,120) | 25/39 | Kogge-Stone; segmented scan |
| scan::prefix_max_u8x8 / count_leading_eq_u8 (scan.rs:160,175) | 14/10 | prefix; SWAR-prefix hybrid |
| penstock::prefilter8_mix / prefilter8 / swar_all_zero_vl (penstock.rs:35,44,64) | 4/14/55 | mult. mix (w^(w>>7))*MIX; SWAR var-len zero detect |
## 2. Verification investment (courts attached)
Test LOC in cgap-branchless: branchless_census.rs 750 + scan_courts.rs 487 +
transpose_courts.rs 516 = 1,753 vs 2,240 src (~0.78:1); plus 34 in-module #[cfg(test)]
units (arith 3, select 7, swar 4, eytz 6, radix 8, scan 1, penstock 5); 19 external
#[test]s (census 4, scan 8, transpose 7).
- Census courts: per-symbol llvm-objdump disassembly census, host-aarch64 AND riscv32imc,
  asserting EXACT per-target branch counts (26-cell grid).
- Differential-oracle courts: naive scalar oracles (scan_courts.rs:40-114;
  transpose_courts.rs:30-52) over boundary + seeded-splitmix corpora; 7 scan courts.
- Property courts: transpose involution + stage self-inverse (transpose_courts.rs:156,175);
  Gray-code property on prefix_xor.
- Inline-site census (lane INL): caller probes filter_lane / swar_scan / eytzinger_probe.
- Sandbox-twin courts: unsafe 0-branch mask-form select_slice as /tmp twin, both targets.
- Anti-vacuity mutation courts, each asserted to FLIP else VACUOUS panic: M-REP1-1
  (if-injection into select.rs sandbox copy -> BRANCHY 11x), M-INL-1 (data-dependent
  branch + black_box into caller probe), M-TS2-1 (runtime-bound mutation, scan.rs sandbox),
  M-TS4-1 (stage-7 mask bit-flip, differential court flips), M-AE4-1 (if-injection into
  cmov_u64; lane-04 receipt).
- cgap-gate penstock: 12 #[cfg(test)] courts in src/penstock.rs (taxonomy, order-pin,
  content-independence, zero-alloc allocator-hook witness, ratio fence) + 3 in
  tests/lever_courts.rs; §8.3 anti-vacuity matrix re-executed per wave with sha256 restore.
- cgap-core: unit_dma_ring 13, identity_court 4, guard_flip_court 11, rv32imc_court 2,
  source_scan_gate 8, cross_lane_gate 8, ingress_layout 6, static_assert 4,
  doc_offset_table 2, manifest 3.
- Compile-fail: NO trybuild/compile_fail machinery in these crates (zero grep hits); ABI
  pinning is runtime/static-assert courts. No proptest; corpora seeded in-house.
## 3. ISA-ceiling analysis (REP1): measured facts
Claim: every RV32 (riscv32imc) 2-way select lowers to a branch (bnez) regardless of
source form; `--disable-cgp-select2branch` does not change it. Receipts:
- repairs/repair-01-branchless-census.md:41-46,67-76 — cmov_u64 `if`/select form emitted
  bnez; the arithmetic mask form (0u64.wrapping_sub) FOLDED BACK to a select and emitted
  the SAME 1 bnez; flag flip witnessed no-change; recorded in census allowed() table.
- convergence/lane-11-branchless-census-witness.md:58-64 — residual table: "rv32imc has
  no Zicond, ISel lowers every 2-way select to a branch"; verbatim llvm-objdump witnesses
  (host select_slice = cbz + 2x csel; riscv cmov_u64 = 1 bnez in both source forms).
- exhaustive/lane-03-branchless-complete.md:30 — host PASS (cmp+csel, 0 br) vs riscv
  BRANCHY (1x bnez), classified "ISA ceiling, not a source defect".
- countermeasures/lane-04-disassembly-court.md:54-78 — per-symbol disassembly court
  re-witnessed; naive_if_cmov self-check flagged BRANCHY.
Statement of record: the 0-branch invariant was achieved on host (aarch64 csel) but on
RV32 the residual is an ISA ceiling — riscv32imc has no conditional-move instruction, so
instruction selection converts EVERY 2-way select into a conditional branch no matter
what the Rust source says. Proving this required empirical disassembly of both source
forms plus the compiler-flag flip; source inspection cannot distinguish "author wrote a
branchy form" from "ISel manufactured a branch out of a branchless form." The census
asserts exact residual counts (1/1, 2/2) rather than faking PASS; the 0-branch unsafe
mask form is proven as a sandbox twin, blocked in-tree only by #![forbid(unsafe_code)]
(lib.rs:16).
## 4. ANALYSIS (labeled analysis — interpretation, not measurement)
1. Compiler-backend knowledge. Correctness is defined at the assembly level: a fn is
   "done" only when objdump shows residuals at exactly their documented counts. The
   cmov_u64 episode shows source-level reasoning is not just insufficient but actively
   misleading — the intuitively "more branchless" arithmetic form produces IDENTICAL
   branch codegen. Maintainers must reason about LLVM IR folding and ISel.
2. ISA ISel behavior. Target ceilings (no Zicond on rv32imc; unmasked variable shifts
   forcing a beqz guard in eytzinger_restore) are load-bearing documentation — the
   residual table is part of the contract. Knowledge normally held by compiler engineers.
3. Differential-oracle discipline. Every bit-trick kernel is pinned by a naive scalar
   oracle + property (involution, Gray code) + an anti-vacuity mutant asserted to flip —
   the suite checks its own teeth (second-order verification, courts on courts). That
   machinery (~0.78 test:src plus sandbox-twin compiles inside cargo test) is the
   dominant LOC cost and does not exist in conventional library development.
4. Contrast: ordinary Rust library work stops at the API/behavior contract verifiable
   from source; here every fn carries (a) a per-target disassembly budget, (b) a naive
   oracle, and (c) a named mutant class whose flip is asserted. The marginal machinery
   (llvm-objdump plumbing, riscv32 no_std census builds, sandbox twins) is the substance
   of this lane's complexity claim. Person-month quantification: Phase 2 per charter.
## Sources
crates/cgap-branchless/src/*.rs; tests/{branchless_census,scan_courts,transpose_courts}.rs; crates/cgap-gate/src/penstock.rs; crates/cgap-core/src/{isomorphism,dma_ring}.rs; docs/jira/v26.11.2/{repairs/repair-01-branchless-census.md, convergence/lane-11-branchless-census-witness.md, exhaustive/lane-03-branchless-complete.md, countermeasures/lane-04-disassembly-court.md, countermeasures/lane-inl-census.md}
