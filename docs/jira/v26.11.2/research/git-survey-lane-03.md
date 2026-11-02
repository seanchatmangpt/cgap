# Git Survey Lane 03: Tree Sorting, Storage Seam, Packfiles

Lane: GS3. Read-only survey of `crates/cgap-git` at the v26.11.2 tree.
Method: grep + targeted reads only; no builds, no mutations.

## 1. Tree-entry sorting

NOT-IMPLEMENTED. `cgap-git` has no git tree objects and no tree-entry
comparator at all. The pack writer serializes blobs only (`write_pack_v2`
takes `&[&[u8]]`, `pack_write.rs:238`; `PackBuilder::push(&[u8])`,
`pack_write.rs:354`). There is no `100644`/`040000` mode parsing, no entry
name handling, and no directory-sorts-as-trailing-`/` comparator anywhere
in the crate (grep for `mode|100644|040000|trailing|b'/'` in `src/`:
zero relevant hits).

The only sorts present are OID sorts, not tree sorts:
- `pack_write.rs:269` — `entries.sort_by_key(|e| e.oid)` in `write_idx_v2`
  (idx fan-out order, dynamic sort).
- `pack_write.rs:620` — same, in a test.
- `idx_read.rs:308,411,422` — `sort_unstable` in test helpers.

`dag.rs`'s `DagTree` (`dag.rs:325`) is a BLAKE3 merkle DAG for the
`.cgap-dag` file, not a git tree; its ordering is a fixed postorder wire
layout (`dag.rs:270`), not name-based sorting.

cgap-branchless / eytzinger consumption: NONE. No dependency on
`cgap-branchless` and no `eytzinger` reference in
`crates/cgap-git/Cargo.toml` or any `src/*.rs` (grep exit 1). The only
"branchless" mentions are comments (`idx_read.rs:175`, `ladder.rs:2`).

## 2. Storage / journal seam

NOT CONNECTED. `cgap-git` does not reference `RingSink`, `mmap_writer`,
or `BlockSink` anywhere (grep zero hits in `crates/cgap-git/`). The
coupling runs the other direction by identity claim only:
`cgap-storage/src/atomic_commit.rs:21,82-83,382-383` re-implements
IEEE CRC-32 (poly 0xEDB88320) and cites `cgap_git::pack_write::crc32`
as the identity source; `cgap-storage/Cargo.toml:21` declares the
`cgap-git` path dependency. `cgap-parquet` does not depend on cgap-git.

Loose-object write path: IN-MEMORY. `PackBuilder` holds
`alloc::vec::Vec<alloc::vec::Vec<u8>>` (`pack_write.rs:344-345`);
`finish()` (`pack_write.rs:367`) calls `build_pack_and_idx`
(`pack_write.rs:310`) which returns `(Vec<u8>, Vec<u8>, _)` — pack,
idx, entries — all heap `Vec`s. No disk path, no scratchpad, no ring
journal. The crate is `#![no_std]` + `extern crate alloc`
(`lib.rs:13-16`); there is no `std::fs` in `src/` at all. Real disk
contact exists only in tests (`tests/git_interop.rs` writes packs into
a temp `.git/objects/pack` via `std::fs`).

## 3. Packfile mechanics verdict table

| Symbol | Verdict | Evidence |
|---|---|---|
| `parse_pack_v2` | EXISTS (tested) | `pack_write.rs:419`; `pack.rs:67`; `lineage_workflow.rs:115` |
| `build_pack_and_idx` | EXISTS (tested) | `pack_write.rs:310`; `lineage_workflow.rs:113` |
| `IdxV2::parse` | EXISTS (tested) | `idx_read.rs:25`; git idx in `git_interop.rs` dir A |
| `IdxV2::verify_idx_checksum` | EXISTS (tested) | `idx_read.rs:196`; `idx_read.rs:340` |
| `write_idx_v2` / `write_pack_v2` | EXISTS (tested) | `pack_write.rs:262,238`; `git fsck` dir B |
| OFS delta (type 6) resolution | NOT-IMPLEMENTED | no delta symbols; parser
  (`pack_write.rs:390-392`) rejects only 0/5, so a type-6 body is misread
  as a zlib blob body, not refused as delta |
| REF delta (type 7) resolution | NOT-IMPLEMENTED | same evidence |
| `apply_delta` | NOT-IMPLEMENTED | symbol absent crate-wide |

Design is documented as delta-free: `lib.rs:6-7` ("no deltas, no new
type codes"). `encode_object_header` (`pack_write.rs:206-207`) accepts
type codes 6/7 syntactically but the writer never emits them; the
parser does not reject them either — a latent gap, not a feature.

Dead code: none of the EXISTS symbols are dead. The `ladder.rs` /
`dag.rs` modules are exercised by `ladder_*` and `fsck_court` tests
(court files present in `tests/`); `lineage_workflow.rs` (259 lines)
exercises the writer→parser→idx chain end to end and asserts typed
refusals on corruption (`lineage_workflow.rs:171`).

## 4. CRC32 / zlib

Both self-contained, zero external deps:

- CRC32: IEEE reflected, poly 0xEDB88320 — `const fn build_crc_table`
  `pack_write.rs:144`; `Crc32` streaming struct `pack_write.rs:172-193`
  (`new`/`update`/`finalize`); one-shot `pub fn crc32`
  `pack_write.rs:195`. Known vectors tested `pack_write.rs:571-575`
  (`crc32(b"123456789") == 0xCBF4_3926`).
- zlib compress: `deflate.rs` — `deflate_fixed` `deflate.rs:254`
  (single fixed-Huffman block over LZ77 tokens, `lz77_tokens`
  `deflate.rs:192`); `zlib_compress` `deflate.rs:287` (0x78 0x9C
  header + adler32 trailer).
- zlib decompress: two independent implementations —
  `deflate.rs:538` `zlib_decompress` (own inflate `deflate.rs:415`)
  and `inflate.rs:426` `zlib_decode` over `inflate` `inflate.rs:260`
  with `adler32` `inflate.rs:407`. Cross-validated against real
  python3 zlib in both files' tests (`deflate.rs:580`,
  `inflate.rs:463`) and a real git fixture (`inflate.rs:585`).

## Commands

| Command | Exit |
|---|---|
| `ls crates/cgap-git/src tests` | 0 |
| `grep -rn "sort_by\|sort_unstable" crates/cgap-git/src/` | 0 (hits above) |
| `grep -rn "branchless\|eytzinger" crates/cgap-git/` | 0 (comments only) |
| `grep -rn "RingSink\|mmap_writer" crates/cgap-git/` | 1 (no hits) |
| `grep -rn "delta\|apply_delta" src/pack.rs inflate.rs pack_write.rs` | 1 (no hits) |
| `grep -rn "cgap_git" crates/cgap-storage/` | 0 (identity-claim hits) |
| `grep -n ... tests/git_interop.rs tests/lineage_workflow.rs` | 0 |

Standing: findings grounded by the greps above, run this session.
No builds run (lane contract). Tree-entry canonical order and delta
resolution are NOT-IMPLEMENTED, not merely untested.
