# Git Survey — Lane GS2: Object Model + Hashing

Static read of `crates/cgap-git/src/` (7 source files, 5096 LOC). No
builds; all file:line from on-disk bytes this session. Reads: `grep`,
`sed -n`, `wc -l`.

## H1. Object envelope + supported types

The canonical git envelope is implemented as a hash-input composer, not
a full serializer:

| Item | Where | Notes |
|---|---|---|
| `git_object_oid(kind, content)` | pack_write.rs:131 | kind + space + len + NUL + content |
| Type argument | `&str` | only "blob" is produced by encoders (pack_write.rs:333, dag.rs:944) |
| tree/commit/tag | none in src/ | only tests/lineage_workflow.rs:193 uses "tree" |

So: no type/len/NUL envelope byte-serializer function
exists (nothing returns the envelope bytes); the envelope exists only as
the hash-update sequence inside `git_object_oid` (pack_write.rs:131-139).
Blob-only encoders: `write_pack_v2` hardcodes type 3 (pack_write.rs:247,
260); `encode_object_header` accepts types 1-4, 6, 7 (pack_write.rs:209)
but no caller passes tree/commit/tag. No tag object support anywhere.

## H2. Hash architecture

| Algorithm | Physical impl | Location |
|---|---|---|
| SHA-1 (FIPS 180-1) | in-crate | pack_write.rs:14-32 struct, `sha1()` at :124 |
| CRC-32 IEEE | in-crate, const table | pack_write.rs:143-206 (`crc32()` at :195) |
| BLAKE3 regular | in-crate | dag.rs:568+, `blake3()` at :828 (`finalize_32`) |
| SHA-256 | absent | grep over src/: zero impl/usage |

BLAKE3 is hardcoded (single-hash), not dual-hash. `DagDigest = [u8; 32]`
(dag.rs:128-132). cgap-blake3 is a **dev-only dep**
(cgap-git/Cargo.toml:22) used for independent KAT cross-checks; the
production hash is the in-crate copy. Documented boundary
(dag.rs:13-17): SHA-1 lives *inside* BLAKE3-hashed bytes, never reverse.

Oid representation: `pub type Oid = [u8; 20]` — lib.rs:25, a plain
20-byte SHA-1 alias. No enum/newtype. 32-byte digests are a separate
type (`DagDigest`), never mixed (dag.rs:130-132). `ZERO_DIGEST`
(dag.rs:139), `GENESIS_OID = [0u8; 20]` (dag.rs:135).

## H3. Heap-allocation inventory (encoders + tree path)

### pack_write.rs (write path)

| Site | Allocates | Class |
|---|---|---|
| :208 `encode_object_header` | `Vec::with_capacity(8)` header varint | hot, per-object |
| pack_write.rs:242 `write_pack_v2` | `Vec::new()` — full pack output buffer | hot, once per pack |
| :276 `write_idx_v2` | `Vec::with_capacity(1032+44*n+40)` idx | hot, per pack |
| pack_write.rs:322 `build_pack_and_idx` | `Vec::with_capacity(blobs.len())` — entries table | hot |
| pack_write.rs:355 `PackBuilder::push` | `blob.to_vec()` — owned blob copy | hot, per-blob |
| pack_write.rs:513/526/527 | oid/crc/offset tables `with_capacity(n)` | reader (parse/verify) |
| pack_write.rs:425 `parse_pack_v2` | `Vec::with_capacity(count)` — parsed objects | reader |
| pack_write.rs:580-583 `hex` | String + format! per byte | test-only |
| pack_write.rs:619 `entries.clone()` | test clone | **test-only** |

### dag.rs (tree construction)

| Site | Allocates | Class |
|---|---|---|
| dag.rs:930 `order: Vec<usize>` | index vec `(0..len)` | hot, per commit |
| dag.rs:938 `oids: vec![GENESIS_OID; n]` | per-block Oid table | hot |
| dag.rs:939 `level: Vec::with_capacity(len)` | leaf node boxes | hot |
| dag.rs:947+ `Box<TreeNode>` per node | 2L-1 boxed nodes (promotion law) | hot |
| dag.rs:973 `next: Vec::with_capacity((level+1)/2)` | per-level scratch | hot |
| dag.rs:972-1007 level loop | one `Vec` realloc per tree level | hot |
| dag.rs:1035 `sibling_path: Vec::new()` | proof path vec | proof gen only |
| payload_bytes (dag.rs:839) | none — `[u8; 128]` stack buffer | zero-alloc |

### deflate.rs / inflate.rs (codec under the pack)

| Site | Allocates | Class |
|---|---|---|
| deflate.rs:288 | `Vec::with_capacity(len/2+16)` deflate output | hot, per object |
| deflate.rs:193 | `Vec::new()` token buffer | hot, per block |
| deflate.rs:80 | `Vec::new()` bit-writer out | hot |
| inflate.rs:262 | `Vec::new()` output, grows to declared size | hot, per object |
| inflate.rs:180/319 | Huffman symbol/length tables | hot, per block |
| deflate.rs:605-618, inflate.rs:561+ | corpora vecs | test-only |

Notable zero-alloc sites: CRC table is `static` const-built
(pack_write.rs:165); SHA-1 and BLAKE3 state are stack structs; idx
fan-out is a stack `[u32; 256]` (pack_write.rs:286). Whole crate is
`no_std` + `alloc` (imports via `alloc::vec`, pack_write.rs:10).

## H4. Lineage receipt cross-reference (p1-lineage-workflow.md)

| Cited symbol | Receipt claim | Current reality | Verdict |
|---|---|---|---|
| `sha1` | pack_write.rs:124 | :124 | no drift |
| `crc32` | :195 | :195 | no drift |
| `git_object_oid` | :131 | :131 | no drift |
| `build_pack_and_idx` | :310 | :310 | no drift |
| `parse_pack_v2` | :411 | :411 | no drift |

Divergences recorded by that lane and still true: entries sorted by OID
(write_idx_v2 `entries.sort_by_key`, :269) not blob order; child parent
head = `git_object_oid("tree", dag bytes)` exists only at
tests/lineage_workflow.rs:193 (production API never mints tree OIDs).

## Standing

ALIVE for static reads: every file:line above was read this session via
grep/sed with exit 0. No builds run (lane contract: static only). No
mutation courts run — this is a census, not a behavioral gate; the
behavioral courts live in cgap-git tests (61 passed per p1 receipt,
as-of-date, not re-run here).
