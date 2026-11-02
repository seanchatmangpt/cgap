# Branchless reuse lane BR4 — cgap-parquet thrift/varint

Standing: ALIVE (read-only survey; evidence commands in receipts below, run this session).
Verdict up front: **no branchless substitution is warranted** on this surface. Both varint
decoders are already tight, bounded loops whose only data-dependent branch is the
continuation bit — and they sit on **cold paths**. Detail per target:

## T1. Varint decode loop (`read_varint`)

- `crates/cgap-parquet/src/thrift.rs:81-97` (`read_varint`, `ThriftError`), twin at
  `crates/cgap-parquet/src/rle.rs:40-57` (`ParquetError`), intentionally duplicated — the
  module doc says "same discipline as `crate::rle`".
- Current logic: byte-at-a-time `src.get(*pos)`, accumulate `value |= (b & 0x7F) << shift`,
  exit on `b & 0x80 == 0`; shift >= 64 → typed EOF error (overlong refusal).
- SWAR assessment (honest): a branchless form exists — load 8 bytes
  (`u64::from_le_bytes`), `leading_ones` counts continuation bytes N, mask + shift chain via
  `((w & 0x7F...) * mul)` or precomputed per-N masks, then
  `pos += N + 1`. For ≤8-byte varints it is branch-free; ≥9-byte varints still need the
  loop fallback (`u64::MAX` at 10 bytes is a real wire value here — footer.rs:1348 writes
  it in a court fixture). Cost: the per-byte `get` + branch predicts well (varints in
  headers are 1-2 bytes); a SWAR path adds an 8-byte bounds concern at end-of-slice
  (headers sit directly against page bodies — over-read must be masked out, not fault).
  Gain class: **marginal** — this is a cold-parse surface.
- Hot-path check (the decisive question): **varints are header/metadata-only.** Value
  decoding on the scan hot path is PLAIN fixed-width direct aliasing —
  `crates/cgap-parquet/src/scan.rs:26-28` ("PLAIN payload bytes ARE the Arrow buffer
  bytes: the scan aliases the mapping, no copy"); the accessor hot path is slice
  arithmetic. Page headers are parsed **at most once per (row group, column)** and
  memoized (scan.rs:9-10, parse at scan.rs:772). Footer `FileMetaData` is parsed exactly
  once at open (scan.rs:6-7). Nothing per-value ever touches `read_varint`.
- Court mutant: corrupt a page-header varint (set byte 0x80 continuation on the last
  byte of `uncompressed_page_size`) in a /tmp sandbox copy → must flip to
  `ScanError::Thrift(UnexpectedEof)`/`MissingRequiredField` typed refusal; restore
  sha256-exact + mtime bump. Not run this session (read-only lane): UNVERIFIED as an
  executed court, design stated.

## T2. Field-ID delta decode (`read_field_header`)

- `crates/cgap-parquet/src/thrift.rs:199-223`. Delta form: `fid = prev_id + (b >> 4)`,
  `i16::try_from` overflow → `ThriftError::InvalidFieldId` (thrift.rs:214-218); delta 0 →
  non-delta form: zigzag-i16 varint (thrift.rs:220).
- Hot? No — same cold-parse classification as T1: field headers appear only in page
  headers (per column chunk, memoized) and footer metadata (per open). One byte,
  one add, one branch — already minimal. A branchless variant would save nanoseconds on
  a path executed a handful of times per column. Gain class: **marginal, skip.**
- Court mutant (design): flip a delta nibble in a sandbox page header so the id gap
  no longer matches the writer's layout → field lands in the `(_, other) => skip_value`
  arm (thrift.rs:480) and required-field assembly fails with `MissingRequiredField`
  typed refusal; restore sha256-exact + mtime bump.

## T3. Const dispatch tables vs `match`

- Sites: `(field_id, type_id)` match ladders at thrift.rs:475-481 (`read_page_header`),
  thrift.rs:522-532 (`read_data_page_header_v2`), and `skip_value`'s type match
  (thrift.rs:270+). Dense small matches over integers — rustc already lowers these to
  jump tables / branchless selects; a `const` lookup table would add an extra load and
  cannot beat the codegen. **[ANALYTIC]** — no measurement run; substitution expected
  net-neutral-to-negative on a cold path.

## Receipt

- Commands (this session): `grep -rn "varint" crates/cgap-parquet/src/` (exit 0);
  `grep -n "read_varint|read_page_header" ...` (exit 0); Read of thrift.rs:78-263,
  380-549, 551-599, rle.rs:30-72, scan.rs:1-60. No build/test run — read-only lane;
  no target-dir lease needed.
- Divergence from dispatch: none material. Dispatch said "no compact.rs/varint.rs" —
  confirmed; varint lives in `thrift.rs:81` and `rle.rs:40`, plus a byte-exact
  writer-mirror in `footer.rs:133` (`fn varint`).
