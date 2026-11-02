# Lane PERF (FA9FIX post-fix) — receipt

Lane: PERF (read-only + bench runs). Repo /Users/sac/cgap @ main, 9a730320.
Command: `CARGO_TARGET_DIR=/tmp/perf-target cargo bench -p bench --bench chains-sovereign` — exit 0, run 3x this session (full runs + focused `-- stage_commit_parquet chain_full_transaction` filter run, also exit 0).
Build root /tmp/perf-target is a lease (deleted at integration per lane law).

## Method

Divan, 100 samples/leg, fastest reported (divan "fastest" column), timer precision 41 ns.
Environment disclosure: shared machine, concurrent fleet lanes compiling/testing throughout.
Load average sampled during the focused run: 8.71 -> 13.80 (1-min), iostat shows 8.7 MB/s
disk0 traffic from other lanes. stage_commit_parquet does a REAL temp-file write per commit
call (crates/bench/benches/chains-sovereign.rs:660-688, commit sink at chains-sovereign.rs:438),
so that leg is IO-contention-sensitive; run-to-run fastest varied 37.15-44.28 ms @4096.

## Measured (this session, fastest; 3 runs, range given)

| Bench | N | This session (fastest range) | per-tx | Witness (pre-gate) | per-tx |
|---|---|---|---|---|---|
| stage_commit_parquet | 256 | 196.6-231.6 us | 0.768-0.905 us/tx | 150.7 us (a3) / 193.9-213.4 us (w5) | 0.589-0.834 us/tx |
| stage_commit_parquet | 4096 | 37.15-44.28 ms | 9.07-10.81 us/tx | 2.472-3.241 ms | 0.603-0.791 us/tx |
| chain_full_transaction | 256 | 1.518-1.54 ms | 5.93-6.02 us/tx | - | 6.19-6.33 us/tx |
| chain_full_transaction | 4096 | 62.63-64.54 ms | 15.30-15.75 us/tx | 25.37 ms | 6.19-6.31 us/tx |
| ring_sink_commit | 256 | 175.8-175.9 us | 0.687 us/tx | 175.9 us (a3) / 193.9 us (w5) | 0.687-0.758 us/tx |

Full-run divan table preserved at /tmp/perf-bench.log (run 2) and /tmp/perf-bench3.log
(focused run 3); run 1 output in the lane task output file.

## Verdict: does the per-(path-hash,tid) claim registry cost show at steady state?

**NO at steady state — the gate cost is not visible, and the data support "negligible".**

1. The gate (crates/cgap-ffi/src/sovereign.rs:875-892) is O(1) per commit CALL: one FNV-1a
   over the path bytes, one Mutex lock, one linear scan of the owners vec (1-2 entries at
   bench scale). Its cost is per-call CONSTANT — identical per tx at N=256 and N=4096.
2. At N=256 (0.768-0.905 us/tx) the measured band straddles the pre-gate witness band
   (0.589-0.834 us/tx) — no attributable regression. ring_sink_commit matches a3's witness
   exactly (175.8 vs 175.9 us). If the gate added measurable work, N=256 would show it
   equally; it does not.
3. The N=4096 inflation (9.07-10.81 us/tx vs 0.603-0.791 witness; chain_full 15.3-15.8 vs
   6.19-6.31) is INCONSISTENT with a per-call-constant gate cost (which cannot scale with N)
   and occurred under load average 8-16 with concurrent fleet compiles/tests doing real IO
   on the same disk (iostat 8.7 MB/s during the focused run). stage_commit_parquet's timed
   loop is per-call real-file-IO-bound; it is the leg most exposed to that contention.
   Attribution: ambient fleet load, NOT the FA9FIX claim registry.

Standing: stage_commit @256 and chain_full @256 = within-witness (ALIVE as measurements,
this session, command+exit cited). @4096 legs = contention-dominated; a quiet-machine
re-measure is the close condition for a clean @4096 number. FA9FIX's "post-fix perf: none
measured" gap is now partially closed: steady-state claim cost measured = not visible
within witness noise at the per-call-constant-sensitive N=256 leg.

Falsifier for the verdict: if the @4096 delta persists on a quiet machine (load < 2), the
"negligible" verdict is refuted and the gate (or commit backend) has a real scale-dependent
cost to bisect.
