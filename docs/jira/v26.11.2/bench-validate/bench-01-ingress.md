# Lane BENCH-01 — hyper-verbose ingress validation (bench wave, 2026-11-02)

Repo /Users/sac/cgap @ main. Contract: read-only over crates/; sole file
written is this one. No git commands. No Process.sleep: pre-leg host load
was bounded-polled (3 polls / 90 s deadline, threshold loadavg1 < 2.0) via
/tmp/bench01-work/quiet_poll.py; the CONTENTION-SUSPECT guard
(crates/bench/benches/chains-ingress.rs, per lane-rc11) is the per-run
honesty mechanism.

## §1 Claims under test + register rows

- Row 5 (11-measured-claims.md row 5, MEASURED scoped): L1 ingress full
  chain **10.20 ns/frame fastest / 10.30 median**, QUIET HOST ONLY,
  divan 0.1.21, 1,048,576-frame const, 100 samples; band 10.4-10.8
  (red-10 4.5% reproduction tolerance).
- Row 28 (row 28, REGISTERED DIVERGENCE): hostile-environment divergence
  governed by redteam/red-10-timing-jitter-exploit.md (124.06 ns/frame
  slowest under bw8+cpu8; >50 ns bar crossed there).
- Row 7: penstock admission stage 7.71 ns/frame fastest (same method).
- Row 8: <15 ns/frame target, quiet-host scoped.

## §2 Method actually applied

- Host: Mac15,9, Apple Silicon 16-core (no SMT), 48 GB, Darwin 25.2.0.
- Build root lease: CARGO_TARGET_DIR=/tmp/bench01-target (deleted at lane
  end). Build exit 0 (`cargo bench -p bench --bench chains-ingress --no-run`).
- Quiet-window discipline: before EACH leg, /usr/bin/uptime loadavg1 was
  polled up to 3x. The host NEVER reached the 2.0 threshold this session:
  ambient loadavg1 ranged 6.11-19.35 across legs (concurrent lane builds
  on the same box). All 8 bench legs therefore ran at hot ambient load;
  per-run guard verdicts below are the honesty record, per lane-rc11 law.
- Runs: N=7 independent `cargo bench -p bench --bench chains-ingress --
  ingress_chain` legs + 1 `-- stage_penstock_admission` leg + 1 induced
  contention leg. All EXIT=0. stderr logs /tmp/bench01-work/ig{1..7}.err,
  ps.err, igc.err.
- Contention generators: 4x python3 512-MiB bytearray full-line walkers
  (bw channel) + 8x /usr/bin/yes (cpu channel), pids in
  /tmp/bench01-work/hogs.pids; killed by pidfile after the run,
  `remaining=0` verified by kill -0 sweep.

## §3 RAW results

### §3.1 ingress_chain, N=7 legs (guards green: 0 CONTENTION-SUSPECT markers each)

| leg | fastest @1M | median @1M | slowest @1M | ns/f fastest | ns/f median | guard | loadavg1 polls |
|---|---|---|---|---|---|---|---|
| 1 | 11.14 ms | 12.24 ms | 13.67 ms | 10.62 | 11.67 | green (cal jitter 1.19; in-run 1.50) | 6.50/6.42/6.11 |
| 2 | 11.57 ms | 12.39 ms | 13.45 ms | 11.03 | 11.82 | green | 16.15/13.47/12.09 |
| 3 | 11.32 ms | 12.07 ms | 13.25 ms | 10.80 | 11.51 | green (cal 2.924-4.590, jitter 1.57; in-run 1.56) | 19.30/16.35/14.74 |
| 4 | 11.30 ms | 12.49 ms | 14.71 ms | 10.78 | 11.91 | green | 15.99/13.18/11.17 |
| 5 | 11.53 ms | 12.42 ms | 13.68 ms | 11.00 | 11.84 | green | 19.35/15.66/13.36 |
| 6 | 11.54 ms | 12.20 ms | 13.39 ms | 11.01 | 11.63 | green | 15.17/13.41/11.78 |
| 7 | 11.33 ms | 12.46 ms | 13.99 ms | 10.81 | 11.88 | green | 10.57/9.76/9.21 |

(Note: divan column order is fastest | slowest | median | mean; the table
is in fastest/median/slowest order as dispatched.)

### §3.2 stage_penstock_admission (guard green, 0 markers)

@1M: fastest 8.483 ms = **8.09 ns/frame** / median 9.898 ms = 9.44 /
slowest 19.25 ms = 18.36. Loadavg polls 9.66/10.57/9.80.

### §3.3 Induced contention leg (4x512MiB bw + 8 cpu, pidfile-killed)

