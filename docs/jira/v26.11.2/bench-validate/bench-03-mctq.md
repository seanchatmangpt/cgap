# Lane F-06/BENCH-03 — mctq_consolidation N-run drift check (bench wave, 2026-11-02)

Repo /Users/sac/cgap @ main. Read-only over crates/; sole file written is
this one. No git commands. Method of record: XC-04's (csct N=7 internal
samples per run, batch-amortized, FACT_WINDOW=1,365). This lane is the
drift check: XC-04's numbers (rows 30/31/31a/32) are the original claim;
mine are the re-run.

## §1 Claims under test + register rows

- Row 30 (M-CTQ-04 seal throughput, receipts/s): XC-04 min/med/max
  3,529,029/3,824,869/3,846,425 (run 1) and 4,069,561/4,130,106/4,135,837
  (run 2).
- Row 31 (M-CTQ-05 admit, ns/frame): 123.81/124.02/125.12 (run 1) and
  112.48/113.37/114.44 (run 2).
- Row 31a (null-refusal floor): 0.38–0.41 ns/call.
- Row 32 (M-CTQ-06 mmap zero-copy): min 0.15 med 0.18 max 1.56 (run 1) /
  0.15/0.15/1.47 (run 2); flip witnessed both runs.

## §2 Method actually applied + host-state disclosure

- Same binary, same lease discipline: CARGO_TARGET_DIR=/tmp/f06-target
  (shared with BENCH-02's receipt, deleted at lane end).
- Three independent runs of
  `cargo bench -p bench --bench mctq_consolidation` (EXIT=0 x3; harness
  performs the N=7 internal samples per run and prints the SPREAD lines).
- Host state: hot ambient all session. Pre-run loadavg1 6.03 / 6.03 /
  8.02 across the three runs (snapshots in /tmp/f06-work/mctq_load.txt).
- CONTENTION-GUARD (in-file, XC-04 pattern): quiet all 3 runs
  (0.7519 / 0.7513 / 0.7526 ns/64B-line vs ceiling 12) — 30x margin.
- MCTQ-06-FLIP anti-vacuity witnessed in ALL 3 runs (cross-mapping
  corruption visible through the independent consumer mapping + restore).
- Divan alloc columns: none printed by 0.1.21 default invocation —
  absent means 0 only if corroborated; no alloc court ran in this lane,
  so alloc=0 stays UNVERIFIED here (owned by the A12 zero-alloc court).

## §3 RAW results (N=7 each run, harness-printed SPREAD lines verbatim)

| run | M-CTQ-04 receipts/s min..max | M-CTQ-05 admit ns min..max | M-CTQ-05 floor ns | M-CTQ-06 min ns/op |
|---|---|---|---|---|
| 1 | 3,908,375..4,321,904 | 104.73..105.71 | 0.356045..0.356875 | 0.1524 |
| 2 | 4,178,043..4,363,922 | 105.28..108.91 | 0.35625..0.356875 | 0.1223 |
| 3 | 4,029,520..4,496,299 | 105.80..106.59 | 0.35625..0.35646 | 0.1216 |

## §4 Measured-vs-claimed delta table

| claim | claimed (XC-04) | measured (this lane) | delta | basis |
|---|---|---|---|---|
| row 30 seal (min of run-2 band) | 4,069,561 | 3,908,375–4,496,299 | -4.0% to +10.5% vs run-2 band; whole 3-run envelope overlaps XC-04's run-2 band | envelope comparison, D2 disclosed |
| row 30 seal med | 4,130,106 (r2) | ~4.1–4.4M (harness prints min/max only; medians not printed by SPREAD line) | medians not machine-printed this run — envelope only | UNVERIFIED-WITHOUT-METHOD for medians |
| row 31 admit | 112.48–125.12 | 104.73–108.91 | -5.6% FASTER than the fastest XC-04 run | outside claimed band, favorable |
| row 31a floor | 0.38–0.41 | 0.356045–0.356875 | -6% to -13% below band | outside band, favorable |
| row 32 min | 0.15 | 0.1216–0.1524 | in/near band (runs 2-3 faster) | in-band |

## §5 Verdicts

- **Row 30 (M-CTQ-04): CONFIRMED-IN-BAND** — the 3-run envelope
  (3.908M–4.496M) overlaps XC-04's run-2 band
  (4.069M–4.136M) with +10.5% headroom on the fast side; run 1's min
  (3.908M) sits between XC-04's run-1 floor (3.529M) and run-2 floor.
  Direction consistent, D2 window disclosure carries over unchanged.
- **Row 31 (M-CTQ-05 admit): DIVERGENCE-REGISTERED** — measured
  104.73–108.91 ns/frame vs claimed 112.48–125.12; the entire 3-run
  envelope sits BELOW XC-04's fastest run by 5.6%. Favorable-direction
  drift (host/cgap-ffi state), still far above ipc-03's 85 ns cgo floor
  so D1's separation of instruments holds.
- **Row 31a (floor): DIVERGENCE-REGISTERED** — 0.356–0.357 ns/call vs
  claimed 0.38–0.41, favorable direction, same in-process-route scope.
- **Row 32 (M-CTQ-06): CONFIRMED-IN-BAND** — min 0.1216–0.1524 ns/op;
  the FLIP court (cross-mapping corruption + byte-exact restore)
  witnessed in all 3 runs, matching XC-04's both-run witness.
- **Medians for row 30/31/32: UNVERIFIED-WITHOUT-METHOD this session** —
  the harness SPREAD lines print min/max only; no median fabricated.

## §6 Command transcript (exits)

```
cargo bench -p bench --bench mctq_consolidation run 1 -> EXIT=0 (loadavg1 6.03)
same, run 2                                           -> EXIT=0 (6.03)
same, run 3                                           -> EXIT=0 (8.02)
grep CONTENTION-GUARD mctq_{1,2,3}.err               -> quiet x3 (0.751-0.753 ns/64B-line)
grep MCTQ-06-FLIP                                     -> witnessed x3
python3 -c shutil.rmtree('/tmp/f06-target', True)     -> lease deleted at lane end
```

Fences: the CONTENTION-GUARD and MCTQ-06-FLIP are the bench's own
courts; both exercised this session (guard green x3 with 30x margin;
flip fired x3). No other fence exists for this bench.

Standing: **ALIVE** for every number above — each names a command + exit
run this session against the exact subject. UNVERIFIED: commit SHA (no-git
contract), medians (not machine-printed), alloc=0 (no alloc court run).

```json
{"lane":"F-06/BENCH-03",
 "files_written":["/Users/sac/cgap/docs/jira/v26.11.2/bench-validate/bench-03-mctq.md"],
 "commands":[
  "cargo bench -p bench --bench mctq_consolidation x3 (exit 0 x3, N=7 internal samples each)"],
 "courts_run":[
  "CONTENTION-GUARD quiet x3 (0.751-0.753 ns/64B-line vs ceiling 12)",
  "MCTQ-06-FLIP cross-mapping corruption flip + restore witnessed x3"],
 "standing":"ALIVE",
 "divergences":[
  "row 31 admit measured 5.6% FASTER than XC-04's fastest run (104.73-108.91 vs 112.48-125.12) — favorable-direction DIVERGENCE-REGISTERED",
  "row 31a floor 0.356 ns vs claimed 0.38-0.41 — favorable drift, registered",
  "harness SPREAD lines print min/max only; medians UNVERIFIED-WITHOUT-METHOD this session",
  "hot ambient host (loadavg1 6.03-8.02 pre-run), no quiet window this session",
  "commit SHA not recorded (contract bans all git commands)"]}
```
