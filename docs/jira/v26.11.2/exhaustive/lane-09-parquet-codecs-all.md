# Lane EX9 — cgap-parquet codec/scan census (100% fn coverage, read-only)

Standing: PARTIAL_ALIVE (executed read-only inspection this session; no builds per lane contract). Files: 8 present + 1 absent-elsewhere; ~55 fns ledtered; all have hot/cold class, loop classification, KEEP/substitute disposition.

### thrift.rs (921 L) — cold-path codec — all KEEP
`ThriftError::fmt` :48; `read_varint` :81 (shift loop, truncation-typed, court truncation_at_every_byte); zigzag enc/dec i16/i32/i64 :102-133; `write_zigzag_*` :139; `write_field_header` :157 (delta 1..=15 branch); `write_bool_field` :169; `write_{i32,i64,binary}_field` :179-191; `read_field_header` :199 (STOP/delta match); `read_{i32,i64,byte,binary}` :226-257 (checked_add/get fail-closed); `skip_value` :270 (per-type match, MAX_SKIP_DEPTH nesting-bomb guard, court crafted_nesting_bomb); `skip_element` :333 / `skip_struct` :346 / `valid_type_id` :361; `write_page_header` :419; `read_page_header` :463; `read_data_page_header_v2` :508 (required-field gates, court missing_required_fields).

### rle.rs (758 L) — hybrid codec
`read_varint` :40 (cold; thrift twin; writer mirror footer.rs:133 byte-exact doc-pinned) — KEEP. `write_varint` :60 (continuation branches) — KEEP. `rle_encode` :88 — run>=3 branch; literal arm is BYTE-wise bit-fill loop (bit<vb, per-byte push), not word-wise; candidate: word-packed literal bytes; court mutant = absorb-to-group-boundary removal (mid-stream partial over-decode, W3-RLE court covers) — KEEP. `rle_decode` :169 (exact-fill wrapper) — KEEP. `decode_into` :192 — HOT; width gate matches!(1|2|4|8); header &1 run/literal branch; budget clamp run_len > out.len()-oi; zero-alloc (court decode_into_zero_allocations) — KEEP.

### scan.rs (1497 L) — footer/index/prune — AS-OF 2026-11-02
Mmap open/as_slice/Drop :86-114 (len==0, MAP_FAILED guards) — KEEP. `ScanError::width/fmt`, `PhysType::from_id` :247-273 (refuses FLOAT/DOUBLE) — KEEP. `rg_prunes` :331 (fail-open on absent bounds; BR5 verdict stands: inherent, marginal) — KEEP. `write/read_list_header`, `each_field` :379-413 — KEEP. `parse_footer` :421 (version/num_rows/columns/row_groups gates) — KEEP. `read_schema_element` :476; `read_row_group` :534 (offset-mismatch guard); `read_column_chunk` :579; `read_column_meta_data` :605 — KEEP. `read_statistics` :634 — **PRUNEFIX LANDED**: inverted bounds (min>max, bit-pattern order) at scan.rs:662-668 return None (untrusted -> fail-open unpruned), doc pins u64/u32/bytes32-only domain. REQUIRED anti-vacuity mutant: inverted-bounds must flip prune->not-prune. `ParquetFile::open/meta/num_rows/row_groups` :698-722 — KEEP. `resolve`/`resolve_uncached` :734-759 (negative-page-size guard :777) — KEEP.

### unpack.rs (147 L) — per-lane scalar, NOT SWAR (doc divergence)
`unpack_{1,2,4,8}bit` :7-77 — lane-wise shift+mask inside per-word while loops; scalar per-lane over u64 words, no SWAR table/pack. Candidate: SWAR table unpack; court mutant = tail-clamp removal (partial_tail_is_clamped exists) — REQUIRED mutant #2. `unpack_bits` :80 (width dispatch, typed UnsupportedBitWidth) — KEEP.

