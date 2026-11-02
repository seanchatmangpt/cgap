# ARCHIVED cgap-parquet sha256 9526e415552cfd05552954c616863c5d206ed2d210cfb06e6b3c4cdf64152529

# cgap-parquet — Parquet persistence kernel

Zero-external-dep, `#![no_std]` Parquet persistence kernel for the cgap
sovereign columnar substrate. RLE hybrid encode/decode with a sealed
verify-chain: `verify_seal` -> `parse_meta` recompute with a bounded meta
Vec, then a word-bounded direct decode stage.

## Status

PUBLISHED-26.11.1 — on crates.io (see
`docs/jira/v26.11.1/waves/p3-api-verify-all.md`: all five kernel crates
PASS at max_version 26.11.1, yanked=false; earlier f3-15 sweeps documented
26.11.1 absent before the publish legs landed).

## Evidence (cited receipts)

- 0 B alloc decode: `rle_decode_direct` (65,536 lanes) measures 0 B alloc;
  positive control `rle_decode_staged` >0 B (not vacuous) —
  `waves/bench-chains-parquet.md` (divan + AllocProfiler).
- Direct-vs-staged: staged overhead vs direct ~+27% (fastest 63.28 vs
  49.95 µs) — `waves/bench-parquet-receipt.md`.
- Suite: 216 tests 0 failed (default) and 236 tests 0 failed
  (`--features std`) — `waves/a02-finish-receipt.md`.
- `rle_decode_direct` 46.45 µs — `waves/bench-chains-parquet.md`.

## Install

```bash
cargo add cgap-parquet@26.11.1
```

## Lane receipt

`docs/jira/v26.11.1/waves/a02-finish-receipt.md` (lane A02) and
`waves/bench-parquet-receipt.md`; registry witness
`waves/p3-api-verify-all.md`.