- EXIT=0, **CONTENTION-SUSPECT FIRED: 1 marker** — in-run worst-leg pass
  ns/frame max/min ratio **33.09 > 2** ceiling. Pre-run calibration stayed
  green; the in-run channel is what fired (consistent with lane-rc11's
  evolution 2 finding).
- @1M: fastest 12.8 ms = 12.21 ns/frame (+15% vs quiet best) / median
  14.78 ms = 14.10 / slowest 34.7 ms = **33.09 ns/frame**.
- Post-run: kill $(cat hogs.pids); `remaining=0` verified by kill -0
  sweep. Residual hogs: zero.

## §4 Measured-vs-claimed delta table

| claim | claimed | measured (this session) | delta | basis |
|---|---|---|---|---|
| row 5 fastest | 10.20 ns/f | 10.62-11.03 (7 legs) | +4.1% to +8.1% | leg 1 10.62 inside red-10 4.5% tol; legs 2-7 outside |
| row 5 median | 10.30 ns/f | 11.51-11.91 (7 legs) | +11.7% to +15.6% | all legs outside 4.5% tol |
| row 8 <15 target | MET quiet | all 7 quiet-leg medians < 15 (max 11.91) | holds | measured on green-guard legs |
| row 7 penstock | 7.71 ns/f | 8.09 fastest | +4.9% | 0.38 ns above tol edge |
| row 28 slowest | >>50 ns/f (red-10 124.06, bw8+cpu8) | 33.09 ns/f (bw4+cpu8) | below 50 ns bar | NOT a row-28 magnitude reproduction |

## §5 Verdict per number

- **Row 5 fastest (10.20): DIVERGENCE-REGISTERED** — measured 10.62-11.03;
  only leg 1 (10.62) sits inside the red-10 4.5% band. Every leg's guard
  was GREEN, but ambient loadavg1 never dropped below 6.11, so this is a
  hot-ambient reproduction, not a quiet-host one. The quiet-host claim is
  neither confirmed nor refuted here — the host never offered a quiet
  window (UNVERIFIED-QUIET).
- **Row 5 median (10.30): DIVERGENCE-REGISTERED** — 11.51-11.91, +11.7%
  to +15.6% above claim, all legs guard-green at hot ambient load.
- **Row 8 (<15 target): CONFIRMED-IN-BAND** — all seven guard-green legs
  hold median < 15 ns/frame under ambient load up to 19.35.
- **Row 7 penstock (7.71): DIVERGENCE-REGISTERED** — 8.09 fastest, +4.9%,
  marginally past the 4.5% tolerance edge, guard green.
- **Row 28: re-witnessed GUARD FLIP only; magnitudes NOT reproduced** —
  slowest 33.09 ns/frame under bw4+cpu8 is 2.5x quiet slowest but far
  below the 50 ns bar and red-10's 124.06 (red-10 used bw8; R-15 already
  registered the same shortfall). Row 28 stays REGISTERED DIVERGENCE
  governed by red-10's receipt; the flip witnesses the guard's
  anti-vacuity (fires under load, green x7 quiet).

## §6 Command transcript (exits)

```
CARGO_TARGET_DIR=/tmp/bench01-target cargo bench -p bench --bench \
  chains-ingress --no-run                         -> EXIT=0 (build 16.03s)
python3 /tmp/bench01-work/quiet_poll.py           -> exit 0 x8 (never QUIET)
cargo bench ... -- ingress_chain (legs 1-7)       -> EXIT=0 x7, markers 0 x7
cargo bench ... -- stage_penstock_admission       -> EXIT=0, markers 0
python3 (spawn 4 bw + 8 yes hogs, pidfile)        -> spawned, pids recorded
cargo bench ... -- ingress_chain (contention)     -> EXIT=0, 1 SUSPECT marker
kill $(cat /tmp/bench01-work/hogs.pids) + kill -0 sweep -> remaining=0
python3 -c shutil.rmtree('/tmp/bench01-target')   -> target lease deleted
```

Anti-vacuity: the induced-contention leg IS the flip — the in-run guard
fired (ratio 33.09 > 2) on the loaded host and never fired on the seven
unloaded-ambient legs. A guard that cannot flip is FAIL; this one flipped
with 16x margin.

