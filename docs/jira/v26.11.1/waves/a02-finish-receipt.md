# A02 Finish Receipt

- **Lane**: A02 (`crates/cgap-parquet` — `mmap_writer.rs`, `footer.rs`,
  RLE/bit-unpack decode in `rle.rs`), lane-finisher pass, campaign
  v26.11.1, repo `/Users/sac/cgap` (one canonical checkout, branch main)
- **Date**: 2026-11-01
- **Dispatch**: a wave landed (commit 62215326, prior to 9be9109d lineage)
  and a prior wave was killed mid-flight — assess on-disk state first,
  never assume. Witness the host gate this session, decide the width-gate
  axis per spec, run >=1 adversarial sandbox flip, close the missing
  receipt gap (no `a02-finish-receipt.md` existed; only
  `a02-wave8-audit-receipt.md` from an earlier audit slice).
- **Verdict**: **ALIVE** (mission-leg definition) — host gate exit 0 in
  both feature configs (default: 177 lib + 39 integration = 216 tests,
  0 failed; `--features std`: 188 lib + 48 integration = 236 tests,
  0 failed), C5 gate measured in release at **min=516.8us** against the
  < 2 ms TARGET (met, 3.9x under; ratio fence 2.6x vs the receipted
  encoder baseline), and one counting-allocator sandbox flip
  (MUT-FOOT-4) executed BASELINE → BEFORE → MUTATION → AFTER → RESTORE
  with sha256-byte-exact restore and a functional green re-run. Shared
  tree untouched: mutation lived only in `/tmp/cgap-a02-sandbox`;
  `git status` clean for `crates/cgap-parquet/` before and after.
- **Lease honored**: writes confined to `crates/cgap-parquet/src/` (none
  needed — nothing was edited) and this receipt. `lib.rs` untouched
  (it already carried `pub mod mmap_writer; pub mod footer;`).

## Assessment (what was already landed; nothing rewritten)

`crates/cgap-parquet/src` on main carries the lane (lib.rs declares
`pub mod mmap_writer; pub mod footer;` at lib.rs:50-51):

- `mmap_writer.rs` (1619 lines): `SlabView` (lane-contract §7.1 local
  definition), `StaticPageBuffer<const CAP>` `#[repr(C, align(4096))]`
  with compile-time ABI pins (mmap_writer.rs:137-162: alignment 4096 at
  CAP 4096/65536, size_of == CAP, span table bare u32 array, header
  scratch bound), `StaticPageWriter::write_slab` (alloc-free flush into
  the static buffer, up-front width refusal), `seal`, `BlockSink` seam
  (§7.3) with `StaticWindow` (Tier M/B) and `FileSink` (Tier H, std).
  Capacity overflow remapped to `ParquetError::RunOverrun` with
  documented claim/requirement semantics (mmap_writer.rs:41-47) — the
  ADR §3-sanctioned alternative ("re-mapped onto RunOverrun"), taken
  because `rle.rs` was outside the wave's ownership.
- `footer.rs` (1573 lines): FNV-1a checksum + `block_digest` -> 32-byte
  digest, `FooterSealer::seal`, `SealedBlock::flush_to(BlockSink)`,
  `verify_seal` with typed `SealError` verdicts (Checksum vs
  Attestation vs Shape{Magic,FooterLen,BodyDigest,Summary,Meta,Empty}),
  meta block with field-5 body-digest for body-vs-trailer localization,
  80-byte P28 attestation slot framed checksum-spanned and
  flags-bit0-gated, hostile-input corpus courts.
- `rle.rs` `decode_into` (:192): direct-into-caller hybrid decode,
  width gate at :196 admits exactly the kernel set {1,2,4,8} — this is
  spec §7.1's pinned contract ("outside widths are refused by
  UnsupportedBitWidth on decode; the writer must not truncate and
  hope"), so the gate is per-spec and stays. Outside-domain widths
  (3, 5-7, 9-32) are REFUSED BY DESIGN per the authoritative spec; the
  refusal is courted, not silent (c6_bad_width_refused_up_front,
  rle_court::out_of_domain_widths_refused_loudly_not_overflow_panics,
  bits12 regression witness). The 1-32 sweep axis therefore resolves as
  spec-mandated refusal, not UNSUPPORTED capability.
- Fail-closed header validation: decode_into's overrun classes are
  typed (`UnexpectedEof`, `UnsupportedBitWidth`, `RunOverrun`,
  `LiteralOverrun`) over checked arithmetic (checked_mul :240,
  checked_add :250, `get()` bounds :256); the single `unwrap` at :264
  is provably guarded by those checks (group-aligned slice inside
  `pos..end`, byte_len a multiple of 8). Non-test-region panic sweep
  over the three leased files: only contract-boundary assertions — the
  documented encoder width panic (rle.rs:82-93, message names
  UnsupportedBitWidth, witnessed-repair of the mid-loop shift-overflow
  panic) and compile-time const asserts. Zero panic paths on external
  DECODE input.

## Commands (all run this session, exact exits)

