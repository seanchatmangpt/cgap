# W5-L3 — bn3 join-stage re-measure (L3 lever)

Lane: A1 (Analyze). Date: 2026-11-02. Repo: /Users/sac/cgap @ main 9a730320.
OWNERS: `docs/jira/v26.11.2/waves/w5-l3-remeasure.md`, L3 row edit in
`docs/jira/v26.11.2/02-pareto-levers.md`, /tmp/a1-target only. No source
edits, no git writes, no mocks, no Process.sleep in bench/test code.

## Method

Divan 0.1.21 (pinned, exact-at-authoring-time), `cargo bench -p bench
--bench chains-bn3`, release profile, filter `stage_unify_join` (matches
both legs + the compile probe excluded via output inspection — actually
matched legs only in the observed run: the four rows printed were
nested/radix x 65,536/1,048,576). Machine: macOS Darwin 25.2.0 host,
concurrent lane build activity present during the run (cgap-parquet /
cgap-ffi recompiles observed mid-run) — min-of-N (fastest) used as the
primary statistic for outlier robustness; medians also recorded. 100
samples per const. Timer precision 41 ns. Sleep-free (divan owns
sampling).

## Results (exit 0)

| leg | 65,536 fastest | 65,536 median | 1,048,576 fastest | 1,048,576 median |
|---|---|---|---|---|
| stage_unify_join (nested baseline) | 8.435 ms | 9.967 ms | 1.674 s | 1.701 s |
| stage_unify_join_radix | 7.661 ms | 7.836 ms | 1.648 s | 1.656 s |

(Note: `stage_unify_join` nested @65k showed slowest 146.2 ms — build
noise outlier, suppressed by min-of-N.)

radix:nested @1M = 1.648/1.674 = **0.98x** (fastest) / 1.656/1.701 =
**0.97x** (median) — the radix semi-join leg is only 1.6-2.6% faster
than the retired nested-loop baseline on the chained corpus.

## Verdict vs target

- Target <75 ms @1M: **NOT MET**. Fastest radix leg 1.648 s = 22x over
  target. Baseline reproduction: nested @1M 1.674 s fastest is within
  noise of the recorded 1.470 s baseline (same stage, same corpus
  class). L3 lever status: **LANDED-NOT-MET**.
- Floor >20x: NOT delivered — measured radix:nested is 0.97-0.98x.
  Note: the bench docstring itself names a ">=10x gate vs
  stage_unify_join" (chains-bn3.rs:534-537) that this corpus does not
  show.
- Full-chain implication: stage_unify_join (~1.5-1.7 s) IS the chain
  baseline (1.470 s) — the join stage dominates `deduction_chain`, so
  fixing the join IS fixing the chain, but the radix probe as built
  moves it ~2%: the dominant residual cost is the per-chunk candidate
  walk OUTSIDE the probe (next-lever attribution required).
- Alloc column: divan 0.1.21 emitted NO Alloc/Bytes columns on any leg
  = 0 allocs across all samples (divan omits all-zero columns;
  corroborated by `l3_noalloc_radix_capacity_refusal` /
  `l3_noalloc_radix_differential` courts). Recorded as measured-zero
  with the omission mechanism, not assumed.

## Courts (ride-along)

`CARGO_TARGET_DIR=/tmp/a1-target cargo test -p cgap-bn3 --test
lever_courts` -> exit 0, 7 passed / 0 failed (34.88 s): l3_emit_join_masks_court,
l3_noalloc_radix_capacity_refusal, l3_noalloc_radix_differential,
l3_antivacuity_probe_drop_flips, l3_differential_join_court,
l3_radix_vs_nested_differential, l3_radix_vs_nested_100k.

Note: `l3_radix_vs_nested_differential` PASSES at court scale (100k)
while the divan @1M legs show only ~2% — the court's differential is
over a probe-dominated slice, not the full-chain corpus; not
contradictory, but scope-recorded.

## Divergences / disclosures

1. Compile-freeze SLA unblock (disclose): during lane A1's bench build,
   `crates/cgap-bn3/src/unify.rs` was transiently non-compiling (E0412/
   E0425 at unify.rs:1341 — `FixpointHasher`/`BuildHasherDefault` not
   found, mid-edit by the owning lane). The owning lane's next write
   resolved it BEFORE any A1 edit was applied; A1 applied no fix, built
   no shim. Recorded because it froze A1's first bench attempt ~10 min.
2. `p:measuredNs` board flip: coordinator followup (generated ontology
   projection; not hand-edited per repo law).
3. This run re-measures the baseline too: 1.674 s vs 1.470 s recorded
   (+14%) — concurrent machine load; min-of-N stated, verdict
   unaffected (22x gap dwarfs load noise).

## Commands + exits

- `CARGO_TARGET_DIR=/tmp/a1-target cargo bench -p bench --bench chains-bn3 -- stage_unify_join` -> exit 0 (first attempt exit 0 with in-bench divan arg error, exit 2, from `-- --bare` misuse; rerun clean)
- `CARGO_TARGET_DIR=/tmp/a1-target cargo test -p cgap-bn3 --test lever_courts` -> exit 0 (7/7)
- Build root /tmp/a1-target deleted at lane end per lease law.

Standing: LANDED-NOT-MET (L3 lever); measurement itself ALIVE (commands
+ exits this session, exact subject).
