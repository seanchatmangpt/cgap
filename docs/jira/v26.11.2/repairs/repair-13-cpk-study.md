# Repair-13: Cpk study run (W15 UNVERIFIED item closes)

Lane F14, 2026-11-02, repo /Users/sac/cgap @ main. Read-only + bin run;
only write = this file. Build lease /tmp/f14-target deleted via
shutil.rmtree after run.

## Method

`crates/bench/src/bin/cpk_study.rs` — 6-sigma capability study (C_pk >= 1.67,
fail-closed exit 1), N >= 100 samples per leg, real spec limits:

- **CTQ-05 admission latency**: end-to-end pipeline per sample (Quad64 batch
  -> hash_many_simd BLAKE3 receipt -> level-free parquet read-back of 3
  pages, byte-identity asserted). USL 35 us, one-sided C_pu. n=200.
- **CTQ-06 throughput**: hash_many_simd over 256 MiB/sample.
  Gate LSL 1.0 GiB/s (ratified 2026-10-31); emit reports lsl=8.5 (stale
  display literal, crates/bench/src/bin/cpk_study.rs:341) — gate math uses
  1.0. n=100.
- **CTQ-04 variance**: transpose+branchless filter, 4096-quad deterministic
  batch, USL = 1.15*mean (retired 15% pin; pure dispersion index). n=100.

Stats: mean, sigma_overall (n-1), sigma_within (MR_bar/d2, d2=1.128);
C_pu/C_pl on both sigmas; PASS iff cpk_overall >= 1.67.

## Run

Command: `CARGO_TARGET_DIR=/tmp/f14-target cargo run --release -p bench
--bin cpk_study` (cwd /Users/sac/cgap). **Exit 1 (fail-closed).**
Verbatim: `"verdict": "FAIL", "failed_gates": ["CTQ-06"]`.

## Per-leg table

| Leg | n | mean | spec | Cpk (overall) | Cpk (within) | Verdict |
|---|---|---|---|---|---|---|
| CTQ-05 admission latency | 200 | 10.898 us | USL 35 us | 193.363 | 236.416 | MEASURED-PASS |
| CTQ-06 BLAKE3 throughput | 100 | 1.66 GiB/s | LSL 1.0 GiB/s | 0.855 | 2.131 | MEASURED-FAIL |
| CTQ-04 transpose+filter variance | 100 | 16.141 us (CV 2.68%) | USL 1.15*mean | 1.863 | 3.529 | MEASURED-PASS |

## Standing

- CTQ-05: MEASURED-PASS (mean 10.9 us vs 35 us USL; 19x headroom).
- CTQ-04: MEASURED-PASS (marginal: 1.863 vs 1.67, CV 2.68%).
- CTQ-06: **MEASURED-FAIL** — mean 1.66 GiB/s, sigma_overall 0.26. Gate
  fails on overall sigma; within-sigma 2.131 clears 1.67, so the miss is
  slow drift between samples, not instantaneous spread. Deep margin: mean
  would need ~2.0 GiB/s to reach Cpl=1.67 even with sigma halved. Charter
  note in bin: "honest expectation per charter status: deep fail (<<1.67)".
- W15's Cpk item converts UNVERIFIED -> MEASURED (2 pass / 1 fail). Exit 1
  is the honest fail-closed result, not a run defect.

Bin quirk (not this lane's write scope): emit for CTQ-06 hardcodes
`lsl: 8.5` in JSON while gate uses 1.0 (crates/bench/src/bin/cpk_study.rs:341).
