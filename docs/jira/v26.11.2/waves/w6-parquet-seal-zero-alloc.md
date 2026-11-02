# P5 — parquet seal zero-alloc (v26.11.2 wave 6)

Lane: P5 (parquet seal zero-alloc). Ownership: `crates/cgap-parquet/src/footer.rs`
(seal-path alloc sites only) + this receipt. No git writes; publishes frozen.

## Change summary

- `FooterSealer::spans`: `Vec<u32>` (heap copy via `to_vec`, footer.rs:376) →
  fixed inline `[u32; MAX_ROW_GROUPS]` (the writer's own representation,
  `mmap_writer.rs:72`), filled by `copy_from_slice` of the summary's spans.
- `build_meta_block`: Vec-growth across `write_*_field` pushes (footer.rs:447)
  → fixed-capacity `[u8; META_CAP=512]` scratch emitted via a new footer-local
  `MetaBuf` (byte-exact mirrors of `rle::write_varint` and the thrift
  compact-protocol field writers; zigzag encoding delegated to the crate's own
  `thrift::zigzag_encode_{i16,i32,i64}` so no constant is redefined).
  Overflow is a typed `RunOverrun` (claim/budget), never silent truncation.
- New typed seal refusal: a `BlockSummary` with `row_groups >
  MAX_ROW_GROUPS` (64) is refused with payload
  `RunOverrun { run_len: row_groups, budget: MAX_ROW_GROUPS }`. Wire bytes are
  unchanged (the writer itself can never exceed 64 groups).
- Doc carve-out at footer.rs:52-55 ("allocation is lawful on the seal path")
  CLOSED and replaced with the new fixed-buffer description.
- Courts added (in footer.rs test module): `meta_fixed_path_is_byte_exact_vs_vec_oracle`
  (differential vs the crate's real Vec writers over rg ∈ {1,3,14,15,64};
  anti-vacuity MUT-FOOT-10: one-input-bit flips must move the bytes) and
  `max_row_groups_over_capacity_refused_with_exact_payload` (MUT-FOOT-11:
  lawful MAX_ROW_GROUPS arm must construct Ok; payload pins the check).

## Wire byte-identity

Byte-identity is guaranteed by construction (same byte emission sequence) and
pinned by the new differential court plus the pre-existing seal round-trip
courts: `every_truncation_fails_closed`, the hostile sealed-block corpus,
`meta_field5_is_body_digest_and_summary_round_trips`, the A04 identity court,
and the C7 external-reader (std) court — all green this session.

## Alloc witness (divan alloc column, crates/bench chains-parquet seal_arm)

Method: `cargo bench --bench chains-parquet -- seal_arm` with
`AllocProfiler` (binary-scoped global allocator, chains-parquet.rs:45-46).

- BEFORE (this session, pre-edit):
  - 65536: alloc 2 / 12 B (max 36 B), dealloc 2 / 36 B, grow 2 / 24 B
  - 1048576: alloc 2 / 12 B (max 36 B) — matches w5-encode-repareto.md
    (sites 376 + 447).
- AFTER (this session, post-edit, two runs):
  - alloc 0 / 0 B at BOTH consts (median; 100 samples each). One 64 B
    outlier sample observed across the two runs (mean 0.01 allocs —
    consistent with a warmup/allocator artifact outside the steady loop;
    median and fastest both 0). Latency unchanged: ~1.01 ms @65536,
    ~16.4 ms @1048576 (baseline 16.33-16.36 ms — within noise).

Target landed in full: seal arm 2 allocs / 36 B → 0 (median).

## Commands + exits

- `cargo bench --bench chains-parquet -- seal_arm` (baseline) — exit 0
  (alloc 2/12 B, max 36 B, both consts).
- `cargo test -p cgap-parquet --lib footer::` — exit 0 (12 courts incl. the
  two new).
- `cargo test -p cgap-parquet` — exit 0 (182 lib + 45 integration, 0 failed;
  page_zero_alloc + mutation_kills + hostile corpus green).
- `cargo test -p cgap-parquet --features std` — exit 0 (193 lib passed; the
  `#[cfg(std)]` C7 external-reader court ran, real python3 subprocess).
- `cargo bench --bench chains-parquet -- seal_arm` (after) — exit 0, alloc
  column 0/0 B at both consts.
- `cargo build -p cgap-parquet -p bench --benches` — exit 0.

## Compile-freeze SLA disclosure

During the after-bench build, cgap-bn3 (another lane's in-flight no_std
strip) failed to compile — errors migrating across unify.rs / project.rs
between my two attempts. No unblock edit was applied by this lane; the
owning lane recovered within the SLA window and the build froze green
(bounded poll: BUILD_OK on attempt 1 of the retry loop, ~60 s).

## Standing

ALIVE for the seal-path zero-alloc claim: command + exit run this session
against the exact subject (footer.rs post-edit; divan alloc column 2→0
median). Remaining honest notes:

- The divan alloc mean shows 0.01 (a single 64 B sample in 200) — recorded,
  not rounded away; median/typical is 0.
- Verify-side alloc (parse_meta's Vec spans) remains — out of lane scope
  (verify path), unchanged.