1. `CARGO_TARGET_DIR=/tmp/cgap-lane-A02-target cargo test -p cgap-parquet`
   -> **exit 0**; lib 177 passed/0 failed (57.65s), integration
   binaries: accessor_perf 4, aligned_interop 4, codec_properties 0
   (feature-gated empty), level_free_multi 0, mutation_kills 17,
   offset_fuzz 0, page_zero_alloc 2, rle_court 8, robustness 3,
   std_feature_court 1. Total 216, 0 failed.
2. `CARGO_TARGET_DIR=/tmp/cgap-lane-A02-target cargo test -p
   cgap-parquet --features std` -> **exit 0**; lib 188 passed/0 failed
   (11 std-gated unit tests incl. all `courts::tier_h`), 48
   integration. Total 236, 0 failed. Log: /tmp/a02-test-std.log.
3. C5 debug measurement: `cargo test -p cgap-parquet --features std
   c5_gate_100k_rows_under_2ms_target -- --exact --nocapture` ->
   exit 0; `min=4090.8us median=4141.8us body=200068B
   encoder_baseline=3405.3us ratio=1.2x` (debug profile: TARGET
   asserted in release only — printed, not asserted).
4. C5 release (TARGET armed): same filter `--release` -> **exit 0**;
   `min=516.8us median=517.2us body=200068B encoder_baseline=196.7us
   ratio=2.6x` -> **TARGET < 2000us MET** (3.9x under); ratio fence
   2.6x < 50x against `rle_encode_fast` on the identical work
   (sleep-free, min-of-30).
5. Sandbox flip sequence (below), each leg an exact exit.

## Courts run (name -> pass/flip evidence)

All under the two green `cargo test -p cgap-parquet` runs above unless
noted; every named mutation court carries its flip witness in-file.

- `c1_body_byte_flip_flips_checksum_verdict_with_body_localization`
  (MUT-FOOT-1) — ok (std run); flips one body byte on real `seal()`
  output -> typed checksum verdict with body localization.
- `c2_meta_byte_flip_localizes_corruption_to_trailer` (MUT-FOOT-2) —
  ok; meta field-3 flip -> checksum mismatch localized to the trailer
  via the field-5 body digest.
- `c3_zeroed_attestation_slot_with_flag_fires_attestation_verdict`
  (MUT-FOOT-3) — ok; zeroed 80B slot with flags bit0 set -> the
  attestation verdict while the checksum verdict still passes.
- `decode_into_zero_allocations` (rle.rs:402, counting
  `#[global_allocator]`) — ok in both runs; measured 0 allocs across a
  full mixed literal+run decode (4096 lanes).
- `c4_flush_bytes_equal_reference_path_and_mutation_court_flips`
  (MUT-FOOT-4) — ok; Part 1 behavioral (static flush byte-identical to
  the receipted Vec path), Part 2 structural flip witness; the COUNTING
  form of the flip is the sandbox battery below.
- `c5_gate_100k_rows_under_2ms_target` (C5) — ok in release; measured
  516.8us vs the 2000us TARGET; the debug leg prints the measurement
  and asserts release-only.
- `c6_bad_width_refused_up_front_and_not_bypassable` +
  `c6_refusals_are_typed_and_untouched` (MUT-FOOT-6) — ok; width 3 ->
  `UnsupportedBitWidth(3)` before any byte; a hand-written width-3 page
  is still refused on decode (the bypass fails).
- `footer::courts::tier_h::c7_external_reader_accepts_real_rejects_
  swapped_v2_ids` (MUT-FOOT-7) — ok (std run); the real external reader
  accepts real pages and rejects the swapped V2 subheader field-id
  mutation (the PCL2 defect class).
- footer shape courts — `every_truncation_fails_closed`,
  `magic_and_footer_len_flips_are_shape_faults`,
  `hostile_sealed_block_corpus`, `zero_body_seal_refused` — ok; typed
  `ShapeFault` verdicts, no panic.
- `robustness.rs::fail_closed_adversarial_corpus` — ok.
- `mutation_kills.rs` (17 tests) — ok (rle/encode mutation kills).
- Cross-lane: `w8_f1_flush_to_oversized_window`, `a04_identity_court`,
  `a05_power_cut_sweep_torn_writes_fail_closed_per_offset`,
  `a20_one_million_row_stress_two_row_groups`,
  `tier_h_file_sink_round_trip_is_byte_exact` — ok (std run).

## Sandbox flip (MUT-FOOT-4, counting-allocator form) — /tmp/cgap-a02-sandbox

Named invariant: `StaticPageWriter::write_slab` (the flush path) is
allocation-free; the zero-alloc court must fail if a heap intermediate
enters it. The in-lib counting form is structurally impossible (rle.rs
owns the test target's single `#[global_allocator]` at rle.rs:382 —
documented at mmap_writer.rs:1017-1027), so the landed Part-2 witness is
text-structural only. This battery supplies the counting form in a
standalone /tmp crate copy (shared tree never mutated):

