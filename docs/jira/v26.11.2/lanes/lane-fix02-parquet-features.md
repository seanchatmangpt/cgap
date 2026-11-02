# Lane FIX-02 — cgap-parquet std-gating of FA9 court

Standing: ALIVE. Branch main. Date 2026-11-02.

## Defect

`cargo test -p cgap-parquet` (default features, `default = []` per
`crates/cgap-parquet/Cargo.toml:12`) exited 101. The FA9 rotation court
(items at former lines ~1801-1915 of `mmap_writer.rs`) referenced
`RingSink`, `SpillSync`, and `private::Sealed` — all `#[cfg(feature =
"std")]`-gated — but sat bare inside `mod courts` which is only
`#[cfg(test)]`. The sibling `tier_h` mod (line 1449) already used the
correct convention: `#[cfg(all(test, feature = "std"))]`.

Before: 18 compile errors (E0405/E0425/E0433) at mmap_writer.rs:1823,
:1844, :1902, :1908 (plus peers).

## Fix

Per-item `#[cfg(all(test, feature = "std"))]` gates added to 6 items in
`crates/cgap-parquet/src/mmap_writer.rs` (RecordingSpill struct + impl,
SharedRec type alias, Sealed impl, SpillSync impl, fa9 test fn). No
runtime behavior change; mirrors the tier_h convention. Zero-string and
source-scan gates untouched (comment-only additions, no string gate
surface).

## Commands + exits (this session)

```text
CARGO_TARGET_DIR=/tmp/fix02-target cargo test -p cgap-parquet
  -> EXIT 101 (before; 18 compile errors, mmap_writer.rs:1823/1844/1902/1908)
CARGO_TARGET_DIR=/tmp/fix02-target cargo test -p cgap-parquet
  -> EXIT 0   (after)
CARGO_TARGET_DIR=/tmp/fix02-target cargo test -p cgap-parquet --features std
  -> EXIT 0   (195 passed; fa9_rotation_drain_is_sync_durable_... ok)
CARGO_TARGET_DIR=/tmp/fix02-target cargo test -p cgap-parquet --no-default-features
  -> EXIT 0   (default = [], so identical surface to default)
```

## Fences not regressed

- `--features std` suite passes (195 lib tests incl. the FA9 court —
  the court still runs, now only under std).
- Default suite (no FA9 under no_std) exits 0; tier_h gating unchanged.

## Divergences from dispatch

1. Dispatch pointed at tests/ FA9 court file; the un-gated reference
   was actually in-module (`src/mmap_writer.rs` `mod courts`), so the
   fix landed there per the dispatch's "whichever carries the un-gated
   reference" clause.
2. `default = []`, so default and --no-default-features are the same
   surface; both exit 0.

Build root `/tmp/fix02-target` deleted at lane end.
