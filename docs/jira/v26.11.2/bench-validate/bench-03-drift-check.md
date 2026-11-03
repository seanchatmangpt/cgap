# Lane FS-02 — bench drift check vs F-06 + XC-04 originals (bench wave, 2026-11-02)

Repo /Users/sac/cgap @ main. Read-only over crates/; sole file written is
this one (`bench-03-drift-check.md`, per coordinator collision notice —
F-06 owns `bench-03-mctq.md` and `bench-02-seal-bn3.md`, both landed and
read by this lane BEFORE any run). No git commands. Method of record:
XC-04's (csct N=7 internal samples per run, batch-amortized,
FACT_WINDOW=1,365) and divan 0.1.21 fastest-column for the chains-bn3 /
bn3 / seal legs.

## §1 Claims under test + register rows

- Register rows 30/31/31a/32 (`docs/jira/v26.11.2/11-measured-claims.md:106-109`,
  consolidation-addendum class): XC-04 originals, cited verbatim from
  that receipt (lane-xc04-mctq-consolidation.md:36-37,51-52,47-49,66-67).
- The 82.24 GB/s / ~460 ns P28 dossier row (REVENUE compendium §6.3,
  REVENUE_GENERATION_COMPENDIUM.md:337): REGISTER STATUS this lane's
  finding — the figure is ABSENT from the v26.11.2 claims register
  (11-measured-claims.md rows 1-38 contain no 82.24 / 460 row); it is
  carried only by v26.11.1 receipts (waves/bench-bn3-receipt.md:69 for
  82.24; paper-rewitness.md:11 / bench-seal-receipt.md:64-65 for ~460 ns)
  and the compendium row itself flags "UNVERIFIED-WITHOUT-METHOD for the
  v26.11.2 claims register". Also: NEITHER figure is a chains-bn3 leg —
  82.24 is `--bench bn3` statement_scan_24b@65536, 460 ns is `--bench
  seal` p28_attestation_verify@1024. Recorded loudly per contract; both
  exact legs were still run with their real names.
- chains-bn3 legs (exact names): deduction_chain, stage_scan_24b,
  stage_compile_rules, stage_unify_join, stage_unify_join_radix,
  stage_project_consequents, closure_ladder — all present with [[bench]]
  name = "chains-bn3" (crates/bench/Cargo.toml:91-92); no leg lacked a
  manifest entry.

## §2 Method actually applied + host-state disclosure

- Build root lease: CARGO_TARGET_DIR=/tmp/fs02-target (deleted at lane
  end). Builds exit 0 x2 (`cargo bench --no-run` for chains-bn3+mctq,
  then bn3+seal; 15.96 s / 3.29 s).
- HOT AMBIENT HOST ALL SESSION: loadavg1 2.92-21.63; no pre-run poll
  reached the 2.0 quiet threshold (3 polls / 90 s deadline each leg,
  /tmp/fs02-work/quiet_poll.py — first version had an uptime-parse bug
  on the piped "load averages:" plural form, fixed in-session before any
  recorded leg).
- Runs: mctq_consolidation x2 (EXIT=0 x2), chains-bn3 filtered legs
  `-- deduction_chain stage_` (EXIT=0, 0 CONTENTION-SUSPECT markers),
  `--bench bn3 -- statement_scan_24b` (EXIT=0), `--bench seal --
  p28_attestation_verify` (EXIT=0).
- Loud skips: closure_ladder NOT re-run (all consts incl. 1M) — F-06's
  bounded-window kill (bench-02-seal-bn3.md:128-132: 268.9 ms @65536 ->
  5.054 s @262144 superlinear, pid-killed ~40 min, exit 101 typed kill)
  stands; re-running the 1M leg inside this lane's window would repeat
  that. closure_ladder@65536 not isolated either (divan filter would
  pull 1M). [SKIP] loud, reason stated, per contract.
