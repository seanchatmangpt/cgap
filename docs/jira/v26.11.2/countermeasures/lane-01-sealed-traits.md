# Lane AE1 — sealed production traits (v26.11.2 countermeasure)

Standing: ALIVE (all commands run this session, exits below).

## What was sealed

- `crates/cgap-parquet/src/mmap_writer.rs` — `SpillSync` is now
  supertrait-bounded on a crate-private `private::Sealed` marker
  (`pub trait SpillSync: private::Sealed`). Out-of-crate impls are
  compiler-rejected (E0277).
- `crates/cgap-core/src/traits.rs` — ABSENT-honest: cgap-core has no
  extensible driver traits (`grep -rn "trait " crates/cgap-core/src/`
  returns zero matches; modules are dma_ring, frame, isomorphism,
  frame/lib). No file created; the seal is scoped to parquet only.

## Deliberately NOT sealed

- `BlockSink` stays open by design: its own docs name it the external
  backend attach seam ("a real `mmap` backend is attachable externally"),
  and `crates/bench/benches/chains-parquet.rs:138,159` implements it
  out-of-crate (`RamSink`, `BorrowedFileSink`). Sealing it would break
  the documented attach contract.

## Court preservation (H3 rotation court)

H3's court instrument is the in-crate `#[cfg(test)]` `RecordingSpill`
(aliased `SharedRec`), injected through the `pub(crate) fn with_dyn`
seam — both inside `cgap-parquet`, so the seal does not touch it. Added
`impl private::Sealed for SharedRec {}` next to its `SpillSync` impl.
Regression: `cargo test -p cgap-parquet --features std` exit 0
(194 passed, 0 failed; `fa9_rotation_drain_is_sync_durable_and_bytes_reach_the_spill ... ok`);
`cargo test -p cgap-ffi` exit 0 (50 passed + integration suites, 0 failed).

## Anti-vacuity court (out-of-crate rejection witness)

Sandbox: `/tmp/ae1-seal-court/` — copy of cgap-parquet (Cargo.toml
literalized, dev-deps dropped; `src/mmap_writer.rs` sha256-identical to
shared tree: `cbde701a66d1f9a3be2b67ab14666b6284d92938774a3972ac91084eadcf91b7`)
plus a dependent crate `mock-impl-attempt` doing
`impl SpillSync for OutOfCrateMock`.

- BASELINE + mutation in one step: the mock crate's only content IS the
  mutation. `cargo build` → exit 101, E0277:
  `SpillSync is a "sealed trait" ... required by this bound in SpillSync`,
  help names `std::fs::File` as the only impl. Log: `/tmp/ae1-reject.log`.
- The court flips: pre-seal the same mock compiles (the seal bound is the
  only gate); the compiler error text itself names the seal. Shared tree
  never mutated in the court; sha256-identical sandbox source confirmed
  above, so no mtime restore was required.

## Commands / exits

- `cargo test -p cgap-parquet --features std` → exit 0 (194 passed)
- `cargo test -p cgap-ffi` → exit 0 (50 passed, 0 failed)
- `cd /tmp/ae1-seal-court/mock && cargo build` → exit 101, E0277 (expected rejection)
