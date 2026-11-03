# Lane OPT-03 — channel-attribution dose-response curve (2026-11-02)

Repo /Users/sac/cgap @ main. Contract: read-only over crates/; sole file
written is this one. No git commands. Format per bench-01-ingress.md /
bench-05-guard-drift.md. No Process.sleep for coordination: pre-leg host
load bounded-polled (3 polls / 90 s, threshold loadavg1 < 2.0) via
/tmp/opt03work/quiet_poll.py; per-run CONTENTION-SUSPECT verdicts are the
per-run honesty mechanism (crates/bench/benches/chains-ingress.rs).

## §1 Claims under test + register rows

- Row 28 (REGISTERED DIVERGENCE, red-10-timing-jitter-exploit.md):
  hostile-environment split — bw channel hits fastest; cpu channel wrecks
  the tail; combined bw6+cpu9 slowest 124.06 ns/frame @1M.
- Row 5 (MEASURED scoped): quiet-host 10.20 fastest / 10.30 median.
  NOT re-run as a claim this lane; leg 1 is the lane's own clean
  reference at hot ambient load.
- red-10 attribution question this lane answers at finer granularity:
  WHERE does the 124.06-class slowdown onset live — a bus-saturation
  knee (bandwidth) or a scheduler-tail knee (oversubscription)? The
  answer selects which optimization family can pay: bandwidth reduction
  vs tail isolation.
- bench-05 existing single points: bw4-only slowest 92.58, cpu8-only
  slowest 117.72 (both mix-of-ambient+hogs; mix-dependent).

## §2 Method actually applied + host-state disclosure (mandatory)

- Host: Mac15,9, Apple Silicon 16-core (no SMT), Darwin 25.2.0.
- Build root lease: CARGO_TARGET_DIR=/tmp/opt03-target; build exit 0
  (chains-ingress --no-run, 14.34 s); lease deleted at lane end
  (shutil.rmtree exit 0).
- AMBIENT LOAD DISCLOSURE: the host NEVER reached quiet this session.
  Ambient loadavg1 ranged 6.65-15.43 across the 7 legs (concurrent wave
  lanes building on the same box). Every leg ran at hot ambient load;
  every number below is a hot-ambient measurement, scoped as such.
