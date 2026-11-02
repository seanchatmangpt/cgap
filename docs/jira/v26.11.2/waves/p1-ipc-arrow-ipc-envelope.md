# p1-ipc — Arrow interchange envelope in cgap-arrow

Lane p1-ipc, v26.11.2 campaign. New module `crates/cgap-arrow/src/ipc.rs`
(inside the existing crate — no new crates), registered in the embedded
(no-OS) surface block of `lib.rs` alongside `slab`.

## What landed

`src/ipc.rs` — Arrow IPC streaming-format encapsulated-message envelope,
`no_std`, zero external deps, zero alloc (emit/read on caller buffers):

- Wire layout: `<continuation 0xFFFFFFFF><metadata_size u32 LE><metadata
  + zero pad so the body starts alignment-aligned><body + zero pad to
  alignment><EOS 0xFFFFFFFF 0x00000000>` (format/Message.fbs
  "Encapsulated message"; format/IPCStreaming.rst stream format).
- `MetaHeader` (64 B, `repr(C)`, static offset asserts): magic `"IPC1"`,
  version, flags, **cgap O-ABI schema digest** (32 B, BLAKE3 donor
  `crate::blake3`) at 0x08, body_len, alignment (8 or 64), seq,
  FNV-1a header digest over 0x00..0x38.
- Emitter/reader with typed refusal per corrupt framing class:
  `BadContinuation`, `BadMetaSize`, `Truncated`, `BadMagic`,
  `HeaderDigestMismatch`, `SchemaMismatch{got: Some(byte_idx)}`,
  `BadBodyLen`, `BadAlignment`, `PadNonZero{at}`, `MissingEos`.
- Framing arithmetic: `meta_span_for(8)=64`, `meta_span_for(64)=120`
  (body starts at 72 / 128, alignment-aligned), `window_len`.

## Recorded scope divergence (honest scope note)

Full IPC **flatbuffer** `Message`/`Schema` metadata is DEFERRED: the
metadata payload is the cgap MetaHeader, schema identity carried as the
O-ABI digest. An external Arrow IPC reader is expected to accept the
framing (continuation/metadata_size/EOS, per Message.fbs encapsulated
message + IPCStreaming.rst) and to REFUSE the metadata payload as an
unknown flatbuffer (typed parse failure there, never a misread).
No external Arrow reader was run against the wire (UNSUPPORTED:
no arrow-rs/arrow-cpp in the dependency closure — zero-dep law).
Flatbuffer-in-behind-current-signatures is the follow-on lane.

## Courts (tests/ipc_courts.rs, 9 tests, all pass)

- C1/C1b byte-layout: real wire window asserts (continuation at 0x00,
  metadata_size 64/120, header fields at pinned offsets, body start
  72/128 alignment-aligned, zero padding, EOS tail).
- C2/C2b roundtrip: real sealed 4-row slab (`ArrowRecordSlab::<4>`:
  push x4, seal) -> emit -> read -> `from_bytes` -> byte-identical
  slab, sealed, len 4; alignment 8 and 64.
- C3 differential vs the real generated `include/cgap_c_data.h`:
  struct ladder (`ArrowArray`/`ArrowSchema` field order) and flag
  macros present with the values the Rust c_abi module pins.
- C4 mutation court: corrupting each of the 4 continuation bytes
  (XOR 0xFF) -> typed `BadContinuation` with the exact observed value;
  EOS continuation corrupted -> `MissingEos`.
- C4b: schema-digest byte flip -> `SchemaMismatch{Some(17)}`; nonzero
  body pad -> `PadNonZero{at exact offset}`; metadata mutation ->
  header-digest or schema refusal; truncated window -> typed refusal.
- C5 full sweep: every single-byte increment over prefix+header
  (0..72) maps to SOME typed error — zero silent accepts.
- C6: emit capacity/shape faults typed (Truncated / BadBodyLen /
  BadAlignment) — no panic on inputs.

## Source-level shim court (non-vacuity of the continuation gate)

/tmp sandbox `/tmp/p1-ipc-shim` (de-workspaced Cargo.toml, dev-deps
stripped, C header copied to /tmp/p1-ipc-include):

