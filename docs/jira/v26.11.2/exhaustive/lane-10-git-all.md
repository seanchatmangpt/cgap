# Lane EX10 — cgap-git exhaustive census (2026-11-02)

Standing: ALIVE (read-only census; every claim below is a this-session
`grep`/`sed` run on /Users/sac/cgap/crates/cgap-git). No builds per lane doc.
File inventory (wc -l this session):

| file | lines | status |
|---|---|---|
| lib.rs | 143 | present |
| pack.rs | 130 | present (test-only facade court) |
| pack_write.rs | 746 | present |
| dag.rs | 1919 | present |
| deflate.rs | 792 | present |
| inflate.rs | 719 | present |
| idx_read.rs | 521 | present |
| ladder.rs | 287 | present |
| loose.rs / tree.rs / oid.rs / index.rs | — | ABSENT (marked, not scaffolded) |

## GITFIX reality check (verified this session)

- Type-6/7 typed refusal LANDED: `pack_write.rs:393-395` — `read_varint_header`
  returns `GitError::UnsupportedDeltaEncoding` for obj_type 6/7 (and BadVarint
  for 0/5). Court `tests/delta_refusal_court.rs` (109 lines) builds real packs
  via `encode_object_header`+`zlib_compress`+`sha1`, writes to real tempdir,
  anti-vacuity witness `w6-git-typed-refusals.md` (exists in waves/).
- `ladder.rs` cfg narrowing LANDED: exactly 5 production cfg sites
  (lines 28, 39, 43, 53, 142), all the narrowed
  `all(debug_assertions, feature = "std")` form; 6th site (line 262) is
  test-gated. Probe counter compiled out in release.
- riscv32imc no_std build: UNVERIFIED this session (no builds lane law);
  prior receipt `w6-rv32imc-branchless-repair.md` on disk.

## Function census (public surface; internal helpers indented)

**lib.rs** (143): `GitError` (Display fmt @55) — 2 inline tests. KEEP.

**pack.rs** (130): facade court, 1 test fn @47. KEEP (court, no prod surface).

**pack_write.rs** (746): sha1 write path — `Sha1::{new,update,finalize,compress}`
(27-124), `sha1` @124, `git_object_oid` @131, `Crc32::{new,update,finalize}`
(173-195), `crc32` @195, `encode_object_header` @206, `write_pack_v2` @238,
`write_idx_v2` @262, `build_pack_and_idx` @310, `PackBuilder::{new,push,len,
is_empty,finish}` (349-367), `read_varint_header` @386, `parse_pack_v2` @414,
`parse_idx_v2` @471. 12 inline tests (sha1/crc vectors, pack/idx roundtrip,
checksum-corruption flips, object-count cap, no_std hygiene @739). KEEP all.

**dag.rs** (1919): `DagNodePayload` @251, `DagTreeNode` @274, `DagHeader` @285,
`LineageProof` @307, `DagTree` @325, `Commit::frame_fields` @368,
`DagError` @388, `payload_bytes` @851, `commit_blocks` @1044, `merkle_root`
@1051, `lineage_proof` @1059, `verify_lineage` @1089, `encode_dag_file` @1145,
`decode_dag_file` @1166; internal: chunk-stack hasher @780, `build_tree`
recursion @915. Inline courts incl. F-1 hostile-counts typed refusal @1658,
prefix-cut Truncated sweep @1699-1744. KEEP all.

**deflate.rs** (792): `Adler32::{new,update,finalize}` (34-57), BitWriter
`{new,put_bits,put_huff,flush_to_byte}` (78-107), Huffman `decode` @441,
BitReader `{new,need,bits,align_byte,consumed}` (357-394), `inflate`-adjacent
surface. Tests: python-zlib cross-checks (631-785) — real-collaborator
subprocess interop. KEEP all.

**fuzz_codec.rs / packfile_fuzz.rs**: 737+740 lines, real python-zlib
differential corpora.

