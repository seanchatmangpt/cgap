---
name: run-gates
description: >-
  Run the cgap release gate ladder: boundary_gate --mutate, zero_string_gate
  + --self-check, cgap-court verdict-run, and the G1-G8 gate-runner blocks
  with expected exits, counts, and flip courts. Source:
  docs/jira/v26.11.2/01-release-gates.md (cited, not redefined).
disable-model-invocation: true
---

# Run gates (release gate ladder)

Source of record: `docs/jira/v26.11.2/01-release-gates.md` (G1-G8 blocks
with COMMAND/RECEIPT/FLIP/BLOCKED verbatim). Exit-code contract (repo
CLAUDE.md): 0 pass / 1 gate fail / 2 subject missing / 3 loud skip.
Gates mutate only /tmp sandbox copies with sha256 fail-closed restore;
never the shared tree. No Process.sleep.

## Fast gates (repo CLAUDE.md commands)

    python3 scripts/boundary_gate.py            # 0 pass / 1 fail
    python3 scripts/boundary_gate.py --mutate   # anti-vacuity flip battery
    python3 scripts/zero_string_gate.py         # 0 pass / 1 fail
    python3 scripts/zero_string_gate.py --self-check  # non-vacuity proof
    cargo test -p cgap-court                    # court_verdict harness

A mutation-free green gate run is VACUOUS (exit nonzero). A court that
cannot flip is FAIL. Skips are loud (`exit 3` + `[SKIP] <reason>`), never
silent. A pass without its receipt number is not a pass.

## G1-G8 pointers (full COMMAND/RECEIPT/FLIP/BLOCKED in 01-release-gates.md)

- G1-PUBLISH: scripts/release_check.py (pin REQUIRED_VERSION first).
  Receipt: 19/22 publishable set; per-crate package exit 0; workspace
  version exact. Flip: strip a crate `description` in /tmp, gate must FAIL.
- G2-PARETO: six cargo bench targets; ns/frame from criterion
  estimates.json via scripts/divan2receipt.py. Receipt: L1 <15 ns/frame;
  L2 <10 ms @1M; L3 >20x vs L2 baseline; L4 <4 us/tx; L5 <1.45 us/row.
  Flip: mutate the kernel in /tmp, threshold must flip.
- G3-MAT-BENCH: mat_closure_bench. Receipt: >50M closure facts/s/core,
  single-core like-for-like vs RDFox 2-6.1M facts/s. Absent target =>
  exit 2 `[SKIP] mat-closure-bench-absent` (observed 2026-11-01).
- G4-DRED: dred_bench. Receipt: <3.0 ns/batch at 90% churn; <0.15 ns
  entropy variance; >15M facts/s; N>=9 per-batch numbers. Absent target
  => exit 2 `[SKIP] dred-bench-absent` (observed 2026-11-01).
- G5-SILICON: riscv32imc + footprint.sh + symbol_gate.py + QEMU
  stack-paint. Receipt: flash <32,768 B; SRAM+stack <4,096 B; 0
  alloc/panic symbols. Missing toolchain => exit 2 `[SKIP]`.
- G6-NO-DF: grep -ri datafusion over both repos (comments count as hits).
  Receipt: zero matches. Known pre-fix hit: Cargo.toml:33 stale CD-1
  comment must be deleted first. Flip: append a datafusion dep in a /tmp
  Cargo.toml copy; gate must FAIL.
- G7-PATENT: verify_ost_design_around.py. Receipt: 4/4
  DISTINCT-MECHANISM + auditor sign-off. Script absent (observed
  2026-11-01) => exit 3 `[SKIP] verify_ost_design_around.py-absent`.
- G8-STARDF-F1: purrdf census/golden/kd_flip scripts. Receipt: census
  exit 0 with 11 mapping rows; 10/10 golden digests; KD-12/KD-14 flips.
  Scripts absent => exit 2 `[SKIP] stardf-census-scripts-absent`.

## Flip discipline

Every gate names its FLIP (anti-vacuity mutation) in its block: run it in a
/tmp sandbox copy, require the gate to FAIL on the mutated copy, sha256
verify byte-exact restore, then bump mtime (`touch`) on any restored file a
rebuild will consume. Threshold met without a flip witness => receipt line
`VACUOUS: ...` and exit 3.
