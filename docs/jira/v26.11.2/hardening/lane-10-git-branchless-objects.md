# Lane H10 Receipt: git envelope varint SWAR conversion (BR10)

Status: ALIVE (all commands run this session, exits cited below)

## Decision

- **Envelope decimal emitter: CHANGED.** The decimal length emission in
  `git_object_oid` (`crates/cgap-git/src/pack_write.rs:159`) went through
  `usize::to_string()` (alloc + core::fmt machinery, `alloc::string::ToString`
  import at :9). Replaced with `emit_u64_decimal`, a branch-light itoa
  2-digits-per-u64-division emitter into a 20-byte stack buffer. Byte-identical
  to `u64::to_string` for every u64 — pinned by the envelope court
  (boundaries + 256-value sweep, corpus seed `0xA20C_0FFE_E5A7_E000` frozen)
  and the real-git interop suite (`git_interop`, `lineage_workflow`,
  `dag::tests::d5` git-KAT, `fsck_court`, `packfile_fuzz`).
- **encode_object_header varint loop: NO-CHANGE.** Measured (host release,
  instructions between symbol and `Lfunc_end`): 102 baseline, 82 post (same
  file also carries the emitter change). ~3 insn/iteration, <=10 bytes per
  object header, cold per object; the pack checksum + real-git interop suite
  is the byte-pin. A SWAR transform would not beat the measured body.
- **NUL delimiter: NO-TARGET (recorded loudly).** BR10's finding confirmed:
  there is no delimiter scan — the NUL is emitted into the streaming SHA-1
  (`h.update(b"\0")`, pack_write.rs:173). Nothing to optimize.
- **OID equality: NO-CHANGE.** `Oid = [u8; 20]` (lib.rs:25) with derived
  PartialEq lowers to unrolled compares; the file-wide asm shows no `memcmp`
  in either function and behavior pinned by
  `oid_equality_is_structural` (envelope court).
- **Mid-flight defect caught by courts, fixed:** the first emitter slice was
  `&lenbuf[..n]` (leading zeros) — flipped `dag::tests::d5` git-KAT and the
  envelope court; fixed to `&lenbuf[20 - n..]`. Witness that the court set
  kills real emitter defects.

## Evidence (commands + exits, this session)

- `cargo test -p cgap-git` (full suite, post-change): **exit 0** — 61 lib +
  4 delta_refusal + 2 envelope_court + 2 fsck + 4 fuzz_codec + 3 git_interop +
  11 ladder_mutations + 6 large_offset + 1 lineage_workflow + 2 packfile_fuzz
  (perf/ladder_timing ignored). lineage_workflow real-git interop both
  directions green = the byte-exactness pin.
- Anti-vacuity mutant battery (digit-flip mutants in /tmp/h10-mutant sandbox
  copy of pack_write.rs, shared tree untouched, SHIM-COURT tagged):
  - 1-digit branch mutant (`v+1`): `d5_git_compat_and_frame_fields` FAILED
    (KAT flipped) — court alive.
  - 2-digit (v>9) branch mutant: `envelope_oid_matches_reference...` FAILED
    — court alive.
  - v>99 loop low-digit mutant (`(d+1)%10`): envelope court FAILED at size
    100 — court alive.
- Asm instruction counts (host aarch64 release, directives excluded, same
  counter both sides): `git_object_oid` 295 → 520 (SHA-1 inlining coupling;
  the win is the removed String alloc + fmt call, not size),
  `encode_object_header` 102 → 82.
- riscv32imc (nightly, `--no-default-features`): build exit 0;
  `git_object_oid` 289 (baseline) → 394 (post), `encode_object_header` 87 → 87.
- Purity: `nm` on riscv32imc release rlib: **0 alloc/panic/unwind symbols**.

## Court file

`crates/cgap-git/tests/envelope_decimal_court.rs` (new, 2 tests): byte-identity
of the envelope emitter vs std `to_string` reference over digit boundaries +
256 seed-derived sizes; structural OID equality pin.

## Divergences / notes

- Lane doc said "encode_object_header region ~:206-217" for the decimal
  emission; the decimal emitter actually lives in `git_object_oid` (:159).
  The :206-217 region is the pack-format binary varint (recorded NO-CHANGE).
- First d5/envelope-court failure exposed a real bug in my own first diff
  (leading-zeros slice); court killed it. Restored correctness before landing.
- Mid-court observation: cargo nightly build-dir layout puts test binaries
  under `build/<hash>/out`; unaffected behavior, noted for future lanes.
