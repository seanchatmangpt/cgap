# cgap-parquet — Parquet persistence kernel

Zero-external-dep, `#![no_std]` Parquet persistence kernel for the cgap
sovereign columnar substrate. RLE hybrid encode/decode with a sealed
verify-chain: `verify_seal` -> `parse_meta` recompute with a bounded meta
Vec, then a word-bounded direct decode stage.

## Status

WORKING-26.11.2 — kernel source at workspace version
26.11.2 (publish law: one attempt per crate, sequenced; prior
PUBLISHED-26.11.1 record archived under readme-archive/). Census: see
`docs/jira/v26.11.2/waves/crate-reality/cgap-parquet.md`: full suite
exit 0 this session at 26.11.2.

## Evidence (cited receipts)

- 0 B alloc decode: `rle_decode_direct` (65,536 lanes) measures 0 B alloc;
  positive control `rle_decode_staged` >0 B (not vacuous) —
  `waves/bench-chains-parquet.md` (divan + AllocProfiler).
- Direct-vs-staged: staged overhead vs direct ~+27% (fastest 63.28 vs
  49.95 µs) — `waves/bench-parquet-receipt.md`.
- Suite: 222 tests 0 failed (1 loud ignored perf court) at 26.11.2 —
  `docs/jira/v26.11.2/waves/crate-reality/cgap-parquet.md`.
- `rle_decode_direct` 46.45 µs — `waves/bench-chains-parquet.md`.

## Install

```bash
cargo add cgap-parquet@26.11.2
```

## Lane receipt

`docs/jira/v26.11.2/waves/crate-reality/cgap-parquet.md` (26.11.2
census); historical: `docs/jira/v26.11.1/waves/a02-finish-receipt.md`,
`waves/bench-parquet-receipt.md`, `waves/p3-api-verify-all.md`.
