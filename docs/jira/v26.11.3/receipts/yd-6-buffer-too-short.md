# YD-6: rename `AoqError::HeaderTooLong` -> `AoqError::BufferTooShort`

XC-5r rec #4 / XD-5 TARGET-UNMEASURED follow-up. The variant fires when an
OUTPUT buffer is too short (`out.len() < SIZE_BYTES` in `codec.rs:36-40`,
`writer.rs` window path, `chain.rs` slab-full) — `HeaderTooLong` inverted
the semantics. Rename is Rust-surface-only: `docs/aoq-header-offsets.md`
does not mention the Rust variant name and the wire class code
`0x00_02` (`class_code()`, `error.rs:61`) is unchanged, so the frozen
FM-AOQ band is untouched.

Landed receipts are historical records; not edited. This receipt notes
the rename.

## Before census (grep -rn HeaderTooLong crates/, session start)

- `crates/cgap-aoq/src/error.rs:24,61,79` (variant, class_code arm, Display)
- `crates/cgap-aoq/src/writer.rs:140,151,192,206` (doc comments + 2 constructions)
- `crates/cgap-aoq/src/chain.rs:214,217` (doc comment + construction)
- `crates/cgap-aoq/src/codec.rs:35,38` (doc comment + construction)
- `crates/cgap-aoq/tests/cbmqs_runner.rs:403` (label string "HeaderTooLong")
- `crates/cgap-aoq/tests/chain_courts.rs:204`
- `crates/cgap-aoq/tests/macro_density_bench.rs:166`
- `crates/cgap-aoq/tests/aoq1_codec_court.rs:104`
- `crates/cgap-aoq/tests/ab8_courts.rs:330`
- docs/aoq-header-offsets.md: NO mention of the variant name (verified;
  the doc pins statuses FM-AOQ-001..003, not this Rust name).

## Changes

- Renamed variant + all construction/match/doc-display sites (sed across
  the 9 files above); Display text `header too long: {got}B` ->
  `buffer too short: {got}B`; runner label string -> `"BufferTooShort"`.
- Wire class code 0x00_02 unchanged. `got` field unchanged.
- docs/aoq-header-offsets.md: no change needed (zero mentions found —
  the task brief's "the variant's mention" does not exist on disk).

## Verification (this session)

- `grep -rn HeaderTooLong crates/ docs/aoq-header-offsets.md` -> 0
  matches (exit 1), run after edit.
- `cargo test -p cgap-aoq` run TWICE: 18 test-target lines all
  `test result: ok`, 0 failed in any target, both passes (incl. doc-tests).
- `cargo check --workspace` -> exit 0.

No Cargo.toml touched (YD-1 concurrent lane). No git commands run.
