# ZD-76 — Terminal Sweep Verification Receipt

- **Subject**: cgap @ `e32331919e2e45aa1015c53cd89b497870c75141` (committed tree; 4 dirty files:
  untracked `crates/cgap-ffi/src/bin/aoq_sidecar.rs` + `crates/cgap-ffi/tests/aoq_sidecar_court.rs`
  — ZD-56 in-flight).
- **Date**: 2026-11-04
- **Lane**: verification (own file only; no source edits, no git).

## Gate table

| # | Gate | Exit | Result |
|---|---|---|---|
| 1 | `cargo check --workspace` | 0 | PASS (2 warnings in untracked sidecar bin: unused `PAD_HI`, 1 more) |
| 2 | `cargo test -p cgap-ffi` | 0 | PASS — lib 60/60, sidecar court 7/7, courts all green (15 `test result: ok` suites) |
| 3 | `cargo test -p cgap-git --release` | 0 | PASS — 40 `test result: ok` suites, 0 failed (ZD-53 courts green on committed tree) |
| 4 | `cargo test -p cgap-core --test manifest_court` | 0 | PASS — 3 passed, 0 failed |

## Intermediate red, classified

First `cargo test -p cgap-ffi --lib` run (pre-`cargo build`) reded one test:
`sovereign::wave9_courts::wave9_a07_zero_string_gate_over_all_built_crates` — GATE-REFUSED
`execution-crate-no-artifacts` for cgap, cgap-beam, cgap-git, cgap-gpu, cgap-integration at
profile=debug. This is **not session-introduced and not lane interference**: the A07 gate's
anti-vacuity precondition requires full debug artifacts, and `cargo check` does not produce
linkable artifacts for the doc execution family. After `cargo build --workspace` (exit 0),
the gate exits 0: `SUMMARY scanned=36 crates=22 hits=0 emitter_hits=2 adv_hits=0 unknown=315015
allowed=6699 skipped_artifacts=0 vocab=9 gate=1.4.3`. Classification: **environmental
(build-coverage precondition), resolved by building; the committed subject is green.**

## Notes

- First full-ffi run took 317.76s for the lib (A05/A07/A20 cargo-suite courts spawn real
  subprocess suites); subsequent runs 48.44s (warm courts).
- Warnings in the ZD-56 in-flight sidecar bin are lane-in-flight, not sweep regressions.
- No git operations performed. Receipt is the only file created.
