# p01-stdio-baseline

Lane P-01 receipt. Real MCP-style JSON-RPC 2.0 echo server + client over
stdio (newline-delimited), Rust + serde_json, zero cgap deps, child-process
server. Honest fences F1-F5 honored: this is the ms/µs-class stdio layer,
NOT the ns/frame kernel admission layer (row 5).

## §1 Claim under test

Paper cell "stdio ms-class round-trip latency" (P-17 target). Prior status:
TARGET (never measured by this repo). After this run: MEASURED — and the
measurement shows the "ms-class" label was pessimistic by ~2 orders of
magnitude on this host.

## §2 Method

- Harness: `/tmp/p01-harness` (standalone cargo crate, edition 2021,
  serde_json 1.0.154, opt-level 3).
- Toolchain: rustc 1.101.0-nightly (12433c307 2026-10-24), cargo
  1.101.0-nightly (cad07cf8e 2026-10-23).
- Host: Mac15,9, Apple M3 Max, 16 cores, loadavg 2.56/2.76/2.66 at run
  start (ambient, not quiet-host — scoped accordingly).
- Protocol: client spawns server as child process (`exe serve`); one
  JSON-RPC 2.0 request object per line on child stdin; one response object
  per line on child stdout; client blocks on `read_line` until the matching
  `id` returns.
- Envelope: `{"jsonrpc":"2.0","id":i,"method":"echo","params":{"seq":i,
  "payload":"x"*256}}` — 329-331 B request, 322-324 B response.
- Metric: FULL round-trip (write+flush request -> read parsed response),
  `Instant::now()` bracketing, N=100 serialized envelopes, 3 runs.
- Fixtures: `/tmp/p01-harness/fixtures/env-0{1,2,3}.json` (representative
  request/response pairs, seq 0/1/2, for P-17).

## §3 Results (N=100 x 3 runs, exit 0 x3)

| run | min µs | median µs | mean µs | p95 µs | max µs |
|---|---|---|---|---|---|
| 1 | 7.25 | 7.38 | 25.15 | 11.96 | 1741.12 |
| 2 | 7.08 | 7.46 | 24.10 | 12.79 | 1639.58 |
| 3 | 7.17 | 7.40 | 24.60 | 13.62 | 1639.58 |

Max outliers ~1.6-1.7 ms are scheduling (one envelope per run, first-run
child wakeup class). All other 99/100 envelopes < 50 µs (binned histogram,
every run represented, run CSVs at /tmp/p01-harness/run{1,2,3}.csv):

| bin | run1 | run2 | run3 |
|---|---|---|---|
| 0-50 µs | 99 | 99 | 99 |
| 1.55-1.75 ms | 1 | 1 | 1 |

Raw per-envelope table (run 3, all 100 envelopes, seq/latency_us/req_B/
resp_B; run1/run2 same shape):

<details>
<summary>run3 raw table</summary>

