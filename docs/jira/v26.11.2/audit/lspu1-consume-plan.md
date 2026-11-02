# lspu1-consume-plan

Branchless-lsp Tier-2 court: cgap consume-plan (LSPU-1, 2026-11-02).
Subject: run_tier2 (lsp-max rule_pack.rs:236): real rustc --emit=obj via
CompilerWorker (:242), DisasmCourt::for_target + judge_classified
(:251-252), per-fn seeded classification (:253-256), ReceiptVector fold
(:267-282); verdict_diagnostics (:300) emits ANDON_REFUSED at
DWARF-mapped spans; attrs via parse_attrs (G04); 80-byte seal (G07).

## Gap matrix: run_tier2 vs verify_disassembly_invariants.py

| dimension | run_tier2 | AE4 script |
|---|---|---|
| targets | aarch64 + riscv32imc (:237-241) | riscv32imc only (TARGET :52) |
| build | debuginfo + obj, one rustc/file | emit=asm opt-level=3 + linked staticlib witness |
| instr classes | full capstone ALU/LS/branch/back-edge (disasm_court.rs:458,:691) | 16 branch mnemonics only (:79-83) |
| attribution | DWARF LineMap PC->line (:322) | substring symbol match (:149-176) |
| anti-vacuity | NONE (court trusts itself) | arms C/D/E + sha256 restore law |
| residual law | attr-aware (parse_attrs) | static RESIDUAL_ALLOWLIST (:67-77) |
| scope | one saved file | whole crate, 19 allowlist fns |

Verdict: consumption must be ADDITIVE, not substitutive — Tier-2 has
targets x2, classes x4, DWARF spans; AE4 has the anti-vacuity arms and
the linked witness (resurrection risk) that Tier-2 lacks.

## Findings

1. cscb bins have ZERO disasm asserts today; black_box uses in c1_c2/c4
   are timing harness, not disasm gates. Rerouting through Tier-2 would
   lose release-profile timing + black_box semantics. NOT recommended.
2. The missing assertion cgap lacks anywhere: cgap-branchless kernels
   hold n_refused == 0 on aarch64 (AE4 is riscv-only). Tier-2 already
   supports aarch64 — this is the additive win.
3. Mutation arm port: feed AE4's arm C mutant text (:262-268) as a
   source string to run_tier2(mutant_source, riscv32imc), assert
   n_refused >= 1 — the only mutation arm portable without disk
   mutation.
4. One-shot CLI gap: no argv mode exists (main.rs:11-19 stdio only);
   hook consumption needs JSON-RPC handshake scripting or a new CLI arm
   in lsp-max.

## Ranked consume-plan

1. **PostToolUse branchless-court.sh hook** (settings.json insert after
   fmt-edited entry :70-77). Skeleton = fmt-edited.sh:8-18 (stdin JSON,
   tool_input.file_path, .rs filter). Assert: no ANDON_REFUSED in the
   touched file's diagnostics; non-empty -> exit 2 with text (exit-2
   feeds stderr back — same channel as flip-tests :59-68). Order after
   fmt so the court sees formatted source. Cost control: real rustc per
   save — tier aarch64 always, riscv32imc only under
   crates/cgap-branchless/; debounce by mtime (no precedent, must be
   new).
   - Falsifier: clean select.rs -> exit 0 <1s non-.rs path; injected
     `if cond { t } else { f }` into cmov_u64 -> exit 2 naming
     ANDON_REFUSED + line.
2. **aarch64 Tier-2 run over cgap-branchless in CI** (`just branchless`
   recipe after `courts`, justfile:56-66): AE4 + one clean + one mutant
   source through run_tier2, n_refused flips 0->>=1.
   - Falsifier: `just branchless` exit 0 clean; exit 1 on mutant text.
3. **Tier-2 mutant arm** (above, ported) with exit ABI mirroring AE4
   (:56-59).
4. **verify-ladder rung** in verify_v26.10.31.sh (justfile:51-53) —
   lowest novelty, do last.
5. NOT recommended: folding into gate_cargo_test.py (parses cargo test
   stdout only, RESULT_RE :31-34 — single-observable law).

## See Also

audit/lspu2-capability-inventory.md · audit/lspu3-adoption-runbook.md
· lanes/lane-g05-lsp-wire-rebuild.md · rule_pack.rs:236