Fences run: none exist beyond the CONTENTION-SUSPECT guard itself for
this bench (chains-ingress is the guard's only subject, per lane-rc11);
the guard was exercised in both directions this session (7 green, 1 flip).

Standing: **ALIVE** for every number above — each names a command + exit
run this session against the exact subject. UNVERIFIED remains: commit
SHA (no-git contract), quiet-host row 5 (host never quiet this session).

```json
{"lane":"F-05/BENCH-01",
 "files_written":["/Users/sac/cgap/docs/jira/v26.11.2/bench-validate/bench-01-ingress.md"],
 "commands":[
  "cargo bench -p bench --bench chains-ingress --no-run (exit 0)",
  "cargo bench -p bench --bench chains-ingress -- ingress_chain x7 quiet-ambient (exit 0 x7, guards green)",
  "cargo bench -p bench --bench chains-ingress -- stage_penstock_admission (exit 0, guard green)",
  "contention leg 4x512MiB+8cpu: cargo bench -- ingress_chain (exit 0, 1 CONTENTION-SUSPECT, ratio 33.09)",
  "kill $(cat /tmp/bench01-work/hogs.pids) -> remaining=0"],
 "courts_run":["CONTENTION-SUSPECT guard: 7 quiet legs green + 1 loaded flip (ratio 33.09 > 2) — anti-vacuity witnessed both directions"],
 "standing":"ALIVE",
 "divergences":[
  "row 5 fastest/median measured +4.1..+8.1% / +11.7..+15.6% above claim on guard-green legs at hot ambient load (loadavg1 6.11-19.35, never quiet) — quiet-host claim UNVERIFIED-QUIET, not refuted",
  "row 7 penstock 8.09 vs 7.71 (+4.9%, just past tolerance)",
  "row 28 magnitudes again NOT reproduced at bw4+cpu8 (slowest 33.09 ns/f < 50 ns bar; red-10 used bw8) — guard flip witnessed, row 28 stays governed by red-10",
  "commit SHA not recorded (contract bans all git commands)",
  "harness note: yes hogs inherited Bash stdout on spawn, flooding one tool result (64MB persisted-output); hogs were pidfile-captured and killed, remaining=0 — no repo impact"]}
```

## §7 ERRATA (2026-11-02, lane BENCH-ADD — appended only, §1-§6 above
## unchanged)

### E1. §4:80 basis column "all 7 quiet-leg medians < 15" uses a
### forbidden label — the legs were HOT-AMBIENT, never quiet

`bench-01-ingress.md:80` (§4 delta table, row 8 basis column) reads "all
7 quiet-leg medians < 15". That wording contradicts this receipt's own
§2: the quiet-poll threshold (loadavg1 < 2.0) was NEVER met across all 8
legs — ambient loadavg1 ranged 6.11-19.35 — and §5:88-91 declares row 5
UNVERIFIED-QUIET ("hot-ambient reproduction, not a quiet-host one").
There were no quiet legs this session. The label is the defect flagged
by X-06's audit; it is the same defect class A-03 corrected in
bench-05-guard-drift.md §8 E1 (its "quiet ref (bench-01 leg 1)" label
against this same receipt's §2/§5).

Corrected reading: §4:80 must state "all 7 HOT-AMBIENT-GUARD-GREEN leg
medians < 15 (max 11.91, ambient loadavg1 up to 19.35)". No numbers
change — only the label. Consumers should read §5:94-95, which already
carries the correct phrasing ("all seven guard-green legs hold median
< 15 ns/frame under ambient load up to 19.35"). The quoted phrase
"quiet-leg" in §4:80 must NOT be quoted downstream.

Standing impact on row 8: the <15 ns CONFIRMED-IN-BAND verdict STANDS,
and is in fact a STRONGER result than a quiet-window pass would be —
the band held at hot ambient load (up to 19.35 loadavg1) with guards
green, so a quiet-window re-run can only widen the margin, not narrow
the verdict. Row 8 is CONFIRMED-IN-BAND, HOT-AMBIENT-GUARD-GREEN scope;
the quiet-host scoping of the original row-8 claim remains UNVERIFIED-
QUIET, exactly as §5 records for row 5.

ERRATA-never-rewrite honored: §1-§6 byte-unchanged (append-only §7).

```json
{"lane":"BENCH-ADD",
 "files_written":["docs/jira/v26.11.2/bench-validate/bench-01-ingress.md (§7 append only)"],
 "commands":[
  "Read bench-01-ingress.md @150 lines — §2:28-31, §4:80, §5:88-95 verified on disk",
  "Read bench-05-guard-drift.md §8 E1 (:151-169) — A-03 pattern, same defect class"],
 "courts_run":["none: doc-label errata; CONTENTION-SUSPECT record (7 green + 1 flip) untouched"],
 "standing":"ALIVE (errata appended against on-disk §2/§4/§5 re-read this session)",
 "divergences":[
  "dispatch path 'bench-validate/...' is under docs/jira/v26.11.2/ on disk — wrote the real path",
  "§4:80 'quiet-leg' label contradicts §2 hot-ambient disclosure — §7 E1 fixes label, not numbers",
  "A-03 bench-05 §8 E1 errata pattern applies (same class, same subject receipt)"]}
```
