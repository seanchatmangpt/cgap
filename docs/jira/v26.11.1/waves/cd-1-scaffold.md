# CD-1 Scaffold Receipt: cgap-datafusion membrane

Lane: CD-1 (lane-runner), 2026-11-01, repo /Users/sac/cgap (branch main).

## Files written (lease: crates/cgap-datafusion/**)

- `crates/cgap-datafusion/Cargo.toml` (NEW; + concurrent CD-3 edit disclosed below)
- `crates/cgap-datafusion/src/lib.rs` (NEW: CgapTableProvider spike + courts)
- `crates/cgap-datafusion/README.md` (stub)
- `Cargo.toml` (root, ONE line + comment: workspace member added)
- `docs/jira/v26.11.1/waves/cd-1-scaffold.md` (this receipt)

## Dependency graph note (HEAVY)

Pin: `datafusion = "55.2.0"` (chosen via `cargo search datafusion`,
latest stable at scaffold; exit 0). Resolved tree in root `Cargo.lock`:
datafusion 55.2.0, arrow-rs 59.3.2 (arrow-array/-schema/-select), parquet
59.3.2, async-trait 0.1.92, tokio 1.54.1, object_store 0.13.2, url 2.5.8,
chrono 0.4.45 — 31 datafusion* crates total (`grep -c 'name = "datafusion'`
= 31). This is the heaviest member by design: the only workspace crate
with an external dep tree of this size; kernels stay zero-dep no_std.
cgap-parquet is consumed with `features = ["std"]` — its `scan` module
(the hosted reader) is std-gated; no_std builds are unaffected.

## Conversion boundary

`CgapTableProvider::from_parquet_file` (src/lib.rs): real on-disk file →
`cgap_parquet::scan::ParquetFile::open` (mmap + footer index) → per-column
typed accessors (`value_u32`/`value_u64`) →
`UInt32Array::from_iter_values` / `UInt64Array::from_iter_values` — the
exact point cgap bytes become arrow-rs buffers → one `RecordBatch` →
`MemorySourceConfig::try_new_exec` behind `TableProvider::scan`.

## Courts (Chicago; all ran this session)

1. `cgap_rows_reach_arrow_values` — 7 seeded rows through the real mmap/
   footer/typed-accessor path, final-state value asserts. PASS.
2. `dataframe_count_matches_seed` — `SELECT count(*) FROM t` through the
   real DataFusion DataFrame API over CgapTableProvider; count equals the
   9 seeded rows. PASS.
3. `mutation_fixture_row_count_flips_observed_count` — anti-vacuity:
   fixture mutated to 5 rows flips the observed count (5 != 9). PASS.
4. `corrupted_bytes_refuse_loudly` — truncated fixture (tail destroyed)
   is refused by decode, not silently decoded. PASS.

`cargo test -p cgap-datafusion`: 4 passed; 0 failed (exit 0). Count(*)
columns in DF 55 are Int64, not UInt64 — fixed mid-court, re-run green.

## Commands + exits

- `CARGO_TARGET_DIR=/tmp/cd1-target cargo test -p cgap-datafusion` — exit 0
  (4 passed; 0 failed; 0 ignored).
- `cargo metadata` parse of root manifest after member add — 23 members.
- Target dir deleted post-run (shutil.rmtree via python3).

## Divergences (disclosed)

- Root `Cargo.toml` edit: coordinator-owned file; minimal one-member add
  with comment, per lease disclosure. This is the disclosed exception.
- Concurrent writer in the lease dir (NOT this lane's files, left
  untouched, not mod-declared): `crates/cgap-datafusion/src/listing.rs`
  and `src/predicate.rs` (CD-3 surface) appeared mid-session; CD-3 also
  added `parking_lot = "0.12"` to the crate manifest with a CD-3 comment.
  My court run passed with those files present but undeclared.
- cgap-parquet `scan` is std-gated (found at first compile; E0432), so
  the dep carries `features = ["std"]` — the no_std kernel build is
  unchanged.
