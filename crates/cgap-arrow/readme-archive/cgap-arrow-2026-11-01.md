# ARCHIVED cgap-arrow sha256 bc4fad5a9ee0c6f85ec025fdea5247c519f17a87977130426b4c8738f5c60c81

# cgap-arrow — Arrow C Data Interface kernels

Zero-external-dep, `#![no_std]` Apache Arrow C Data Interface kernels for the
cgap sovereign columnar substrate. Columnar transforms (mask/filter/transpose)
over a sealed slab plane; built to run on host and bare metal.

## Status

PUBLISHED-26.11.1 — on crates.io (see
`docs/jira/v26.11.1/waves/p3-api-verify-all.md`: all five kernel crates
PASS at max_version 26.11.1, yanked=false; earlier f3-15 sweeps documented
26.11.1 absent before the publish legs landed).

## Evidence (cited receipts)

- 30,157 B riscv32imc release rlib (blake3::hash 27,914; seal_with_mask x3;
  .rodata 1) — `waves/gate-5-silicon.md` (pre-link object; linked-ELF
  authoritative).
- Purity court 0/0: 0 alloc-family, 0 panic-family symbols over the real
  riscv32imc release rlib — `waves/a01-finish-receipt.md`.
- Host suite: 177/0 — `waves/a01-finish-receipt.md` (session receipt cites
  "host 177/0, riscv exit 0, symbol audit").
- Bare-metal `riscv32imc-unknown-none-elf` build exit 0, ELF symbol audit
  clean — `waves/a01-finish-receipt.md`.

## Install

```bash
cargo add cgap-arrow@26.11.1
```

## Lane receipt

`docs/jira/v26.11.1/waves/a01-finish-receipt.md` (lane A01) and
`waves/a01-wave9-receipt.md`; silicon gate `waves/gate-5-silicon.md`;
registry witness `waves/p3-api-verify-all.md`.
