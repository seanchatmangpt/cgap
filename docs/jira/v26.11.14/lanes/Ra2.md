# Ra2 — penstock contended band re-run (KD-2 residue)

Standing: PARTIAL (measured; host-scoped, does not replace the L57 band)

Note on premise: the residue named in `docs/jira/v26.11.5/receipts/l27-kd2-claims.md:49-52`
was already retired by its appendix (L57, lines 99-118; `l57-penstock.md`). This lane
is an independent second-host replication of that paired measurement.

## Method
- Bench exists: `crates/bench/benches/chains-ingress.rs` leg `stage_penstock_admission`
  (1M const leg, divan, 100 samples). Driver `/tmp/Ra2/run.sh` (L57 method; repo untouched).
- Host: Linux 6.18, **4 cores** (nproc=4), 16 GiB RAM. Prior L27/L57 host core count is
  not stated in their receipts (macOS arm64) - not comparable.
- Load: `scripts/l27-loadgen.c` built with gcc to `/tmp/Ra2/loadgen`, `combined 6 9`
  = 15 threads on 4 cores (3.75x oversubscribed; 3 GiB bw buffers). Contention is real
  and heavier per core than the L57 profile likely was. No sleeps.
- N=3 paired rounds, SOLO then CONTENDED, min-of-3. CARGO_TARGET_DIR=/tmp/target-Ra2.
- Commands: `cargo bench --bench chains-ingress -- stage_penstock_admission` x7
  (1 warm + 6 paired), all rc=0 (`/tmp/Ra2/rc.log`).

## Raw (`/tmp/Ra2/results.tsv`)
```
round profile    fastest   median    slowest
1 solo           22.23 ms  24.86 ms  32.51 ms
1 contended      132.7 ms  624 ms    1.063 s
2 solo           20.93 ms  22.34 ms  35.71 ms
2 contended      21.91 ms  598.9 ms  979.4 ms
3 solo           21.08 ms  24.3 ms   45.37 ms
3 contended      21.03 ms  310.1 ms  597.1 ms
```

## Results (ns/frame = ms x 1e6 / 1,048,576; min-of-3 fastest/median, worst slowest)
| profile | fastest | median | slowest |
|---|---|---|---|
| SOLO | 19.96 | 21.31 | 43.27 |
| CONTENDED bw6+cpu9 | 20.06 (+0.5%) | 295.73 (13.9x) | 1013.76 |
Fence worst/best = 1013.76 / 19.96 = **50.8x** (L57 host: 39.7x).

## Guard witnesses
- solo legs: CONTENTION-GUARD quiet x3 (jitter ratio 1.09 / 1.41 / 1.39).
- contended legs: CONTENTION-SUSPECT emitted (round 1: ratio 5.06) - expected, valid
  as contended-profile numbers only. Disclosure: guard-line values for cont_2/cont_3
  are in `/tmp/Ra2/b_cont_*.log`; only round 1 was quoted here.

## Verdicts
- Fastest column survives contention (+0.5%), reproducing L57's direction (+20%).
- Tail is unbounded-class: worst 1.0 us/frame; L57's 39.7x fence is a lower bound
  on this host class. Consumers must not quote solo penstock ns as a worst case.
- Absolute solo is 2.5x L57's (19.96 vs 8.02 ns/frame): different host; numbers
  are host-scoped measurements, not TARGET-promotable to the quiet band.
- UNVERIFIED: ratio on the L57 host class; effect of oversubscription (15 threads/4
  cores) versus true cross-core contention is not separated.

## Mutation / anti-vacuity
Named mutation: omit loadgen (SOLO leg) - the paired contended-vs-solo delta
flips (median 13.9x, guard SUSPECT vs quiet). The solo legs are the mutation-free
control; the guard marker flipping quiet->SUSPECT is the in-run witness.

## Fences / files
- Repo files written: this receipt only. Fences untouched (no source diff).
- Cleanup: /tmp/target-Ra2 removal below.
