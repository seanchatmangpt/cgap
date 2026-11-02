# Lane H11 — arrow bitmask SWAR count() + validate census (receipt)

Date: 2026-11-02. Repo: /Users/sac/cgap @ main. Lane-id: h11.
Build roots (leases): /tmp/h11-target, /tmp/h11-target-rv (deleted at end).

## 1. Diff

`crates/cgap-arrow/src/bitmask.rs`:
- `PresenceMask::count()` (:126-134 pre-change) replaced: per-byte
  `count_ones` loop -> word-wise u64 fold. Word-padded storage is
  structural: `mask_bytes` (:42-49) returns `ceil(cap/64)*8`, a
  multiple of 8, so `BYTES / 8` words cover the buffer with no tail
  byte. The byte->u64 fold is const-friendly (no `read_unaligned`,
  which is not const-stable, and the `[u8]` storage has alignment 1).
- Oracle retained verbatim: `PresenceMask::count_per_byte_oracle()`
  (`#[doc(hidden)] pub const fn`) — the pre-change per-byte form,
  used by the differential courts.
- Byte-exactness law: `count() == count_per_byte_oracle()` for every
  mask state (exhaustive small masks, word-boundary probes on
  PresenceMask<65,16>, seeded sweeps on Mask256/Mask1024 — frozen seed
  `0xBF58_476D_1CE4_E5B9`).

`crates/cgap-arrow/tests/bitmask_swar.rs` (NEW, 3 courts):
- C-H11-1 differential count-vs-oracle; C-H11-2 count() stateless +
  bytes untouched; C-H11-3 pair-law const pins.

## 2. Commands + exits (this session)

| command | exit |
|---|---|
| `cargo test -p cgap-arrow` (baseline, pre-change) | 0 — 189 green |
| `cargo test -p cgap-arrow` (post-change) | 0 — 192 green (189 + 3 new courts) |
| `cargo rustc ... --emit=asm` host probe (before/after) | 0 / 0 |
| cargo riscv32imc probe build + asm emit (before/after) | 0 / 0 |
| mutation court M-H11a (padding corrupted, pins intact) | compile-abort E0080 (const pins fire — layer 1) |
| mutation court M-H11b (padding + pins relaxed, sandbox) | 101 — FLIP: `left: 0, right: 1` |
| control court (clean module, same differential) | 0 — 6/6 pass |
| `sha256sum -c` shared bitmask.rs | 0 — shared tree byte-identical |

## 3. asm witness (loop elimination)

Method: /tmp/h11-probe with #[inline(never)] concrete instantiations
(Mask64/Mask256/Mask1024/Mask65), `cargo rustc --emit=asm`, aarch64
host + riscv32imc nightly (1.101.0-nightly, target re-verified).
Instruction counts per fn (excluding cfi/globl):

| fn | host before | host after | rv32imc before | rv32imc after |
|---|---|---|---|---|
| count_64 | 27 | 6 | 26 | 5 |
| count_65 | 33 | 9 | (n/a) | (n/a) |
| count_256 | 50 | 15 | 49 | 14 |
| count_1024 | 36 | 44 | 35 | 43 |

- count_64 host after body: `ldr d0; cnt.8b; addv.8b; fmov; ret` —
  one word popcount, loop eliminated (was 5 byte-popcounts).
- count_1024 was already fully unrolled NEON before (36 insns, zero
  branches); after is 44 unrolled insns, zero branches — same
  complexity class, word-aligned form. Loop ELIMINATION witnessed on
  the small/medium masks; the 128B mask was loop-free in both forms.
- rv32imc: no branches, no `__popcountsi2` libcall either side.
## 4. Anti-vacuity (shim-court discipline, /tmp sandbox only)

- M-H11a: `mask_bytes` corrupted to byte-padding
  (`cap.wrapping_add(7) >> 3`) with the const pins intact -> the
  module's own const asserts refuse the build (E0080,
  `mask_bytes(1) == 8` / `mask_bytes(65) == 16` fire). Defense layer 1.
- M-H11b: same corruption + pins relaxed in the sandbox copy ->
  runtime FLIP: `PresenceMask<70,9>`, bit 64 set: count() = 0, oracle
  = 1, `MUT-B-EXIT 101` — the differential court has teeth.
- Control: same differential court against the clean module -> 6/6
  pass, `CTRL-EXIT 0`. Non-vacuous in both directions.
- Shared tree integrity: `sha256sum -c` on bitmask.rs = OK (mutation
  never touched the checkout; restore law satisfied trivially by
  never mutating the shared copy).

## 5. validate census (item 2 of dispatch)

`cargo test -p cgap-arrow` post-change exit 0, 192 green, including
the A2P validate courts (`validate::tests::*` — length overflow,
unaligned base typed-error without dereference, error display — and
`tests/validation.rs`, 36 passed). No pre-existing failures.

## 6. BR3 rejection (recorded, with cite)

The dispatch item "constant-time validation without branches" is
REJECTED per BR3. A validator's branches ARE its typed refusals:
`crates/cgap-arrow/src/validate.rs` refuses via typed errors
(`BitmaskError::OutOfRange` pattern — `Err` returns on out-of-range /
length-overflow / unaligned-base), and removing those branches removes
the refusal capability itself. Branch-free validation is either (a)
vacuous (refusals still branch, hidden in a helper) or (b) an
unconditional-accept validator (fail-closed behavior destroyed). The
fail-closed 5-court behavior is intact and pinned by the suite (§5).

## 7. Standing

ALIVE for: count() word-wise rewrite, differential courts, asm
witness (host + riscv32imc), anti-vacuity flip, suite census.
Perf claims are TARGETs: no cycle counts measured this session —
instruction-count deltas above are structural, not wall-clock.
UNVERIFIED: anything not named above.

## Files

- crates/cgap-arrow/src/bitmask.rs (modified)
- crates/cgap-arrow/tests/bitmask_swar.rs (new)
- docs/jira/v26.11.2/hardening/lane-11-arrow-swar-validate.md (this)
