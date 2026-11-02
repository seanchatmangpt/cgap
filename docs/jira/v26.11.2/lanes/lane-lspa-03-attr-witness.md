# Lane LSPA-3 — parameterized-attr live-wire witness (inventory #14)

Repo: /Users/sac/lsp-max (branch main) + /Users/sac/cgap (docs). Date: 2026-11-02.

## Files written (ownership)

- `/Users/sac/lsp-max/crates/branchless-lsp/tests/did_save_court.rs` —
  ADDITIVE ONLY: +3 live-wire arms (LSPA-3 block, ~150 lines appended; the
  4 existing G06 courts are byte-untouched above the append point; sha256
  of the pre-append file preserved in the append delta).
- `/Users/sac/cgap/docs/jira/v26.11.2/lanes/lane-lspa-03-attr-witness.md` —
  this receipt.
- UNPLANNED, disclosed per compile-freeze SLA (both-directions law):
  `/Users/sac/lsp-max/crates/branchless-lsp/src/lib.rs` — ONE hunk in
  `shutdown` (LSPA-1 seal log line). The shared lib did NOT compile before
  my work (three errors in the LSPA-1 shutdown hunk: format-string arg
  count 6-vs-9, `[u8]: LowerHex` not implemented, `log_message` called
  with struct where (MessageType, impl Display) positional signature
  expected). Under the ~10-minute compile-freeze SLA I applied the minimal
  unblock: hex tag via byte map, positional `log_message`, and copying the
  session totals out of the MutexGuard before the await (guard-across-await
  made the future `!Send`). No semantic change intended; the seal values
  logged are the same numbers. LSPA-1 lane must re-own this hunk.

## Measured N (dispatch law: measure, do not guess)

Measured AT RUNTIME by the tests themselves, from the real court: each arm
first calls `rule_pack::run_tier2(BRANCHY, Aarch64AppleDarwin)` (real
`rustc --emit=obj -O -g` + capstone disasm) and reads
`functions[find(name == "g06_branchy")].branch_sites.len()`. Witnessed
value this session: **N = 2** branch sites for `g06_branchy` at -O on
aarch64-apple-darwin. No guessed budget anywhere.

## FINDING (primary result — the gate is unreachable on the wire)

Arm (i) FAILED, and the failure is the finding, not a flake: a VALID
parameterized allowance `#[allow_isel_residual(target = "aarch64",
max_branches = 2, reason = "lspa-3 measured budget")]` on the branchy body
is REFUSED on the did_save wire, even though the measured budget admits it
per `ResidualParams::classify_count`. Root cause, cited:

1. `extract_attr_paths` (rule_pack.rs:133-166) splits the attribute-payload
   inner text on `[',', ' ']` (:156), shredding
   `allow_isel_residual(target = "aarch64", max_branches = 2, reason = ...)` into
   fragments `allow_isel_residual(target`, `=`, `"aarch64"`, ... The token
   `allow_isel_residual(target` fails `strip_suffix(')')` in parse_attrs
   (attributes.rs:203-207) -> MissingCloseParen.
2. `fn_attr_flags` (rule_pack.rs:125) does `parse_attrs(...).unwrap_or_default()`
   (rule_pack.rs:125) — the typed parse error is SILENTLY swallowed into
   DEFAULT (bare=false, no residual_params) attrs.
3. Classification then falls to `classify_count(attrs, branches)` with
   bare=false -> `ErrorRefusedNoAttr` -> ERROR diagnostic, regardless of
   target or budget. `target_matches` (attributes.rs:77-93) and
   `ResidualParams::classify_count` have NEVER executed on the wire: the
   wire extraction is structurally incapable of delivering a parsed
   ResidualParams to classify_seeded_count.

Consequences witnessed on the wire this session (exact messages):
- Arm (i) diagnostics: `ANDON_REFUSED: fn `g06_branchy` carries a forward
  conditional branch; ...` x2 (duplicated) + `A blocking diagnostic was
  emitted without a corresponding ANDON push event.`
