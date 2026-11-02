---
name: run-gates
description: >-
  Run the cgap release gate ladder: boundary_gate --mutate, zero_string_gate
  + --self-check, cgap-court verdict-run, and the G1-G8 gate-runner blocks.
  Source: docs/jira/v26.11.2/01-release-gates.md (cited, not redefined).
disable-model-invocation: true
---

# Run gates (release gate ladder)

Source of record: `docs/jira/v26.11.2/01-release-gates.md` (G1-G8 blocks
with COMMAND/RECEIPT/FLIP/BLOCKED verbatim). Gates mutate only /tmp
sandbox copies with sha256 fail-closed restore; never the shared tree.
No Process.sleep. Exit-code contract: repo CLAUDE.md.

## Exit ABI (H12 normalization; boundary/zero_string carry full 0/1/2/3)

All gates normalize to: 0 pass / 1 typed refusal (VACUOUS GATE, red
baseline) / 2 subject or prerequisite missing (loud) / 3 loud skip
(`[SKIP] <reason>`, nothing observed). A compile-abort is NOT a failure
count — classify it exit 2, name the broken file. A pass without its
receipt number is not a pass.

## Fast gates (repo CLAUDE.md commands)

    python3 scripts/boundary_gate.py            # 0 pass / 1 refuse
    python3 scripts/boundary_gate.py --mutate   # anti-vacuity flip battery
    #  arms: --deps --strings --register --shim-court-only --cross-gate-court --interference-court
    python3 scripts/zero_string_gate.py         # 0 pass / 1 refuse
    python3 scripts/zero_string_gate.py --self-check  # non-vacuity proof
    python3 scripts/zero_string_gate.py --register    # register arm
    cargo test -p cgap-court                    # court_verdict harness

A mutation-free green gate run is VACUOUS; a court that cannot flip is FAIL.

## Wider gate fleet (inventory; full flags via `--help`)

    python3 scripts/enclave_budget_gate.py --check              # budget
    #  --profile-deployment --subject <crate>=<tier> --verify-symbols(-mutant) --self-check
    python3 scripts/verify_dead_library.py [--self-check]       # also disasm
    python3 scripts/gate_cargo_test.py [--crates ... --json]
    cargo test -p cgap-core --test source_scan_gate             # source scan
    python3 scripts/verify_ost_design_around.py [--test]        # G7-PATENT
    python3 scripts/release_check.py                            # G1-PUBLISH
    python3 scripts/registry_sweep.py                           # registry
    python3 scripts/symbol_gate.py --target <t>                 # G5-SILICON
    python3 scripts/timing_gate.py --claim <c>                  # timing
    python3 scripts/shacl_negative_corpus{,_b}.py               # SHACL A/B
    # cpk_study: no scripts/cpk_study.py on disk (observed 2026-11-02).

## FA10R verdict wording caveat (enclave --profile-deployment)

Success stdout is `MINIMUM ENTRY SURFACE FITS: ...` — NOT "BUDGET GATE"
(renamed 2026-11-02; countermeasures/lane-fa10r-verdict-rename.md). It
carries a mandatory COVERAGE CAVEAT and per-crate linked/audit-rlib
ratios over frozen FA10_AUDIT_RLIB_BYTES — exercised minimal-entry
subset only, not the whole engine. Exit codes unchanged.

## G1-G8 pointers (full COMMAND/RECEIPT/FLIP/BLOCKED in 01-release-gates.md)

- G1-PUBLISH: release_check.py (pin REQUIRED_VERSION). 19/22 publishable;
  flip: strip a crate `description` in /tmp, gate FAIL.
- G2-PARETO: six bench targets via divan2receipt.py (L1 <15 ns, L2 <10 ms
  @1M, L3 >20x, L4 <4 us/tx, L5 <1.45 us/row). No flip => `VACUOUS` exit 3.
- G3/G4: mat_closure_bench / dred_bench; absent target => exit 2 `[SKIP]`.
- G5-SILICON: riscv32imc + footprint.sh + symbol_gate.py + QEMU stack
  paint (flash <32768 B, SRAM+stack <4096 B, 0 alloc/panic); no toolchain
  => exit 2 `[SKIP]`.
- G6-NO-DF: grep -ri datafusion, zero matches. Flip: datafusion dep in a /tmp copy, FAIL.
- G7-PATENT: verify_ost_design_around.py 4/4 DISTINCT-MECHANISM; absent =>
  exit 3 `[SKIP]`.
- G8-STARDF-F1: census 11 rows, 10/10 golden digests, KD-12/KD-14 flips;
  absent scripts => exit 2 `[SKIP]`.

## Flip discipline

Every gate names its FLIP: run it in a /tmp sandbox copy, require the
gate to FAIL on the mutated copy, sha256 verify byte-exact restore, then
bump mtime (`touch`) on any restored file a rebuild will consume.
Threshold met without a flip witness => `VACUOUS: ...` exit 3. Law: flip
before PASS — a PASS is admissible only alongside its FLIP witness from
the same session and subject.
