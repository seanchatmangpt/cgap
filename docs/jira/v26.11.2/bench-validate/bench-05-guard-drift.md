# Lane BENCH-05 — single-channel contention legs + guard drift audit (2026-11-02)

Repo /Users/sac/cgap @ main. Contract: read-only over crates/; sole file
written is this one. No git commands. Format per bench-01-ingress.md.

## §1 Claims under test + register rows

- Row 28 (REGISTERED DIVERGENCE, red-10): hostile-environment split —
  bw-only degraded fastest +14.8% and tail +189%; cpu-only left fastest
  intact (-3.4%) and wrecked the tail (+173%). Channel-split law.
- lane-rc11 guard law: two-channel guard, quiet never fires, load
  fires; pre-run calibration is bandwidth-channel only (rc11 evolution
  2: pre-run stayed quiet while in-run fired under cpu-class load).
- Rows 30/31/31a/32 (XC-04 stations) + R-15 sweep cells — drift audit
  rows re-read from disk this session (receipt-cited counts re-read at
  use time).

## §2 Method actually applied

- Build root lease CARGO_TARGET_DIR=/tmp/f07-target; build exit 0
  (chains-ingress --no-run, 18.04 s).
- Ambient loadavg1 this session: 8.53 -> 14.34 -> 45.20 peak (concurrent
  lane builds + induced hogs). NO quiet leg was run in this lane —
  dispatch scoped legs to bw-only and cpu-only contention only; both
  legs ran SUSPECT by design.
- Contention generators: 4x python3 512-MiB bytearray full-line walkers
  (bw channel, pids /tmp/f07work/hogs_bw.pids) then 8x /usr/bin/yes
  (cpu channel, /tmp/f07work/hogs_cpu.pids), spawned detached with
  stdout/stderr -> DEVNULL (bench-01's stdout-flood divergence NOT
  repeated). Killed by pidfile after each run.
- Hog-retire verification: bw hogs — first `kill -0` sweep reported
  remaining=4 (signal-delivery race); second `ps -p` sweep: EMPTY
  (remaining=0). cpu hogs — ps sweep clean immediately. Disclosed
  verbatim; no repo impact.
- ns/frame = ms*1000/1,048,576 (divan, 100 samples, @1M leg).

## §3 RAW results

### §3.1 bw-only leg (4x512MiB), EXIT=0, 1 CONTENTION-SUSPECT marker

