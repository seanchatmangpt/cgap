# Lane EX5 — cgap-arrow arrays/bitmaps exhaustive census (part 2)

Read-only census, no builds. Files enumerated 2026-11-02 against
`crates/cgap-arrow/src/`. array.rs ABSENT (marked per dispatch); real
array-manipulation files: bitmap.rs, bitmask.rs, slab.rs, buffer.rs,
batch.rs, varlen.rs, views.rs, slice.rs, runend.rs.

## Per-file loop census (non-test code only)

- **bitmap.rs** (253 L): `mask_words` (L34-41) word-wise u64 AND per 64
  rows — KEEP. `count_valid` (L46-55) word-wise popcount — KEEP.
  `is_all_valid` (L181-200) word-wise popcount + masked tail — KEEP.
  `count_one_bits` (L211-224) word-wise popcount + masked tail — KEEP.
  `word()` (L164-171) byte-copy tail-safe — KEEP. Verdict confirms BR3:
  scan kernels already word-wise; zero per-byte hot loops.
- **bitmask.rs** (240 L): `count()` (L126-134) is the sole per-byte
  loop: `count_ones()` per u8 over `MASK_BYTES` bytes, `const fn`.
  SUBSTITUTION CANDIDATE (swar): read `bits` as u64 words via
  `u64::from_ne_bytes` + `count_ones` — but `bits` is a fixed `[u8;
  BYTES]` in a `const fn`, so it needs a const-compatible word read or
  dropping `const` (callers: C-ABI negotiation surface, cold). COLD;
  MASK_BYTES ∈ {8,32,128}; keep-priority LOW, opportunistic.
- **slab.rs** (1930 L): `fnv1a32` (L285-295) per-byte FNV-1a — inherent
  to the algorithm (§4.1 frozen wire law); KEEP (not swar-able without
  breaking the frozen digest). `seal()` padding zero-fill (L372-378)
  per-slot `[0;3]` write, CAP ≤ 1024, COLD — KEEP. `from_bytes` slot
  decode (L496-512) per-slot byte window parse — wire-format inherent,
  COLD (ingest path) — KEEP. `seal_with_mask` (L547-561)
  `get_unchecked` at L554: justified (i < len ≤ CAP invariant itemized,
  GATE-2 no-panic law, riscv32imc bounds-check elision) — KEEP, not a
  select_slice-class violation (see below).
  blake3.rs get_unchecked (L278, 303-304, 339) — out of lane scope
  (EX4/BL3 jurisdiction), noted only.
- **buffer.rs** (163 L): no loops in non-test code; SAFETY-commented
  raw accessors. KEEP as-is.
- **batch.rs** (448 L): L172-175 duplicate-name check is O(n²) via
  `columns[..index].iter().any(...)` over column count (small, COLD,
  builder-path) — KEEP. All other `.iter()` are metadata/debug — KEEP.
- **varlen.rs** (262 L): `offsets_monotonic` (L128-138) per-offset
  compare — correctness loop, COLD-ish; pair-compare is inherently
  scalar; KEEP. `finish()` offsets write (L197-203) per-offset
  `copy_from_slice` of 4/8 LE bytes, COLD freeze path — KEEP.
- **views.rs** (188 L): zero loops in non-test code — KEEP. Nothing to
  substitute.
- **slice.rs** (377 L): zero loops in non-test code (Sliced is
  zero-copy view); loops at L213/274/342+ are `#[cfg(test)]`. KEEP.
- **runend.rs** (359 L): `decode_to_dense` (L129-139) per-row
  `dst[row] = vals[run]` — scalar i32 store per row, run-length
  amortized; possible widening (fill word-wise per run) but runs are
  O(runs) writes; LOW-priority candidate, COLD-to-warm. Builder
  `finish` (L206-216) per-run LE write — wire-format inherent, KEEP.

## select_slice / get_unchecked sweep

REJECT class (BR3) stands: no re-proposal. `get_unchecked` sites in
array-manipulation files: **slab.rs:554 only** (seal_with_mask,
invariant-backed, disclosed above). No `select_slice` symbols exist in
the crate. blake3.rs get_unchecked is outside this lane's file set —
noted, not adjudicated.

## Coverage ledger

100%: all 9 present array-manipulation files censused; every non-test
loop classified word-wise / per-byte / wire-inherent / KEEP /
candidate. Files absent: array.rs (marked). Full-src get_unchecked
sweep: bitmap/bitmask/buffer/batch/varlen/views/slice/runend = 0 hits;
slab.rs:554 (KEEP); blake3.rs (out of scope).

Standing: UNVERIFIED-for-substitution (census only, no builds run, no
mutation courts; substitution candidates are TARGETs). Commands: `ls`+
`wc -l` src/, three grep/sed sweeps — all exit 0.