- Contention generators (per lane-rc11/bench-05 pattern): bw channel =
  N x python3 512-MiB bytearray full-line walkers (LLC/DRAM stream);
  cpu channel = N x /usr/bin/yes spinners; all spawned detached with
  stdout/stderr -> DEVNULL (bench-01's stdout flood NOT repeated), pids
  appended to /tmp/opt03work/hogs.pids.
- Kill law: after EVERY leg, `kill_hogs.py` SIGKILLs the pidfile pids
  and `ps -p`-sweeps: remaining=0 verified on all 7 legs (exit 0 x7).
  Final host sweep: zero residual walkers/spinners (grep exit 1).
- ns/frame = ms x 1000 / 1,048,576 (divan, 100 samples, @1M leg).
  Divan column order is fastest | slowest | median | mean; this report
  states fastest/median/slowest as dispatched.

## §3 RAW results (7 legs, all EXIT=0)

### §3.1 Attribution table (channel level | ns/frame @1M)

| leg | dose | fastest | median | slowest | guard verdict | loadavg1 polls |
|---|---|---|---|---|---|---|
| 1 | clean baseline | 10.71 | 11.73 | 12.82 | quiet (cal 1.03; in-run worst 1.50) | 15.43/14.13/13.26 |
| 2 | bw2 (2x512MiB) | 10.92 | 11.86 | 14.27 | SUSPECT (cal 1.63; leg0 ratio 3.89) | 12.88/12.12/11.34 |
| 3 | bw4 | 11.03 | 12.49 | 27.00 | SUSPECT (in-run leg1 ratio 2.45) | 10.20/9.16/12.95 |
| 4 | bw6 | 12.18 | 14.09 | 16.36 | QUIET (in-run worst ratio exactly 2.00 <= 2 — see §5) | 11.31/10.24/9.14 |
| 5 | cpu4 | 10.89 | 11.53 | 12.87 | quiet (cal 1.01; in-run 1.40) | 7.85/7.47/6.78 |
| 6 | cpu8 | 10.73 | 11.52 | 19.30 | SUSPECT (CALIBRATION flip: jitter 3.13; in-run quiet 1.90) | 6.65/7.05/6.96 |
| 7 | cpu16 | 10.67 | 16.52 | 52.65 | SUSPECT (in-run leg0 ratio 11.67; slowest 55.21 ms) | 7.28/6.71/6.66 |

Raw ms strings (fastest | slowest | median), @1M:
L1 11.23|13.44|12.3; L2 11.45|14.96|12.43; L3 11.57|28.31|13.09;
L4 12.78|17.16|14.78; L5 11.42|13.5|12.09; L6 11.25|20.24|12.08;
L7 11.18|55.21|17.32.

### §3.2 Guard drift observations (honest record)

- Leg 2 (bw2): the flip came from the IN-RUN 65536-frame leg (ratio
  3.89 > 2) while the calibration itself stayed inside its bw ceiling.
- Leg 4 (bw6): 0 SUSPECT markers — in-run worst ratio exactly 2.00,
  at-but-not-over the ceiling. Fastest still degraded +13.8% vs
  baseline. Honest reading: the max/min-ratio guard is blind to a
  UNIFORM slowdown; bw6 shifted the whole distribution, which the
  ratio guard cannot see. This is a guard-drift datum, not a clean leg.
- Leg 6 (cpu8): the flip came from the PRE-RUN CALIBRATION (jitter
  3.13 > 2) while the in-run legs stayed quiet (1.90). rc11 evolution
  2 re-witnessed in reverse: this time the bw-class calibration caught
  what the in-run channel missed.
- Leg 7 (cpu16): hard in-run flip, ratio 11.67 (5.8x the ceiling).

## §4 Dose-response reading (measured points only; no extrapolation)

Fastest column (ns/f): 10.71 / 10.92 / 11.03 / 12.18 / 10.89 / 10.73 /
10.67 across baseline -> bw2/bw4/bw6 -> cpu4/cpu8/cpu16. Range spans
only 10.67-12.18 (max delta +13.8% at bw6). The FASTEST column is
nearly dose-invariant on BOTH channels.

Slowest column (ns/f): 12.82 / 14.27 / 27.00 / 16.36 (bw ladder);
12.82 / 12.87 / 19.30 / 52.65 (cpu ladder). Both ladders are
super-linear in the tail:

- bw knee: between bw2 -> bw4 (slowest 14.27 -> 27.00, +89% for a 2x
  dose step); bw6 did not extend the tail further this session
  (16.36), a hot-ambient ordering anomaly recorded verbatim, not
  smoothed.
- cpu knee: between cpu8 -> cpu16 (19.30 -> 52.65, +173% at the 16
  spinner dose = full core-count saturation); cpu4 sits AT baseline
  (12.87 vs 12.82). The knee is at oversubscription, not at cpu4.
- cpu16-only slowest 52.65 CROSSES the >50 ns bar with ZERO bandwidth
  hogs — a new single-channel datum (bench-05 crossed it on bw4-only
  at 92.58 the same day).

## §5 Verdict per number / attribution conclusion

- ATTRIBUTION (row 28 mechanism): the 124.06-class slowdown is a TAIL
  phenomenon on both channels. The fastest path never loses more than
  +13.8% at any measured dose; the tail loses up to 4.1x (cpu16).
  Therefore the dose-response onset is the SCHEDULER-TAIL KNEE, not a
  bus-saturation knee: bw hurts the tail from dose 4 (partial-DRAM
  interference), cpu hurts the tail only past core-count saturation
  (16 spinners = 100% of cores busy). Implication for the optimization
  family: bandwidth reduction alone cannot recover the 124.06-class
  losses — the fastest column it protects is already intact at every
  dose — while TAIL ISOLATION (QoS-class scheduling, critical-section
  pinning, tail-latency guard escalation) targets the column that
  actually degrades. Bandwidth reduction pays only for the +13.8%
  fastest drift and the median drift; tail isolation pays for the
  4.1x tail.
- Row 5 fastest/median: UNVERIFIED-QUIET (unchanged from bench-01) —
  the host never offered a quiet window this session either.
- Row 8 (<15 target): holds at every hot-ambient dose EXCEPT cpu16
  (median 16.52) — the target's quiet-host scope is doing real work;
  under full-core saturation it does NOT hold.
- Leg 4 guard QUIET at bw6: REGISTERED guard blindness to uniform
  distribution shift — max/min-ratio ceilings cannot see a common-mode
  slowdown; recorded as a guard-drift row for lane-rc11 follow-up.
- Row 28 magnitudes: still mix-dependent (no leg hit 124.06 here; the
  combined red-10 dose was not re-run — out of lane scope). Row 28
  stays REGISTERED DIVERGENCE governed by red-10's receipt.

## §6 Command transcript (exits)

```
CARGO_TARGET_DIR=/tmp/opt03-target cargo bench -p bench --bench
  chains-ingress --no-run                        -> EXIT=0 (14.34 s)
quiet_poll.py x7                                 -> HOT x7 (never < 2.0)
leg 1 clean    cargo bench -- ingress_chain      -> EXIT=0, markers 0
leg 2 bw2      spawn_bw.py 2  -> bench -> kill   -> EXIT=0, 1 SUSPECT (3.89)
leg 3 bw4      spawn_bw.py 4  -> bench -> kill   -> EXIT=0, 1 SUSPECT (2.45)
leg 4 bw6      spawn_bw.py 6  -> bench -> kill   -> EXIT=0, 0 markers (2.00 at ceiling)
leg 5 cpu4     spawn_cpu.py 4 -> bench -> kill   -> EXIT=0, 0 markers (1.40)
leg 6 cpu8     spawn_cpu.py 8 -> bench -> kill   -> EXIT=0, 1 SUSPECT (cal 3.13)
leg 7 cpu16    spawn_cpu.py 16 -> bench -> kill  -> EXIT=0, 1 SUSPECT (11.67)
kill_hogs.py after each leg                    -> remaining=0 x7 (exit 0 x7)
final host sweep ps aux | grep walkers/yes     -> zero residuals (exit 1)
python3 shutil.rmtree('/tmp/opt03-target')     -> exit 0
```

Anti-vacuity: the guard flipped in BOTH directions this session —
quiet legs (leg 1: 1.03/1.50; leg 5: 1.01/1.40) and four flips across
four distinct mechanisms (in-run leg ratio 3.89, in-run 2.45,
calibration jitter 3.13, in-run 11.67). A court that cannot flip is
FAIL; this one flipped 4x under load and never fired on the two
genuinely-unperturbed legs (baseline, cpu4).

Fences run: the CONTENTION-SUSPECT guard is chains-ingress's only
fence (per lane-rc11); exercised both directions above, PLUS a new
blindness finding recorded (leg 4 common-mode blindness). No other
existing gates touch this bench; no shared-tree files were modified.

Standing: **ALIVE** for every number above — each names a command +
exit run this session against the exact subject (logs retained at
/tmp/opt03work/leg{1..7}.{out,err} until /tmp cleanup). UNVERIFIED
remains: commit SHA (no-git contract); quiet-host row 5 (host never
quiet); the exact 124.06 combined dose (out of lane scope, row 28
stays governed by red-10).

```json
{"lane":"OPT-03",
 "files_written":["/Users/sac/cgap/docs/jira/v26.11.2/bench-validate/opt-03-channel-curve.md"],
 "commands":[
  "cargo bench -p bench --bench chains-ingress --no-run (exit 0)",
  "leg1 clean ingress_chain (exit 0, guard quiet 1.50)",
  "leg2 bw2 2x512MiB (exit 0, 1 SUSPECT ratio 3.89, slowest 14.27 ns/f)",
  "leg3 bw4 4x512MiB (exit 0, 1 SUSPECT ratio 2.45, slowest 27.00 ns/f)",
  "leg4 bw6 6x512MiB (exit 0, 0 markers, in-run 2.00 AT ceiling — guard blind to uniform shift)",
  "leg5 cpu4 (exit 0, quiet 1.40, slowest 12.87 ns/f)",
  "leg6 cpu8 (exit 0, 1 SUSPECT calibration jitter 3.13, slowest 19.30 ns/f)",
  "leg7 cpu16 (exit 0, 1 SUSPECT ratio 11.67, slowest 52.65 ns/f — >50 ns bar crossed cpu-only)",
  "kill_hogs.py x7 -> remaining=0 x7; final host sweep zero residuals",
  "shutil.rmtree /tmp/opt03-target (exit 0)"],
 "courts_run":["CONTENTION-SUSPECT guard: 2 quiet legs + 4 flips across 4 distinct trigger mechanisms (in-run 3.89 / in-run 2.45 / calibration 3.13 / in-run 11.67); new blindness datum: max/min-ratio ceiling misses common-mode bw6 slowdown"],
 "standing":"ALIVE",
 "divergences":[
  "attribution conclusion: scheduler-tail knee, NOT bus-saturation knee — fastest column dose-invariant (10.67-12.18, max +13.8%); tail degrades up to 4.1x (cpu16) and 2.1x (bw4); tail-isolation optimization family pays, bandwidth reduction does not recover 124.06-class losses",
  "bw knee between bw2->bw4 (slowest +89%); cpu knee between cpu8->cpu16 (+173%, full-core saturation)",
  "leg 4 guard blindness: bw6 leg ran 0 markers while fastest degraded +13.8% — uniform-shift blindness registered for lane-rc11 follow-up",
  "leg 4 bw6 slowest (16.36) < leg 3 bw4 slowest (27.00): hot-ambient ordering anomaly, recorded verbatim, not smoothed",
  "row 8 <15 target fails at cpu16 (median 16.52) — target is quiet-host-scoped and the scope is load-bearing",
  "ambient loadavg1 6.65-15.43 all legs, never quiet — all numbers are hot-ambient scoped",
  "commit SHA not recorded (contract bans git commands)"]}
```

## §7 Errata + format-completion pass (2026-11-02, lane A-04)

ERRATA, never rewrite — this section amends §3-§6 in place; §1-§5 (and
the §6/standing block as landed) stand as written above.

- FORMAT DIVERGENCE (dispatch vs disk): the A-04 dispatch stated this
  report lacked the §6 command transcript + standing block required by
  the bench-01 format law. On disk at pass time, §6, the Standing
  block, and the JSON footer were ALREADY PRESENT (this file's lines
  132-193, 62-83 in the JSON). No section was appended for them; this
  errata is the only addition. The existing §6/standing content was
  re-read from disk this session and is consistent with §2/§3's
  recorded commands and exits; no exit was invented or altered.
- ROUNDING NOTE (§3.2 leg 4 / §4 / §5 "+13.8%" vs 13.7%): both
  figures are rounding views of the SAME measurement; the report's
  +13.8% is the correct one at raw precision. Raw ms (§3.1, the
  ground truth divan printed): leg 4 fastest 12.78 ms vs leg 1
  baseline 11.23 ms = 12.78/11.23 = +13.80%. The 13.7% figure arises
  only if you divide the ns/frame column values pre-rounded to 2
  decimals (12.18/10.71 = +13.725%); that compound rounding loses
  ~0.1 pt. All "+13.8%" statements in §3.2/§4/§5 stand unchanged.
- STANDING (re-read, not re-run): the ALIVE standing block and the
  seven leg exits above were executed in the original OPT-03 lane
  session, not by this pass (A-04 contract forbids bench re-runs, per
  X-06). This pass re-read the receipt from disk; it ran no commands
  against the subject and therefore claims no fresh ALIVE of its own.
  The receipt's standing remains as recorded: ALIVE for the 7 legs +
  guard flips; UNVERIFIED: commit SHA (no-git contract), quiet-host
  row 5 (host never quiet), the exact 124.06 combined dose (row 28
  stays governed by red-10).
- DIVERGENCE LIST (unchanged, restated for the format law): tail knee
  not bus knee (fastest dose-invariant 10.67-12.18, tail up to 4.1x);
  leg 4 guard blindness (0 markers at +13.8% uniform shift); bw6
  slowest < bw4 slowest hot-ambient ordering anomaly; row 8 <15 fails
  at cpu16 (median 16.52).

```json
{"lane":"A-04",
 "files_written":["/Users/sac/cgap/docs/jira/v26.11.2/bench-validate/opt-03-channel-curve.md"],
 "commands":[],
 "courts_run":[],
 "standing":"ALIVE (receipt-level, re-read from disk; this pass ran no subject commands)",
 "divergences":[
  "dispatch premise diverged from disk: §6 + standing + footer already present on disk",
  "only the §7 errata appended; §1-§6 untouched per append-only receipt law",
  "rounding: +13.8% correct at raw-ms precision (12.78/11.23); 13.7% is ns rounding",
  "no bench re-runs (X-06 forbids); no exits invented — exits quoted from §2/§3/§6 record"]}