- Arms (ii)/(iii) PASS, but vacuously: their expected refusal currently
  arrives via the parse-drop path, not via the budget/target semantics
  they name. They become non-vacuous the moment rule_pack extraction is
  fixed, and arm (i) going green is the flip witness.

Dispatch asked: "if the mismatch silently passes, that is a FINDING". The
observed defect is the dual: the ADMIT path silently fails. Recorded, not
hidden.

## SECOND FINDING (LSPA-1 hang, witnessed, not mine to fix)

`session_totals_observable_before_shutdown` (crates/branchless-lsp/tests/
conformance_seal_court.rs) HANGS: single-test bisect with a 30s wall — 3
sibling seal tests finish in <=0.09s, this one never returns (and a bare
`cargo test -p branchless-lsp` therefore never completes; three lanes'
full-suite runs were all parked on it simultaneously at 0% CPU). Not a
compile break, so outside my SLA authority; reported here for the owning
lane. Bounded-witness commands:
- conformance_seal_court binary `--list` -> 4 tests.
- per-test 30s bisection: 3 ok, `session_totals_observable_before_shutdown`
  HANG(30s) — cited, not guessed.

## Commands + exits (this session, exact subject)

- `CARGO_TARGET_DIR=/tmp/lspa3-target cargo test -p branchless-lsp --test did_save_court`
  -> exit 1: 6 passed / 1 FAILED — the FAILED is
  `lspa3_measured_budget_admits_branchy_on_wire` (the FINDING above);
  4 pre-existing G06 courts all green.
  sha256(did_save_court.rs after append) =
  26382b1f8f0cdb6316bfa20fcb3fdac5d33ba4fbf53c0ba63272c9841dee5596
- Full census: `CARGO_TARGET_DIR=/tmp/lspa3-target cargo test -p branchless-lsp --no-fail-fast -- --skip session_totals_observable_before_shutdown`
  -> exit 101 (the only failure is my arm-(i) FINDING court). Per-target:
  lib 72/0, main 0/0, code_actions 5/0, compile_note 5/0,
  conformance_seal 3/0 (1 filtered = the hanging LSPA-1 test, loud skip),
  did_save 6/1 (FAILED = lspa3_measured_budget_admits_branchy_on_wire,
  the FINDING), e9_precision 6/0, riscv32imc 3/0. Grand total 100 passed /
  1 failed, 1 loud skip. The unskipped bare `cargo test -p branchless-lsp`
  cannot complete (hangs, see SECOND FINDING).
- Lib compile fix verified by the same runs (lib compiled clean after the
  SLA hunk; prior state was 3 hard compile errors, exit 101).

## Standing

- Arm (ii)/(iii) + all 4 pre-existing courts: ALIVE (ran this session).
- Arm (i) and the parameterized gate itself: the court is RED with a
  verified root-cause finding in rule_pack.rs:133-166/:125. Standing for
  "parameterized allowance honored on the wire": REFUTED-by-witness (the
  court flip is real; it cannot pass against the current wire path).
- Full-suite census: ALIVE (no-fail-fast run above, exit 101 with the
  single documented arm-(i) failure + 1 loud skip).
- Standing vocabulary: PARTIAL — court written, arms (ii)/(iii) + prior
  courts green, arm (i) red with root-caused production defect handed to
  the owning lane.

## Fences (must-not-regress) and their runs

- Fence 1: the 4 existing G06 courts — RAN, green (6/7 pass includes
  them).
- Fence 2: full `cargo test -p branchless-lsp` — RAN (in flight; errata
  will pin the exact totals).
- Fence 3: lsp-max root crate / cgap source: read-only (cgap touch is this
  receipt only). No git commands anywhere.

## Divergences from dispatch

1. Dispatch said arm (iii) records what actually happens for target
   mismatch; code-read predicted typed refusal (fallback to bare=false ->
   ErrorRefusedNoAttr). Wire confirms refusal, but via the parse-drop
   defect, so (iii) is currently vacuous until rule_pack is fixed.
2. Dispatch expected arm (i) zero-ERROR -> NOT achieved: production
   defect, documented as the FINDING. Court left red intentionally (a
   court that cannot fail is decoration).
3. Unplanned lib.rs SLA unblock (disclosed above).
