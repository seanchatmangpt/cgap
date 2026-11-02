# w5-stats-skip

Lane D8 receipt (v26.11.2).

Lane: D8 (Design). Repo: `/Users/sac/cgap`, branch `main`. Date: 2026-11-02.
Subject: whole-row-group pruning from fixed-width footer statistics in
`crates/cgap-parquet/src/scan.rs`.

## Diff summary

- `crates/cgap-parquet/src/scan.rs` (+297/-129):
  - New `Bounds { min, max }` (bit-pattern u64 pair) and `RowGroupEntry`
    extended with `bounds: Vec<Option<Bounds>>` (loses `Copy`, keeps
    `Clone`/`PartialEq`/`Eq`).
  - Footer parse: `ColumnMetaData.statistics` (parquet.thrift field 12),
    `Statistics.min_value`/`max_value` (fields 6/5) read as 8-byte PLAIN
    LE bit patterns; partial/wrong-width stats fail open to `None`
    (advisory bounds never change results, only cost).
  - `RowGroupEntry::rg_prunes(col, (lo, hi)) -> bool`: predicate range
    entirely outside `[min, max]` (bit-pattern ordering) prunes the whole
    row group; missing bounds never prune.
  - `ParquetFile::scan_u64_pred(col, (lo, hi))`: skips pruned row groups
    whole (no page header parsed), scans survivors value by value.
  - `ParquetFile::pages_parsed()` observable: counts Thrift page headers
    actually parsed; a pruned row group does not move it.
  - Writer side: `write_fixture_file_with_bounds(columns, num_rows,
    bounds)` emits the `Statistics` struct; `write_fixture_file` is the
    bounds-stripped variant (the court's mutation seam).
- `crates/cgap-parquet/tests/stats_skip_court.rs` (new): the court.

## Commands + exits (this session)

- `cargo test -p cgap-parquet --features std --test stats_skip_court`
  -> exit 0, `2 passed; 0 failed`.
- `cargo test -p cgap-parquet` (default features, full suite)
  -> exit 0, all `test result: ok` (180 passed / 1 ignored in lib tests
  plus 13 more ok sections; pre-existing warnings in `bitslice.rs:894`
  and `unpack_filter.rs:209,252` are not this lane's files).
- `cargo test -p cgap-parquet --features std` (full suite) -> exit 0,
  14x `test result: ok`, 0 failed.

## Anti-vacuity flip (sandbox, sha-exact restore)

- Mutation (in `/tmp/d8-sandbox/cgap-parquet/src/scan.rs` copy):
  `rg_prunes` `None => true` (prune even with no bounds).
- Court on mutated copy: `stripped_bounds_mutation_flips_prune` panics
  `MUTATION FLIP FAILED: prune fired without bounds` -> cargo exit 101.
- Restore: sandbox `scan.rs` reverts to sha256 `4a321b15e40bd6e3...`,
  byte-exact with the shared tree. Shared tree was never mutated.

## sha256 (landed files)

- `crates/cgap-parquet/src/scan.rs`:
  `4a321b15e40bd6e3f38189a42b1333c34a913eb9b10f5f646ae0cf8737b5856a`
- `crates/cgap-parquet/tests/stats_skip_court.rs`:
  `8d740f1e383e352f0a7645cd8c720f63ef3cc05a2228cc8adcf9dcbf29d34495`

## Standing

ALIVE for the pruning surface landed this session (commands + exits
above, run against the exact landed sha256s).

## Divergences

1. Dispatch asked for a /tmp mutation of a written file with sha256-exact
   restore; the implemented mutation seam is writer-side (bounds argument
   omitted -> no `Statistics` struct on the wire), plus the sandbox
   `rg_prunes` mutation above for the exit-101 witness. Both are real
   mutations of the subject; the restore law holds (sandbox restored
   sha-exact, shared tree untouched).
2. Dispatch named RowGroupEntry as scan.rs:302-305; the landed file has
   the struct at a shifted line after this lane's own edits, same fields
   plus `bounds`.
3. A concurrent lane (D7) left a placeholder edit inside
   `read_row_group` mid-flight (compile-freeze SLA window); this lane
   completed the region as part of its owned file. The D7 placeholder
   comment (`bounds: Vec::new()` fail-open note) was superseded by the
   real bounds parse; no code of D7's remains.
4. The crate's fixture writer models one ColumnChunk per row group (all
   columns share one page), so per-ROW-GROUP bounds attach to chunk 0;
   `rg_prunes` indexes bounds per column chunk and a multi-chunk footer
   would prune per column as specified.
