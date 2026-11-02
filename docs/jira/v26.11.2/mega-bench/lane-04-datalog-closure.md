# Lane 4 — mat_closure (G3) mega-bench receipt

- Date: 2026-11-02. Branch: main. Lane contract: READ-ONLY source; only
  write = this file.
- Command: `CARGO_TARGET_DIR=/tmp/mb4-target cargo bench -p bench --bench mat_closure`
  (cwd /Users/sac/cgap) -> EXIT 0. In-bench anti-vacuity fence
  (benches/mat_closure.rs:69-72, closure_facts > mutant_base_only_count)
  passed every bucket (no VACUOUS panic). The /tmp sandbox mutant court
  itself was NOT re-run this session (standing from
  w5-g3-mat-closure.md, sha256 0df12641... flip exit 101); this session
  witnesses the bench + in-bench fence only.

## Per-bucket table (measured 2026-11-02, this session)

| bucket | base_facts | closure_facts | rounds | best_secs | facts/s/core | w5 witness | drift |
|---|---|---|---|---|---|---|---|
| N=65,536, len=8 | 57,344 | 229,376 | 7 | 0.022148 | 10,356,433 | 11,249,252 | -8.0% |
| N=262,144, len=16 | 245,760 | 1,966,080 | 15 | 0.260436 | 7,549,177 | 8,226,371 | -8.2% |
| N=1,048,576, len=32 | 1,015,808 | 16,252,928 | 31 | 2.815235 | 5,773,205 | 4,847,437 | +19.1% |

- Fact counts and rounds are byte-identical to the w5 witness
  (deterministic corpus, splitmix64; corpus seed
  0xA20C_0FFE_E5A7_E000; 0xBF58_476D_1CE4_E5B9 is the SplitMix64 DERIVE
  multiplier inside the generator, never the corpus seed — kept
  distinct). Throughput drift is timing noise: -8% small buckets,
  +19% at N=1M (this run's best-of-3 was faster). All buckets within
  the V1 audit range 4.85M-11.2M facts/s/core.
- Success audit: >= 1M facts/s/core — MET every bucket (min 5.77M).
  G3 50M threshold: still TARGET, NOT met (consistent with w5).

## Shape taxonomy (honest scoping)

- The mission's linear-chain / DAG / cycle-rich taxonomy does NOT exist
  as separate datasets. What exists: one corpus shape per bucket
  (`chains_corpus`, benches/mat_closure.rs:31-47) — node-id-ordered
  chains of length `chain_len`, then splitmix64-shuffled edge order.
  Measured facts/s/core are chains-shape numbers only. DAG/cycle-rich
  shapes: MISSING (loudly, not silently covered by chains numbers).

## Derivation vs dedup split

- NOT INSTRUMENTED. The bench times the whole `semi_naive_closure`
  loop (closure.rs:94-136: round delta loop + SeenSet
  `pair_key` dedup at :105/:123); separating derivation from dedup
  would require a code change — not made (READ-ONLY lane). Structural
  facts only: delta-driven join `dR x B` via `Adjacency::stream`
  (:120-130); SeenSet dedup gates emission; fresh
  `next_x`/`next_z` Vec per round (:118-119).

## Borrowed-prefix (BFX) driver status

- LANDED at run time: `crates/cgap-bn3/src/unify.rs` (2853 lines)
  carries ZERO `to_vec()` sites; `SplitStore` borrowed-prefix +
  owned-delta-tail documented at :409-418; W6-BFX marker at :1470.
  The legacy :1371-1373 `to_vec()` copies no longer exist. This is
  context only — mat_closure drives cgap-storage closure
  (closure.rs), not unify; no BFX-vs-legacy A/B was measured (no
  legacy driver remains to A/B against).
- Zero-heap re-allocation during convergence on the closure path is
  NOT true of the shipped code and was witnessed, not assumed.

## Zero-alloc verdict: NOT MET (measured)

- Counting-allocator probe (/tmp/mb4-allocprobe, counting GlobalAlloc
  over System, counter window = closure call only, best-of-4 reps,
  N=1,048,576/len=32, CARGO_TARGET_DIR=/tmp/mb4-target, EXIT 0):
  `N=1048576 best_secs=2.901682 rounds=31 allocs=1117 bytes=1204813020`
  — 1,117 allocation events, ~1.20 GB churned, consistent with the
  fresh delta Vec per round (closure.rs:118-133). Retained-alloc
  channel: ALLOCATING. Zero-heap claim: NOT MET on the shipped path.
- Probe is /tmp-only; no source change. The bench itself emits no
  divan alloc column (plain Instant main).

## Compile-freeze disclosure

- On first bench attempt, `cgap-parquet` failed to compile:
  `crates/cgap-parquet/src/footer.rs` contained a spliced
  `pub fn dummy() {}` fragment (unclosed delimiter, cargo exit
  nonzero, 0 tests executed — compile-abort, NOT a failure count;
  corruption was present in the working-tree diff itself, no clean
  +side ancestor). This lane probed a faithful in-file
  reconstruction but wrote NOTHING (probe asserts fired before any
  write). The owning lane repaired footer.rs mid-session; the
  bench-relevant closure path was never affected. No fix applied by
  this lane; no source file modified by this lane.

## Standing

- mat_closure bench: ALIVE (ran this session, exit 0, fence green).
- G3 50M facts/s/core threshold: TARGET, not met (measured
  5.77M-10.36M this session).
- Shape taxonomy beyond chains: MISSING (no dataset).
- Derivation/dedup split: NOT INSTRUMENTED.
- Zero-alloc on closure path: REFUTED by measurement
  (1,117 allocs / 1.20 GB @ N=1M bucket).
- BFX borrowed-prefix driver: LANDED (unify.rs:1470), unmeasured vs
  legacy (legacy removed).
