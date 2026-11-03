# Charter §7 Gates T-1/T-2 verification (lane CE-09, 2026-11-02)

Repo /Users/sac/cgap @ main, branch main. Sole file written: this receipt.
No git commands. No Process.sleep: pre-leg host load was bounded-polled
(3 polls / 90 s deadline, threshold loadavg1 < 2.0) via
/tmp/ce09-work/quiet_poll.py; the CONTENTION-SUSPECT guard
(`crates/bench/benches/chains-ingress.rs`, per lane-rc11) is the per-run
honesty mechanism. Ambient discipline follows
`../bench-validate/bench-01-ingress.md` (the dispatch's named precedent).

## §1 Environment + build

- Host: Mac15,9, Apple Silicon 16-core (no SMT), Darwin 25.2.0.
- Build root lease: CARGO_TARGET_DIR=/tmp/ce09-target (deleted at lane end).
- Build: `cargo bench -p bench --bench chains-ingress --no-run` EXIT=0.
- Pre-leg quiet poll: loadavg1 2.58/3.52/3.49 — **HOT** (never < 2.0 this
  session; the host did not offer a quiet window).

## §2 Gate T-1: chains-ingress `ingress_chain` @1M, N=3 legs

Command per leg: `cargo bench -p bench --bench chains-ingress --
ingress_chain`, CARGO_TARGET_DIR=/tmp/ce09-target. stderr logs
/tmp/ce09-work/ig{1,2,3}.err; stdout /tmp/ce09-work/ig{1,2,3}.out.
divan column order is fastest | slowest | median | mean.

| leg | fastest @1M | median @1M | slowest @1M | ns/f fastest | ns/f median | guard | loadavg1 (pre / post) |
|---|---|---|---|---|---|---|---|
| 1 | 11.01 ms | 12.00 ms | 13.63 ms | 10.50 | 11.44 | green (0 markers; cal jitter 1.00; in-run worst 1.56) | 2.41 / 11.47 |
| 2 | 11.05 ms | 11.86 ms | 12.86 ms | 10.54 | 11.31 | green (0 markers) | 7.85 / 7.85 |
| 3 | 11.03 ms | 11.71 ms | 11.71 ms | 10.52 | 11.17 | green (0 markers) | 6.88 / 6.88 |

Cross-leg: fastest min 10.50 / median-of-medians 11.31 / slowest max
13.00 ns/frame. CONTENTION-SUSPECT markers: 0 across all 3 legs.

## §3 Gate T-2: quota/posh purity courts

Command: `cargo test -p cgap-git --test quota_alloc_purity_court --test
posh_alloc_sentry_court` (both binaries, one invocation). Log
/tmp/ce09-work/t2.out. **EXIT=0.**

- `quota_alloc_purity_court` — 1 passed / 0 failed / 0 ignored (0.00s).
  Court asserts allocs==0 AND bytes==0 inside the open CountingAlloc
  window (`crates/cgap-git/tests/quota_alloc_purity_court.rs:8`).
- `posh_alloc_sentry_court` — 1 passed / 0 failed / 0 ignored (5.85s).
  Witness counts: **0/0** (0 alloc witnesses / 0 byte witnesses under the
  purity window; the sentry leg separately proves the harness flips —
  witness line `posh_alloc_sentry_court.rs:19-20`).

## §4 Verdicts (honestly scoped)

- **T-1 PASS-WITH-SCOPE (hot ambient)**: fastest 10.50-10.54 ns/frame
  across all 3 legs (min 10.50), median 11.17-11.44. Against row 5's
  quiet-host claim (10.20/10.30):
  - fastest: leg 1's 10.50 sits inside the red-10 4.5% band (10.66 was
    red-10's own quiet re-check; band ~9.74-10.67 around 10.20 —
    fastest min 10.50 inside); legs 2-3 also inside (10.52/10.54).
  - median: 11.17-11.44, +8.4% to +11.1% above the 10.30 claim —
    outside the 4.5% tolerance, consistent with bench-01's hot-ambient
    median profile (11.51-11.91 there).
  - Guard verdicts were GREEN on every leg (0 CONTENTION-SUSPECT
    markers) — the measured drift is ambient-load-shaped, matching the
    bench-01 precedent at similar loadavg. Per the consumption rule and
    bench-01, this is a **hot-ambient reproduction, not a quiet-host
    one**: the row-5 quiet-host claim is **UNVERIFIED-QUIET** on this
    host this session — neither confirmed nor refuted (no quiet window
    was offered).
  - Row 8 (<15 ns/frame): all medians < 15 — holds, guard-green.
- **T-2 PASS**: both courts exit 0, 1/1 passed, 0 failures, 0/0
  alloc/byte witnesses under the purity window.

## §5 Fences (must-not-regress, run this session)

- CONTENTION-SUSPECT guard (chains-ingress.rs:716-802): 0 markers on all
  3 clean legs — guard quiet-detection exercised live on each leg (the
  guard's flip witness is RC-11's, not re-run here; no mutation battery
  was in this lane's charter).
- Row-5 quiet-host scoping (06-bench-protocol §8): honored — a marked
  run would void the claim; none occurred; the hot-ambient scope is
  disclosed rather than rounded to quiet.

## §6 Standing

- Gates T-1, T-2: **ALIVE** (commands + exits above, this session).
- Row 5 quiet-host claim: UNVERIFIED-QUIET (scoped, per bench-01
  precedent; not a failure-to-failure round-up).
- Artifacts: /tmp/ce09-work/{ig1,ig2,ig3}.{out,err}, t2.out; build-root
  lease /tmp/ce09-target deleted at lane end.
