# Branchless reuse map — cgap-arrow (lane BR3)

Read-only survey, 2026-11-02. All file:line verified on disk this session.

## 1. validate.rs bounds checks: HOT vs COLD

`validate_fixed_width` (crates/cgap-arrow/src/validate.rs:76-99) runs 3 checks
per column: null-column refusal, `align_offset(8)`, `saturating_mul(8) > bytes`.

**Verdict: KEEP-BRANCHY, all paths.** A validator's job is refusal; every
branch here is COLD-path typed refusal (ValidateError at :16-33). None of the
three checks is a per-element hot loop — cost is O(3 columns) per frame, not
per row. N2's word-popcount substitution class does not apply. No candidate.

## 2. bitmask.rs / bitmap.rs loop taxonomy

- `PresenceMask::count` (bitmask.rs:126-134): BYTE-wise `bits[b].count_ones()`
  over MASK_BYTES (8/32/128 bytes for frozen aliases Mask64/256/1024).
  Per-BYTE bulk loop → N2's 4-8x class applies, but masks are tiny (max 128 B)
  so gain is bounded; an existing word-wise kernel (`bitmap::count_valid`)
  cannot be called from a `const fn` — substitution requires a const-expr
  chunked count or a non-const twin. Candidate B2.
- `count_valid` (bitmap.rs:46-55): already WORD-wise (u64 `count_ones` per
  word) — N2's byte-LUT claim does not apply here; verify noted, no change.
- `mask_words` (bitmap.rs:34-41): already word-wise, cyclomatic 1, NEON
  auto-vectorized (module doc :4-10). No candidate.
- `count_one_bits`/`is_all_valid`/`null_count` (bitmap.rs:211-225, 181-199,
  204-206): word-wise popcount + tail mask. No candidate.
- `Bitmap::from_bools` (bitmap.rs:103-117): per-BIT loop (byte_idx/bit_idx
  per element). Not popcount class; a SWAR byte-broadcast pack is possible
  but gain class is [ANALYTIC] only — deferred, low value (constructor path).
  No candidate.
- `word`/`word_at`/`word_valid` (bitmap.rs:75-90, 164-171): single-word loads
  via copy_from_slice + bounds checks; these are the memory-safety boundary —
  REJECT any get_unchecked substitution (see §3).

## 3. select_slice / unchecked bounds-check avoidance

No `select_slice` surface exists in cgap-arrow today (verified: zero matches
across src/). Any proposal to trade bounds checks for branches on "valid
slices" is **REJECT**: bounds checks are the memory-safety boundary; the
`word_at` panic-on-OOB contract (bitmap.rs:83-90) is load-bearing. If a
checked-precondition wrapper is ever admitted, it must (a) assert the range
once per call, (debug_assert or typed checked slice), then (b) document the
unchecked inner read — not land as a bare `get_unchecked`. Standing: REJECT
class, documented, not built.

## Candidates

| id | file:line | current | target | gain class | court mutant |
|----|-----------|---------|--------|-----------|--------------|
| B2 | bitmask.rs:126-134 | per-byte `count_ones` | const word-chunked popcount (u64 load per 8 bytes, `count_ones`) OR non-const twin calling `bitmap::count_valid`-style loop; padding stays clear so tail byte needs no mask | [ANALYTIC] bounded: 8x fewer popcount ops on Mask1024 only; masks ≤128 B so wall-clock gain is small | mutate `mask_bytes` to non-word-fill (`cap.div_ceil(8)`): `count` must still return 1024 for Mask1024::ALL_VALID only if word-wise impl reads padded bytes as zero — actually padding-clear law means mutant is expected to FAIL count for CAP=63 alias pairs is const-refused already; better mutant: flip count accumulation to `count_zeros` — expected: c4 courts fail (`Mask1024::ALL_VALID.count() == 1024` pin flips) |
| B2b | bitmap.rs:103-117 | per-bit set in from_bools | SWAR byte pack (8 bools/byte via 0x0101.. broadcast + multiply trick) | [ANALYTIC] 8x fewer iterations, constructor path only | mutation: swap `byte_idx = i >> 3` shift to `i >> 2` → round-trip `null_count`/`get` courts (bitmap_tests) must fail |

## Anti-vacuity

Both mutants flip existing pinned courts (c4 LSB probe, bitmap_tests
round-trip); neither is a vacuous green. No court was run this session —
this lane is read-only mapping; standing UNVERIFIED (map, not measurement).
Perf claims above are TARGETs ([ANALYTIC]) until measured per BM method.
