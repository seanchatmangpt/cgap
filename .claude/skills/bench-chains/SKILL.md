---
name: bench-chains
description: >-
  Run the five chains-* end-to-end benches (ingress, parquet, bn3, seal,
  sovereign), land the PARETO receipts, and read the divan alloc column.
  Sources: crates/bench/Cargo.toml [[bench]] list, P3-16/P3-17 receipts,
  docs/jira/v26.11.2/02-pareto-levers.md.
---

# Bench chains (five staged end-to-end benches)

Five `[[bench]]` targets in `crates/bench/Cargo.toml` (lines 79-97):
chains-ingress, chains-parquet, chains-bn3, chains-seal, chains-sovereign.
Each is one seeded corpus fed through a real pipeline with per-stage
checksum/identity asserts; each names its anti-vacuity mutations in its
header comment. Harness = false (divan), pin `divan = "=0.1.21"`.

## Run

    CARGO_TARGET_DIR=/tmp/<lane-id>-target cargo bench -p bench --bench chains-ingress
    ... --bench chains-parquet
    ... --bench chains-bn3
    ... --bench chains-seal
    ... --bench chains-sovereign

Chain law: each stage feeds the next over real wire bytes; final state is
asserted (identity + folded checksum), never interaction counts. Corpus:
SplitMix64 canonical seed `0xBF58_476D_1CE4_E5B9`, per-lane spread
`seed ^ shape*0x2544_5491_4F6C_DD1D`. Frozen; never vary.

## Receipts (where they land)

- v26.11.1 coordinates (prior wave):
  `docs/jira/v26.11.1/waves/bench-chains-{ingress,parquet,bn3}.md`.
- v26.11.2 re-measures land in
  `docs/jira/v26.11.2/waves/bench-chains-<chain>.md`, one file per chain,
  PARETO lever coordinates cite them (02-pareto-levers.md method: divan
  0.1.21, bench profile, 100 samples, Apple Silicon). All perf numbers are
  TARGETs until a v26.11.2 bench receipt re-measures them.

## Alloc column semantics

The alloc column is the verdict for zero-alloc claims. Divan's own
`#[global_allocator]` is `#[cfg(test)]`-only, so each bench binary installs
`#[global_allocator] static ALLOC: divan::AllocProfiler =
divan::AllocProfiler::system()` itself (documented pattern, parquet.rs /
chains-parquet.rs). Without the profiler installed, tallies are never
collected and any 0 B claim is vacuous. Read alloc/grow/dealloc bytes per
iteration from the divan table (chains-parquet receipt shape: "alloc 3/16 B,
grow 2/24 B, dealloc 40 B per iteration"); a residual per-row-scaled alloc
is attributed to its source (e.g. verify_seal's bounded meta-Vec) rather
than averaged away. The staged decode arm in `parquet.rs` is the >0 B
positive control for the direct read-back arm.