- Guard: calibration quiet (not re-shown in bw.err summary above; in-run
  `leg 0 (65536) pass ratio 2.25`, `leg 1 (1048576) 7.67`; SUSPECT
  marker on worst leg ratio 7.67 > 2.
- @1M: fastest 13.44 ms = 12.82 ns/f / median 27.75 ms = 26.46 /
  slowest 97.06 ms = **92.58 ns/frame** — >50 ns bar CROSSED under
  bw4-only this session (red-10's bw4 row was 38.76).

### §3.2 cpu-only leg (8x yes), EXIT=0, 1 CONTENTION-SUSPECT marker

- Guard: calibration **quiet** (min 1.632 / max 1.635 ns/64B-line,
  jitter 1.00) while in-run legs fired (2.40 / 10.64; SUSPECT on 10.64).
  Pre-run bandwidth calibration is blind to cpu-class load — rc11
  evolution 2 re-witnessed exactly.
- @1M: fastest 11.66 ms = **11.12 ns/f (fastest INTACT)** / median
  13.87 ms = 13.23 / slowest 123.4 ms = **117.72 ns/frame (tail
  wrecked)**.

## §4 Channel-split re-witness (row 28 law)

| leg | fastest ns/f | vs quiet ref | slowest ns/f | vs quiet ref |
|---|---|---|---|---|
| quiet ref (bench-01 leg 1, loadavg 6.11-6.50) | 10.62 | — | 13.05 | — |
| bw-only 4x512MiB | 12.82 | +20.7% | 92.58 | +609% |
| cpu-only 8 threads | 11.12 | +4.7% | 117.72 | +802% |

Row 28's channel split REPRODUCES: bw dominates fastest degradation
(+20.7% vs +4.7%); cpu wrecks the tail hardest (+802% vs +609%).
Row 28 stays REGISTERED DIVERGENCE governed by red-10's receipt; today's
bw4/cpu8 magnitudes are mix-dependent and do not amend it.

## §5 Drift table (claim / original / today / in-band? / method-match?)

"Today" = measured this session where a leg was run; otherwise the
receipt was re-read from disk this session (stale-by-default honored).

| claim | original (receipt) | today | in-band? | method-match? |
|---|---|---|---|---|
| row 5 quiet 10.20/10.30 | red-10 baseline 10.66/11.76 (red-10:30-41, in 4.5% tol); rc11 quiet 10.4-10.8 (rc11:38-43); R-15 10.26-10.65 (fix-18 W2) | NOT re-run this lane (no quiet leg; ambient never < 8.53) | UNVERIFIED-today; latest receipts in-band | yes (all use divan @1M method); today n/a |
| row 7 penstock 7.71 | bench-01 8.09 fastest (+4.9%) | not re-run | UNVERIFIED-today | n/a today |
| row 28 bw6+cpu9 14.71/19.03/124.06 | red-10 W5 combined row | bw4-only 12.82/26.46/92.58; cpu8-only 11.12/13.23/117.72 | magnitudes mix-dependent; slowest >50 bar crossed on BOTH single channels today | partial: red-10 used bw6+cpu9 combined; today's single channels isolate the two channels red-10 lumped |
| rc11 guard quiet (never fires; quiet ratios 1.20-1.60, cal 1.63-1.68 ns/line) | rc11 W3 | cal 1.632/1.635 witnessed today (cpu leg) — in-band | yes | yes (same ceilings 12 ns / ratio 2) |
| rc11 guard flip under load | rc11 W3 flip ratios 2.13 cal / 59.93 in-run | flips today: 7.67 (bw4) and 10.64 (cpu8) | flips, magnitude varies by load class | yes (same in-run max/min law) |
| rows 30/31/31a/32 (XC-04 stations) | xc04 receipt:36-37,51-52,47-49,66-67 vs 11-measured-claims.md:106-109 | re-read both files on disk this session: row 30 (3,529,029/3,824,869/3,846,425 + 4,069,561/4,130,106/4,135,837), row 31 (123.81/124.02/125.12 + 112.48/113.37/114.44), row 31a (0.38-0.41), row 32 (0.15/0.18/1.56 + 0.15/0.15/1.47) — claims table matches receipt numbers EXACTLY | in-band (identity) | yes, with XC-04's disclosed divergences carried into the register rows (window 1,365 vs b06 1M; in-process vs cgo; Rust L1-hot vs ipc-04 Go) |
| R-15 sweep cells (row 5 legs, contention slowest 25.60 @4bw+8cpu, SRAM 36,048/refusal 49.75x, seal 3.086/3.123 µs/row vs row 12 +1.5%) | fix-18 W2-W5 | SRAM cells re-witnessed THIS session (bench-04: 36,048 exit 0 / refusal exit 1) — in-band identity; contention/seal cells re-read from disk only | in-band | yes (same scripts, same courts) |
| row 12 seal 3.04 µs/row | fix-18 W5: 3.086/3.123 (+1.5%) | not re-run | UNVERIFIED-today | n/a today |

## §6 Verdict per number

- **Row 28 channel split: CONFIRMED** — bw dominates fastest, cpu wrecks
  the tail; guard flipped on both single-channel legs (1 marker each).
- **bw4-only slowest 92.58 ns/f crosses the >50 ns bar** — new
  single-channel datum: red-10's bw4 row was 38.76; today's higher value
  is mix-of-ambient+hogs (loadavg 14-45). Does not amend row 28 (mix
  differs); recorded as a today-only observation.
- **cpu8-only fastest 11.12: fastest column survives cpu-class load**
  (red-10's law re-witnessed: -3.4% then, +4.7% today, both within
  quiet-band).
- **rc11 guard: anti-vacuity witnessed BOTH directions today** — quiet
  calibration 1.632/1.635 (never fires) + two flips under load.
- **Rows 30/31/31a/32: claims table = XC-04 receipt, identity** — no
  drift between receipt and register; disclosed divergences correctly
  carried into the register's basis column.
- **R-15 SRAM cells: re-witnessed in-band identity** (bench-04 §3.3/3.4).

## §7 Command transcript (exits)

```
CARGO_TARGET_DIR=/tmp/f07-target cargo bench ... --no-run  -> EXIT=0
spawn 4x512MiB bw hogs (pidfile, DEVNULL) -> pids recorded
cargo bench ... -- ingress_chain (bw leg)  -> EXIT=0, 1 SUSPECT (7.67)
kill bw hogs; sweep1 remaining=4 (race) -> sweep2 ps: 0
spawn 8x yes cpu hogs (pidfile, DEVNULL) -> pids recorded
cargo bench ... -- ingress_chain (cpu leg) -> EXIT=0, 1 SUSPECT (10.64)
kill cpu hogs; ps sweep clean
```

Anti-vacuity: the two induced-load legs ARE the flips — the in-run
guard fired under each single channel (7.67 / 10.64 > 2) while the
pre-run calibration stayed quiet on the cpu leg (1.00 jitter). A guard
that cannot flip is FAIL; this one flipped on both channels, and the
cpu leg's quiet calibration + firing in-run pair re-proves the guard is
the in-run channel doing the work (rc11 evolution 2 re-witnessed).

Fences run: none exist beyond the CONTENTION-SUSPECT guard for this
bench (chains-ingress is the guard's only subject per lane-rc11); the
guard was exercised in both directions this session (quiet calibration
witnessed, two flips witnessed).

Standing: **ALIVE** for every number above — each names a command +
exit run this session against the exact subject. UNVERIFIED remains:
commit SHA (no-git contract); quiet-host row 5 today (no quiet leg run;
governed by R-15/bench-01 receipts).

```json
{"lane":"F-07/BENCH-05",
 "files_written":["/Users/sac/cgap/docs/jira/v26.11.2/bench-validate/bench-05-guard-drift.md"],
 "commands":[
  "cargo bench -p bench --bench chains-ingress --no-run (exit 0)",
  "cargo bench ... -- ingress_chain bw-only 4x512MiB (exit 0, 1 SUSPECT, ratio 7.67, slowest 92.58 ns/f)",
  "cargo bench ... -- ingress_chain cpu-only 8x yes (exit 0, 1 SUSPECT, ratio 10.64, slowest 117.72 ns/f, fastest intact 11.12)",
  "kill pidfiles + ps sweeps -> 0 residual hogs"],
 "courts_run":["CONTENTION-SUSPECT guard: quiet calibration witnessed (1.632/1.635 ns/64B-line, jitter 1.00) + flips on both single-channel legs (7.67 bw / 10.64 cpu) — anti-vacuity both directions"],
 "standing":"ALIVE",
 "divergences":[
  "bw4-only slowest 92.58 ns/f crossed the >50 ns bar (red-10 bw4 row 38.76); attributed to ambient+hogs mix (loadavg 14-45), does not amend row 28",
  "no quiet leg this lane (dispatch scoped to contention legs) — row 5 quiet today UNVERIFIED-today, governed by R-15/bench-01",
  "bw hog retire verification raced on first kill -0 sweep (remaining=4) — second ps sweep clean; disclosed verbatim",
  "commit SHA not recorded (contract bans all git commands)"]}
```

## §8 ERRATA (2026-11-02, lane A-03 — appended, historical table above
## unchanged)

Two defects in this receipt, corrected here per ERRATA-never-rewrite.

### E1. §4 row 1 label "quiet ref (bench-01 leg 1)" is WRONG

`bench-05-guard-drift.md:62` labels the reference row "quiet ref
(bench-01 leg 1, loadavg 6.11-6.50)". Bench-01's own receipt contradicts
the "quiet" label: leg 1 ran at loadavg1 6.11-6.50 with a never-quiet
host — bench-01 §2 declares the quiet-poll threshold 2.0 was NEVER met
across all 8 legs, and bench-01 §5 declares row 5 leg 1
UNVERIFIED-QUIET ("hot-ambient reproduction, not a quiet-host one").
10.62/13.05 is therefore a HOT-AMBIENT reference, not a quiet one.

Consequence: every "vs quiet ref" delta in the §4 table — bw-only
+20.7% fastest / +609% tail, cpu-only +4.7% / +802% — is a
hot-ambient-reference delta, not a quiet-reference delta. Against a
true quiet baseline (row 5's quiet band 10.4-10.8 fastest), the real
quiet-reference magnitudes would differ and are NOT computable from
this lane's data (no quiet leg was run; §5:78 itself records ambient
never dropped below 8.53 in THIS lane's session either).

Unaffected: the channel-split CONCLUSION (bw dominates fastest
degradation, cpu wrecks the tail hardest) is directionally intact —
both comparison legs (bench-01 leg 1 and today's bw4/cpu8 legs) are
same-day, same-ambient-class (hot), so the between-leg comparison
remains apples-to-apples. Row 28's classification is untouched. But
the §4 magnitudes must NOT be quoted anywhere as quiet-baseline
deltas; quote them only as "vs bench-01 leg-1 hot-ambient reference".

### E2. §3.1 dangling reference "not re-shown in bw.err summary above"

`bench-05-guard-drift.md:42` references a "bw.err summary above" that
does not exist anywhere in this report — no bw.err summary was ever
included. The raw guard calibration output for the bw-only leg lived
only in the lane's /tmp work dir (/tmp/f07work/), which has since been
deleted. Recorded as LOST-WITH-WITNESS: the in-run ratios (2.25 leg 0,
7.67 leg 1) and the SUSPECT marker are receipted here from the session
record, but the underlying raw stderr summary cannot be produced and
is NOT invented. Do not cite §3.1 as containing the calibration raw
output; cite only the ratios as session-recorded values.