- SANDBOX: `/tmp/cgap-a02-sandbox/cgap-parquet` — byte-copy of the
  canonical `src/` (baseline sha256 of mmap_writer.rs
  `f5f4da9e47040e89289b199e2b066690b6f0b5e8926a63788dbfe8621ff12a20`),
  standalone Cargo.toml (workspace inheritance inlined; dev-dep
  cgap-arrow -> absolute path), court `tests/zero_alloc_flip.rs`
  (counting GlobalAlloc over System, atomic counters, flush of a 4096
  -lane bits=8 slab, asserts delta == 0).
- BASELINE sha256: f5f4da9e... (above).
- BEFORE (clean): `cargo test --test zero_alloc_flip` -> **exit 0**,
  1 passed ("allocated 0" asserted, ok).
- MUTATION (SHIM-COURT MUT-FOOT-4): `let _mutated_heap =
  alloc::vec::Vec::<u8>::with_capacity(4096);` injected at the top of
  `write_slab` (anchor unique-checked; only the sandbox file touched).
- AFTER (mutated): court **FAILS** —
  `assertion left == right failed: MUT-FOOT-4: flush path allocated 1
  times — zero-alloc law violated` at zero_alloc_flip.rs:67; true exit
  **101** (no pipe). **FLIPPED.**
- RESTORE: sandbox src re-copied from canonical tree; sha256
  f5f4da9e... byte-exact match.
- BACK-TO-GREEN leg: first re-run after restore returned **FAILED**
  despite byte-exact restore — the restore preserved the ORIGINAL
  (older) mtimes, so cargo's fingerprint kept the MUTATED binary and
  re-ran stale code. `touch` on sandbox src forced rebuild -> **exit 0**,
  1 passed. Lesson recorded below.

## Findings (this session)

1. **Stale-binary restore hazard (new, witnessed)**: a sha256-byte-exact
   restore that preserves pre-mutation mtimes does NOT make cargo
   rebuild — the fingerprint sees the artifact as fresh and re-runs the
   MUTATED binary (witnessed: FAILED verdict on restored source; touch
   -> rebuild -> green). Shim-court "restore byte-exact + green re-run"
   legs must bump mtimes or clean the target between mutation and
   restore legs, or the green re-run is not evidence of the restored
   source. (smalt scripts/shim_court/lib.sh is the canonical pattern;
   this repo's court discipline should adopt the mtime-bump rule.)
2. **C4 counting form**: now witnessed end-to-end in the sandbox; if a
   `crates/cgap-parquet/tests/` binary is ever authorized for this lane
   (file ownership), port `zero_alloc_flip.rs` in-tree so the counting
   form runs with the standard gate instead of requiring a sandbox.
3. **Doc-vs-reality**: spec §4 names `page_aligned.rs` "the alignment
   kernel"; it is in reality an Arrow payload framer. Alignment lives
   in `StaticPageBuffer`'s `#[repr(C, align(4096))]`
   (mmap_writer.rs:97-100 documents this). Spec yields to reality;
   recorded, not "fixed" (spec text is not a code defect).
4. **Width gate**: {1,2,4,8} up-front refusal is spec §7.1's pinned
   contract — decision: gate stays; refusal is courted loud. Widths
   3, 5-7, 9-32 are REFUSED BY DESIGN (typed UnsupportedBitWidth), not
   an UNSUPPORTED lane capability and not a silent skip.
5. **CapacityExceeded**: the ADR §3 alternative (remap onto RunOverrun
   with claim/requirement semantics) was taken by the landed wave; kept
   (spec explicitly authorized either; rle.rs ownership churn avoided).

## Divergences (spec vs reality)

- `page_aligned.rs` role mismatch (Finding 3) — handled in module docs;
  no code change warranted.
- No `CapacityExceeded` variant (Finding 5) — authorized alternative in
  the spec's own ADR text; documented at mmap_writer.rs:41-47.
- C5 TARGET asserted in release profile only; debug runs print the
  measurement (4.09ms in debug — debug-build timing is not the gate).
- Spec §8 C4 asked for an alloc-count flip witness; the in-tree form is
  text-structural (single-global-allocator wall) and the counting form
  ran in the sandbox this session (above). Court is FLIPPED, not FAIL;
  the port-in instruction is Finding 2.

## Standing

**ALIVE** for the mission legs: decode kernel, writer, footer, and
courts ran green this session (216 default + 236 std tests, exits 0);
C5 gate measured and met (516.8us vs 2000us TARGET); MUT-FOOT-4 flip
witnessed with counting-allocator evidence; fail-closed overrun
validation witnessed via typed-error courts. Not witnessed: no_std
target build (wasm32/riscv) this session — out of this lane's mission
(scope of the A02 spec's Tier M gate is the hosted static path);
recorded UNVERIFIED, not assumed.

## Cleanup

- `/tmp/cgap-lane-A02-target` deleted (lease build root).
- `/tmp/cgap-a02-sandbox` deleted (disposable sandbox, per the lane
  build-root cleanup law).
