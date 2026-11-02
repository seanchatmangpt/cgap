# BR10: branchless reuse map — cgap-git OID/envelope

## 1. Header decimal-length formatting (`<type> <len>\0`)

- Where: `crates/cgap-git/src/pack_write.rs:131-137` (`git_object_oid`) — formats via
  `content.len().to_string()` (core fmt, not itoa-loop). Write path per-object: called
  once per object in `write_pack_v2`/`PackBuilder` (pack_write.rs:245+), so hot in pack writes.
- Current: `to_string` on `usize`; envelope hash input. Not the pack varint header
  (`encode_object_header`, pack_write.rs:209-221) — that is bit-shift only, no decimal.
- Primitive: SWAR 2-digits-at-a-time decimal itoa (div 100, 2-digit LUT) into a fixed
  `[u8;20]` buffer — removes fmt machinery + allocs.
- Gain class: LOW-to-MEDIUM. Per-object cost is dominated by SHA-1 (20-block absorb);
  fmt+to_string overhead is real but a small fraction; only pays on multi-thousand-object packs.
- Court: mutation `git_object_oid` header bytes (flip one decimal digit) must fail
  `git fsck`/pack checksum courts (cgap-git tests/fuzz_codec.rs, delta_refusal_court.rs).

## 2. NUL-delimiter search for envelope termination

- Where: no NUL-search loop exists. Envelope is assembled and streamed into SHA-1
  (`update` calls at pack_write.rs:133-136); dag.rs payload is fixed 128-byte wire form
  (dag.rs:843-846+) with fixed offsets — no delimiter scan.
- Primitive: `swar_has_zero_byte(x) = (x.wrapping_sub(0x0101..) & !x & 0x8080..) != 0`
  applicable to any ≤128-byte stack scan if one is introduced.
- Gain class: NONE-TODAY ([ANALYTIC-no-target]). No per-byte delimiter loop to replace;
  do not invent one to optimize. Revisit only if a buffer-parse envelope reader lands.

## 3. Oid equality ([u8;20])

- Where: `Oid = [u8; 20]` (lib.rs:25), derived `PartialEq` at IdxEntry (pack_write.rs:229),
  parse-side lookups (pack_write.rs:630).
- Primitive: u64-chunked SWAR equality (16+4 bytes → 2.5 words).
- Gain class: [ANALYTIC-no-gain]. rustc already emits unrolled loads/compares for
  fixed-array `PartialEq` (LLVM vectorizes [u8;20] to 2×u64 + 1×u32); a SWAR rewrite
  changes nothing codegen-wise. Verify once in asm at integration, do not rewrite.
- Court: existing parse/idx roundtrip courts already pin equality semantics; no new court.

## 4. idx fanout table (pack_write.rs:286-296)

- Where: write side builds cumulative counts (pack_write.rs:286-296); read side validates
  monotonicity (pack_write.rs:496-503) via `windows(2)`.
- Primitive: none needed — 256-entry first-byte table is O(1) exact-index lookup by
  construction (`counts[e.oid[0]]`), and read-side binary search is not present.
- Gain class: NONE ([ANALYTIC-no-gain]). No substitution; no branchless search to add.

## Net branchless surface

Only lever #1 is actionable (SWAR itoa in `git_object_oid`), gain class LOW-MEDIUM,
SHA-1-dominated path. All others analytic no-gain/no-target; zero code written (read-only lane).
