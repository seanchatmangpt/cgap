# cgap-git — Git packfile v2 kernel

Zero-external-dep, `#![no_std]` Git packfile v2 + DAG kernel for the cgap
sovereign columnar substrate. Dual-register design: the git-compat register
(`write_pack_v2`, git-identical parsers, `git_object_oid`, SHA-1) and the
receipt register (`commit_blocks`, content-addressed block receipts) — one
real snapshot round-trips through both registers with cross-register OID-set
binding, and any one-byte mutation moves the encoded root address.

## Status

PUBLISHED-26.11.1 — on crates.io (see
`docs/jira/v26.11.1/waves/p3-api-verify-all.md`: all five kernel crates
PASS at max_version 26.11.1, yanked=false; earlier f3-15 sweeps documented
26.11.1 absent before the publish legs landed).

## Evidence (cited receipts)

- Suite 88/0: `cargo test -p cgap-git` exit 0
  (`waves/a03-finish-receipt.md`).
- Three mutation flips, all targeting real courts, sha256 byte-exact
  restores (`waves/a03-finish-receipt.md`): SHIM-OFFSET (d9 flipped + 3
  wire-leg collateral, 56/60), SHIM-STABILITY (d1 flipped, 56/60),
  SHIM-PARENT (exactly d3 + d8 flipped, 58/60, isolated).
- Dual-register round-trip court `facade_court_both_registers_round_trip`
  (pack.rs) — both content-addressing registers on one snapshot.

## Install

```bash
cargo add cgap-git@26.11.1
```

## Lane receipt

`docs/jira/v26.11.1/waves/a03-finish-receipt.md` (lane A03) and
`waves/a03-wave5-audit-receipt.md`; registry witness
`waves/p3-api-verify-all.md`.
