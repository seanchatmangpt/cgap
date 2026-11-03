# ADV-15 — claims boundary injection (corrosion wave, 2026-11-02)

Repo /Users/sac/cgap @ main. Contract: read-only over crates/; sole file
written is this one. No git commands. Method format follows
bench-01-ingress.md / bench-05-guard-drift.md; guard law per
lane-rc11-contention-guard.md.

## W1 Subject + claims under test

Whether any current latency claim SURVIVES an actively hostile host.
Rows: 11-measured-claims.md row 5 (10.20/10.30 ns/frame, QUIET-HOST
scoped), row 28 (REGISTERED DIVERGENCE, red-10:43-66), row 7 penstock
7.71. Audit targets: dflss/00-dflss-master-dossier.md technical-
readiness rows + dflss/02-voc-ctq-matrix.md M-CTQ-01 (post-QUAL-01).
Hogs: /tmp/f08work/f08_hog.c (clang -O2, repo untouched):
- evict: 4 threads x 512 MiB, stride 32768 B full-line read+write —
  cache-set pollution targeting L1/L2/LLC with LOW DRAM streaming.
- bw: 6 threads x 512 MiB, 64 B stride read+write — memory-bus
  saturation (heavier than F-05 bw4, near red-10 bw8).
- both: 10 threads (bw6+evict4). Pidfile-captured, killed per leg,
  remaining=0 verified via `ps -p $(cat hogs.pids)` after EVERY leg;
  final comm-based sweep `pgrep -x f08_hog` = 0 processes. Zero
  residual hogs at lane end.

## W2 Battery (4 legs, ingress_chain, divan @1M, EXIT=0 each)

Per-frame = ms*1000/1,048,576. Ambient loadavg1 hot all session
(6.14-11.31; never quiet — same regime as bench-01/F-07).

| leg | guard verdict | fastest | median | slowest | ns/f fast/med/slow | loadavg1 |
|---|---|---|---|---|---|---|
| evict-1 | GREEN (0 markers; cal 1.658/1.674 jitter 1.01; in-run 1.50) | 11.81 ms | 12.97 ms | 15.40 ms | 11.26 / 12.37 / 14.69 | 8.72-11.31 |
| evict-2 | GREEN (0 markers; cal 1.761/3.055 jitter 1.74; in-run 1.73) | 12.64 ms | 15.25 ms | 20.15 ms | 12.06 / 14.54 / 19.21 | 6.14 |
| bw6 | SUSPECT x2 (cal 2.491/5.920 ratio 2.38; in-run 3.71) | 16.18 ms | 18.44 ms | 21.62 ms | 15.43 / 17.59 / 20.62 | 6.78 |
| both | SUSPECT x2 (cal 2.950/7.947 ratio 2.69; in-run 2.53) | 18.07 ms | 20.13 ms | 26.91 ms | 17.23 / 19.19 / 25.66 | 7.05 |


## W3 The evict-channel guard blind spot (PRIMARY FINDING)

Both evict legs ran GUARD-GREEN (0 CONTENTION-SUSPECT markers) while
DEGRADING the numbers past the row-5 tolerance:

- evict-1 fastest 11.26 ns/frame = +10.4% vs row 5's 10.20 (+6.1% vs
  bench-01 quiet ref 10.62) — outside the red-10 4.5% band, no marker.
- evict-2 fastest 12.06 ns/frame = +18.2% vs 10.20 — outside the band,
  no marker. Slowest 19.21 = +47% vs quiet-tail ref 13.05, no marker.

Mechanism: the evict profile degrades UNIFORMLY — per-pass max/min
ratio stays under the 2.0 in-run ceiling (1.50 / 1.73) and the
bandwidth calibration channel sees only cache-pollution, not DRAM
streaming (cal jitter rose 1.01 -> 1.74 approaching but never crossing
ratio 2.0). A guard built on variability (ratio) plus bandwidth (12 ns
ns/64B-line ceiling) is structurally blind to a smooth, uniform,
low-bandwidth degradation profile. A hostile host running the evict
profile would receive guard-green certification for numbers outside
the quiet-host band. This does NOT falsify any doc claim (all are
scoped, W4) — it narrows what guard-green certifies: NOT "quiet host",
only "not contended in the two guarded channels".

