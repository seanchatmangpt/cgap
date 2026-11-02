# Lane 6 - Branchless Census (mega-bench)

Date: 2026-11-02. Repo `/Users/sac/cgap` (canonical checkout, read-only
source). Lane writes: this receipt only. Build scratch: `/tmp/b6-target`
(the census's own `TARGET_DIR` constant, `tests/branchless_census.rs:31`)
and `/tmp/mb6-latency` (probe crate; deleted below). `/tmp/mb6-target` was
not needed - the census binary hard-codes `/tmp/b6-target`.

## Gate ladder

| Gate | Command (cwd) | Exit | Evidence |
|---|---|---|---|
| Unit tests | `cargo test -p cgap-branchless` (repo root) | 0 | 26 passed; 0 failed (0.07s) |
| Census court build | `rustup run nightly rustc --edition 2021 crates/cgap-branchless/tests/branchless_census.rs -o /tmp/mb6-census-bin` | 0 | standalone harness=false binary; `--emit=obj`, `-C link-dead-code=yes` (load-bearing per w6-b6) |
| Census court run | `/tmp/mb6-census-bin` | 1 | 19 cells: 15 PASS / 3 BRANCHY; anti-vacuity self-check tripped (`naive_if_cmov` BRANCHY 3x [tbz, b, b]) - court non-vacuous |
| Latency probe | `cargo build --release` + run in `/tmp/mb6-latency` | 0 | 1M-iter loops x5, min, black_box'd inputs, opt3+lto, host aarch64 |

## 19-cell verdict grid (as of this run)

Host = aarch64 (opt-level=3). riscv = riscv32imc-unknown-none-elf,
`#![no_std]` wrapper; select.rs excluded on riscv (needs std/alloc - loud
`[SKIP]` emitted by the court itself).

| fn (module) | aarch64 | riscv32imc |
|---|---|---|
| cmov_u64 (select) | PASS | n/a (module skipped) |
| select_slice (select) | BRANCHY 12x (b.eq x2, cbz x3, b.ne x2, b.hs, b.lo, b, tbz) | n/a |
| pred_inc (select) | BRANCHY 1x (tbz) | n/a |
| saturated_subs_p28 (arithmetic) | PASS | BRANCHY 1x (beq) |
| poison_mask (arithmetic) | PASS | PASS |
| swar_eq_u64 (swar) | PASS | PASS |
| swar_has_zero_byte (swar) | PASS | PASS |
| swar_range_mask (swar) | PASS | PASS |
| eytzinger_next (eytzinger) | PASS | PASS |
| eytzinger_restore (eytzinger) | PASS | PASS |
| radix_bucket_12 (radix) | PASS | PASS |

Count: 15 PASS / 3 BRANCHY - **identical cell set to w6-b6's final run
(select_slice + pred_inc host; saturated_subs_p28 riscv). Zero cell drift**
from B2's landed bodies. Every BRANCHY cell is explained:

- `select_slice` / `pred_inc` (host): slice-length fast-path and predicate
  lowering that the optimizer materializes as real branches; structural to
  the current bodies, unchanged since w6-b6.
- `saturated_subs_p28` on rv32imc: u64 saturating sub on a 32-bit ISA with
  no conditional-move instruction (RV32 base + M have no cmov) - the
  borrow/compare lowering emits one `beq`. Structural to RV32, not a
  regression.

Honesty notes:
- `eytzinger_next` / `eytzinger_restore` are `#[inline(always)]`
  (`crates/cgap-branchless/src/eytzinger.rs:42,54`); standalone per-fn
  symbols generally do not survive codegen, so their PASS verdicts are read
  at their inlined sites inside `eytzinger_search` (w6-b4 convention). At
  this run's opt-level=3 + link-dead-code=yes standalone symbols happened
  to be present and PASS. Their bodies are pure arithmetic
  (`2*curr + pred`, `idx >> (trailing_ones+1)`) - branchless by
  construction.
- The module has no built-in benches; a standalone probe crate
  (`/tmp/mb6-latency`, path-pulls the module files directly) is the
  standardized harness for this latency census, stated per lane contract.

## Latency census (host aarch64, min-of-5 x 1M iters, black_box'd inputs)

Method note: a first pass without input black_boxing produced 0.00 ns/op for
swar_eq_u64 / swar_has_zero_byte - LLVM constant-folded the loop. Those
readings were discarded; the table below is from the black_box'd rerun
(authoritative).

| primitive | ns/op |
|---|---|
| cmov_u64 | 0.70 |
| saturated_subs_p28 | 1.07 |
| poison_mask | 0.82 |
| swar_eq_u64 | 1.26 |
| swar_range_mask | 1.66 |
| swar_has_zero_byte | 0.69 |
| pred_inc | 0.26 |
| radix_bucket_12 | 0.56 |
| eytzinger_search | min 1.54 / max 12.85 over n=1..199 virtual-padded layouts (max at n=61) |

pred_inc at 0.26 ns/op is below a 4 GHz cycle - read as the sub-ns pipeline
effect of a 1-instruction hot loop (single tbz + add), not a folded
measurement: its output is counter-state-dependent and acc-chained.

## Compliance verdict (honest)

100% branchless compliance is NOT met:

- aarch64: 9/11 cells PASS; `select_slice` (12 branches) and `pred_inc`
  (1 branch) are BRANCHY.
- riscv32imc: 7/8 censusable cells PASS; `saturated_subs_p28` is BRANCHY
  (1 beq, structural to RV32's missing cmov).
- Aggregate: 16/19 PASS, 3 BRANCHY, 0 MISSING. Court exit 1 is the CORRECT
  verdict for this state (gate fail on BRANCHY cells), and the court is
  provably non-vacuous (branchy reference flagged). The court stays red
  until select.rs's two fns are rewritten branchless and/or the RV32
  saturated-sub is restructured; neither landed in this lane (read-only).

## Reproduce

```
cargo test -p cgap-branchless
rustup run nightly rustc --edition 2021 \
  crates/cgap-branchless/tests/branchless_census.rs -o /tmp/mb6-census-bin
/tmp/mb6-census-bin   # exit 1 (3 BRANCHY cells, self-check tripped)
```