### bitslice.rs (930 L) — vertical bit-slice kernels
`pack_vertical` :77 — 64-groups, `if v==0` skip, per-bit while; lane-wise — candidate: precomputed per-level masks. `scan_range_vertical` :127 -> counted :135 — HOT; domain gate lo>domain_max; chunks_exact(wf); rev() per-level; (eq_hi|eq_lo)==0 early-skip — KEEP. `scan_range_vertical_128` :216 — HOT; 2-group unroll + scalar tail branch :283; differential court vs scalar ref; candidate: 4-group unroll, mutant = swap unroll pairing. `mask_of` :207. `sum_masked_vertical_128` :323 — algebraic-parity court. `semi_join_masks` :348 / `semi_join_two_hop` :379 — length guards. Seeds pinned at :687-699 (splitmix64 0xBF58_476D_1CE4_E5B9 vs corpus 0xA20C_0FFE_E5A7_E000) — present, distinct, do not sweep.

### dictionary.rs (505 L) — first-seen intern
`required_bits` :65 (width table, len==0 typed refusal). `DictBuilder::{new,push,len,is_empty,dict_len,finish}` :97-139 (contains_key branch). `encode_page` :167 — width + dict-overflow + per-index domain gates (courts encode_page_gates_fail_typed, in_domain_by_construction). `decode_page` :202 — dict-empty refusal + exact-fill + per-lane ix>=dict.len() fail-closed (court decode_page_fail_closed_on_hostile_pages); candidate: word-wise 8-lane batch domain check. All KEEP. W3-RLE mid-stream-run fix cited in tests :423 — landed, intact.

### deflate.rs — ABSENT from cgap-parquet
Not in crates/cgap-parquet/src (ls witness this session). Lives at crates/cgap-git/src/deflate.rs — ABSENT-elsewhere, out of EX9 lane partition. Marked, not silently skipped.

### stream_compact.rs (101 L) — branchless exemplar
`compact_lanes_branchless` :10 — 0 conditional branches (unconditional store + validity-bit head advance) — KEEP, existing exemplar. `neon_table_compact_8` :38 — aarch64 vtbl1_u8 shuffle + popcount — KEEP. Candidate: u8 mask -> u64 mask 64-lane batches via carry-less prefix scan; court = mixed-selectivity batch count flip.

### level_free.rs (727 L) — zero-level writer/reader — all KEEP
`LevelFreeError::fmt` :63; `width`/`code` :156-165; `write_list_header` :201; `write_schema_elements` :214; `write_root_element` :233; `write_schema_list` :242; `parse_schema_elements` :251; `admit_schema` :298 (optional/repeated/nested/unsupported-type courts refuses_*); `align_rows_256b` :355; `write_page` :368; `size_i32` :413; `write_row_group_page` :437; `write_column_meta_data` :485; `size_i64` :499; `write_row_group` :506; `write_footer` :527; `write_file` :551; `read_page_payload` :575 (length-mismatch typed).

### ABSENT files (marked)
statistics.rs / filter.rs / compact.rs / varint.rs — ABSENT from cgap-parquet/src (ls witness). Stats/prune live in scan.rs (`read_statistics`, `rg_prunes`); varint lives in thrift.rs:81 + rle.rs:40 (writer mirror footer.rs:133).

## Branch-site summary
Hot fns: rle.decode_into; bitslice.{scan_range_vertical_counted, scan_range_vertical_128, sum_masked_vertical_128}; stream_compact.{both}; unpack 4 kernels; dictionary.decode_page. Everything else cold. Two REQUIRED anti-vacuity mutants: (1) scan.rs:662 inverted-bounds prune flip, (2) unpack tail-clamp removal.

## Divergences from lane doc
1. unpack.rs is per-lane scalar over u64 words, NOT SWAR — doc claim corrected.
2. deflate.rs not in cgap-parquet; in cgap-git/src/deflate.rs (out of lane scope, marked).
3. PRUNEFIX min<=max guard CONFIRMED LANDED at scan.rs:662-668 this session.
