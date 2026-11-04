# AF-10 — CBMQS Seven-Gate Evaluator (v26.11.3)

- **Lane**: AF-10 IMPLEMENTATION, Chicago Bare-Metal Qualification Standard
  as an executable evaluator composing the LANDED courts' machinery.
- **Deliverable**: `/Users/sac/cgap/crates/cgap-aoq/tests/cbmqs_gates.rs` (new).
- **Date**: 2026-11-03.

## Deliverable

One driver test, `cbmqs_seven_gates_summary_table`, runs eight gate
functions (G1 + G1b floor leg) inside `catch_unwind`, prints ONE CBMQS
summary table (typed PASS/FAIL/SKIP per gate), and fails the driver on
any FAIL. Skips are loud with typed reasons — never fake passes.

## Gate map (compose, don't re-implement)

| Gate | Composes | Result ×2 |
|---|---|---|
| G1 zero-alloc hot path | AB-9 counting-allocator court: positive control, then 100k IPC1 windows (256B bodies) emit+read, 0 allocs / 0 bytes after init | PASS ×2 |
| G1b slab rate floor | AB-19 method: fill(64×64B)+reset, min-of-3 pushes/sec, floor 100k/s (receipt 12,204,201/s) | PASS ×2 (6.23M / 5.40M pushes/s) |
| G2 wire framing ceiling | AB-6 bench method: min-of-7×100k encode+decode ns/header, ceiling 100 ns | **FAIL ×2** (717.0 / 719.7 ns) — see below |
| G3 spec-drift | AB-6 drift court: landed TTL field table == generated layout; injected offset mutation flips (flip witness) | PASS ×2 |
| G4 anti-vacuity KAT | AB-2 SHAKE-128/256 empty KATs + AB-3 RFC 8439 §2.5.2 Poly1305 tag + 2 flip witnesses | PASS ×2 |
| G5 storm soak | AB-10 method scaled: 50k frames through a 1-byte-per-accept sink (13.2M accepts, >1× frames) + AB-11 `FrameTooLarge` typed refusal on a REAL wire-shm segment (len=4081, max=4080) | PASS ×2 |
| G6 churn | AB-17 200-cycle court | **SKIP (typed)** — lane not landed |
| G7 receipt chain | AB-14/AF-7 machinery: Shake256 digest on AoqChain, receipt per frame (2,048 appends), `verify_records` green, tamper flip witnessed, **measured** per-link cost recorded | PASS ×2 (measured 59.7k / 72.1k ns/link — recorded, NOT fenced) |

## G2 — fail-honest, with the evidence

The prompt's gate law (encode+decode < 100 ns, from AB-6's receipted
86 ns) does not reproduce on current disk state. Both my gate and the
LANDED bench binary itself measure ~700 ns debug today:

- `cargo test -p cgap-aoq --test aoq1_bench_court -- --nocapture` →
  `bench: encode 399.4 ns/header, decode 302.2 ns/header (min of 7x100000)`.
- My G2: 717.0 / 719.7 ns combined (min-of-7×100k, same method) — agrees
  with the landed court within ~3%.

Per no-overclaiming law, a receipted number is as-of-date; the current
measurement refutes the 86 ns baseline. I did NOT relax the fence: G2 is
a typed FAIL against the stated 100 ns ceiling, with the landed court's
own number as corroboration. Un-regressing encode+decode from ~700 ns to
the receipted 86 ns is an owner-lane task (AB-6/AB-7; header.rs/codec.rs
carried disclosed mid-wave convergence edits after the receipt). Re-run
`cbmqs_gates` when the kernel re-baselines; the gate flips to PASS
mechanically.

## G7 gate-law correction (implemented as instructed)

The prompt's "<25 ns, 18.1 ns baseline" is UNSUPPORTED (AF-2 audit):
no 18.1 ns number exists in any landed receipt; AB-12's 64B scalar is
disclosed as latency-noise. G7 therefore appends a receipt per frame,
requires chain verify green + tamper-flip, and records the MEASURED
per-link cost (~60-72 µs/link, Shake256 debug, 2,048 links) without a
fence. No invented number anywhere in the gate.

## Gates run (exact commands + exits, this session)

- `cargo test -p cgap-aoq --test cbmqs_gates -- --nocapture` — pass 1:
  verdict `6 PASS / 1 FAIL / 1 SKIP`, `test result: FAILED` (driver
  correctly fails on the G2 FAIL), 2.87s.
- Same command pass 2: `6 PASS / 1 FAIL / 1 SKIP`, 3.92s. Gate run ×2
  MET (same verdict both passes; measured values recorded above).

## Loud skips

- G6 SKIP typed: no AB-17 receipt in
  `docs/jira/v26.11.3/receipts/` and no 200-cycle court in
  `crates/*/tests`. Lawful per the charter: skip-with-typed-reason,
  never fake.

## Disclosed cross-lane edits (same-checkout fan-out law)

1. `crates/wire-shm/Cargo.toml`: removed the empty `[workspace]` table
   (one-line manifest unblock) — cargo refuses a path dev-dep into a
   crate that declares itself a workspace root ("multiple workspace
   roots"). Standalone `--manifest-path` builds are unaffected (implicit
   single-package workspace). This is what G5's FrameTooLarge leg needs
   to compose the landed AB-11 machinery from inside cgap-aoq.