**inflate.rs** (719): `InflateError` @48, Reader `{new,align_byte,read_bits,
bytes_consumed}` (114-143), `Huffman::{from_lengths,decode,empty}` (158-254),
inflate core; std-gated fixture test @103. Tests: stored-block, corrupt
typed-error flips (527-583), python zlib multi-level roundtrip @590, git
fixture @614. KEEP all.

**idx_read.rs** (521): `Idx::{parse,len,is_empty,fanout,oid,crc,offset}`
(25-87), `lookup` @137, `verify_idx_checksum` @196; test-side
`build_idx_v2` @253. Courts: n1000 every-oid, miss→None, n65536 boundary,
oversized-partition typed error, 64-bit large-offset escape, empty roundtrip.
KEEP all.

**ladder.rs** (287): `debug_probes` @38, `debug_tick` @52, `fold` @62,
`oid_lt_eq` @73, `oid_lt_mask` @91, `ct_eq_oid` @99, `ladder_search` @119;
tests: ct_eq hit/miss, order flips, n1000 finds-every, lower-bound misses,
n65536 window boundary, constant iteration-count court @263. KEEP all.

## Branch-site / lever census (BR10 / BR11)

- **BR10 SWAR decimal-itoa**: real lever is `encode_object_header`
  (pack_write.rs:206-217): byte-at-a-time varint push loop while s!=0 —
  7-bit chunks, data-dependent loop. SWAR decimal itoa (2 digits/lane,
  branchless length) applies. **Substitution candidate.** Court: flip pack
  SHA-1 trailer / corrupt one header byte → `PackChecksumMismatch` /
  parse refuses (existing corrupted_pack_trailer_fails_checksum @651 +
  delta_refusal_court cover the flip surface). Hot path: once per object
  in write_pack_v2 / parse_pack_v2.
- **BR11 idx ladder**: `ladder_search` (ladder.rs:119) + `Idx::lookup`
  (idx_read.rs:137) already the 16-stage branchless ladder (constant
  iteration-count court @263 pins it). KEEP — no substitution. Tree
  comparator (dag.rs DagTree ordering) remains DESIGN-TARGET only; no
  branchy comparator hot site found this session to justify promotion.
- Everything else KEEP: deflate/inflate bit loops are algorithmically
  data-dependent (no branchless rewrite candidate identified);
  dag.rs sha chunk-stack is the existing BLAKE-style compression, hot but
  already unrolled.

## Heap-site inventory (GS4 §5, post-GITFIX)

- pack_write.rs: `alloc::vec::Vec` throughout (headers, packs, idx) —
  write-side, cold/serialization, accepted.
- dag.rs:110 `use alloc::{boxed::Box, vec, vec::Vec}` — `LineageProof.siblings`
  @318, `DagTree.nodes` @335, `Commit.oids` @345, chunk_stack @780,
  build_tree Box @915/922/929. Post-GITFIX: no new heap sites introduced by
  the refusal path (refusals are early-return typed errors, zero alloc).
- idx_read.rs / ladder.rs: **zero heap in production body** (Vec only under
  `#[cfg(test)]`). no_std lookup path is allocation-free. CLEAN.
- inflate/deflate: output `Vec` (inherent to stream decode). Accepted.

## tests/ inventory

delta_refusal_court(109, GITFIX court), fsck_court(316), fuzz_codec(737),
git_interop(360), ladder_mutations(878), ladder_timing_stress(319),
large_offset(354), lineage_workflow(259), packfile_fuzz(740),
perf_baseline(214). 10 files, all present.

## Coverage ledger

8/8 real source files censused (100%); 4 absent files marked ABSENT; every
public fn enumerated; branch/lever disposition assigned (1 SUBSTITUTION
candidate BR10, 1 KEEP-with-target BR11, rest KEEP); heap sites inventoried;
GITFIX state verified on disk; riscv32imc build claim UNVERIFIED this
session (lane forbids builds; prior receipt cited, not re-run).
