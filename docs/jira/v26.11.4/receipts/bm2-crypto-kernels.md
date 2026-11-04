# BM2 — Crypto Kernel Surfaces (Benchmark Lane 2)

Date: 2026-11-04 · Repo: /Users/sac/cgap · Lane: bm2 · No git operations.

## Machine-condition witness

```
12:00  up  1:51, 3 users, load averages: 7.63 10.98 9.69
```

Concurrent-fleet disclosure: GAP/C benchmark lanes were running concurrently with this lane.
These are CPU-bound short benches; contention is disclosed as load (1-min LA 7.63) and not
hidden. Per-call p99 figures are contention-sensitive; min-of-N is the primary statistic.

Timer-floor awareness: the ~30-40ns `Instant::now()` call+timer overhead is INSIDE every
per-call number below (the test itself states "call+timer overhead included"). The 64B p50
(125.0ns) is therefore roughly 4x timer floor; small-size numbers are floor-dominated and
should not be read as pure hash cost. Large sizes (1024B, 64KiB) are far above floor.

## (1) Per-call latency — `cargo test -p cgap-blake3 --release --test percall_latency -- --nocapture`

Exit 0, 1 passed. Raw output verbatim:

```
AF-22 per-call hash(64): rounds median [125.0, 125.0, 125.0] ns | best-round p50 125.0 p90 167.0 p99 167.0 min 83.0 max 292.0 ns (n=10000 x 3 rounds, call+timer overhead included)
AF-22 per-call hash(1024): rounds median [2125.0, 2125.0, 2125.0] ns | best-round p50 2125.0 p90 2167.0 p99 2292.0 min 2041.0 max 8334.0 ns (n=10000 x 3 rounds, call+timer overhead included)
AF-22 per-call hash(65536): rounds median [32792.0, 32791.0, 32750.0] ns | best-round p50 32750.0 p90 32875.0 p99 35209.0 min 32583.0 max 47291.0 ns (n=10000 x 3 rounds, call+timer overhead included)
```

Per number: plane=userland, profile=release(opt), method=n=10000 samples x 3 rounds, median
of rounds + best-round percentiles; min-of-N (3x10000) stated per row.

| size | p50 | p99 | min-of-N |
|---|---|---|---|
| 64B | 125.0 ns | 167.0 ns | 83.0 ns |
| 1024B | 2125.0 ns | 2292.0 ns | 2041.0 ns |
| 64KiB | 32750.0 ns | 35209.0 ns | 32583.0 ns |

Comparison vs prior band (64B p50 125.0ns, min 83.0ns): **CONFIRMED** — exact match on both
p50 and min.

## (2) Batch perf — `cargo test -p cgap-blake3 --release --test independent_batch_perf -- --nocapture`

Exit 0, 1 passed. Raw output verbatim:

```
PL15 perf leg (N=64 x 1 KiB, min-of-5): batch=55083 ns per_frame=136583 ns speedup=2.480x (PL15 predicted 1.4-1.6x)
```

Plane=userland, profile=release, method=min-of-5 frames. Batch-vs-scalar speedup at
N=64x1KiB = **2.480x** (prior: 2.515x — CONFIRMED within ~1.4%, same band; both well above
the PL15 predicted 1.4-1.6x).

## (3) Anti-vacuity dev-profile witness (loud-SKIP)

`cargo test -p cgap-blake3 --test percall_latency -- --nocapture` (dev profile). Exit 0.
Witnessed line verbatim:

```
LOUD-SKIP percall_latency_64b_1kib_64kib: non-release profile: debug_assertions=true opt-level=unset
```

SKIP witness witnessed, not fixed. The gate refuses to emit perf numbers under a
non-release profile — vacuity guard is live (it flips: release prints numbers, dev prints
LOUD-SKIP).

## (4) ML-KEM surface — `cargo test -p cgap-crypto --release --test mlkem768_bench -- --nocapture`

Present; ran. Exit 0, 1 passed. Raw output verbatim:

```
BENCH keygen min-of-100: 25.50 us/op
BENCH encaps min-of-100: 28.44 us/op
BENCH decaps min-of-100: 31.33 us/op
BENCH_SUMMARY keygen=25.50us encaps=28.44us decaps=31.33us
```

Plane=userland, profile=release, method=min-of-100 ops per phase. No prior band stated for
ML-KEM — first sample, UNVERIFIED baseline.

## Standing

- BLAKE3 64B band: CONFIRMED (125.0ns p50 / 83.0ns min, exact).
- Batch speedup: CONFIRMED (2.480x vs prior 2.515x at same shape).
- Loud-SKIP anti-vacuity: witnessed firing (ALIVE).
- ML-KEM 768: ALIVE, first-sample baseline (no prior band to compare).
- All runs under concurrent-fleet load (LA 7.63/10.98/9.69) — disclosed above.