New channel law extending row 28's bw/cpu split to three channels:
- evict-class load: moves the FASTEST column past the quiet band
  (+10-18%) while every guard channel stays green.
- bw-class load: fires the guard's BOTH channels; moves fastest +51%
  and median +71%.
- combined: monotone worst (fastest 17.23, +69%); tail 25.66 stayed
  below the 50 ns bar.

## W4 Claims audit (dossier + CTQ matrix vs measured hostile numbers)

- 00-dflss-master-dossier.md:24 — row 5 quote carries "quiet-host-
  scoped" verbatim + explicit row-28 pointer ("row 28 is the registered
  divergence ... 14.71/19.03/124.06 ... red-10:43-66,77-88") and
  forbids hostile/constant-time readings via row 29. Under my hostile
  numbers (fastest 11.26-17.23) the scoped claim is not asserted
  hostile-host, so nothing is falsified.
- 00-dflss-master-dossier.md:46 — M-CTQ-01 "VERIFIED (SCOPED
  QUIET-HOST)" — scope carried.
- 02-voc-ctq-matrix.md:202-209 (M-CTQ-01 cell) — quotes 10.20/10.30
  with "the quiet-host scope is part of the claim", registers row 28
  with full numbers, and pins "constant-time" UNSUPPORTED (row 29).
- Grep sweep over both files: these two latency sites are the ONLY
  ingress-latency cells; no bare unscoped ns/frame number exists in
  either file. No claim my hostile-host numbers falsify.

AUDIT VERDICT: **IMMUNE** — the scoping law holds under injection.
Every surviving latency claim carries its quiet-host scope or the
row-28 pointer. ZERO SPOILER-CONFIRMED cells. But the enforcement
mechanism that BACKS the scope (the rc11 guard) has the W3 blind
spot — the doc law is immune, the runtime detector is only 2-of-3
channels. Until an evict-class detector lands, guard-green must be
read as "not bw/cpu-contended", not "quiet".

## W5 Row 28 re-witness (corroboration-or-divergence)

- Flip: CORROBORATED — combined leg fired BOTH guard channels (cal
  ratio 2.69, in-run 2.53 > 2). Anti-vacuity witnessed both directions
  this session: 2 green legs (evict) + 2 flips (bw6/both).
- Magnitude: DIVERGES — combined slowest 25.66 ns/frame < the 50 ns
  bar, far under red-10's 124.06 (bw6+cpu9) — third consecutive
  non-reproduction of row-28 magnitude (bench-01 33.09 @bw4+cpu8,
  F-07 92.58/117.72 single-channel at loadavg 14-45, mine 25.66 @
  loadavg 6-7). Row 28 stays REGISTERED DIVERGENCE governed by
  red-10's receipt; today's numbers are mix-dependent and do not amend
  it. Today's mixed ambient (6-7 vs F-07's 14-45) is the noted
  covariate.

## W6 Command transcript (exits)

```
clang -O2 -o /tmp/f08work/f08_hog /tmp/f08work/f08_hog.c -lpthread -> 0
CARGO_TARGET_DIR=/tmp/f08-target cargo bench -p bench --bench \
  chains-ingress --no-run                              -> EXIT=0 (15.23s)
spawn f08_hog evict (pidfile); cargo bench -- ingress_chain -> EXIT=0, 0 markers
kill pidfile; ps -p sweep                          -> remaining=0
spawn f08_hog bw; cargo bench -- ingress_chain     -> EXIT=0, 2 SUSPECT (2.38/3.71)
kill pidfile; sweep                                -> remaining=0
spawn f08_hog both; cargo bench -- ingress_chain   -> EXIT=0, 2 SUSPECT (2.69/2.53)
kill pidfile; sweep                                -> remaining=0
spawn f08_hog evict (leg 2); cargo bench           -> EXIT=0, 0 markers (blind spot confirm)
kill pidfile; sweep                                -> remaining=0
pgrep -x f08_hog                                   -> 0 processes
python3 -c shutil.rmtree('/tmp/f08-target', True)  -> lease deleted
```