| seq | lat_us | req_B | resp_B |
|---|---|---|---|
| 1 | 1639.58 | 329 | 322 |
| 2 | 16.17 | 329 | 322 |
| 3 | 16.29 | 329 | 322 |
| 4 | 11.88 | 329 | 322 |
| 5 | 17.79 | 329 | 322 |
| 6 | 11.38 | 329 | 322 |
| 7 | 12.92 | 329 | 322 |
| 8 | 9.21 | 329 | 322 |
| 9 | 11.46 | 329 | 322 |
| 10 | 10.00 | 329 | 322 |
| 11 | 13.25 | 331 | 324 |
| 12 | 8.38 | 331 | 324 |
| 13 | 10.29 | 331 | 324 |
| 14 | 8.21 | 331 | 324 |
| 15 | 7.62 | 331 | 324 |
| 16 | 7.54 | 331 | 324 |
| 17 | 7.54 | 331 | 324 |
| 18 | 9.21 | 331 | 324 |
| 19 | 9.92 | 331 | 324 |
| 20 | 7.92 | 331 | 324 |
| 21 | 7.79 | 331 | 324 |
| 22 | 7.67 | 331 | 324 |
| 23 | 7.46 | 331 | 324 |
| 24 | 7.42 | 331 | 324 |
| 25 | 7.42 | 331 | 324 |
| 26 | 8.04 | 331 | 324 |
| 27 | 7.42 | 331 | 324 |
| 28 | 7.46 | 331 | 324 |
| 29 | 7.42 | 331 | 324 |
| 30 | 7.25 | 331 | 324 |
| 31 | 13.62 | 331 | 324 |
| 32 | 8.83 | 331 | 324 |
| 33 | 7.75 | 331 | 324 |
| 34 | 15.33 | 331 | 324 |
| 35 | 10.29 | 331 | 324 |
| 36 | 10.50 | 331 | 324 |
| 37 | 7.83 | 331 | 324 |
| 38 | 7.42 | 331 | 324 |
| 39 | 7.46 | 331 | 324 |
| 40 | 7.54 | 331 | 324 |
| 41 | 7.38 | 331 | 324 |
| 42 | 7.38 | 331 | 324 |
| 43 | 7.38 | 331 | 324 |
| 44 | 7.29 | 331 | 324 |
| 45 | 7.29 | 331 | 324 |
| 46 | 7.25 | 331 | 324 |
| 47 | 7.29 | 331 | 324 |
| 48 | 7.29 | 331 | 324 |
| 49 | 7.29 | 331 | 324 |
| 50 | 7.33 | 331 | 324 |
| 51 | 7.33 | 331 | 324 |
| 52 | 7.33 | 331 | 324 |
| 53 | 7.25 | 331 | 324 |
| 54 | 7.33 | 331 | 324 |
| 55 | 7.33 | 331 | 324 |
| 56 | 7.33 | 331 | 324 |
| 57 | 7.25 | 331 | 324 |
| 58 | 7.33 | 331 | 324 |
| 59 | 7.38 | 331 | 324 |
| 60 | 7.33 | 331 | 324 |
| 61 | 7.42 | 331 | 324 |
| 62 | 7.46 | 331 | 324 |
| 63 | 7.33 | 331 | 324 |
| 64 | 9.04 | 331 | 324 |
| 65 | 7.42 | 331 | 324 |
| 66 | 7.29 | 331 | 324 |
| 67 | 7.42 | 331 | 324 |
| 68 | 7.25 | 331 | 324 |
| 69 | 7.38 | 331 | 324 |
| 70 | 7.33 | 331 | 324 |
| 71 | 7.29 | 331 | 324 |
| 72 | 7.38 | 331 | 324 |
| 73 | 7.29 | 331 | 324 |
| 74 | 7.38 | 331 | 324 |
| 75 | 7.33 | 331 | 324 |
| 76 | 7.42 | 331 | 324 |
| 77 | 7.33 | 331 | 324 |
| 78 | 7.38 | 331 | 324 |
| 79 | 7.29 | 331 | 324 |
| 80 | 7.42 | 331 | 324 |
| 81 | 7.25 | 331 | 324 |
| 82 | 7.46 | 331 | 324 |
| 83 | 7.42 | 331 | 324 |
| 84 | 7.42 | 331 | 324 |
| 85 | 7.38 | 331 | 324 |
| 86 | 7.17 | 331 | 324 |
| 87 | 7.29 | 331 | 324 |
| 88 | 7.33 | 331 | 324 |
| 89 | 7.33 | 331 | 324 |
| 90 | 7.42 | 331 | 324 |
| 91 | 7.25 | 331 | 324 |
| 92 | 7.29 | 331 | 324 |
| 93 | 7.29 | 331 | 324 |
| 94 | 7.33 | 331 | 324 |
| 95 | 7.33 | 331 | 324 |
| 96 | 7.29 | 331 | 324 |
| 97 | 7.29 | 331 | 324 |
| 98 | 7.33 | 331 | 324 |
| 99 | 7.25 | 331 | 324 |
| 100 | 7.29 | 331 | 324 |
</details>

## §4 Measured-vs-asserted delta

- Asserted (paper cell): "ms-class". Measured: median ~7.4 µs, p95 < 14 µs,
  max ~1.7 ms (single scheduling outlier per run). The ms-class figure is
  only observed at the max tail, not typical-case. Delta: the cell should
  read "median ~7.4 µs, tail to ~1.7 ms on an ambient M3 Max" — µs-class
  typical, ms-class only in the scheduling tail.
- Layer fence (F1/F5): the 10.20 ns/frame figure (row 5, claims row 5,
  quiet-host ingress admission) is a DIFFERENT layer; not comparable to
  these numbers; ~700x between the two figures is a layer gap, not a win.
- F3 boundary: commit-leg receipts are UNSUPPORTED (plan-p07 on file, not
  executed); this receipt covers only the events executed this session.

## §5 Verdict

MEASURED (ambient-host, scoped): stdio JSON-RPC round-trip median 7.40 µs,
min 7.17 µs, max 1.64 ms over N=100, 3 runs, exit 0 x3. The paper stdio
cell may quote: median ~7.4 µs / p95 ~14 µs / max ~1.7 ms, envelope
~330 B request + ~323 B response, single child process, newline-delimited.
If the paper needs ms-class framing, the tail figure ~1.7 ms is the only
honest support; typical-case is µs-class. This receipt does NOT cover
shared-memory channel (F4: ShmRingChannel not exercised by this lane).

## §6 Command transcript

    $ cd /tmp/p01-harness && cargo build --release 2>&1 | tail -5
        Finished `release` profile [optimized] target(s) in 8.95s
    $ ./target/release/p01-stdio-baseline 100        # warmup run, exit 0
    N=100
    latency_us min=7.25 median=8.83 mean=25.71 max=1640.83
    bytes request_min=329 request_max=331 response_min=322 response_max=324
    $ for i in 1 2 3; do ...; done                   # 3 measured runs, exit 0 x3
    run1: min=7.25 med=7.38 mean=25.15 max=1741.12   # csv -> run1.csv
    run2: min=7.08 med=7.46 mean=24.10 max=1610.58   # csv -> run2.csv
    run3: min=7.17 med=7.40 mean=24.60 max=1639.58   # csv -> run3.csv
    $ sysctl -n hw.model hw.ncpu machdep.cpu.brand_string kern.loadavg; uptime
    Mac15,9 / 16 / Apple M3 Max / loadavg 2.56 2.76 2.66 (22:37 up 22:25)
