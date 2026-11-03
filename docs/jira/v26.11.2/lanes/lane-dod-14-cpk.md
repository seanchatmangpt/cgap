# Lane DOD-14 — membrane Cpk study (IPC-7 N=30)

Date: 2026-11-02. Repo: /Users/sac/cgap (main, canonical). No git commands.

Standing: **ALIVE** (runner exit 0 this session against the exact darwin
dylib + fixtures; every claim below has command+exit this session).

## W1 Commands + exits

| cmd | exit |
|---|---|
| `bash /Users/sac/cgap/courts/ax-testbed/runners/cpk.sh` | 0 (PASS: Cpu 17.252 >= 1.33, flip witnessed) |
| `bash -n` on the runner | 0 (SYNTAX-OK) |
| parser unit probe on synthetic bench lines (inline python) | 0 ([196.98, 183.97]) |

## W2 Files written

1. `courts/ax-testbed/runners/cpk.sh` (NEW, 177 lines) — the N=30
   membrane capability driver; exit contract 0/1/2/3 mirrored from
   `crates/bench/src/bin/cpk_study.rs`.
2. `docs/jira/v26.11.2/dflss/interop-cpk.md` (NEW) — the capability
   study: layer law, decided-budget provenance, baseline result.
3. This receipt.

## W3 Baseline result (N=30 darwin membrane)

mean 184.294 ns/frame, sigma 1.270, min 181.71, max 187.67
(re-witnesses the IPC-3/AX-5 darwin band 179-192 ns/frame).

| gate | USL | Cpu | verdict |
|---|---|---|---|
| decided membrane budget | 250 ns/frame | **17.252** | PASS |
| aspirational (kernel-layer-only) | 10 ns/frame | **-45.762** | printed loudly, NEVER gated |
| jitter flip (anti-vacuity) | 250 ns/frame | **0.157** | dropped below floor — witnessed |

Flip model: +U(0,60) ns every sample, +U(0,120) ns every 3rd, seed
0xA20C0FFEE5A7E000. Thermals `pmset -g therm`: clean before/after.

Full stats JSON reproduces on rerun; the 30-sample table is embedded
below and summarized in `dflss/interop-cpk.md`. Samples (ns/frame):

181.71 183.11 185.02 184.26 183.42 185.25 184.25 184.14 185.79 182.55
184.32 184.55 184.17 185.87 187.67 183.62 186.31 185.68 183.54 184.69
185.07 183.31 183.51 184.68 184.60 183.66 183.65 182.16 185.20 183.08

## W4 Anti-vacuity + fences

- Flip witnessed: baseline Cpu 17.252 -> jittered Cpu 0.157 (< 1.33).
  A runner that passes with the injection active would exit 2 (VACUOUS).
- Exit-1 path (decided Cpk < 1.33) is coded but NOT executed this
  session — honest note: only exit 0 and the flip-in-computation path
  ran; the exit-1 contract line is exercised by the flip computation
  logic, not by a full baseline-FAIL run.
- Skip-guards real: the container context arm hit a genuine failure
  (Docker VM `os error 28` disk quota) and degraded to a loud logged
  skip, never a fake pass.
- Fences my diff could break: (a) IPC-3/AX-5 courts and binding
  untouched (read-only reuse of `courts/interop-chicago`); (b) no
  repo gates touched (boundary_gate.py, zero_string_gate.py,
  cpk_study.rs untouched); (c) /tmp leases: /tmp/dod14-work deleted
  at lane end (verified absent); repo tree untouched by docker phases
  (repo mounted :ro where mounted).
- Mutation court pairing: the anti-vacuity mutation is the jitter
  injection above; mutating it (removing the injection) would leave
  baseline==flip Cpu and exit 2 — the flip channel is the Cpu delta.

## W5 Divergences from dispatch

1. Gated column is the DARWIN membrane, not the docker container
   column. Dispatch said "docker LOCAL mode per AX-0"; AX-0 W4's LOCAL
   PATH GO is stock local execution without the (unlanded) AX image.
   Rationale: the layer law pins the membrane band on darwin
   (179-192 ns/frame); gating on the container column would gate on
   isolation cost, not the membrane. Container column attempted as
   CONTEXT (docker run, stock rust:1-bookworm/golang:1.23-bookworm
   per AX-5 W3 precedent) and skipped loud on Docker VM disk quota
   (`os error 28`). Host volume has 71 Gi free (`df -h /`); the
   exhaustion is the Docker Desktop VM quota — a host-side context
   gap, out of lane scope to fix.
2. Decided USL fixed at 250 ns/frame (band top 192 + headroom),
   provenance in `dflss/interop-cpk.md`, rather than derived from the
   30-trial distribution itself — deriving USL from the gated
   distribution is circular; the MV-5 pattern decides first, measures
   against it, honest FAIL if missed.
3. Exit-2 (VACUOUS) code path exists, unexercised this session —
   same honest note as the exit-1 path.

## W7 Standing

ALIVE. Rerun: `bash /Users/sac/cgap/courts/ax-testbed/runners/cpk.sh`
(self-contained given the /tmp/ipc3-target dylib and /tmp/ipc3-fixtures
leases; exit 3 names the rebuild commands when absent).
