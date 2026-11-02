# m2 no_std census (v26.11.2)

Target: riscv32imc-unknown-none-elf, CARGO_TARGET_DIR=/tmp/m2-target. Date 2026-11-02. Command per crate: `CARGO_TARGET_DIR=/tmp/m2-target cargo build -p <crate> --target riscv32imc-unknown-none-elf`.

| crate | class | exit | err count | #![no_std] | top std symbols / notes |
|---|---|---|---|---|---|
| cgap-audit | STD-DEPENDENT | 101 | 272 | 0 | result/assert/option macros, derive, write!; E0432 unresolved cgap_arrow::bitmap/silicon/views (cfg'd modules absent without std) |
| cgap-arrow | NO_STD-CLEAN | 0 | 0 | 1 | extern crate alloc |
| bench | STD-DEPENDENT | 101 | 717 | 0 | result/assert macros, io::Write, fmt::Write, option macros |
| cgap | STD-DEPENDENT | 101 | 2 | 0 | transitive via cgap-git (extern crate std) |
| cgap-beam | TARGET-UNSUPPORTED | 101 | 1 | 3 | cdylib crate-type unsupported by riscv32imc (linking surface, not std) |
| cgap-blake3 | NO_STD-CLEAN | 0 | 0 | 1 |  |
| cgap-bn3 | STD-DEPENDENT | 101 | 438 | 0 | core::result/assert macros, io::, fmt::, derive |
| cgap-bont | STD-DEPENDENT | 101 | 409 | 0 | core::result/option macros, derive, assert, io:: |
| cgap-bosym | STD-DEPENDENT | 101 | 99 | 0 | core::result/assert macros, write!, derive |
| cgap-core | STD-DEPENDENT | 101 | 96 | 0 | result/assert macros, mem::offset_of, io::, fmt:: |
| cgap-court | NO_STD-CLEAN | 0 | 0 | 0 | test-only [[test]] court harness; lib builds clean |
| cgap-crypto | NO_STD-CLEAN | 0 | 0 | 1 | extern crate alloc |
| cgap-ffi | NO_STD-CLEAN | 0 | 0 | 1 | extern crate alloc |
| cgap-gate | NO_STD-CLEAN | 0 | 0 | 1 | extern crate alloc |
| cgap-git | STD-DEPENDENT | 101 | 1 | 1 | explicit `extern crate std;` at src/lib.rs:13 despite #![no_std] |
| cgap-gpu | HOST-ONLY(build-tooling) | 101 | 1 | 0 | cc-rs build script: `failed to find tool "riscv32-unknown-elf-gcc"` — cross C toolchain absent, not a std error |
| cgap-p28 | NO_STD-CLEAN | 0 | 0 | 1 | extern crate alloc |
| cgap-parquet | NO_STD-CLEAN | 0 | 0 | 1 | extern crate alloc |
| cgap-storage | STD-DEPENDENT | 101 | 180 | 0 | result/assert macros, option macros, mem::offset_of, derive |
| cgap-integration | STD-DEPENDENT | 101 | 3 | 0 | E0463 std + prelude import (lib.rs) |
| wasm-footprint | TARGET-UNSUPPORTED | 101 | 1 | 3 | cdylib crate-type unsupported by riscv32imc; lib-only build ALSO resolves cdylib (rc 2 on --lib rerun) |

Standalone crates built from own dir: cli (rc=101, target-unsupported via cgap-beam cdylib dep), wasm-footprint (above).
D5 strip backlog baseline: STD-DEPENDENT set = {cgap-core, cgap-bont, cgap-bosym, cgap-bn3, cgap-storage, cgap-git, cgap-audit, cgap-integration, bench, cgap(transitive)}. Dominant error categories: prelude import (std prelude), assert!/debug_assert! family, io::Write/fmt::Write usage, derive attribute resolution, mem::offset_of (cgap-core 13, cgap-storage 20).
