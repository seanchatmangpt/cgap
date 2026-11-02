# Lane G04 — lsp-max branchless-lsp attributes (verify-and-gap-fill)

Repo: `/Users/sac/lsp-max` (branch main), branch `main`. Date: 2026-11-02.

## Files written

- `/Users/sac/lsp-max/crates/branchless-lsp/src/attributes.rs` — +4 tests
  (G04 block), zero production-code edits.
- `/Users/sac/cgap/docs/jira/v26.11.2/lanes/lane-g04-lspr-attributes.md` —
  this receipt (cgap docs tree per dispatch).

## Inventory confirmed (pre-existing, cited)

- Key-value form `#[allow_isel_residual(target="...", max_branches=N,
  reason="...")]` parses: `parse_attrs` (attributes.rs:196) →
  `parse_residual_params` (:124), typed `UnknownParamKey`/`DuplicateParam`
  /`MalformedParam`/`MissingCloseParen`; witness tests
  `parameterized_attr_parses_full_payload`,
  `unmeasured_parameterized_form_has_no_budget`.
- Refusal tests already covering the mutants: `unknown_param_key_is_
  typed_refused`, `malformed_and_duplicate_params_are_typed_refused`
  (duplicate `target`, non-numeric `max_branches = "three"`, unclosed
  paren). Zero `unwrap`/`expect`/`panic` in the parse path
  (attributes.rs:123-237, awk-swept: 0 hits). The only `expect` near the
  parser is `ResidualGrid::canonical()` on the compile-time embedded
  `include_str!` — not the parse path.

## Gap-fill tests added (attributes.rs, tests mod)

1. `key_value_form_round_trips_through_reason_note` — parse key-value
   payload, assert fields, assert `reason_note()` reproduces every
   literal, then RE-PARSE the note and assert `residual_params` equal.
2. `positional_form_is_fail_closed_typed_refused` — there is NO
   positional form in the grammar (bare or `key = value` only); both
   positional payloads (`"riscv32", 3` and bare `riscv32, 3`) are
   typed-refused `MalformedParam`. Fails closed, no panic.
3. `mutant_non_numeric_max_branches_flips_happy_path` — Ok(3) parses;
   `3x` flips to `MalformedParam` naming `3x`.
4. `mutant_duplicate_key_flips_happy_path` — otherwise-valid payload
   with duplicated `target` flips to `DuplicateParam` naming `target`.

## Anti-vacuity (mutant flips, cited)

The two G04 mutant tests carry their own flips: each shows the
happy-path literal parsing Ok AND the mutated literal flipping to a
typed Err in the same test. The pre-existing
`malformed_and_duplicate_params_are_typed_refused` (:686) is the
independent witness. A court that cannot flip is FAIL; these flip.

## Commands + exits (this session)

- `cargo test -p branchless-lsp` (first run): compile error E0382 in my
  new round-trip test (partial move) — fixed with `.as_ref()`.
- `cargo test -p branchless-lsp` (final): exit 0 — 72 lib, 0/0/0/6/5/3
  across the other targets, 0 FAILED (incl. `disasm_court` flake-witness
  run: passed in-suite and solo).

## Compile-freeze SLA disclosure (both-directions law)

While G04 ran, another lane's in-flight edit to
`crates/branchless-lsp/src/conformance.rs` made `mod tests` private while
`crates/branchless-lsp/src/receipts.rs` calls
`crate::conformance::tests::alloc_watch_*` (E0603, shared compile
frozen). G04 applied the minimal unblock: `mod tests` →
`pub(crate) mod tests` (conformance.rs:133-134). **The owning lane must
disclose this in its receipt and supersedes the diff on landing.** A
transient "multiple global allocators" E0527 abort during the same
window was the same lane's mid-edit state; it cleared without G04 action.

## Standing

ALIVE for the verify-and-gap-fill scope: key-value form parse, round-trip,
fail-closed positional refusal, mutant flips — all ran this session, exit 0
against the exact subject `/Users/sac/lsp-max` branch main. No perf claims.

## Divergences

- Dispatch named `docs/jira/v26.11.2/lanes/` under cgap; the directory
  did not exist — created it. lsp-max has no `docs/jira/v26.11.2/`
  tree at all; per dispatch the receipt lands in cgap.
- "Round-trip for positional AND key-value forms": no positional form
  exists in the grammar; recorded as a fail-closed pinned behavior
  (test 2) rather than a feature to add — grammar is bare or key=value.