Disclosed method incidents (no repo impact): (1) one aborted leg attempt
ran cargo from /tmp cwd — "could not find Cargo.toml", EXIT=101, zero
repo writes, re-run correctly; (2) an interim grep counted the shell
wrapper's own command line as a hog match — resolved by comm-based
`pgrep -x` (0); the pidfile kills were correct on every leg.

## W7 Anti-vacuity + fences

- Flip witnessed: bw6 and both legs each fired BOTH guard channels
  (jitter 2.38/2.69 > 2; in-run 3.71/2.53 > 2). The evict legs are the
  NEGATIVE control that is not vacuous: they show the guard discrimina-
  tes (fires on bw/combined, stays green on evict) — and that
  discrimination is itself the finding (W3): it is a false negative
  class, not a broken guard.
- Fences my diff could break: none — sole write is this receipt;
  crates/ untouched, bench file untouched, shared lib.rs/Cargo.toml
  untouched. No gates exist over this receipt path.

Standing: **ALIVE** — every number above names a command + exit run
this session against the exact subject (stderr evidence /tmp/f08work/
{evict,evict2,bw6,both}.{out,err}, load.txt). UNVERIFIED remains:
commit SHA (no-git contract); true quiet-host row 5 today (host never
below loadavg1 6.14).

```json
{"lane":"F-08/ADV-15",
 "files_written":["/Users/sac/cgap/docs/jira/v26.11.2/redteam/corrosion/adv-15-claims-boundary-injection.md"],
 "commands":[
  "clang -O2 f08_hog.c (exit 0)",
  "cargo bench chains-ingress --no-run (exit 0)",
  "ingress_chain evict leg 1 (exit 0, guard GREEN, fastest 11.26 ns/f)",
  "ingress_chain bw6 leg (exit 0, 2 SUSPECT, fastest 15.43 / slowest 20.62)",
  "ingress_chain both leg (exit 0, 2 SUSPECT, fastest 17.23 / slowest 25.66)",
  "ingress_chain evict leg 2 (exit 0, guard GREEN, fastest 12.06)",
  "pidfile kills + ps -p sweeps x4 -> remaining=0 each; pgrep -x f08_hog -> 0",
  "shutil.rmtree('/tmp/f08-target') -> lease deleted"],
 "courts_run":[
  "CONTENTION-SUSPECT guard: 2 green negative controls (evict x2) + 2 flips (bw6/both, both channels) — anti-vacuity both directions",
  "claims audit: dossier:24/:46 + ctq-matrix M-CTQ-01 all carry quiet-host scope or row-28 pointer — IMMUNE, 0 SPOILER-CONFIRMED"],
 "standing":"ALIVE",
 "divergences":[
  "GUARD BLIND SPOT: evict-channel (large-stride L1/L2/LLC pollution, low DRAM) degrades fastest +10.4%/+18.2% past the 4.5% band with ZERO guard markers (x2 legs) — guard certifies 'not bw/cpu-contended', not 'quiet'",
  "row-28 magnitude NOT reproduced third session running (combined slowest 25.66 < 50 ns bar vs red-10 124.06); flip corroborated, magnitudes mix-dependent; row 28 stays governed by red-10",
  "audit verdict IMMUNE: zero unscoped latency cells found in dossier or CTQ matrix",
  "commit SHA not recorded (contract bans all git commands)",
  "host never quiet (loadavg1 6.14-11.31); all legs hot-ambient like bench-01/F-07"]}
```
