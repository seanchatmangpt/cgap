# LANE EX3 — cgap-branchless exhaustive census (complete, all pub fns)

Lane EX3, 2026-11-02, read-only lane (sole write: this file). Method: cargo +nightly
`RUSTFLAGS="--emit=asm -C opt-level=3 -C link-dead-code=yes -C debuginfo=0"`
build of `crates/cgap-branchless` (zero deps) to `/tmp/ex3-target`; per-symbol
instruction census of the emitted asm, host aarch64-apple-darwin and
riscv32imc-unknown-none-elf (rustup nightly). RVC scan = explicit grep for
`c.`-prefixed mnemonics (c.beqz/c.bnez/c.j/c.jal — the FA6 classifier gap).

## Census divergence vs dispatch

FA6 counted 16 pub fns; a source grep this session enumerates **19** pub fns
(arithmetic 2, eytzinger 4, penstock 3, radix 1, select 4, swar 5). All 19 are
censused below; the 16-row grid undercounted by 3 (penstock_prefilter8_mix,
swar_lt, swar_range_mask class). Private helper `padded_at` (eytzinger) is
censused for completeness but is not a pub-fn row.

## Symbol census (19 rows)

| fn | host (aarch64) verdict | riscv32imc verdict | RVC found | consumers | classification |
|---|---|---|---|---|---|
| arithmetic::saturated_subs_p28 | PASS (9 insns, 0 br) | PASS (19 insns, 0 br) | 0 | cgap-p28 | PURE-ALU — confirms RV1 repair (witnessed 0 branches this emit) |
| arithmetic::poison_mask | PASS (3, 0 br) | PASS (5, 0 br) | 0 | cgap-p28 | PURE-ALU |
| swar::swar_lt | PASS (10, 0 br) | PASS (27, 0 br) | 0 | swar_range_mask | PURE-ALU |
| swar::swar_eq_u64 | PASS (10, 0 br) | PASS (25, 0 br) | 0 | penstock, gate | PURE-ALU |
| swar::swar_has_zero_byte | PASS (8, 0 br) | PASS (18, 0 br) | 0 | penstock | PURE-ALU |
| swar::swar_range_mask | PASS (25, 0 br) | PASS (52, 0 br) | 0 | gate hot path | PURE-ALU |
| swar::swar_splat | PASS (4, 0 br) | PASS (5, 0 br) | 0 | swar_range_mask | PURE-ALU |
| radix::radix_bucket_12 | PASS (3, 0 br) | PASS (5, 0 br) | 0 | partitioned scans | PURE-ALU |
| select::cmov_u64 | PASS (cmp+csel, 0 br) | BRANCHY (1× bnez; RV32 has no cmov) | 0 | select helpers | AS-OF-IN-FLIGHT (REP1 owns select.rs); host PASS / riscv branchy is an ISA ceiling, not a source defect |
| select::pred_inc | PASS (5, 0 br) | PASS (5, 0 br) | 0 | gate counters | PURE-ALU |
| select::select_slice | BRANCHY (1× cbz, len==0 guard before cond[0] load; rest csel) | BRANCHY (2: beqz len-guard + bnez cond byte) | 0 | gate penstock | AS-OF-IN-FLIGHT (REP1); matches FA6 grid "residual cbz 18/19" |
| select::checked_uniform_select | REFERENCE-DRIVER (Iterator::all loop, 8 br) | REFERENCE-DRIVER (loop, 9 br, 3 calls) | 0 | gate uniform select | REFERENCE-DRIVER (loop back-edge, honest) |
| penstock::penstock_prefilter8 | REFERENCE-DRIVER (chunks_exact loop, 8 br) | REFERENCE-DRIVER (loop, 8 br, 8 calls) | 0 | cgap-gate | REFERENCE-DRIVER |
| penstock::penstock_prefilter8_mix | PASS (7, 0 br) | PASS (17, 0 br) | 0 | penstock_prefilter8 | PURE-ALU |
| penstock::swar_all_zero_vl | REFERENCE-DRIVER (loop, 5 br) | REFERENCE-DRIVER (loop, 5 br) | 0 | gate prefilter | REFERENCE-DRIVER |
| eytzinger::eytzinger_next | PASS (3, 0 br) | PASS (3, 0 br) | 0 | eytzinger_search | PURE-ALU |
| eytzinger::eytzinger_restore | PASS (6, 0 br) | BRANCHY (1× beqz, index/remainder guard) | 0 | eytzinger_search | PASS-host / residual-branch on RV32 |
| eytzinger::eytzinger_search | REFERENCE-DRIVER (loop, 5 br) | REFERENCE-DRIVER (loop, 10 br) | 0 | search kernels | REFERENCE-DRIVER (covers prior SKIP #3) |
| eytzinger::to_eytzinger_layout | REFERENCE-DRIVER (loop + alloc, 14 br, calls) | REFERENCE-DRIVER (loop, 17 br, 10 calls) | 0 | layout build | REFERENCE-DRIVER (covers prior SKIP #1; alloc-driver, out of kernel scope) |

## Prior-skip reconciliation

The 5 previously-skipped fns are all covered above with honest classification:
`to_eytzinger_layout` (REFERENCE-DRIVER: layout-build loop + Vec alloc — not a
hot-kernel claim), `checked_uniform_select` (REFERENCE-DRIVER: Iterator::all
back-edge), `swar_lt` (PURE-ALU PASS both targets), `swar_splat` (PURE-ALU PASS
both targets), `eytzinger_search` (REFERENCE-DRIVER: search loop back-edge).
None is a silent PASS; no [SKIP] rows remain in this census.

## Coverage assertion

- pub fns in source (grep): 19; rows in table: 19; rows censused in BOTH asm
  emits (host + riscv32imc): 19. 19/19 covered — zero uncovered pub fns.
- RVC scan: explicit scan for c.beqz / c.bnez / c.j / c.jal over the entire
  riscv32imc emit returned **0 compressed mnemonics** despite the ELF arch
  attribute confirming RVC is enabled
  (`rv32i2p1_m2p0_c2p0_zmmul1p0_zca1p0`). LLVM chose 32-bit encodings for
  this emit under `-C link-dead-code=yes`; FA6's RVC-count gap is therefore
  reported as RVC=0 (nothing to classify), not inferred.
- Grid reconciliation: RV1's saturated_subs_p28 repair is re-witnessed (19
  insns / 0 branches, riscv emit, this session); FA6's select_slice residual
  cbz is re-witnessed (host 1x cbz len==0 guard, riscv 2 branches).
- select.rs rows are AS-OF-IN-FLIGHT (REP1 owns select.rs +
  tests/branchless_census.rs this wave); re-census of select::select_slice and
  select::cmov_u64 is owed post-REP1.

## Standing

- ALIVE: both asm emits + per-symbol census commands ran this session against
  /Users/sac/cgap/crates/cgap-branchless (exit 0; see commands below).
- Perf/size figures above are insn COUNTS from the census method, not timing
  TARGETS.

## Commands (all exit 0)

```
cd /Users/sac/cgap/crates/cgap-branchless && RUSTFLAGS="--emit=asm -C opt-level=3 -C link-dead-code=yes -C debuginfo=0" CARGO_TARGET_DIR=/tmp/ex3-target cargo +nightly build --release --target riscv32imc-unknown-none-elf
cd /Users/sac/cgap/crates/cgap-branchless && RUSTFLAGS="--emit=asm -C opt-level=3 -C link-dead-code=yes -C debuginfo=0" CARGO_TARGET_DIR=/tmp/ex3-target cargo +nightly build --release
grep -c "	c." <riscv .s>          # RVC scan -> 0
python3 per-symbol census (host .s, riscv .s)  # 19/19 rows
grep -n "pub fn" crates/cgap-branchless/src/*.rs  # 19 pub fns
```

Build root /tmp/ex3-target is a lease; deleted at lane end.