- BASELINE: `cargo test --test ipc_courts` -> `9 passed; 0 failed`,
  ipc.rs sha256 `0c3474f946e34456896381a661c3fc44f56dcae55f2a0920d777d97fa75d765a`.
- SHIM (marked SHIM-COURT): `if cont != CONTINUATION` weakened to
  `if false && ...` -> `c4_corrupt_continuation_refused` FAILED,
  exit 101 (`test result: FAILED. 0 passed; 1 failed`).
- RESTORE: byte-exact copy back, sha256 match, mtime bump (BSD `touch`).

A gate that accepts the corrupt frame is a VACUOUS gate; the court flips.

## Standing

- Module + courts: ALIVE (commands + exits this session, above).
- Full Arrow IPC flatbuffer compat: UNSUPPORTED (deferred; see
  divergence note) — the framing layer is what landed.
- Embedded target: riscv32imc-unknown-none-elf build green
  (`cargo build -p cgap-arrow --target riscv32imc-unknown-none-elf`,
  exit 0) — ipc.rs rides the alloc-free embedded surface.

## Files written

- `crates/cgap-arrow/src/ipc.rs` (new)
- `crates/cgap-arrow/src/lib.rs` (one line: `pub mod ipc;` in the
  embedded-surface block)
- `crates/cgap-arrow/tests/ipc_courts.rs` (new)

## Verification ladder (commands + exits)

- `CARGO_TARGET_DIR=/tmp/p1-ipc-target cargo build -p cgap-arrow` exit 0
- `CARGO_TARGET_DIR=/tmp/p1-ipc-target cargo test -p cgap-arrow` exit 0:
  lib 134 passed / 0 failed (pre-existing suites unchanged), ipc_courts
  9 passed / 0 failed, plus existing integration suites green.
- `cargo build -p cgap-arrow --target riscv32imc-unknown-none-elf` exit 0.
- Shim court: baseline exit 0 (9/9), shim exit 101 (flip), restore
  sha256-exact + mtime bump.

```json
{
  "lane": "p1-ipc",
  "files_written": [
    "crates/cgap-arrow/src/ipc.rs",
    "crates/cgap-arrow/src/lib.rs",
    "crates/cgap-arrow/tests/ipc_courts.rs",
    "docs/jira/v26.11.2/waves/p1-ipc-arrow-ipc-envelope.md"
  ],
  "commands": [
    "CARGO_TARGET_DIR=/tmp/p1-ipc-target cargo build -p cgap-arrow (exit 0)",
    "CARGO_TARGET_DIR=/tmp/p1-ipc-target cargo test -p cgap-arrow (exit 0; lib 134/0, ipc_courts 9/0)",
    "cargo build -p cgap-arrow --target riscv32imc-unknown-none-elf (exit 0)",
    "/tmp/p1-ipc-shim: cargo test --test ipc_courts baseline exit 0 (9/9); SHIM-COURT exit 101 (FAILED 1); sha256-exact restore + mtime bump"
  ],
  "courts_run": [
    "c1_byte_layout", "c1b_body_start_is_64b_aligned_in_64_mode",
    "c2_roundtrip_4row_slab", "c2b_roundtrip_align8",
    "c3_differential_vs_generated_c_header",
    "c4_corrupt_continuation_refused", "c4b_schema_and_pad_mutations_refused",
    "c5_prefix_header_byte_sweep_all_typed", "c6_emit_typed_faults",
    "SHIM-COURT: continuation-gate weakening flip (exit 101)"
  ],
  "standing": "ALIVE (framing layer; full IPC flatbuffer compat UNSUPPORTED/deferred)",
  "divergences": [
    "IPC flatbuffer Message/Schema metadata deferred; cgap MetaHeader with O-ABI schema digest instead (documented in src/ipc.rs module docs)",
    "No external Arrow IPC reader run against the wire (zero-dep law; UNSUPPORTED)",
    "C3 court asserts header content as text (struct ladder + flag macro values), not a compiler-level ABI diff"
  ]
}
```
