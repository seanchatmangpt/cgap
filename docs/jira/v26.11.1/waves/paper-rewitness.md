# paper-rewitness

PP-5 re-witness receipt, 2026-11-01, gate-runner lane, /Users/sac/cgap.
All commands first-hand this session, CARGO_TARGET_DIR=/tmp/pp5-target.

| claim | paper/receipt ref | this-session re-run | value | verdict |
|---|---|---|---|---|
| statement_scan_24b 82.24 GB/s (N=65536) | bench-bn3-receipt.md:69 | cargo bench -p bench --bench bn3, min-of-3 reps | fastest 81.88/81.70/81.70 GB/s | MATCH (band -0.5%, host-noise; cold first rep 67.89 under load, discarded) |
| statement_scan_24b 78.94 GB/s (N=1M) | bench-bn3-receipt.md:70 | same | fastest 79.00/78.88/78.60 GB/s | MATCH (±0.5%) |
| unify_join 6.499 ns/pair (pairs=4096) | bench-bn3-receipt.md:71 | same, N=1024 bench | fastest 26.62 us / 4096 = 6.499023 ns/pair | MATCH (exact) |
| p28 verify ~460 ns/attestation | challenger-sales-report.md (p28_attestation_verify) | cargo bench -p bench --bench seal | 471 us fastest / 1024 = 459.96 ns | MATCH; FFI direct C-ABI leg is a loud [SKIP] (exports not linked, receipted in-band) |
| rle_decode_direct 49.95 us, 0 alloc / 0 B | bench-parquet-receipt.md:77 | cargo bench -p bench --bench parquet (x2) | fastest 46.33/46.41 us; allocs 0, max alloc 0 B | MATCH (-7.3%, faster side of band) |
| cgap-arrow riscv32imc object-class 30,157 B | gate-5-silicon.md:15 | build --profile audit --target riscv32imc exit 0; llvm-size over rlib objects | 28,355 B: blake3::hash 26,047 (+.rodata 1) + seal_with_mask x3 = 2,242 + copy_from_slice 66 | DRIFT (named, non-fatal): blake3::hash 26,047 vs 27,914 = -6.7% (compile-output drift, tree moved since receipt; NOT host noise). seal_with_mask x3 = 2,242 B and .rodata = 1 B match byte-exact; shape identical. Authoritative linked-ELF subject: xip_t1_riscv_elf_size_court exit 0, flash 798 B / sram 0 B |
| alloc/panic symbol purity 0 / 0 | gate-5-silicon.md (nm purity) | llvm-nm (rust-toolchain nightly bin) over all carved objects | alloc 0, panic 0, every object | MATCH |
| A13 zero-indirection: 0 blr / 0 br xN / 0 brk | a13-finish-receipt.md:38-43 | rustc --emit=asm release, dispatch region census (python counter) | 238 raw lines; 0 blr, 0 br xN, 0 brk; exactly 3 direct bl (hash_many_simd, hash, grow_one); 1 adrp | MATCH (line-for-line identical to receipt) |

## Notes

- Default `cargo build --release -p cgap-arrow --target riscv32imc` emits a
  metadata/bitcode-only rlib (0 object bytes, .llvmbc-deferred class) — the
  30,157 B subject is the no-LTO object-class build (--profile audit).
- The linked-ELF subject is authoritative per spec section 4; the 30,157 B
  pre-link row should be re-pinned to this session's 28,355 B.
