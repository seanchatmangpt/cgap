# p2-footprint: flash-footprint ladder

## Method

- Target riscv32imc-unknown-none-elf, `--profile audit` (lto=false, cgu=16,
  strip=false -> real ELF objects, not thin-LTO bitcode).
- Target wasm32-unknown-unknown, same profile (wasm objects, CODE/DATA
  sections via llvm-size -A).
- llvm-size/llvm-nm from nightly llvm-tools-preview.
- CARGO_TARGET_DIR=/tmp/p2-footprint-target (lease; rmtree at lane end).

## riscv32imc build ladder (per-crate, exact subject /Users/sac/cgap)

| crate | exit | verdict |
|---|---|---|
| cgap-gate | 0 | builds clean no_std |
| cgap-bn3 | 101 | BLOCKED: 438 errors, E0463 can't find crate `std` |
| cgap-bont | 101 | BLOCKED: 372 errors, E0463 can't find crate `std` |
| cgap-storage | 101 | BLOCKED: 140 errors, E0463 can't find crate `std` |

bn3/bont/storage are std-dependent crates (`#![no_std]` absent from their
lib.rs); full cargo stderr was captured at /tmp/p2-<crate>-riscv.log during
the run (scratch logs since removed with the lane lease; the E0463 signature
is reproducible by rerunning command 2).

## Budget table (ceiling 32,768 B)

| artifact | size (B) | class | vs ceiling |
|---|---|---|---|
| libcgap_gate rlib, riscv32imc, .text | 940 | flash | UNDER (32,768-940 headroom) |
| libcgap_gate rlib, riscv32imc, rmeta | 143 | metadata | n/a (not code) |
| libcgap_arrow rlib, wasm32, CODE | 85,411 | wasm | OVER ceiling by 52,643 B [TARGET-v26.11.2]; wasm-class, not flash-class |
| libcgap_parquet rlib, wasm32, CODE | 65,459 | wasm | OVER ceiling by 32,691 B [TARGET-v26.11.2]; wasm-class, not flash-class |
| cgap-bn3/bont/storage riscv artifact | none | flash | typed BLOCKED (no artifact exists) |

cgap-gate .text = `penstock::ingress` (940 B, single CGU), verified by
llvm-nm: 7 symbol lines, one T symbol
`_RNvNtCs1RYQQXcmVLe_9cgap_gate8penstock7ingress`.

## Alloc/panic symbol census

- cgap-gate (riscv32imc): alloc=0, panic=0 (0/0 required for no_std: PASS).
- arrow/parquet (wasm32, no_std+alloc crates): alloc=101/114 symbol lines,
  panic=11/16 — `extern crate alloc` closure symbols compiled into the CGUs,
  not allocator instantiation; recorded as measured, not a no_std violation.

## Commands (all this session, exit codes witnessed)

1. `cargo build --profile audit -p cgap-gate --target riscv32imc-unknown-none-elf` exit 0
2. same for cgap-bn3/cgap-bont/cgap-storage: exit 101 each (typed BLOCKED)
3. `cargo build --profile audit -p cgap-arrow -p cgap-parquet --target wasm32-unknown-unknown` exit 0 both
4. llvm-size -A / llvm-nm on the three surviving rlibs

## Court status

No mutation court run this lane (measurement lane; no behavior claim to
flip). Anti-vacuity: the census 0/0 claim is falsifiable by the nm command
embedded in command 4; rerunning it on gate's rlib reproduces 0.

## Contract checks

Naming law: forbidden word absent from all files this lane wrote. No git
writes; no new crates; no sleeps; frozen pins untouched; DataFusion count 0.

```json
{"lane":"p2-footprint","files_written":["docs/jira/v26.11.2/waves/p2-footprint-flash-ladder.md"],"commands":["cargo build --profile audit -p cgap-gate --target riscv32imc-unknown-none-elf (exit 0)","cargo build --profile audit -p {cgap-bn3,cgap-bont,cgap-storage} --target riscv32imc-unknown-none-elf (exit 101 x3, typed BLOCKED E0463)","cargo build --profile audit -p cgap-arrow -p cgap-parquet --target wasm32-unknown-unknown (exit 0 x2)","llvm-size -A + llvm-nm on libcgap_gate.rlib (text=940, census 0/0), arrow/parquet wasm rlibs (CODE=85411/65459)"],"courts_run":[],"standing":"PARTIAL","divergences":["cgap-bn3/cgap-bont/cgap-storage cannot build riscv32imc (std-dependent, E0463) - typed BLOCKED per crate, not fixed this lane (no new features lane contract)","arrow/parquet wasm CODE sections exceed the 32,768 B Flash ceiling (85,411 / 65,459 B) - recorded as TARGET-v26.11.2, wasm-class not flash-class","pre-existing naming-law violation observed: rust-toolchain.toml contains the forbidden ecosystem word (not edited this lane; out of lane scope)"]}
```