- Courts exercised: CONTENTION-GUARD quiet x2 (0.809 / second-run value
  below) with 15x margin; MCTQ-06-FLIP cross-mapping corruption flip +
  restore witnessed x2; chains-bn3 in-band Chicago asserts (oracle
  counts expected_sel/expected_join, checksum ck_sel/ck_join) passed
  100x per leg — the flip surface, per the bench's own mutation-class
  doc comments.

## §3 RAW results (this lane's runs)

### §3.1 mctq_consolidation (N=7 internal samples per run, harness SPREAD lines verbatim)

| run | M-CTQ-04 receipts/s min..max | M-CTQ-05 admit ns min..max | floor ns | M-CTQ-06 min ns/op | guard | FLIP |
|---|---|---|---|---|---|---|
| 1 | 3,972,839..4,320,194 | 105.04..105.89 | 0.356..0.357 | 0.1216 | quiet (0.809) | witnessed |
| 2 | 3,869,135..4,465,644 | 106.17..108.67 | 0.356..0.368 | 0.1223 | quiet (0.819, 15x margin, 0 SUSPECT markers) | witnessed |

Medians seen on stdout this session (not machine-printed by SPREAD
lines): M-CTQ-04 med 4,291,332 / 4,386,134; admit med 105.19 / 106.56;
M-CTQ-06 med 0.15 / 0.15 — recorded as stdout witness only, NOT
machine-verified SPREAD output.

### §3.2 chains-bn3 filtered legs (exit 0, 0 SUSPECT markers, loadavg1 5.78-8.91)

| leg | N | fastest | median | throughput |
|---|---|---|---|---|
| deduction_chain | 65536 | 1.732 ms | 1.76 ms | 907.6 MB/s |
| deduction_chain | 1,048,576 | 27.92 ms | 28.7 ms | 901.3 MB/s |
| stage_scan_24b | 65536 | 125.9 µs | 126.2 µs | 12.49 GB/s |
| stage_scan_24b | 1,048,576 | 2.333 ms | 2.356 ms | 10.78 GB/s |
| stage_compile_rules | 65536 | 421.5 ns | 429.3 ns | — |
| stage_compile_rules | 1,048,576 | 415.6 ns | 458.6 ns | — |
| stage_unify_join (nested) | 65536 | 8.366 ms | 8.431 ms | — |
| stage_unify_join (nested) | 1,048,576 | 1.926 s | 1.942 s | — |
| stage_unify_join_radix | 65536 | 1.439 ms | 1.463 ms | — |
| stage_unify_join_radix | 1,048,576 | 23.42 ms | 23.66 ms | — |
| stage_project_consequents | 65536 | 159.1 µs | 160.5 µs | — |
| stage_project_consequents | 1,048,576 | 2.078 ms | 2.091 ms | — |

## §4 Cross-reconciliation (drift check) vs F-06's landed numbers + register rows

### §4.1 mctq stations vs rows 30/31/31a/32 and F-06's bench-03-mctq.md

