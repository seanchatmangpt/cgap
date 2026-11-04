# cgap-fibo

FIBO O-ABI pack for the cgap sovereign columnar substrate: a
build-time-emitted binary ontology pack plus a `#![no_std]` const/types
facade. Lane A09
(`docs/jira/v26.11.1/lanes/A09-fibo-oabi.md`).

## What ships

- `fibo_sec_dbt_v1.oabi` (496 B, byte-stable) — emitted binary ontology
  container. Verified on disk this session: `ls -l` -> 496 bytes.
- `src/lib.rs` — `#![no_std]` facade, zero dependencies: const token
  tables, slot goldens, equivalence table, metadata only. No alloc, no
  lazy statics (DoD 1).
- `src/bin/emit-pack.rs` — build-time emitter. Harvester + collision-vet
  through A06's real `CollisionDetector`, emits A06's real container
  types, and validates its own output through A06's real loader before
  writing (Chicago: real collaborators end to end). Collision-checked
  against the BLAKE3 harvester (`cgap-blake3`, build-time dep).
- `tests/` — 5 files, 21 tests / 0 failures, witnessed this session:
  `cargo test` -> exit 0 (wave1 1+6, wave4 9, wave8 1, wave9 4).

## Status

- Publish: UNPUBLISHED at time of writing. Live check this session:
  `curl -A cgap-publish-receipt https://crates.io/api/v1/crates/cgap-fibo`
  -> `crate cgap-fibo does not exist` (404). Cargo.toml carries
  `publish = true` (P3-8 flip, 2026-11-01); landing in flight.
- Standing: ALIVE (build + 21/0 test run this session, exit 0).

## See Also

- Lane spec: `docs/jira/v26.11.1/lanes/A09-fibo-oabi.md`
- Sibling pack: `ontology-packs/cloud` (cgap-cloud, same build/runtime split)
- Frozen byte-pin: license URN
  `urn:avec:license:p28-sil:v2.1.0-sovereign` — never sweep.
