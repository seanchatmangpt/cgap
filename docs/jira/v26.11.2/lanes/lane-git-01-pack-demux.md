# Lane GIT-01 Receipt: pack_demux (v26.11.2)

Standing: **ALIVE** (courts ran this session, exact commands + exits below).

## Files written (complete ownership list)

- `crates/cgap-git/src/pack_demux.rs` (new, ~430 lines)
- `crates/cgap-git/tests/pack_demux_court.rs` (new, ~350 lines)
- `docs/jira/v26.11.2/lanes/lane-git-01-pack-demux.md` (this receipt)

Nothing else touched: `lib.rs`, `pack.rs`, `pack_write.rs`, `delta_resolver.rs`
and all other crates/files are read-only to this lane. `lib.rs` promotion
(`pub mod pack_demux;`) is deliberately NOT landed (lib.rs contract-off-limits);
the module is courted via the census pattern `#[path]` include in the court
file, same pattern the pack facade documents at pack.rs:11-16.

## Reused vs added (drift-law statement)

REUSED (no redefinition): pack header byte law mirrors
`pack_write::parse_pack_v2` (pack_write.rs:448-451); object varint header law
mirrors private `read_varint_header` (pack_write.rs:416-438, delta types
excluded); pack trailer verified in court via the existing public
`pack_write::sha1`; test-side object-header encoding reuses the public
`encode_object_header`. ADDED: OFS_DELTA biased offset varint decode
(first byte bias-free, continuation bytes `((acc+1)<<7)|low7`), REF_DELTA
20-byte base capture, streaming byte-at-a-time state machine.
Stated divergence: pack_write REFUSES type 6/7 by design (writer emits no
deltas); the demuxer DECODES 6/7 — that is its reason to exist — so the
private parser could not be reused for delta records.

## Commands + exits (this session)

- `cargo fmt -p cgap-git` — exit 0.
- `CARGO_TARGET_DIR=/tmp/git01-target cargo test -p cgap-git --test pack_demux_court` —
  **10 passed; 0 failed** (exit 0).
- `cargo test -p cgap-git --lib` — **61 passed; 0 failed** (exit 0).
- Per-target sweep of all other cgap-git suites (each exit 0): delta_refusal 4,
  envelope_decimal 2, fsck 2, fuzz_codec 4, git_interop 3, ladder_mutations 11,
  ladder_timing_stress 0+1 ignored (loud), large_offset 6, lineage_workflow 1,
  packfile_fuzz 2, perf_baseline 0+1 ignored (loud), object_store 6,
  ingress_ring 6, mcp_driver 6.
- Full `cargo test -p cgap-git` (compile-all) — exits nonzero ONLY on the
  sibling lane's in-flight untracked file (see divergences).

## Courts (each names its anti-vacuity twin)

- C1/C2 magic twin: real 3-object pack demuxes (C1); PACK->XACK byte
  mutation flips to typed `DemuxError::SubjectMissing` (exit-2-class), C2.
- C3/C3b OFS chain twin: hand-computed offsets 12/17/25, trailer at 53,
  pack len 73 asserted byte-exact; flipping the offset varint byte
  (12^1=13) moves the decoded base (C3b proves the decode is live).
- C4 stride invariance: FACT_WINDOW (1,365) strides inside 32 KiB windows
  == single-shot feed, byte-identical records; trailer captured == REUSED
  `pack_write::sha1` over pack content.
- C5/C5b multi-byte varint twin: base offset 200 ([0x80 0x48]) decodes to
  200; flipping the varint byte moves the decoded value.
- C6 truncation twin: phantom 4th object -> `finish()` returns
  typed `Truncated`, never fake Done.
- C7 zero-alloc instrument: tracking global allocator counts ZERO alloc
  calls across Demux::new + full feed + fixed-table drain of the 3-object
  pack; flip twin proves the counter moves (builder allocates, >0).
  Stated instrument: tracking-alloc count, NOT a static claim.
- C8 core-only source scan: no `std::`/`alloc::`/heap types in the module.

## Fences run (must-not-regress surfaces)

All existing cgap-git suites re-run per target (counts above), plus lib
unit tests 61/0. Only failure in the crate is the sibling lane's file.

## Divergences

1. `soc_full_chip_court.rs` + `delta_resolver.rs` are ANOTHER lane's
   in-flight untracked files; at receipt time soc_full_chip_court FAILS at
   its own soc_full_chip_court.rs:168 (copy_from_slice 4 vs 12) —
   pre-existing relative to this lane (files untracked, not owned here,
   untouched). Compile-abort churn on that target earlier in the session
   was the owner lane mid-edit; compile-abort is not a failure count.
2. FACT_WINDOW provenance: no prior constant exists in the repo; named as
   a parameter with coprimality rationale (1,365 = 3x5x7x13, coprime with
   12/20/64-byte structural strides), no lore invented.
3. Module not wired into lib.rs (contract); promotion is a one-line
   coordinator transition.

## Cleanup

Build-root lease `/tmp/git01-target` deleted at lane end (below), per the
build-root lease law.
