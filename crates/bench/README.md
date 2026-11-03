# bench — Divan membrane for the v26.11.1 moonshot

Phase 2 (Measure) perf-target graduation harness for the cgap sovereign
substrate. Divan `=0.1.21` (exact pin, alloc profiling via
`divan::AllocProfiler::system()`), canonical seed `0xBF58_476D_1CE4_E5B9`
(SplitMix64). `publish = false`; member of the root workspace but the only
place `divan` exists — the kernel crates stay zero-dep `#![no_std]`.

## What it benches

Chains (end-to-end business chains, `benches/chains-*.rs`):

| target | chain | receipt |
|---|---|---|
| chains-ingress | admit → penstock → digest fold | waves/bench-chains-ingress.md |
| chains-sovereign | C-ABI admit → eval → commit → verify | waves/bench-chains-sovereign |
| chains-bn3 | scan → compile → unify → project | waves/bench-chains-bn3.md |
| chains-seal | p28 validate → metering → chatman seal | waves/bench-chains-seal.md |
| chains-parquet | sealed-block commit chain | receipt pending (see Divergences) |

Per-lane/component benches (`benches/{ingress,parquet,bn3,seal,kernels}.rs`,
`challenger_{parquet,arrow,git,crypto}.rs`, `v26_10_31.rs`) attribute the
chain costs stage by stage; receipts under
`docs/jira/v26.11.1/waves/` (`bench-{ingress,parquet,seal}-receipt.md`,
`f2-15-bench-flips.md`).

## 80/20 PARETO results (measured 2026-11-01, bench profile)

- **chains-ingress**: stage `digest_fold` (BLAKE3 payload digest) is the
  ~80% segment — 93.38 ms of 109.7 ms segment work ≈ 85% of segment cost /
  ~91% of the 102.7 ms chain mean; frame ≈ 98 ns end-to-end, ~89 ns hash.
  Freshness: figures are as-of the 2026-11-01 receipt — receipted 10.1x;
  fresh 9.6x (audit/tps-01-reality-ledger.md:24) — DRIFT, not
  refutation: fastest 10.70 ns/frame vs receipted 10.20/10.30, the
  <15 ns/frame target is met on fresh numbers. Cite both figures.
  Lever: hash only on policy triggers or escalate from an 8-byte prefilter
  (>10x chain-level).
- **chains-sovereign** (N=4096 fastest): commit = 164.4 ms of 187.6 ms
  chain = **87.6%** — the A02 sealed-block Parquet commit. Boundary: A02
  commit lease accepts 65376 lanes, refuses 65377 typed
  `CGAP_ERR_LEASE_EXHAUSTED` (0x43470004).
- **chains-bn3**: unify join is 81.2% of the chain at N=65,536 rising to
  **99.9%** at N=1,048,576 (1.472 s of 1.473 s); scan runs at 12.5 GB/s in
  chain, 80.3 GB/s isolated.
- **chains-seal**: no 80% stage — a three-way split: asc450 metering ~45%,
  chatman seal+verify ~35%, p28 §5 validate ~19%. Per-row cost is flat
  (2.378 vs 2.373 µs/row across 1024→65536, ≤0.3% drift).
- **parquet** (isolated, F2-12): zero-intermediate-heap claim graduated
  TARGET → MEASURED (0 B alloc rows on decode paths, system allocator
  profiler installed; kernel counting-allocator court green).

## No-contamination rule

`divan` and every `[[bench]]` target live ONLY in `crates/bench`; a sweep
this session confirms zero `[[bench]]` sections in any `crates/cgap-*`
kernel manifest. Kernel crates are zero-dep `#![no_std]` with an optional
`std = []` feature; `crates/bench` enables `features = ["std"]` on
`cgap-parquet` from outside, so std never enters a kernel dep graph. No
[[bench]] in kernel crates — ever.

## Run

    cargo bench -p bench --bench chains-ingress
    cargo bench -p bench --bench chains-sovereign
    cargo bench -p bench --bench chains-bn3
    cargo bench -p bench --bench chains-seal
    cargo bench -p bench --bench parquet

All targets also run standalone via
`cargo bench --manifest-path crates/bench/Cargo.toml --bench <name>`.
