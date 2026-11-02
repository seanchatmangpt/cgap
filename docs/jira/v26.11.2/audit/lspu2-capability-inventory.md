# lspu2-capability-inventory

branchless-lsp capability inventory (LSPU-2, 2026-11-02). One live wire:
did_save -> run_tier2(content, Aarch64AppleDarwin) (lib.rs:186-190) ->
classify_seeded (rule_pack.rs:266) -> verdict_diagnostics ->
publish + Tier2State (lib.rs:191-201); served via max/conformanceVector
(lib.rs:204-216); consumed by client diagnostics_for_files
(branchless_client.py:374-426) and gate (gate-branchless-lsp.py:115).

## Capability table (courted / consumed / gap)

| # | capability | courted | consumed by | verdict |
|---|---|---|---|---|
| 1 | run_tier2 (rule_pack.rs:236) | yes | lib.rs:186; client :410; gate :115 | HEALTHY — the only fully-wired chain |
| 2 | verdict_diagnostics ANDON (rule_pack.rs:303,:333) | yes | lib.rs:191; gate :120-137 | HEALTHY |
| 3 | max/conformanceVector + Tier2State (lib.rs:89,:204) | yes | client :275-279 | HEALTHY |
| 4-5 | conformance_from_vector, sanitize_for_compile | yes | internal | HEALTHY |
| 6 | SessionConformance record/seal (conformance.rs:58/:85) | unit only | NONE — shutdown() (lib.rs:155-157) is Ok(()) | ORPHANED: 80-byte sealed format, no wire producer |
| 7 | Tier-1 lint_source (ast_linter.rs:43) | yes | NOT WIRED — did_open Tier-1 regex is never-matching \b\B (rule_pack.rs:50) | UNWIRED: ANDON_BRANCH_SYNTAX never fires live |
| 8 | did_open Tier-1 publish | implicit | always-empty publish; clients drain as no-op | DEAD WEIGHT + masks |
| 9 | quick_fixes + 3 rewrites (~600 lines, code_actions.rs) | yes | NO codeAction handler in lib.rs:145-217 | ORPHANED |
| 10 | hover_card (hover.rs:12) | — | no hover handler | ORPHANED |
| 11 | ReceiptWriter/.cgap/receipts (receipts.rs:185-248) | unit | NONE — lib.rs:17 docstring advertises persistence the server never does; only the gate writes (its own JSON) | ORPHANED |
| 12 | CompilerWorker::should_dispatch (compiler_worker.rs:76) | unit | none — did_save recomputes every save | ORPHANED dedup seam |
| 13 | compile non-debuginfo | yes | tests only | fine |
| 14 | parameterized allow_isel_residual(target,max_branches) | unit only | classification reachable live (:266) but did_save_court's ALLOWED fixture uses BARE attr (:41-45) | NOT EXERCISED END-TO-END — missing witness below |
| 15 | riscv32imc leg | yes (G05) | lib.rs:187 hardwires aarch64; test-only | UNWIRED by design, unadjudicated: RVC compressed encodings, freestanding cross-compile, 2nd-ISA mutation |
| 16 | gate chain | — | client :374; hook idempotent (install-gate-hook.sh:23-39) | HEALTHY, honest [SKIP] |
| 17 | client analyze() | — | wait_for_diagnostics (:263) returns FIRST publish = tier-1 empty | BUG: diagnostics field wrong for every caller; diagnostics_for_files' drain is a race, not a fix |

## Double-silence finding (worse than cosmetic)

rustc stderr is captured by CompilerWorker (compiler_worker.rs:160-166),
stowed in Tier2Outcome.note (rule_pack.rs:249-251), and verdict_diagnostics
(:303-330) NEVER emits out.note — a rustc-error save publishes an EMPTY
diagnostics array, indistinguishable from a clean save. Net: a rustc
failure is silently reported as "0 refusals" to client AND gate.
Falsifier: save a syntax-error source; the gate passes.

## Missing witness for #14 (exact)

(i) did_save live-wire test with `#[allow_isel_residual(target = "aarch64",
max_branches = N)]` on the BRANCHY body, N >= actual measured branch count,
asserting zero ERROR; (ii) negative arm max_branches = N-1 asserting ERROR
survives; (iii) target-mismatch arm target = "riscv32imc" under aarch64
court asserting refusal. Without these, target_matches (:77-93) has never
run against a real rustc object.

## Top-5 activation moves

1. Wire SessionConformance into did_save/shutdown. Falsifier: grep '\.seal()'
   in lib.rs returns nothing today.
2. Emit out.note as WARNING on compile-failure saves (kills the false-pass).
   Falsifier: gate passes a commit staging `fn f() -> u32 { 1 } }`.
3. Add the parameterized-residual live witness (close #14). Falsifier:
   grep 'target = "aarch64"' in did_save_court.rs returns nothing.
4. Fix client analyze() masking (drain tier-1 publish or key on code ==
   ANDON-REFUSED-001). Falsifier: analyze() on BRANCHY returns 0 ERRORs
   while diagnostics_for_files returns >=1.
5. Expose code_action + hover handlers (capabilities already court-tested,
   lib.rs:31,:34). Falsifier: textDocument/codeAction returns -32601 today.

## See Also

audit/lspu1-consume-plan.md · audit/lspu3-adoption-runbook.md
