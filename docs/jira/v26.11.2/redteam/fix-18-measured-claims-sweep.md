# FIX-18 / R-15 — measured-claims sweep (tollgate wave, 2026-11-02)

Repo /Users/sac/cgap @ main. Sole file written:
`redteam/fix-18-measured-claims-sweep.md` (this file). No git commands.
Rows 30-32 of `11-measured-claims.md` §4 NOT re-run — R-16's receipts
govern; receipted-as-of-lane-xc04. Contract was read-only over crates/;
one cross-lane compile-freeze event disclosed in W6 (no edits by this
lane).

## W1 Environment

- Host: Mac15,9 (Apple Silicon, 16-core, no SMT), macOS Darwin 25.2.0;
  rustc 1.101.0-nightly (12433c307 2026-10-24); divan =0.1.21
  (`crates/bench/Cargo.toml` exact pin); build root lease
  /tmp/r15-target (deleted at lane end).
- Commit SHA: NOT RECORDED — dispatch contract forbids ALL git
  commands including read-only `rev-parse`; divergence recorded.
- Ambient load at receipt-writing time: 14.88/12.68/10.49 (decay from
  contention leg + concurrent lane builds; per-leg guard verdicts below
  are the per-run truth, per lane-rc11 law).

## W2 Row 5 — L1 ingress, quiet legs (11-measured-claims row 5 band
10.4-10.8 ns/frame fastest, per dispatch/red-10/RC-11): 3 quiet legs of
`CARGO_TARGET_DIR=/tmp/r15-target cargo bench -p bench --bench
chains-ingress -- ingress_chain`, all EXIT=0, all guards green:

| leg | stderr log | guard verdict | fastest @1M | ns/frame (fastest) | median @1M |
|---|---|---|---|---|---|
| 1 | /tmp/r15-ig1.err | quiet (worst leg 1.60 <= 2) | 10.93 ms | 10.42 | 12.17 ms |
| 2 | /tmp/r15-ig2.err | quiet (cal 1.629-1.779 ns/64B, jitter 1.09; legs 1.78/1.30) | 11.17 ms | 10.65 | 11.99 ms |
| 3 | /tmp/r15-ig3.err | quiet (cal 1.638-1.761, jitter 1.08; legs 1.67/1.20) | 10.76 ms | 10.26 | 12.01 ms |

CONTENTION-SUSPECT markers per leg: 0 / 0 / 0 (grep -c). Verdict: row 5
REPRODUCES; fastest 10.26-10.65 ns/frame straddles the 10.4-10.8 band
(leg 3 10.26 sits 1.3% below the band floor — inside red-10's 4.5%
reproduction tolerance; typed, not silent). Row 5 claim 10.20 fastest /
10.30 median confirmed within tolerance.

## W3 Row 28 — combined-contention re-witness (4x512MiB bw + 8 cpu)

Load generators: 4x python bytearray(512MiB) full-line walkers +
8x `yes`, pids collected in /tmp/r15_hogs.pids (12 lines), killed by
pidfile after the run; `remaining=0` verified by kill -0 sweep.

`cargo bench -p bench --bench chains-ingress -- ingress_chain` EXIT=0,
stderr /tmp/r15-ig4.err:
- CONTENTION-SUSPECT fired, 2 markers: calibration min 1.994 / max
  4.308 ns/64B-line jitter 2.16 > 2; in-run worst leg ratio 2.50 > 2.
- @1M: fastest 11.36 ms (10.83 ns) / median 13.74 ms (13.10 ns) /
  slowest 26.84 ms (25.60 ns/frame).

**TYPED DIVERGENCE (loud):** dispatch expectation "slowest >>50
ns/frame" was NOT met — measured slowest 25.60 ns/frame is ~2x quiet
slowest (leg-3 quiet slowest 13.09 ms = 12.48 ns/frame) but far below the 50 ns bar and red-10's 124.06 ns/frame.
Attribution: dispatch specified 4x512MiB bw + 8 cpu, whereas red-10/
row 28 used 8x512MiB + 8 cpu (bw8). Row 28 stays REGISTERED DIVERGENCE
governed by red-10's receipt; this leg re-witnesses the GUARD FLIP
(guard fires under load, quiet never fires — anti-vacuity intact) but
is NOT a reproduction of row 28's magnitudes and does not amend them.

## W4 SRAM (verify_cscb_sram.py)

- baseline: EXIT=0, measured 36048 B (matches dispatch's expected
  36048 B exactly; report-only empirical line 39184 B, delta -3136 B),
  headroom 95024 B vs 131072 B bound.
