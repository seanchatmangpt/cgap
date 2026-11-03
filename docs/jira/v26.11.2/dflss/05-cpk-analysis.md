# 05-cpk-analysis (DFSS Measure/Analyze phase)

Campaign v26.11.2 starDF. Companion to `../15-ctq-tree.md` (VOC -> CTQ
flowdown; that file owns the tree, this file owns the capability
arithmetic). Rig of record: `crates/bench/src/bin/cpk_study.rs`
(lane-g09-cpk-stabilization.md landed the n=30 sliding-window version
this analysis re-derives from). Prior deep runs: repair-13
(`../repairs/repair-13-cpk-study.md`, n>=100) and lane-g09 (n=30).

## 1. Fresh derivation (this session, 2026-11-02)

Build: `CARGO_TARGET_DIR=/tmp/df4-target cargo build --release -p bench`
-> exit 0 (14.83 s; 3 pre-existing manifest warnings, none new).
Run: `/tmp/df4-target/release/cpk_study` -> **exit 1, verdict FAIL,
failed_gates [CTQ-06, CTQ-04]** (host under concurrent wave load at
run time). Machine: Apple M3 Max.

| Gate | n | mean | spec | Cpk(overall) | Cpk(within) | Verdict @1.33 floor |
|---|---|---|---|---|---|---|
| CTQ-05 admission latency | 30 | 8.447 us | USL 35 us | 190.949 | 204.471 | PASS |
| CTQ-06 BLAKE3 throughput | 30 | 1.574 GiB/s | LSL 1.0 GiB/s | 0.694 | 0.773 | FAIL |
| CTQ-04 transpose+filter | 30 | 16.640 us (CV 12.26%) | USL 1.15*mean | 0.408 | 1.016 | FAIL |

Exact figures are the rig's own JSON v2 (window_n=30,
cpk_exit_floor=1.33, cpk_target=1.67 reported as context). Both CTQ-06
and CTQ-04 sampled higher dispersion on this loaded host than g09's
quiet-window run (CTQ-06 Cpl 28.97, CTQ-04 Cpu 4.176); the gate math is
unchanged. Verdict: load-sensitive, honestly FAILING today.

## 2. Anti-vacuity flip (CPK_INJECT_FAIL)

`CPK_INJECT_FAIL=1 /tmp/df4-target/release/cpk_study` -> exit 1,
anti_vacuity_injection=true, CTQ-06 Cpk(overall) **-9.6525** at the
forced LSL 10.0 GiB/s. FLIPPED.

Honest caveat: the baseline run this session already exits 1 (CTQ-06
and CTQ-04 fail under load), so the exit-code channel alone cannot
discriminate injected-vs-healthy today; the flip is witnessed through
the Cpk sign change (0.694 -> -9.65) and the JSON injection flag, not
through an exit-code delta. Under g09's quiet-window baseline (exit 0),
the same injection flips exit 0 -> 1 (witnessed there, run 3 of
lane-g09-cpk-stabilization.md). A one-sided rig would be vacuous.

## 3. The 0.855 drift log, honestly

