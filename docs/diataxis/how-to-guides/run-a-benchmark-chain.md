# Run a benchmark chain

Measure a kernel end to end and get the stage attribution (PARETO) that names
the next optimization lever. Numbers are host-local medians; a claim without
this session's run is a target, not a measurement.

## Pick the chain

Five chains live in `crates/bench/benches/`: `chains-bn3`, `chains-ingress`,
`chains-parquet`, `chains-seal`, `chains-sovereign` (plus isolated
non-chain targets like `bn3`, `ingress`, `parquet`, `seal`).

## Run one

```bash
CARGO_TARGET_DIR=/tmp/bench-target cargo bench -p bench --bench chains-bn3
```

Substitute the chain name. Witnessed at exit 0:
`waves/bench-chains-bn3.md` (Divan, 100 samples, bench profile, same
session/host). Each chain has its own receipt in
`docs/jira/v26.11.1/waves/bench-chains-<name>.md` — ingress, parquet, seal,
sovereign included.

## Filter to one case when iterating

```bash
CARGO_TARGET_DIR=/tmp/bench-target cargo bench -p bench --bench chains-parquet \
  -- chain_ram --sample-count 2 --max-time 5
```

Witnessed at exit 0 in `waves/bench-chains-parquet.md` — use low sample
counts while iterating, full samples for the receipt.

## Read the PARETO

Each receipt carries a stage table (scan/compile/unify/project shares) and
names the ~80% segment and the 20% lever. Example: chains-bn3 attributes
81.2% at N=65,536 to unify, collapsing probe work via a hash/trie join on
the bound (s,o) pair is the named lever
(`waves/bench-chains-bn3.md`).

## Keep it Chicago

Asserts ride the benched closure (count/oracle/popcount asserts, named
mutation classes). If your chain run prints no asserts, you ran a target
without the differential oracle — re-check the bench name.

## Gotchas

- Do not run `--benches` (all targets): a sibling lane's in-flight bench
  can abort the whole build (witnessed E0277 in
  `waves/bench-chains-bn3.md`). Build only `--bench <name>`.
- Delete the `/tmp` target dir after the run.
