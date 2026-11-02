# cgap-cloud

Cloud ontology O-ABI pack for the cgap sovereign columnar substrate:
TOSCA/CIMI lexemes, build-time-emitted binary pack, `#![no_std]`
const/types facade. Lane A10
(`docs/jira/v26.11.1/lanes/A10-cloud-oabi.md`).

## What ships

- `tosca_cimi_v1.oabi` (1,552 B) — emitted binary ontology container.
  Verified on disk this session: `ls -l` -> 1552 bytes.
- `src/lib.rs` — `#![no_std]` facade, zero dependencies: lexeme table,
  slot goldens, equivalence table, metadata.
- `src/bin/emit-pack.rs` — build-time emitter; collision-vets through
  A06's real `CollisionDetector`, emits A06's container types, and
  validates its own output through A06's real loader before writing.
- `tests/` — `courts.rs`, `raw_consumer.rs`, `wire_parity.rs`;
  13 tests / 0 failures, witnessed this session: `cargo test` -> exit 0
  (8 + 1 + 4). The raw-bytes consumer court reads the `.oabi` container
  without the emitting crate in scope.

## Status

- Publish: PUBLISHED-26.11.1, live-witnessed this session:
  `curl -A cgap-publish-receipt https://crates.io/api/v1/crates/cgap-cloud`
  -> exit 0; `num_versions: 1`, `max_version: 26.11.1`, created
  2026-11-02T03:43:38Z.
- Standing: ALIVE (build + 13/0 test run this session, exit 0).

## See Also

- Lane spec: `docs/jira/v26.11.1/lanes/A10-cloud-oabi.md`
- Manufacturing receipt: `RECEIPT.md` (waves 4 + 6)
- Sibling pack: `ontologies/fibo` (cgap-fibo, same build/runtime split)
