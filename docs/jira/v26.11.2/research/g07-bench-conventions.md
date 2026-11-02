# R7 — ggen bench suite conventions (Explore brief)

Lane: R7. Sources: `/Users/sac/ggen/benches/`, `/Users/sac/ggen/crates/bcinr-pddl/benches/`,
`/Users/sac/ggen/crates/ggen-abb-sbb/`. Read-only; claims cited file:line.

## 1. Framework landscape

ggen is predominantly **Criterion**; Divan in exactly two sub-crates, zero in root `benches/`.

- Root: `criterion = { version = "0.8", features = ["html_reports"] }` (Cargo.toml:253, :754);
  `[profile.bench]` at :791-796 (opt-level 3, lto, codegen-units 1).
- Divan: `divan = "0.1.21"` (crates/bcinr-pddl/Cargo.toml:68); `divan = "0.1"`
  (crates/ggen-engine/Cargo.toml:176, declared but its benches are Criterion through-patterns).
- Root bench targets all `harness = false` (Cargo.toml:798-843). Two archived-dead (kept on
  disk per fix-forward doctrine): `a2a_bench` gated off via
  `required-features = ["a2a-integration-tests"]` (:827-828); `ggen_benchmarks` gated by
  `ggen-core-retired` (:840-842, comment :835-839). Exclude from live target lists.

## 2. Root benches/ workload shapes

| Bench | Workload |
|---|---|
| runtime_overhead.rs | async/sync bridge `execute()` overhead; SLO doc header <10µs/call, <100ms init |
| async_runtime_benchmarks.rs | simulated async work (lazy_static shared runtimes) |
| memory_profiling.rs | custom TrackingAllocator (GlobalAlloc) counting alloc/dealloc — not timing |
| quick_runtime_validation.rs | fast dev-loop check |
| conventions_performance.rs:1-42 | discovery/plan/watch/cache-hit; targets table in doc comment |
| marketplace_performance.rs | TempDir registry ops, Runtime::block_on |
| cli_startup_performance.rs:14-33 | subprocess benches: builds target/release/ggen in-bench, times spawn |
| comprehensive_slo_benchmarks.rs:13-40 | `mod slo` of P50/P95/MAX Duration constants |
| schema_layer_slo.rs | TTL→Signature transpile (10/50/100 shapes), schema gen, validation 10K objects; measurement_time 10s/15s (:249,:282,:319,:358); paired SCHEMA_LAYER_SLO_README.md |
| v2_performance.rs:1-20 | Chicago-TDD "REAL execution times, REAL templates, REAL data, REAL I/O" |
| fortune500_performance.rs:35-36 | sample_size 10–100, measurement 20–30s, Throughput::Elements, BatchSize variants (:217,:272,:569) |
| receipt_bench.rs / canonical_bench.rs | SHA-256 hashing, canonical JSON (re-pointed ggen_core→ggen_config, canonical_bench.rs:3-8) |
| fixtures/ontologies/ | small/medium/large/very_large_ontology.ttl sized fixtures |

## 3. Receipt conventions

**A. Markdown SLO Performance Receipt** — `benches/PERFORMANCE_RECEIPT_TEMPLATE.md`: command
line, SLO compliance matrix (Target/Measured/Variance/PASS), per-group blocks, system-specs
block (platform, CPU, RAM, rustc, criterion version — :167-177), regression thresholds
**±10% acceptable / >15% alert / >50% critical** (:199-207), reproduction command, sign-off,
"END OF RECEIPT" footer (:286). Worked example: SCHEMA_LAYER_SLO_README.md.

**B. JSON bench receipt + committed regression bound (strongest convention)** —
`crates/ggen-abb-sbb`:
- `bench/receipt.json`: schema `ggen.abb-sbb.bench-receipt.v1`; subject (repo, PR, parent
  commit sha, measured_commit, measured_src_tree, measured_src_lib_rs_sha256), command, host
  (CPU, toolchain, load_average_1m with note that ratios—not absolutes—are the regression
  signal under load), per-bench median_us + ci95_us.
- `tests/bench_bound.rs:1-52`: a test that re-witnesses the receipt every CI run via
  `include_str!` (:9); named ratio bounds (MAX_ADMIT_OVER_DIGEST = 1.6 at :14; 64-vs-16
  artifacts linear-scaling bound under 8x at :17); paired_medians/paired_mins interleaved
  helpers (:29-52) so both sides of a ratio see identical ambient load.

## 4. PARETO / before-after conventions

No formal PARETO harness. The ggen equivalent: **receipt once, enforce ratios as a committed
test** — "admit/graph_digest ~2.06 before the single-digest fix, ~1.1 after"
(bench_bound.rs:11-13) is before/after receipted as a ratio bound, not an absolute. Governing
doctrine (receipt.json host.note): absolute numbers are upper-biased; ratios are the signal.
Markdown receipts use variance-vs-target margins ("27.5x margin", "1.54x margin") plus the
10/15/50% thresholds (PERFORMANCE_RECEIPT_TEMPLATE.md:199-207).

Directly relevant to cgap's no-sleep ratio-fence law (no-sleep-testing): the abb-sbb
paired-ratio pattern is the same shape — relative fence vs sleep-free-by-construction baseline,
paired rounds, min-of-N.

## 5. Divan exemplars to cite

- `crates/bcinr-pddl/benches/scaling.rs`: canonical Divan shape — `fn main() {
  divan::main(); }` (:9-11), `const ARGS: &[usize] = &[8,16,32,64,128,256,512]` (:39),
  `#[bench(args = ARGS)] fn bench_ir(bencher: Bencher, n: usize)` with
  `bencher.bench_local(...)` + `divan::black_box` (:41-49).
- `pddl_80_20.rs` uses bare `divan::black_box(...)` (:60).
- Sub-crate bench preamble: `#![allow(clippy::unwrap_used, ...)]` with comment citing
  `.claude/rules/rust/testing.md` (scaling.rs:1-2, manufacture.rs:5-6).

## 6. Caveats

1. "ggen uses Divan" is false at root — scope to crates/bcinr-pddl.
2. Two archived-gated dead benches — exclude from live lists.
3. Benchmark-as-test (bench_bound.rs) is the regression-gate convention; `save_baseline` used
   nowhere.
