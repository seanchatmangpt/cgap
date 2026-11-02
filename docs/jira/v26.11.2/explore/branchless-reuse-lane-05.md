# Branchless reuse map — parquet stats/pruning (BR5)

Subject: `crates/cgap-parquet/src/scan.rs` (read-only map; no code changed).
Anchors: `rg_prunes` at scan.rs:331-335; per-RG prune branch at
scan.rs:759-763 (`scan_u64_pred`); `read_statistics` min<=max guard at
scan.rs:634-671 (lands via PRUNEFIX — not re-proposed). MB8: prune-only
0.9 ns/RG measured; FA8: fail-open defect (now fixed) measured.

## Candidate 1 — branchless rg_prunes mask

- file:line: scan.rs:333, consumer scan.rs:762-763
- current: `Some(b) => pred.0 > b.max || pred.1 < b.min` — two
  compares + short-circuit OR, result is a single bool.
- primitive: cmp+or into a bitmask; but the consumer is
  `if prune { continue; }` — the branch is on the RESULT, inherent
  per row group: the divergence is "skip 65k rows vs scan them",
  a control-flow decision that cannot itself be data-dependent-
  branch-free without restructuring the whole scan.
- gain class: [ANALYTIC] marginal. Loop shape: per-RG branch in a
  loop over `row_groups.len()` RGs (typical file: 256 RGs). The
  branch is highly predictable when RGs are uniformly pruned or
  uniformly kept (the common case — statistics are homogeneous);
  256 predictable branches at ~0.9 ns/RG (MB8) already cost
  ~230 ns/file. A mask-compute-then-bulk-skip form only pays if
  many RGs share one fate AND the mask path avoids per-RG work;
  here the mask would still need a per-RG bounds load (Vec of
  Option<Bounds>, not contiguous), so the branchless form adds a
  gather it does not remove a branch worth removing. Honest
  caveat: no counterexample measured; this is analysis, not a
  measurement. [ANALYTIC]
- court mutant: invert `pred.0 > b.max || pred.1 < b.min` to
  `!(...)` in a /tmp sandbox copy of scan.rs; must flip
  `pages_parsed`/hits count in an existing scan court
  (stats_skip_court). Non-flip = vacuous court = FAIL.

## Candidate 2 — &&/|| short-circuit placement

- file:line: scan.rs:333 (hot), scan.rs:832 (`v >= pred.0 && v <= pred.1`, hot inner loop), scan.rs:668 (cold parse guard).
- current: scan.rs:333 is on the HOT prune decision (executed per
  RG on the scan path); scan.rs:832 is the hot inner-loop value
  predicate (per row); scan.rs:668's `min > max` guard is COLD
  (footer parse, once per column chunk at open).
- primitive: none warranted. scan.rs:333's OR already orders the
  cheapest/most-discriminating compare first
  (`pred.0 > b.max` — range-above test) and short-circuits; the
  inner predicate at :832 is two dependent compares where
  branchless (mask + masked push) is a real restructure candidate
  for a future lane, but it is the VALUE loop, not the stats path
  — out of BR5 scope (stats/pruning), register it for a
  value-loop lane.
- gain class: [ANALYTIC] no change recommended on the stats path.
- court mutant: for :668 the PRUNEFIX mutant already covers it
  (inverted-bounds file must fail open to unpruned scan — FA8).

## Candidate 3 — aarch64 vs riscv32imc divergence for scan.rs

- file:line: scan.rs (whole crate), census doc:
  docs/jira/v26.11.2/waves/p2-asm-0blr-census.md
- current: census coverage of scan.rs prune path: none found
  (grep of the census doc for scan.rs returned no rows).
- UNMEASURED. No asm census row covers scan.rs prune logic on
  either target; no aarch64/riscv32imc divergence claim is
  lawfully makeable for this path.
- proposal: extend the p2-asm census with a scan.rs prune row —
  emit `rg_prunes` for both targets (`--emit=asm-llvm` on
  aarch64-apple-darwin and riscv32imc-unknown-none-elf), count
  conditional branches in the prunes+scan_u64_pred region, and
  assert the riscv32imc form has no hidden per-RG mispredicted-
  class branch the aarch64 form folds. Gain class: census
  extension, UNMEASURED until run.

## Verdict

No branchless rewrite recommended on the stats/prune path this
wave: the per-RG branch is on a control-flow result inherent to
row-group granularity, MB8's 0.9 ns/RG leaves ~230 ns/file total,
and the only real restructure candidate (inner value predicate,
scan.rs:832) belongs to a value-loop lane, not stats/pruning.
Standing: analysis-only; [ANALYTIC] claims are not measurements.