| claim | XC-04 original (register) | F-06 (bench-03-mctq.md) | FS-02 (this lane) | verdict |
|---|---|---|---|---|
| row 30 seal receipts/s | 3.529M-4.136M (r1/r2 bands) | 3.908M-4.496M (3 runs) | 3.869M-4.466M (2 runs) | CONFIRMED-IN-BAND — envelopes overlap; this lane's medians 4.29M/4.39M straddle F-06's 3-run envelope top |
| row 31 admit ns/frame | 112.48-125.12 | 104.73-108.91 | 105.04-108.67 | CONFIRMS F-06's DIVERGENCE-REGISTERED: favorable drift below XC-04's band, consistent across 2 independent lanes |
| row 31a floor ns/call | 0.38-0.41 | 0.356-0.357 | 0.356-0.368 | CONFIRMS F-06's favorable divergence (0.356-0.368 vs 0.38-0.41) |
| row 32 mmap ns/op | 0.15 med, max 1.47-1.56 | min 0.1216-0.1524 | min 0.1216-0.1223, med 0.15 | CONFIRMED-IN-BAND — med 0.15 both runs; run-1 max 1.25 / run-2 max 1.68 warmup tail, same shape as XC-04's |
| row 12 (context, F-06's bench-02 subject) | 3.04 / 2.37 µs/row | 3.075 / 2.406 | not re-run (not dispatched) | F-06's +1.15-1.53% verdict UNCHANGED, no counter-evidence |

### §4.2 dossier row vs F-06's bench-02 §3.4/§3.5

| figure | dossier / v26.11.1 | F-06 | FS-02 | verdict |
|---|---|---|---|
| 82.24 GB/s (statement_scan_24b@65536, divan fastest) | 82.24 | 80.49 (-2.1%) | **82.24 GB/s fastest (19.12 µs) — exact reproduction** | CONFIRMED-IN-BAND on both re-runs; this lane's number is bit-identical to the dossier figure |
| scan@1M | 78.94 | 78.53 | 78.3 | CONFIRMED-IN-BAND (-0.8%) |
| ~460 ns P28 (p28_attestation_verify@1024) | 459.96 | 462.3 (+0.51%) | **458.0 ns (468.8 µs/1024)** | CONFIRMED-IN-BAND (-0.4%) |
| @65536 seal | 30.09 ms | 30.18 | 30.2 ms | CONFIRMED-IN-BAND |

## §5 Verdicts per number

- **Row 30 (M-CTQ-04): CONFIRMED-IN-BAND** — 2-run envelope
  3.869M-4.466M receipts/s overlaps XC-04's run-2 band and F-06's 3-run
  envelope; medians 4.291M/4.386M straddle F-06's top. D2 window
  disclosure carries over unchanged (1,365 vs b06's 1M).
- **Row 31 (M-CTQ-05 admit): CONFIRMS F-06's DIVERGENCE-REGISTERED
  verdict** — 105.04-108.67 ns/frame, entire envelope below XC-04's
  112.48-125.12 band, favorable direction, same shape and magnitude as
  F-06's 104.73-108.91. Two independent lanes on one session now agree
  the drift is real, not single-run noise. D1 (in-process vs cgo,
  ipc-03's 85 ns floor) carries over — this lane's floor figure is
  still 20+ ns above the cgo floor.
- **Row 31a (floor): CONFIRMS favorable divergence** — 0.356-0.368
  ns/call vs claimed 0.38-0.41.
- **Row 32 (M-CTQ-06): CONFIRMED-IN-BAND** — med 0.15 ns/op both runs,
  FLIP court witnessed x2, warmup-tail max same shape as XC-04/F-06.
- **82.24 GB/s dossier figure: CONFIRMED-IN-BAND, register-absent
  status recorded** — this lane reproduced it EXACTLY (19.12 µs
  fastest = 82.24 GB/s, --bench bn3 statement_scan_24b@65536). The
  figure is a v26.11.1 receipted number ABSENT from the v26.11.2
  register; with F-06's -2.1% and this lane's exact hit, it now has two
  in-campaign re-measurements and is a promotion candidate for an
  11-measured-claims addendum row (coordinator-owned file, not written
  by this lane).
- **~460 ns P28 dossier figure: CONFIRMED-IN-BAND, register-absent
  status recorded** — 458.0 ns (-0.4%), second in-campaign
  re-measurement agreeing with F-06's 462.3. Same promotion-candidate
  note.
- **chains-bn3 stage legs: datapoints only, no register row** — no
  drift (this lane vs F-06 within noise: scan@65536 12.49 vs 11.7 GB/s,
  unify_join radix 1.439 vs 1.574 ms — this lane slightly faster,
  hot-ambient both).
- **Radix gate (M-L3-1 >=10x @65536): NOT met again** — 8.366/1.439 =
  5.81x this lane (F-06 saw 4.91x; v26.11.1 receipt's gate arithmetic
  was against different corpus shape). Registered datapoint; the gate
  owner decides.

## §6 Command transcript (exits)

```
cargo bench -p bench --bench chains-bn3 --bench mctq_consolidation --no-run -> EXIT=0 (15.96s)
cargo bench -p bench --bench bn3 --bench seal --no-run                      -> EXIT=0 (3.29s)
python3 /tmp/fs02-work/quiet_poll.py (per leg)          -> NOT-QUIET every leg (loadavg1 2.92-21.63)
cargo bench -p bench --bench mctq_consolidation run 1   -> EXIT=0, guard quiet (0.809), FLIP witnessed
cargo bench -p bench --bench mctq_consolidation run 2   -> EXIT=0, guard quiet (0.819), FLIP witnessed
cargo bench -p bench --bench chains-bn3 -- deduction_chain stage_ -> EXIT=0, 0 SUSPECT markers
cargo bench -p bench --bench bn3 -- statement_scan_24b  -> EXIT=0 (82.24 GB/s @65536)
cargo bench -p bench --bench seal -- p28_attestation_verify -> EXIT=0 (458.0 ns @1024)
[SKIP] closure_ladder (all consts): loud skip, F-06's bounded-window kill stands
python3 -c shutil.rmtree('/tmp/fs02-target', True)      -> lease deleted at lane end
```

Fences: the mctq bench's own courts (CONTENTION-GUARD, MCTQ-06-FLIP)
exercised both green this session; the chains-bn3 in-band Chicago
asserts are the mutation surface and passed 100x per leg. Must-not-
regress fences the runs could NOT break: this lane edited zero crate
files (read-only over crates/); the only fence touched was wall-clock.

Anti-vacuity: MCTQ-06-FLIP fired x2 (a shared-page court that cannot
flip would be FAIL); the mctq zero-rate / non-OK-status typed FAIL legs
were armed but never fired because every station measured real,
non-zero, in-band values.

Standing: **ALIVE** for every number above — each names a command +
exit run this session against the exact subject. UNVERIFIED: commit
SHA (no-git contract), mctq medians (stdout witness only, not
machine-printed SPREAD output), closure_ladder (skipped loud).

```json
{"lane":"FS-02",
 "files_written":["/Users/sac/cgap/docs/jira/v26.11.2/bench-validate/bench-03-drift-check.md"],
 "commands":[
  "cargo bench -p bench --bench chains-bn3 --bench mctq_consolidation --no-run (exit 0)",
  "cargo bench -p bench --bench bn3 --bench seal --no-run (exit 0)",
  "cargo bench -p bench --bench mctq_consolidation x2 (exit 0 x2)",
  "cargo bench -p bench --bench chains-bn3 -- deduction_chain stage_ (exit 0)",
  "cargo bench -p bench --bench bn3 -- statement_scan_24b (exit 0)",
  "cargo bench -p bench --bench seal -- p28_attestation_verify (exit 0)"],
 "courts_run":[
  "CONTENTION-GUARD quiet x2 (0.809 / 0.819 ns/64B-line vs ceiling 12)",
  "MCTQ-06-FLIP cross-mapping corruption flip + restore witnessed x2",
  "chains-bn3 in-band Chicago asserts (oracle counts + checksums) 100x per leg",
  "[SKIP] closure_ladder all-consts: loud, F-06 bounded-window kill stands"],
 "standing":"ALIVE",
 "divergences":[
  "82.24 GB/s / 460 ns dossier figures are NOT chains-bn3 legs: exact subjects are --bench bn3 statement_scan_24b@65536 and --bench seal p28_attestation_verify@1024; both run and confirmed; NEITHER figure has an 11-measured-claims row (register-absent, compendium itself flags UNVERIFIED-WITHOUT-METHOD) — promotion candidates, coordinator-owned file",
  "row 31 admit drift CONFIRMED by a second independent lane (105.04-108.67 vs XC-04's 112.48-125.12, favorable)",
  "row 31a floor drift confirmed (0.356-0.368 vs 0.38-0.41)",
  "radix gate M-L3-1 @65536: 5.81x, again below the 10x gate (F-06 saw 4.91x)",
  "closure_ladder loud-skipped (F-06's typed kill stands); hot ambient host all session (loadavg1 2.92-21.63, never quiet); commit SHA not recorded (no-git contract)"]}
```