2. `crates/cgap-aoq/Cargo.toml`: added `wire-shm` path dev-dep (AF-10).
   AF-14's concurrent edits (cgap-blake3 dev-dep + `[[test]]
   cbmqs_runner` harness=false entry) landed mid-wave; its manifest
   briefly hit "duplicate [dev-dependencies]"/missing-test-file errors
   that self-resolved as AF-14 landed (cbmqs_runner.rs landed while I
   was polling, ~30s). I applied NO edit to AF-14's sections and no SLA
   fix to its manifest — its errors cleared on its own landing.
3. `crates/cgap-crypto` was compile-broken by a sibling lane mid-wave
   (~50s window, E0061 x3); resolved by the owning lane. I made no
   edit; I polled to green under the compile-freeze SLA.

## Standing

- cbmqs_gates: ALIVE (ran ×2 this session, exact subject, commands +
  exits above) — with G2 fail-honest FAIL and G6 typed SKIP.
- G2 re-baseline: owner lane (AB-6/AB-7) to re-receipt encode+decode;
  then the gate flips mechanically.
- G6: flips to evaluable when AB-17's 200-cycle court lands.

---

## Coordinator fix relay (appended 2026-11-03, post-convergence)

Two coordinator-directed updates; appended, not rewritten.

### 1. G6 SKIP -> PASS-CITED (coordinator correction)

The G6 loud-SKIP was stale: AB-17 LANDED cross-repo in lsp-max — receipt
`/Users/sac/lsp-max/docs/jira/v26.11.3/receipts/ab-17-churn-stress.md`,
court `/Users/sac/lsp-max/crates/lsp-max-compositor/tests/child_churn_stress.rs`
(200-cycle spawn/SIGKILL/evict/respawn churn, 4 courts green x2 in 12.14s
/ 13.54s; leak witnesses: fds 34->10 plateau, RSS plateau 13088 KiB from
cycle 100 with 0 growth, every killed pid reaped). The original cgap-
scoped receipt-dir lookup missed it.

Implemented as PASS-CITED, disk-grounded: the gate asserts the receipt
AND court file exist on disk and that the receipt still carries the
measured witnesses ("200-cycle", fd "34", RSS "13088", court filename);
a vanished referent or stripped witness is a typed FAIL, never a silent
pass. Cross-repo citation disclosed (no same-host re-run; the court's
subject is lsp-max-compositor, not a cgap crate — coordinator offered
cite-or-rerun; cite chosen).

### 2. G2 release profile pin (coordinator G2 methodology ruling)

The 100 ns ceiling is a RELEASE law. G2 is now profile-pinned:
- RELEASE: amortized-min (min-of-7x100k, 100k warmup) is FENCED vs
  100 ns. Measured this session: **87.1 / 86.1 ns** (encode ~44.5 +
  decode ~42.6) — matches AB-6's receipted 86 ns. PASS x2.
- DEBUG: loud typed SKIP of the fence (never a fake pass), with the
  debug numbers recorded as UNFENCED context (711.8 ns amortized /
  750.0 ns per-call-median this session).
- AB-6's lane landed the two-axis method inside this gate file
  (amortized-min fenced + per-call-median unfenced, n=1000 individually
  timed black_box'd calls: 84.0 ns) — disclosed cross-lane edit to my
  owned file, method per the coordinator's AF-17 axis ruling. Kept.

### 3. G1b release anti-DCE fix (session-found, this window)

The release run exposed a measurement defect in my G1b: 11.08 BILLION
pushes/s — LLVM folded the unobserved slab away; the number was vacuous.
Fix: black_box the slab byte image + len each cycle. Post-fix release:
28.18M / 28.77M pushes/s (min-of-3), debug 5.76M — real numbers, floor
100k/s still cleared by >250x.

### Gates (post-fix, exact commands + exits)

- `cargo test --release -p cgap-aoq --test cbmqs_gates -- --nocapture`
  — pass 1: `8 PASS / 0 FAIL / 0 SKIP`, `test result: ok` (0.25s).
- Same command pass 2: `8 PASS / 0 FAIL / 0 SKIP`, `test result: ok`
  (0.24s). GREEN x2 MET (release profile, the gate law's profile).
- `cargo test -p cgap-aoq --test cbmqs_gates -- --nocapture` (debug,
  context run): `7 PASS / 0 FAIL / 1 SKIP` — the one SKIP is the G2
  release-profile pin itself, loud with unfenced debug context.

### Standing (updated)

- cbmqs_gates: ALIVE — release profile GREEN x2 (8 PASS / 0 FAIL /
  0 SKIP); debug run green with the single profile-pin SKIP.
- G2: PASS at 87.1 / 86.1 ns amortized-release (ceiling 100 ns); debug
  context recorded unfenced.
- G6: PASS-CITED cross-repo, disk-grounded each run (missing receipt /
  court file or stripped witnesses -> typed FAIL).
- G1b: PASS with real (anti-DCE) release numbers 28.2M / 28.8M pushes/s.
