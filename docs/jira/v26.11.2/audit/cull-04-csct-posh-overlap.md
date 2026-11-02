# cull-04-csct-posh-overlap

CULL-04 csct/POSH court overlap audit (2026-11-02, read-only;
static read, no tests executed).

## The two "<=10 ns amortized" gates are NOT the same assertion

- cscb_c1_c2_ingress.rs: C2 gate p99 <= 10.0 ns/frame
  (C2_P99_NS_MAX :49), kernel = process_frame (:70-87): one
  scan::prefix_max_u8x8 + two rotate/xor folds per 24-B frame,
  batch=64 amortization (:147-183), p99 over ~15,625 batches.
  PASS ~5-6 ns.
- posh_batch_court.rs (:199-289): gate median <= 10.0
  (GATE_NS_PER_FRAME :55), kernel = unify_batch_probe two-atom
  JOIN over 1,365 facts (:146-154), median-of-7 runs. HONEST
  FAIL 23.35 ns.
- Reconciliation: same threshold + method, DIFFERENT kernels
  and statistics (p99-of-15k-batches vs median-of-7-runs). A
  join with probe scratch + match-set emission is strictly more
  work than a prefix-max scan. POSH-2's fail does not
  contradict C2's pass; neither subsumes the other.
  VERDICT: both UNIQUE.

## Per-file verdicts

- posh_batch_court.rs UNIQUE (only timing court over
  unify_batch_probe; own branchy flip :242-256).
- posh_etag_court.rs UNIQUE (etag fixture exists only here;
  5 courts :85-163).
- csct_coverage_court.rs OVERLAP-PARTIAL: exit-variant wire
  pins (:113-115) vs andon.rs:109 + types.rs:191 (same
  0/1/2/3/101 pin, three sites); digest sensitivity courts vs
  types.rs:157-189 (coverage adds structural reorder +
  per-field exhaustiveness — partial delta).
- csct_pipeline_court.rs OVERLAP-PARTIAL: strongest redundancy
  found = mutation_controls_pass (:73-78) duplicates
  src/mutation.rs:66 verbatim in intent; receipt determinism
  (:82-96) triple-covered (receipt.rs:169 + coverage :344-355);
  vacuum guard + digest-flip falsifier are UNIQUE meta altitude.
- src stage courts UNIQUE individually (each pins its own pass
  + one mutation flip).
- drowning_benchmark.rs OVERLAP-PARTIAL: re-asserts ledger
  constants drowning.rs also asserts; its delta = the C8
  ROI-floor + zeroed-spark mutant (:72-85) which drowning.rs
  lacks.
- src/bin/*.rs UNIQUE (each a distinct kernel/budget gate).

## Cull recommendation

- CULL: pipeline mutation_controls_pass (csct_pipeline_court
  :73-78) — duplicate of src/mutation.rs:66.
- KEEP both timing gates (different kernels; culling either
  silently narrows latency coverage).
- KEEP coverage court; dedupe the exit-wire pin later.

## See Also

audit/cull-03-git-court-census.md · lanes/lane-posh-02-batch.md
