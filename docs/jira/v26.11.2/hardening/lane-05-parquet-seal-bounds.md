# Lane H5 — parquet seal bounds (rows >= 2^63 seal/verify asymmetry)

Date: 2026-11-02 · Repo: /Users/sac/cgap (main) · Standing: **ALIVE**

## Defect

`FooterSealer::new` had no upper bound on `rows`; `build_meta_block`
emits §6.2 field 3 as a zigzag i64 (`self.rows as i64`), so
`rows >= 2^63` cast-wraps negative on the wire while `verify_seal`
refuses `rows < 0` (`MetaMalformed`) — writable-but-unverifiable
blocks (W6-F1/W7-F1 asymmetry class at the row-count bound).

## Diff

- `crates/cgap-parquet/src/footer.rs` — typed refusal in
  `FooterSealer::new` when `summary.rows > i64::MAX as u64`:
  `ParquetError::RunOverrun { run_len: rows, budget: i64::MAX as usize }`.
  Doc comment updated. Fixture constant
  `0xABCD_1234_5678_9A00` (top bit set — violates the new bound)
  retuned to `0x5BCD_1234_5678_9A00` in the P5 byte-exact-oracle
  helper + oracle calls (3 sites), top bit clear; flip arms are
  span-xor / rows+1, unaffected.
- `crates/cgap-parquet/tests/seal_rows_bound_court.rs` — new court
  (3 tests, below).

## Court grid (tests/seal_rows_bound_court.rs)

| arm | rows | assertion | result |
|---|---|---|---|
| seal_refuses_rows_above_i64_max | 2^63 | typed RunOverrun, budget == i64::MAX, run_len > budget, never Ok | ok |
| boundary_rows_i64_max_still_seals_and_verifies | i64::MAX | seal Ok + verify_seal Ok + rows round-trips exactly (tight bound, not off-by-one) | ok |
| wrapped_negative_wire_rows_are_unverifiable | 2^63 | refusal re-asserted (asymmetry documentation arm) | ok |

Boundary feasibility: `rows` is a summary field, not a physical row
count; the body is a real 4-byte collaborator, so the i64::MAX
boundary seals in-process (no scale-BLOCKED arm needed).

## Anti-vacuity mutation (SHIM-COURT, /tmp sandbox)

- Subject: /tmp/h5-sandbox (byte-copy of cgap-parquet; Cargo.toml
  de-workspaced, cgap-arrow path dep absolute).
- Mutation: `if false && summary.rows > i64::MAX` (bound disabled).
- BEFORE (unmutated): 3 passed, exit 0.
- SHIM: 1 passed, 2 FAILED — the >2^63 arm SEALS Ok without the
  bound (flip witnessed, exit 1). Court is not vacuous.
- RESTORE: footer.rs re-copied from shared tree; 3 passed, exit 0.
  Shared tree never mutated (mutation ran in the sandbox copy);
  mtime bump applied to the shared footer.rs per the mtime-bump law.

## Verification runs (this session)

- `cargo test -p cgap-parquet` (default/no_std): all targets
  `test result: ok` — 182 lib passed incl. FA7-class arms
  `w7_f1_span_wire_bound_refused_with_exact_payload`,
  `max_row_groups_over_capacity_refused_with_exact_payload`,
  `meta_fixed_path_is_byte_exact_vs_vec_oracle` (byte-diff),
  `zero_body_seal_refused...`; exit 0. 0 panics.
- `cargo test -p cgap-parquet --features std`: 194 lib passed, all
  targets ok, exit 0 (both feature configs).
- Alloc witness: `rle::alloc_proof::decode_into_zero_allocations` ok,
  `tests/page_zero_alloc.rs` ok — 0/0 alloc/panic witness intact.

## Divergences / notes

- The dispatch's "FA7 7-arm battery" does not exist as a single named
  test; the equivalent court set is the four footer courts above (65 RG
  and 10,000 RG arms map to
  `max_row_groups_over_capacity_refused_with_exact_payload` at
  MAX_ROW_GROUPS). All green; recorded here per doc-yields law.
- Mid-run the `--features std` config compile-aborted on
  `crates/cgap-parquet/src/mmap_writer.rs` (`Box`/`format` scope
  errors) — another lane's in-flight edit, not mine; it self-healed
  within the session (194 passed on rerun). No unblock fix applied by
  this lane.
- Build roots /tmp/h5-target, /tmp/h5-target-mut deleted at lane end.
