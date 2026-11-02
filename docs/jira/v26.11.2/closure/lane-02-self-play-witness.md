# Lane CL2 - Self-Play Suite Witness (SP1-SP11)

Date: 2026-11-02. Repo /Users/sac/cgap, branch main. Read-only witness + test runs; build root CARGO_TARGET_DIR=/tmp/cl2-target (deleted post-run).
Verdict: PARTIAL - 11/12 test targets green (33 passed / 2 failed), RISC-V check exit 0; ONE FAILING TARGET (alloc_boundary, AE7 owner), reported unvarnished.

## Commands + exits

| Command (cwd /Users/sac/cgap) | Exit | Note |
|---|---|---|
| CARGO_TARGET_DIR=/tmp/cl2-target cargo test -p cgap | 101 | fail-fast run: stops after alloc_boundary FAILED (2 failed / 0 passed) |
| CARGO_TARGET_DIR=/tmp/cl2-target cargo test -p cgap --no-fail-fast | 101 | full census: 33 passed / 2 failed across 12 targets (/tmp/cl2-full3.log) |
| CARGO_TARGET_DIR=/tmp/cl2-target cargo check -p cgap --no-default-features --target riscv32imc-unknown-none-elf | 0 | re-witnessed post all lanes; matches SP1 landed exit 0 |
| CARGO_TARGET_DIR=/tmp/cl2-target cargo test -p cgap --benches --no-run | 0 | SP10/SP11 benches compile |

## SP-lane witness table (from --no-fail-fast census)

| Lane | Test target (crates/cgap/tests/) | Tests passed | Exit |
|---|---|---|---|
| SP1 (lib courts) | unittests src/lib.rs (pipeline::wiring_courts) | 4/4 | ok |
| SP2 | self_play_ingress_lease.rs | 2/2 | ok |
| SP3 | self_play_lease_arrow.rs | 2/2 | ok |
| SP4 | self_play_arrow_radix_join.rs | 4/4 | ok |
| SP5 | self_play_radix_dred_cascade.rs | 2/2 | ok |
| SP6 | self_play_datalog_parquet_seal.rs | 4/4 | ok |
| SP7 | self_play_parquet_git_commit.rs (header says SP7, file:1) | 2/2 | ok |
| SP8 | self_play_concurrency_spill.rs | 3/3 | ok |
| SP9 | self_play_full_closed_loop.rs | 3/3 | ok |
| SP10 | (bench) benches/bench_self_play_ingress_parquet.rs | n/a - bench, not in cargo test census | compiles (--benches --no-run exit 0) |
| SP11 | (bench) benches/bench_self_play_datalog_git.rs | n/a - bench, not in cargo test census | compiles (--benches --no-run exit 0) |
| AE7 | alloc_boundary.rs | 0/2 FAILED | FAILED |

Raw counts: 12 test targets, 35 tests executed, 33 passed, 2 failed, 0 ignored, 0 measured. Plus 0-test targets (bin enclave_self_play_entry, doc-tests) at 0/0 ok.

## The failure, unvarnished (AE7 owner: alloc_boundary / tracking_alloc)

- Target: crates/cgap/tests/alloc_boundary.rs - tests ae7_alloc_boundary_courts and ae7_positive_control_breaches, both FAILED.
- Panic 1: `crates/cgap/tests/alloc_boundary.rs:175:5: budget(0,0) must breach on a Vec alloc`
- Panic 2: `crates/cgap/../../cgap-core/src/tracking_alloc.rs:156:5: ALLOC BUDGET BREACH` (exact path as printed: crates/cgap/tests/../../cgap-core/src/tracking_alloc.rs:156:5)
- Transient compile-abort witnessed mid-session in the same file (E0425 OFF_MAGIC/OFF_OABI_DIGEST not found, 4 errors) - an in-flight lane edit; the file compiled on re-run. Compile-abort is NOT a failure count; the run's real failure count is the 2 panics above.

## Warnings census (verbatim subjects)

Native run (cargo test -p cgap):
1. cgap-branchless/src/swar.rs:22:11 - warning: use of deprecated module `core::u64`: all constants in this module replaced by associated constants on the type
2. cgap-branchless/src/swar.rs:25:7 - warning: constant `LO` is never used (dead_code)
3. cgap-gate/src/penstock.rs:208:21 - warning: unused imports: `String` and `ToString`
4. cgap-gate/src/penstock.rs:209:5 - warning: unused import: `alloc::vec::Vec`
5. cgap/src/pipeline.rs:65:5 - warning: fields `hash` and `pack_write` are never read (dead_code)
6. cgap/tests/alloc_boundary.rs:34:37 - warning: unused import: `na_unify_radix`
7. cgap/tests/self_play_arrow_radix_join.rs:37:57 - warning: unused import: `SIGIL_SHIFT`

RISC-V check (riscv32imc-unknown-none-elf, --no-default-features): same swar.rs and penstock.rs warnings plus bin enclave_self_play_entry generated 2 unused_variables warnings (cargo fix -p cgap --bin suggested).

## Standing

- PARTIAL. SP1-SP9 courts: ALIVE (ran this session, cited exits). SP10/SP11: benches only, compile-ALIVE, not executed (not part of cargo test census). AE7 alloc_boundary: FAILING - owner to repair; the panics above are the exact failure text.
- Build root /tmp/cl2-target deleted via shutil.rmtree at receipt close.
