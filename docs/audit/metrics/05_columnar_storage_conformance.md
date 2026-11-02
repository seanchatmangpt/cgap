# AU5 Phase 1 — Columnar Storage Conformance (05)

Tree: /Users/sac/cgap. Method: wc -l, grep, targeted reads; no builds, no Process.sleep. Measured at HEAD this session.

## 1. Zero-dependency inventory (LOC, wc -l)

### cgap-arrow (src total 10,271 LoC)
- slab.rs 1,930 — fixed-cap slab / COMMIT_LEASE allocator
- ipc.rs 399 — CONTINUATION=0xFFFF_FFFF (:36), 64-B repr(C) MetaHeader (:54), seal_meta/emit/read (:198,246,295)
- c_abi.rs 284 — ArrowArray/ArrowSchema twins, release-once Drop (:199-247), CAS no-op double-release (:255-283)
- validate.rs 315 — ValidationLevel ladder (nanoarrow R6), validate_minimal (:142)
- bitmask.rs 271 — PresenceMask const-generic mask, word-wise SWAR (:126)
- NOTE: unpack.rs is NOT in cgap-arrow; unpack.rs (147 LoC) lives in cgap-parquet/src

### cgap-parquet (src total 14,552 LoC)
- thrift.rs 921 — hand-rolled compact-protocol forward parser
- footer.rs 2,114 — FooterSealer::seal (:586), verify_seal(bytes,&[u8;32]) (:789), MetaBuf fixed path
- scan.rs 1,497 — offset-index scan; PLAIN payload IS the Arrow buffer bytes (aliasing, :26); fail-closed stats pruning (:302-325)
- mmap_writer.rs 1,916 — RingSink (:721), BlockSink on &mut RingSink (:717)
- rle.rs 758 — RLE/bit-pack hybrid; level_free.rs 727; unpack.rs 147 — SWAR unpack
- deflate: absent — no deflate.rs in cgap-parquet; zlib lives in cgap-git

### cgap-git (src total 5,287 LoC; per GS3 survey, git-survey-lane-03.md)
- pack_write.rs 776 — pack V2 writer/parser, idx V2, in-crate SHA-1 (:63-123), CRC32 table (:144) + streaming (:172-195)
- idx_read.rs 521 — IdxV2::parse (:25), verify_idx_checksum (:196); dag.rs 1,919 — BLAKE3 merkle DagTree, postorder wire (:270)
- deflate.rs 792 — fixed-Huffman deflate (:254), zlib_compress 0x789C (:287), own inflate (:415); inflate.rs 719 — second independent inflate (:260), adler32 (:407)
- No external deps in any hash/compression/pack path of the three crates.

## 2. Conformance evidence (measured)

### Arrow C Data Interface header (task "N4" framing)
- include/cgap_c_data.h: 62 lines, ggen-generated from ontology.ttl, spec-cited (arrow.apache.org C_DataInterface); ArrowArray (80/8 LP64), ArrowSchema (72/8), C11 _Static_assert pins; mirrors c_abi.rs twins (E13 COURT 2, offset_of!).
- BYTE-IDENTICAL CAVEAT: not verified byte-identical to upstream arrow-array.h this session; measured standing is spec-cited + ggen-frozen + struct-exact via E13.
- E13 (docs/jira/v26.11.2/countermeasures/lane-e13-abi-court.md): 31 compile-time const assert pins (size/align/offset, three twins); COURT 3/4 mutation anti-vacuity (cc -std=c99 freestanding sandbox must FAIL on swapped ArrowArray field pair, pristine compiles; Rust mirror flips same mutation, pristine scores EQUAL). Court: crates/cgap-ffi/tests/c_header_abi_court.rs.
- Mutation corpus: crates/cgap-parquet/tests/mutation_kills.rs (hand-derived wire bytes from rle spec, :248).

### Parquet wire parity (P5)
- waves/w6-parquet-seal-zero-alloc.md:23-24: court meta_fixed_path_is_byte_exact_vs_vec_oracle — fixed path vs the crate's own Vec writers, rg ∈ {1,3,14,15,64}; MetaBuf = "byte-exact mirrors of rle::write_varint and the thrift" (:13).
- Differential-oracle courts: lever_courts.rs (FNV oracle vs kernel), codec_properties.rs (wire bytes asserted), mutation_kills.rs.

### Git zlib + interop (GS3) — cited precisely
- crates/cgap-git/src/deflate.rs:628-641: real system python3 zlib as court collaborator ("court precondition"); cross-validation at deflate.rs:580 and inflate.rs:463; real-git fixture inflate.rs:585. This is cgap-git's zlib, NOT parquet's — parquet has no deflate.
- tests/git_interop.rs, both directions: git→us via `git pack-objects` (:118-145) with idx object-count parity (:211); us→git via write_pack_v2 into temp .git/objects/pack, read back by real git (dir B, GS3 §3 table); IdxV2 verify vs git idx (dir A).

## 3. Honest divergences (measured / repo-documented)
1. IPC MetaHeader vs Arrow IPC flatbuffers: DEFERRED, documented (waves/p1-ipc-arrow-ipc-envelope.md:29-34,78,124-126) — cgap MetaHeader + O-ABI schema digest; unknown-flatbuffer typed parse failure, never misread. Feather research (v26.11.1/waves/research-feather-format.md:79) records the flatcc/compiler-step requirement. The task-supplied "flatcc 3,250 LoC + 25,669-line header" figures were NOT found in the repo — UNVERIFIED if used.
2. Delta resolution NOT-IMPLEMENTED: no apply_delta symbol crate-wide; parser rejects types 0/5 only (pack_write.rs:390-392), so a type-6/7 body is misread as zlib, not refused (GS3 §3; lib.rs:6-7 "no deltas").
3. Blob-only object writers: write_pack_v2(&[&[u8]]) (:238), PackBuilder::push(&[u8]) (:354); no tree objects/modes/tree-sort comparator (GS3 §1).
4. Validation ladder (validate.rs) narrower than nanoarrow's four-level contract pre-lane; unions unsupported (synthesis R8b).
5. Verdict: spec SUBSET, not spec-complete; subset boundaries documented in-repo, not silent.

## 4. ANALYSIS (labeled: interpretation, not measurement)
The class of work is format-implementer conformance without reference-library linkage: each wire grammar (Arrow C ABI, compact thrift, pack V2, DEFLATE/zlib, IPC framing) is re-derived in-crate, then conformed not by linking Arrow/Parquet/git libraries but by (a) struct-exact headers frozen by codegen with compile-time offset pins, and (b) differential courts against independent oracles — the crate's own Vec writers, hand-derived wire bytes, system python3 zlib, real `git pack-objects`/`index-pack`. This matches the repo's nanoarrow oracle synthesis (research/nanoarrow-oracle-synthesis.md): vendor the reference implementation as a read-only comparison corpus, extract the constructs worth adopting (release protocol, validation ladder, SWAR unpack — verdicts R1-R11 with explicit DIVERGENT-RECORDED), and synthesize a one-harness A/B oracle rather than adopt the 186 KB-class C99 tree wholesale; same struct inventory, same mutation anti-vacuity legs as E13. Limit of the method: differential courts against oracles you also own (Vec writers) prove internal consistency, not ecosystem interoperability; the python3-zlib and real-git legs are what convert subset conformance into externally witnessed conformance, and they exist only where a real external collaborator is available.
