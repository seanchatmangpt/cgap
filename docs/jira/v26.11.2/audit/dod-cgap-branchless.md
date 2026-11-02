# dod-cgap-branchless

Audit lane EXP-01 — DoD audit of `crates/cgap-branchless`, re-verified from
live commands on 2026-11-02 (not cited from G01/G02 receipts).

## 1. Disassembly invariants — VERIFIED-DONE

Command: `python3 scripts/verify_disassembly_invariants.py` — exit 0.
Script verdict: `AE4 disassembly-invariants court: CLEAN (census + linked
witness + mutant + self-check + binary mutant all held)`. Allowlist of 28
functions; per-row verdicts for the three EXP-01 targets:

| Function | Verdict | Evidence |
|---|---|---|
| count_leading_eq_u8 | PASS | `[PASS] ... 0 forbidden branches` |
| prefix_max_u8x8 | PASS | `[PASS] ... 0 forbidden branches` |
| transpose_64x64 | PASS | `[PASS] ... 0 forbidden branches` |

Additional residuals (allowed, documented in-script): cmov_u64 (1x bnez),
eytzinger_restore (1x beqz), select_slice (2: beqz len-guard + bnez), all
tagged "RV32 ISA ceiling". G01/G02 receipts claim these three rows; this
audit re-verified them live, independent of the receipts.

## 2. Dead accumulators / unused vars — VERIFIED-DONE

The formerly-warned `acc` in `scan.rs` is live and consumed: every write to
`acc` is followed by a read (scan.rs:123-152, prefix_sum chain feeding
`out[0..8]`). No dead store. `transpose.rs` has a single `mut` binding
(`let mut x = rows;`, transpose.rs:79), consumed by the stage shifts. The
`cargo test -p cgap-branchless` compile emitted zero `warning:` lines.

## 3. Test suites — VERIFIED-DONE

Command: `cargo test -p cgap-branchless` — exit 0.

| Suite | Count | Expected | Match |
|---|---|---|---|
| lib unittests (src/lib.rs) | 34 passed | 34 | yes |
| boundary_sweep_courts | 7 passed | 7 | yes |
| branchless_census | 3/3 | 3/3 | yes |
| scan_courts | 8/8 | 8/8 | yes |
| transpose_courts | 7/7 | 7/7 | yes |

Mutation teeth (anti-vacuity arms, named):

- `branchless_census`: arm (a) branchy reference `naive_if_cmov` must flag
  BRANCHY; arm (b) mutation **M-REP1-1** (if-injection into a select.rs
  sandbox copy must push select_slice past its residual); arm (c) **M-INL-1**
  (data-dependent branch into a caller probe must flip the caller HOT cell
  off 0). VACUOUS-COURT panic guards at branchless_census.rs:466, :696.
- `scan_courts`: anti-vacuity arm (a) naive serial reference must be
  BRANCHY; arm (b) mutation **M-TS2-1** — runtime-bound loop injected into
  prefix_xor_u64x8 must flip the census; guard at scan_courts.rs:483.
- `transpose_courts`: **M-TS4-1** single-bit flip in the stage-7 mask of a
  /tmp copy of transpose.rs must flip the differential court
  (transpose_courts.rs:389-409, guard at :483
  `anti_vacuity_mutant_flips_differential_court`).
- `boundary_sweep_courts`: `reference_model_has_teeth` — a deliberately
  wrong expected lane (one injected bit-flip, boundary_sweep_courts.rs:374)
  must be rejected.

## 4. no_std buildability — VERIFIED-DONE

Command: `cargo check -p cgap-branchless --no-default-features --target
riscv32imc-unknown-none-elf` — exit 0 (`Finished \`dev\` profile`).
Toolchain: nightly (active); riscv32imc target installed.

## Summary

All four areas VERIFIED-DONE. No DEFECT-OPEN, no ASPIRATIONAL-UNVERIFIED,
no LOUD-BLOCKED.
