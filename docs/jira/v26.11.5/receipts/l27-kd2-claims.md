# l27-kd2-claims — KD-2 load-profile re-derivation (paired solo/contended bench)

Standing: PASS

- Date: 2026-11-04. Lane: L27. Repo: /Users/sac/cgap @ main
  (11f8640b at run time).
- Surface: the ns/frame claims of
  `docs/jira/v26.11.4/receipts/bm6-benchmark-claims-table.md` /
  `docs/jira/v26.11.2/11-measured-claims.md` rows 5-8 (quiet-host-scoped
  ingress chain), contested by RED-10 W5 (row 28: contended slowest
  124.06 ns/frame, +826% vs quiet fastest 10.66).
- Divergence closed via its close condition: rows re-published with
  load-profile columns + ratio fence against a sleep-free baseline
  (KD-2 row updated with the receipt path; court that documents is not
  a fix — the measurement is the fix).

## Method (no-sleep ratio-fence law)

- Paired rounds, min-of-N: N=3 rounds; each round runs the SAME bench
  (divan `chains_ingress::ingress_chain`, 1M const leg, 100 samples)
  twice — SOLO then CONTENDED — only the environment differs.
- Contended profile = RED-10 W4 load classes reproduced by
  `scripts/l27-loadgen.c` (`combined 6 9`: 6 bw threads x 512 MiB
  private-buffer 64B-stride streams + 9 cpu threads of dependent
  int/FP chains). Sleep-free: no sleeps anywhere; load is real
  threads; timing is divan's own 41 ns timer.
- Driver: `scripts/l27-kd2-paired-bench.sh` (exit 0; all 6 bench legs
  `cargo bench --bench chains-ingress -- ingress_chain`, CARGO_TARGET_DIR=/tmp/l27-kd2-target, rc=0 x6, log `/tmp/l27-kd2-paired/rc.log`).
- Raw paired table `/tmp/l27-kd2-paired/results.tsv`:

```
round	profile	fastest_ms	median_ms	slowest_ms
1	solo	11.43	12.10	12.91
1	contended	11.75	24.00	80.22
2	solo	11.60	12.43	21.07
2	contended	13.97	23.39	103.00
3	solo	10.85	11.22	39.67
3	contended	13.47	23.35	43.90
```

## Re-published rows (ns/frame = ms x 1e6 / 1,048,576)

| Claim row | Load profile (machine-stated) | fastest | median | slowest | Ratio vs solo (same column, min-of-N) |
|---|---|---|---|---|---|
| 5/6 ingress chain 1M const | SOLO (min-of-3, paired method) | 10.35 | 10.70 (min) | 37.83 (worst) | 1.00x baseline |
| 5/6 ingress chain 1M const | CONTENDED bw6+cpu9 (min-of-3, paired) | 11.21 (+8.3%) | 22.27 (+108%) | 98.23 (worst) | — |
| — fence (worst/best) | CONTENDED slowest-worst / SOLO fastest-min | — | — | — | **9.49x** |

- Penstock stage (row 7): NOT re-run this lane — its load-profile
  column carries the RED-10 W5 witness (fastest +1%, tail 12.73 ns/frame
  under bw-only), status UNVERIFIED-new-band until a paired re-run.
- Ratio fence statement: under RED-10-class combined memory+CPU load
  the chain's worst-sample latency is 9.49x the solo fastest sample
  (98.23 vs 10.35 ns/frame, this session). This replaces the unbounded
  "quiet ns/frame" reading; consumers must not quote solo numbers as
  worst-case bounds. The quiet fastest (10.35, min-of-3) reproduces the
  receipted 10.20-10.66 band within 1.5%.
- Cross-check vs RED-10 (2026-11-02): their contended slowest 124.06
  (+826% vs 10.66); ours 98.23 (+849% vs 10.35 fastest-min). Same
  class, same order — the fence is reproduced on a different day/host
  load. Fastest column again survived contention best (+8.3% here vs
  -3.4% cpu-only in RED-10; the cpu-only class is weaker than combined).

## Contention-GUARD witnesses (in-run, per leg, from bench stderr)

- solo_1 / solo_2: 0 SUSPECT flags; quiet guard jitter ratio 1.01 (under ceiling 2).
- solo_3: 1 SUSPECT (slowest outlier 39.67 ms — fleet noise; disclosed,
  kept in min-of-N set because the fence uses min).
- cont_1: 1, cont_2: 2, cont_3: 2 SUSPECT flags — expected under
  injected load (the guard is the bench's own in-run noise witness).
- Disclosure: the host was NOT quiet fleet-wide (20-lane wave in
  flight); every number is an upper bound under that background, same
  for both profiles of a paired round, so the fence ratio is
  background-symmetric.

## Verdicts

- Row 5/6 (ingress chain quiet band): CONFIRMED as SOLO-scoped
  (min-of-3 fastest 10.35 reproduces 10.20-10.66 within 1.5%).
- Row 28 / KD-2 fence: CONFIRMED (9.49x worst/best fence, same class
  as RED-10's 12.2x).
- Row 7 (penstock): UNVERIFIED new band (prior witness only).
- Exit code: 0 (driver), 0 x6 (bench legs).

## Receipt

- Subject: /Users/sac/cgap @ 11f8640b (working tree for the run; docs
  committed after).
- Scripts (new, owned): scripts/l27-loadgen.c, scripts/l27-kd2-paired-bench.sh
- Table re-published: docs/jira/v26.11.4/receipts/bm6-benchmark-claims-table.md
  (addendum "Load-profile re-publication (L27)").
- KD-2 row: docs/jira/v26.11.4/KNOWN_DIVERGENCES.md (close condition
  met; receipt path cited).
- Standing: PASS (all six legs exit 0, fence computed from raw TSV
  above; UNVERIFIED residue: penstock new band, fleet background load).