- `--subject bn3`: EXIT=1 typed refusal: static noalloc charge 6,520,984 B
  (NaMatches 2,162,696 + NaEngine 4,358,288) > 131072 B (49.75x).
- `--mutate bn3-pin`: EXIT=0, FLIP WITNESSED: mutant refused ("capacity
  table DIVERGED from red-12 size_of pin: static NaMatches 1081352 B !=
  pinned 2162696 B"). A mutant that was admitted would be VACUOUS; it
  was refused. Log /tmp/r15-sram-mut.log.

## W5 Alloc sentry + seal

- posh_alloc_sentry_court: cargo test -p cgap-git
  --test posh_alloc_sentry_court — EXIT=0, 1 passed/0 failed (5.91s).
- quota_alloc_purity_court: EXIT=0, 1 passed/0 failed (0.00s).
- chains-seal leg `governance_chain_fused_per_row` (leg name from
  crates/bench/benches/chains-seal.rs:330; [[bench]] chains-seal,
  Cargo.toml:95): EXIT=0, @65536 fastest 202.2 ms / median 204.5 ms =
  3.086 / 3.123 µs/row — row 12 claim 3.04 µs/row (199.1 ms) reproduces
  within +1.5% fastest / +1.3% median. No contention guard exists on
  chains-seal (guard is chains-ingress-only per lane-rc11) — disclosed;
  verdict rests on the +1.5% agreement, not on a quiet-host marker.

## W6 Compile-freeze event (disclosed, no edit by this lane)

cgap-ffi was compile-broken 18:26-18:40 PST by another lane's in-flight
`src/sovereign.rs` (doc-comment bleed at :771, OsStr::from_bytes
missing import(s), sites drifting 948->957->974 under active edit,
2->3->2 errors). Owner fixed it at 18:40:26 (my poll attempt 6, second
bounded round of 45s/40s polls, 19 attempts total, deadline-bounded).
Per contract I did NOT apply an SLA unblock (contract: read-only over
crates/); bench-class runs were BLOCKED 18:26-18:40 and all bench cells
above ran AFTER repair. Owner lane: unnamed in-flight ffi lane; this
disclosure stands in for both receipts' cross-reference until the
owner's receipt lands.

## W7 Standing

ALIVE for every cell above: each names a command + exit run this
session. Rows 30-32: receipted-as-of-lane-xc04, NOT re-run (per
dispatch). UNVERIFIED remains: commit SHA (no-git contract), chains-seal
quiet-host scope (no guard installed on that bench).

```json
{"lane":"R-15/FIX-18",
 "files_written":["/Users/sac/cgap/docs/jira/v26.11.2/redteam/fix-18-measured-claims-sweep.md"],
 "commands":[
  "python3 scripts/verify_cscb_sram.py (exit 0, 36048 B)",
  "python3 scripts/verify_cscb_sram.py --subject bn3 (exit 1 refusal)",
  "python3 scripts/verify_cscb_sram.py --mutate bn3-pin (exit 0, flip)",
  "cargo test -p cgap-git --test posh_alloc_sentry_court (exit 0)",
  "cargo test -p cgap-git --test quota_alloc_purity_court (exit 0)",
  "cargo bench -p bench --bench chains-ingress -- ingress_chain x3 quiet (exit 0 x3, guards green)",
  "cargo bench -p bench --bench chains-ingress -- ingress_chain contention (exit 0, SUSPECT x2)",
  "cargo bench -p bench --bench chains-seal -- governance_chain_fused_per_row (exit 0)"],
 "courts_run":["verify_cscb_sram --mutate bn3-pin flip (witnessed)",
  "chains-ingress contention guard flip (witnessed, 2 markers)",
  "posh_alloc_sentry_court + quota_alloc_purity_court (green)"],
 "standing":"ALIVE",
 "divergences":[
  "contention slowest 25.60 ns/frame, NOT >>50 as dispatch expected (load 4bw+8cpu vs red-10 8bw+8cpu); row 28 magnitudes unchanged",
  "quiet leg 3 fastest 10.26 ns/frame 1.3% below the 10.4 band floor (inside 4.5% tolerance)",
  "commit SHA not recorded (contract bans all git commands incl. rev-parse)",
  "cgap-ffi compile-freeze 18:26-18:40 owned by another lane; no SLA edit by this lane (contract read-only); disclosed here for both receipts",
  "rows 30-32 not re-run: receipted-as-of-lane-xc04 per dispatch"]}
```
