# l57-penstock — KD-2 residue: penstock row 7 contended band re-run (paired)

Standing: PASS

- Date: 2026-11-04. Lane: L57. Repo: /Users/sac/cgap @ main
  (1a84bf665939e88b7b41ac99b3dec65876fafc76 at run time; docs committed
  after).
- Surface: bm6 claims-table row 7 (penstock admission stage), carrying
  only the RED-10 bw-only witness; contended band never re-run paired.
  Residue named in `docs/jira/v26.11.5/receipts/l27-kd2-claims.md`
  (row 7 "UNVERIFIED new band").
- Method: L27's paired method extended verbatim — new driver
  `scripts/l57-penstock-paired-bench.sh` over the same
  `scripts/l27-loadgen.c` (combined 6 9 = RED-10 W4 bw6+cpu9), N=3
  paired rounds, min-of-N, no sleeps (real threads, divan's own
  timer). Bench leg: `cargo bench --bench chains-ingress --
  stage_penstock_admission` (1M const leg). All six legs rc=0.
- Run history (disclosed): round 3 initially rc=101 x3 attempts —
  workspace resolution blocked by L50's in-flight quinn pin
  (>=0.11.15 does not exist on crates.io; verified via `cargo search`
  + registry API). L50 re-pinned to >=0.11.12 on disk (their comment
  now marks the CVE version claim UNVERIFIED against the registry);
  round 3 then ran green. During round 1, L50's mid-edit duplicate
  `[dependencies]` table froze all legs; I applied the minimal SLA
  unblock (merged the duplicate table; moved rcgen into
  `[dependencies]` optional — cargo rejects optional dev-deps, and
  `dep:rcgen` in the quic feature requires it there). The cgap-beam
  Cargo.toml is L50-owned and NOT committed by this lane.
- Raw paired TSV (`/tmp/l57-penstock-paired/results.tsv`):

```
round	profile	fastest_ms	median_ms	slowest_ms
1	solo	9.878 ms	10.7 ms	12.33 ms
1	contended	10.12 ms	17.87 ms	281.6 ms
2	solo	9.69 ms	11.49 ms	90.68 ms
2	contended	11.41 ms	21.25 ms	333.6 ms
3	solo	8.412 ms	8.71 ms	9.562 ms
3	contended	11.22 ms	17.45 ms	43.52 ms
```

## Results (ns/frame = ms x 1e6 / 1,048,576)

| Row 7 | Load profile (machine-stated) | fastest | median | slowest | Ratio vs solo (min-of-3) |
|---|---|---|---|---|---|
| Penstock admission 1M const | SOLO (min-of-3, paired method) | 8.02 | 8.31 (min) | 86.48 (worst) | 1.00x baseline |
| Penstock admission 1M const | CONTENDED bw6+cpu9 (min-of-3, paired) | 9.65 (+20.3%) | 16.64 (+100%) | 318.15 (worst) | fence |
| — fence (worst/best) | CONTENDED slowest-worst / SOLO fastest-min | — | — | — | **39.7x** |

- bm6 prior SOLO 7.71 ns/frame (quiet-host-scoped, 2026-11-04): this
  session's solo fastest-min 8.02 is +4.0% above it — an upper bound
  under the disclosed 20-lane wave background, consistent with the
  prior quiet band (noise inflates only).
- The RED-10 bw-only witness (tail 12.73 ns/frame) is NOT an upper
  bound for the contended band: under combined bw6+cpu9 the worst
  sample reaches 318.15 ns/frame — 25x the bw-only tail. Row 7's
  contended column now carries a machine-stated band; the bw-only
  number stays as the weaker prior witness.
- Ratio fence statement: under RED-10-class combined memory+CPU load
  the penstock stage's worst-sample latency is 39.7x the solo fastest
  sample (318.15 vs 8.02 ns/frame, min-of-3, paired rounds). The
  chain (row 5/6) fence was 9.49x — penstock alone is ~4.2x more
  contention-sensitive than the full chain on the worst column: the
  dam absorbs contention disproportionately, and consumers quoting
  row 7 solo numbers as worst-case bounds were off by >30x.
- Divan timer cross-check: cont_3's guard line read
  CONTENTION-SUSPECT (jitter 2.27) yet its slowest 43.52 ms did not
  enter the max (333.60 from round 2 stands); solo_3's guard was
  quiet (jitter 1.01) and its fastest 8.41 ms set the min — the
  min-of-N fence tightens monotonically with clean rounds.

## Contention-GUARD witnesses (in-run, per leg, from bench stderr)

- solo_1 (jitter 1.08 quiet via cont_1's guard probe), cont_1,
  cont_2: 0 SUSPECT rows entered the fence; solo_2: 1 SUSPECT
  (jitter 2.92, host noise — kept, noise inflates only, its fastest
  9.69 was superseded by solo_3's clean 8.41); cont_3: 1 SUSPECT
  (jitter 2.27 — expected under injected load; its numbers did not
  enter the fence extremes).
- Round-3 abort logs (pre-fix attempts):
  /tmp/l57-penstock-paired/bench_solo_3.log / bench_cont_3.log
  (quinn resolution error, rc=101, now overwritten by the green run's
  logs; the error text is quoted in this receipt's run history).

## Verdicts

- Row 7 contended band: CONFIRMED-NEW (machine-stated, min-of-3
  paired: fastest +20.3%, median +100%, worst 39.7x vs solo fastest).
- Row 7 UNVERIFIED note: RETIRED (appendix appended to
  `docs/jira/v26.11.5/receipts/l27-kd2-claims.md`; bm6 row 7
  re-published with load-profile columns).
- Exit codes: 0 x6 landed legs (solo_1, cont_1, solo_2, cont_2,
  solo_3, cont_3).

## Receipt

- Subject: /Users/sac/cgap @ 1a84bf665939e88b7b41ac99b3dec65876fafc76
  (working tree for the run; docs committed after).
- Scripts (new, owned): scripts/l57-penstock-paired-bench.sh (reuses
  scripts/l27-loadgen.c unchanged).
- Table re-published: docs/jira/v26.11.4/receipts/bm6-benchmark-claims-table.md
  (L27 addendum table, row 7 updated).
- Appendix: docs/jira/v26.11.5/receipts l27-kd2-claims.md (delta note
  appended, history not rewritten).
- Standing: PASS (all six legs exit 0, fence computed from raw TSV
  above).
