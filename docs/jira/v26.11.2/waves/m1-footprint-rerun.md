# m1-footprint: Tier-0 footprint ladder re-run (Measure lane)

Method of record: `docs/jira/v26.11.2/waves/p2-footprint-flash-ladder.md`.
Exact subject /Users/sac/cgap, branch main (clean at lane start).
llvm-tools added for pinned nightly: `rustup component add llvm-tools` exit 0
("up to date"). Active: rustc 1.101.0-nightly (12433c307 2026-10-24),
nightly-aarch64-apple-darwin (floating nightly per rust-toolchain.toml).

## Build ladder (CARGO_TARGET_DIR=/tmp/m1-footprint-target, profile audit)

| command | exit |
|---|---|
| `cargo build --profile audit -p cgap-gate --target riscv32imc-unknown-none-elf` | 0 |
| `cargo build --profile audit -p cgap-arrow -p cgap-parquet --target wasm32-unknown-unknown` | 0 |
| `cargo build --profile audit -p cgap-blake3 --target riscv32imc-unknown-none-elf` | 0 |
| `cargo build --profile audit -p cgap-p28 --target riscv32imc-unknown-none-elf` | 0 |
| `cargo build --profile audit -p cgap-core --target riscv32imc-unknown-none-elf` | 101 |

cgap-core BLOCKED (typed): 95 errors, 1x E0463 class — "`std` is required by
`cgap_core` because it does not declare `#![no_std]`"; stderr log kept at
/tmp/m1-cgap-core-riscv.log (scratch). Same std-dependent class as p2's
bn3/bont/storage rows. Compile-abort, not a test failure count.

## Per-number comparison vs receipted (p2)

| artifact | receipted | this run | verdict |
|---|---|---|---|
| gate .text (riscv32imc) | 940 | 940 | REPRODUCED |
| arrow wasm CODE sum | 85,411 | 84,951 | DRIFTED -460 (-0.54%) |
| parquet wasm CODE sum | 65,459 | 66,780 | DRIFTED +1,321 (+2.02%) |

Drift attribution: floating nightly moved (p2 ran on an earlier nightly);
audit profile is cgu=16/lto=false, so codegen layout shifts with the
compiler. No repo source changed between runs (git clean).

## Tier-0 census additions (new measurements, no prior receipt)

| artifact | .text sum (B) | .rodata sum (B) |
|---|---|---|
| cgap-blake3 rlib, riscv32imc | 113,360 | 388 |
| cgap-p28 rlib, riscv32imc | 7,140 | 1,212 |
| cgap-gate rlib, riscv32imc | 940 | 0 |

Dominant blake3 object: `hash_wide_simd` 50,088 B; `hash` 30,176 B;
`hash_many_simd_x2` 27,710 B.

## Gate alloc/panic census (llvm-nm)

7 symbol lines, 0 matching alloc|panic -> alloc=0 / panic=0 REPRODUCED
(single T symbol `_RNvNtCs1RYQQXcmVLe_9cgap_gate8penstock7ingress`).

## Anti-vacuity witness

Truncated copy of gate rlib (truncate -s 800, /tmp/m1-truncated.rlib):
llvm-size exit 1, .text sum 0 (vs 940 full) — the tool reads the real
artifact, census is not vacuous. Copy removed after witness.

## Commands (all this session, exits witnessed)

1. `rustup component add llvm-tools` — exit 0 (up to date)
2. builds per ladder table above (exits 0/0/0/0/101)
3. `llvm-size -A` per rlib, awk CODE/.text sums; `llvm-nm` census on gate
4. truncation witness exit 1 with .text=0

## Contract checks

No repo source edits; no git writes; no new crates; no sleeps; no mocks;
frozen pins untouched; lane lease /tmp/m1-footprint-target deleted at lane
end. Naming law: forbidden ecosystem word not introduced.

```json
{"lane":"m1-footprint","files_written":["docs/jira/v26.11.2/waves/m1-footprint-rerun.md"],"commands":["rustup component add llvm-tools (exit 0)","cargo build --profile audit -p cgap-gate --target riscv32imc-unknown-none-elf (exit 0)","cargo build --profile audit -p cgap-arrow -p cgap-parquet --target wasm32-unknown-unknown (exit 0)","cargo build --profile audit -p cgap-blake3 -p cgap-p28 --target riscv32imc-unknown-none-elf (exit 0, 0)","cargo build --profile audit -p cgap-core --target riscv32imc-unknown-none-elf (exit 101, typed BLOCKED E0463 std-dependent)","llvm-size -A sums + llvm-nm census (gate text=940 REPRODUCED, alloc/panic=0/0; arrow CODE=84951 DRIFTED -460; parquet CODE=66780 DRIFTED +1321; blake3 text=113360; p28 text=7140)","truncation anti-vacuity witness (llvm-size exit 1, text=0 vs 940)"],"courts_run":[],"standing":"PARTIAL","divergences":["arrow/parquet wasm CODE drifted from p2 receipt (-460 / +1321) under floating nightly; numbers are per-nightly, not stable pins","cgap-core cannot build riscv32imc (std-dependent, E0463) — typed BLOCKED, matches p2 bn3/bont/storage class"]}
```
