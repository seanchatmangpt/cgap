# Repair-15: nanoarrow C1 — minimal validation tier (lane NA1)

Date: 2026-11-02. Lane NA1, v26.11.2 starDF campaign.

## Scope

Adopt nanoarrow's validation-level ladder semantics (first tier) into
`crates/cgap-arrow/src/validate.rs`, per
`docs/jira/v26.11.2/research/nanoarrow-oracle-synthesis.md` C1 (tag
`[TARGET-v26.11.2]`) and nanoarrow
`ArrowArrayViewValidateMinimal` (vendor/nanoarrow/src/nanoarrow/common/array.c:1595-1676).

## Ladder tiers

- `ValidationLevel::None` / `ValidationLevel::Minimal` enum mirrors the
  ladder; DEFAULT/FULL nested walks stay ABSENT (cgap fixed-width only,
  N7 — recorded divergence, not a gap).
- `validate_minimal(&ArrayView) -> Result<MinStats, MinimalError>`:
  length >= 0, offset >= 0, offset+length <= i64::MAX (u64 math,
  no overflow panic), data buffer need = (offset+length)*elem_size,
  validity need = bytes_for_bits(offset+length); validity tier skipped
  when null_count <= 0 and buffer absent/empty (array.c:1632-1636).
- Observable skip: `MinStats { validity_checks, validity_skipped,
  offset_plus_length }` — arithmetic counters, not mocks.
- The pre-existing A2P 5-court module
  (`validate_fixed_width` + `ValidateError`) is untouched; its 5
  unit tests stay green in the 136-test lib suite.

## Files

- crates/cgap-arrow/src/validate.rs (extended; existing API unchanged)
- crates/cgap-arrow/tests/validate_minimal.rs (new, 4 courts)
- this receipt

## Court grid

| Court | Law | Result |
|---|---|---|
| court1_minimal_passes_wellformed | well-formed views pass; offset window; bytes_for_bits rounding | ok |
| court2_undersized_buffers_refused | typed BufferUndersized {Validity, Data} refusals | ok |
| court3_validity_skipped_when_null_count_le_zero | validity_checks==0 iff null_count<=0 && buffer empty/absent; ==1 when null_count>0 | ok |
| court4_bounds_refused | NegativeLength / NegativeOffset / OffsetLengthOverflow typed | ok |
| A2P legacy 5 (lib tests) | LengthOverflow / Misaligned / ColumnCount / display | ok |
| sandbox baseline (7 tests) | real cargo build of validate.rs in /tmp court crate | ok exit 0 |
| sandbox MUTANT (data size check removed) | undersized court MUST fail | FAILED (flipped, exit 1) |
| sandbox restore | sha256sum -c pre.sha | OK, mtime bumped |

## Anti-vacuity (mutation court)

Sandbox: /tmp/na1-sandbox (copy of crates/cgap-arrow + a zero-dep court
crate at /tmp/na1-sandbox/court including validate.rs via #[path]).
Baseline: cargo test exit 0, 7 passed. Mutation: removed the
`need > v.data_bytes` size check from the /tmp sandbox copy ->
`courts::undersized_data_refused` FAILED (6 passed / 1 failed). Shared
tree was never mutated. Restore: cp from canonical bytes,
`sha256sum -c pre.sha` OK, mtime bumped.

## Verification receipt

- CARGO_TARGET_DIR=/tmp/na1-target cargo test -p cgap-arrow -> exit 0,
  190 passed / 0 failed across 9 suites (136 lib incl. the 5 legacy
  validate courts; 4 new minimal-tier courts; others unchanged).
- CARGO_TARGET_DIR=/tmp/na1-target cargo test -p cgap-arrow
  --test validate_minimal -> exit 0, 4 passed.
- Mutation baseline/mutant runs: exit 0 (baseline) / exit 1 (mutant).

## C1 TARGET tag flip

`[TARGET-v26.11.2]` C1 -> DONE (this receipt): minimal tier landed with
mutation-court flip witness; falsifier per C1 (FULL-vs-NONE geometry)
applies to the DEFAULT/FULL tiers, which remain ABSENT by N7 — recorded
divergence: the landed falsifier is the undersized-buffer mutant flip
above.
