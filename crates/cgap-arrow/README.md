# cgap-arrow — Arrow C Data Interface kernels

Zero-external-dep, `#![no_std]` Apache Arrow C Data Interface kernels for the
cgap sovereign columnar substrate. Columnar transforms (mask/filter/transpose)
over a sealed slab plane; built to run on host and bare metal.

## Status

WORKING-26.11.5 — kernel source at workspace version
26.11.5 (publish law: one attempt per crate, sequenced; prior
PUBLISHED-26.11.1 record archived under readme-archive/). Census: see
`docs/jira/v26.11.2/waves/crate-reality/cgap-arrow.md`: full suite
exit 0 this session at 26.11.2.

## Evidence (cited receipts)

- 30,157 B riscv32imc release rlib (blake3::hash 27,914; seal_with_mask x3;
  .rodata 1) — `waves/gate-5-silicon.md` (pre-link object; linked-ELF
  authoritative).
- Purity court 0/0: 0 alloc-family, 0 panic-family symbols over the real
  riscv32imc release rlib — `waves/a01-finish-receipt.md`.
- Host suite: 142/0 at 26.11.2 —
  `docs/jira/v26.11.2/waves/crate-reality/cgap-arrow.md`
  (134 unit + 1 mutation-kill + 1 property + 6 validation).
- Bare-metal `riscv32imc-unknown-none-elf` build exit 0, ELF symbol audit
  clean — `waves/a01-finish-receipt.md`.

## Deprecation notice: `nested-types` feature (26.11.2)

The `nested-types` feature (`pub mod timestamps` + the `TimestampMicro`/
`TimestampMilli`/`TimestampNano`/`TimestampSecond` re-exports) is DEFAULT ON
for one release cycle as a semver remedy — a v26.11.2 draft moved them behind
a default-off gate, which would have removed API reachable under 26.11.1
default features (BREAKING; audit `docs/jira/v26.11.2/audit/
pub-04-semver-delta.md`).

**This feature is deprecated for removal in the NEXT MAJOR.** Migrate off the
`Timestamp*` re-exports now (gate them explicitly via
`features = ["nested-types"]` or inline the timestamp constructors) so the
major cut that returns `default = []` and drops the module is non-breaking
for you.

## Install

```bash
cargo add cgap-arrow@26.11.2
```

## Lane receipt

`docs/jira/v26.11.2/waves/crate-reality/cgap-arrow.md` (26.11.2 census);
historical: `docs/jira/v26.11.1/waves/a01-finish-receipt.md`,
`waves/a01-wave9-receipt.md`, `waves/gate-5-silicon.md`,
`waves/p3-api-verify-all.md`.