Repair-13 recorded CTQ-06 Cpk(overall) 0.855 (mean 1.66 GiB/s, sigma
0.26, n=100). The draft-era causal story ("optimization moved 12.4 ns
to 6.00 ns") is WRONG and is corrected here per ERRATA law: those
figures are the CSCB C2 p99 ladder numbers (lane-b01-cscb-c1-c2.md:19;
AS-OF-RECEIPT per audit/doctrine-commercial-proofs.md:26), a different
metric entirely, and no codegen change explains a between-sample
spread. The honest narrative is a SAMPLING-METHODOLOGY repair: the
0.855 was load-induced between-sample drift under the concurrent wave
(n=1-style sampling could not separate process capability from host
contention). Evidence: within-sigma Cpk was 2.131 (clears 1.67) while
overall-sigma was 0.855 - the signature of slow between-sample drift,
not instantaneous spread (repair-13-cpk-study.md:36-41;
lane-g09-cpk-stabilization.md:15-20). The fix was the n=30 sliding
window (control-chart-style separation of sigma_overall from
sigma_within via MR_bar/d2, d2=1.128), not a kernel change; the hash
kernel and input are unchanged since 0539fc7f.

## 4. Gage R&R: BATCH=64 amortized Instant is the measurement system

For CTQ-02 (frame ingress cost, `../15-ctq-tree.md` Section 3: USL 15 ns
/ns-per-frame, measured 10.20-10.30 ns MEASURED in
`../waves/w3-l1-ingress.md:63,69-70`; quiet host, divan fastest/median
columns — contended divergence registered `../11-measured-claims.md`
row 28, 124.06 ns/frame slowest under combined bw+cpu): the gauge is
BATCH=64 frames
amortized over one Instant pair, not per-frame timing. Literal per-frame
Instant timing on macOS cannot resolve a 10 ns quantity: clock overhead
is ~40 ns, i.e. the instrument exceeds the measured quantity ~4x and
any per-frame number would be gauge variance, not process variance.
The gauge, not the process, defines the number: the w3 receipt's
10.20 ns/frame is a batch-amortized derivation, and the p99-of-batch
protocol (audit/doctrine-commercial-proofs.md:73, M1) must carry that
method label or it overclaims. Every capability number above inherits
the same rule: a Cpk is only as lawful as its gauge; sigma_of_gauge
must be stated with sigma_of_process.

## 5. CTQ-05 capability: ASPIRATIONAL-UNVERIFIED

The dispatch asks about "CTQ-05 25 ms": there is NO 25 ms gate in this
rig or anywhere in the campaign surface. The DECIDED gate lives in
`crates/csct/src/bin/cscb_c3_closure.rs:40`: DECIDED_BUDGET_MS = 60,000.0
(lane MV-5 gate re-scope, witnessed consts re-read fresh this session:
ASPIRATIONAL_MS = 25.0 const exists at :42 but is NOT a gate; RUNS=7
min-of-N; corpus seed 0xA20C_0FFE_E5A7_E000 at :28, SplitMix64
multiplier 0xBF58_476D_1CE4_E5B9 at :30). Prior decision run under
load: min 36330.134 ms / median 40258.075 ms / max 52868.876 ms - the
aspirational 25.0 ms figure is ~1.5-2.1x below even the measured minimum
and was never a gate anywhere (EXP-03 finding, in-source). Therefore:
any 25 ms-class closure capability is **ASPIRATIONAL-UNVERIFIED** - no
active gate exists to measure it against; the only active gates are
the 60 s DECIDED budget and this rig's three Cpk gates. It never rounds
up to MEASURED.

## 6. Standing

| Claim | Standing | Ground |
|---|---|---|
| Fresh Cpk table (Sec 1) | MEASURED (FAIL, honest) | exit 1 this session |
| CPK_INJECT_FAIL flip | FLIPPED (Cpk 0.694 -> -9.65, exit 1) | run this session |
| 0.855 = load-induced drift | MEASURED (prior receipts, re-read) | repair-13:36, g09:15-20 |
| "12.4 -> 6.00 ns codegen" story | REFUTED (ERRATA vs draft) | lane-b01:19 is C2 p99, different metric |
| BATCH=64 gauge claim | MEASURED method label | w3-l1-ingress.md:63,69-70 |
| 25 ms closure capability | ASPIRATIONAL-UNVERIFIED | cscb_c3_closure.rs:40-42 consts fresh-read |
| Quiet-window healthy PASS | ALIVE as-of-g09, UNVERIFIED today | g09 run 2, exit 0; not reproduced this session |

## 7. Cross-links

- `../15-ctq-tree.md` - VOC flowdown tree (owner of CTQ definitions).
- `../repairs/repair-13-cpk-study.md` - first deep run (n>=100), 0.855.
- `../lanes/lane-g09-cpk-stabilization.md` - n=30 sliding-window rig
  this analysis re-derives from; supersedes repair-13 verdict rule
  (1.33 exit floor, disclosed there).
- `../audit/doctrine-commercial-proofs.md` - M1 method-label duty for
  the C2 p99 figures; the errata source for Section 3.
