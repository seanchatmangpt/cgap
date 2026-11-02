# cgap-blake3

# cgap-blake3 — BLAKE3 hash kernel

Zero-external-dep, `#![no_std]` BLAKE3 hash kernel for the cgap sovereign
columnar substrate. Implements the full BLAKE3 suite — extendable-output
hashing, keyed hashing, and key derivation — against the canonical
`test_vectors.json` recipe: known-answer tests cover every published length,
plus small-batch and multi-chunk vector suites, a thread differential, and a
zero-alloc court.

## Status

WORKING-26.11.2 — kernel source at workspace version
26.11.2 (publish law: one attempt per crate, sequenced; prior
PUBLISHED-26.11.1 record archived under readme-archive/). Census: see
`docs/jira/v26.11.2/waves/crate-reality/cgap-blake3.md`: full suite
exit 0 this session at 26.11.2.

## Evidence (cited receipts)

- Suite green 27/0: `cargo test -p cgap-blake3` exit 0
  (`docs/jira/v26.11.2/waves/crate-reality/cgap-blake3.md`).
- KAT corpus: `tests/official_vectors.rs` follows the canonical
  BLAKE3-team `test_vectors.json` recipe; `multi_chunk_vectors.rs`,
  `small_batch_vectors.rs`, `thread_differential.rs`, `zero_alloc.rs`.
- 5 real test suites; a 564 B print-only `bench_tmp.rs` was purged with a
  sha256-fail-closed receipt (`waves/r12-bench-tmp.md`).

## Install

```bash
cargo add cgap-blake3@26.11.2
```

## Lane receipt

`docs/jira/v26.11.2/waves/crate-reality/cgap-blake3.md` (26.11.2 census);
historical: `docs/jira/v26.11.1/waves/r12-bench-tmp.md`,
`waves/f3-09-pub-blake3.md`, `waves/p3-api-verify-all.md`.
