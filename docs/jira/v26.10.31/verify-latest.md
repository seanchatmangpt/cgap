# Verify Latest: v26.10.31 (lane I20)

Generated: 2026-11-04T09:38:07Z · HEAD: `5b04b0ae15f71de506e895e8a1dcf6326658335a` · host: arm64

## Gate Summary

| Gate | Status | Exit |
|------|--------|------|
| `check-workspace` | PASS | 0 |
| `test-cgap-arrow` | FAIL | 101 |
| `test-cgap-parquet` | FAIL | 101 |
| `test-cgap-git` | PASS | 0 |
| `test-cgap-blake3` | PASS | 0 |
| `test-integration` | PASS | 0 |
| `test-cgap-beam` | FAIL | 101 |
| `audit-branchless` | PASS | 0 |
| `cli-verify` | PASS | 0 |
| `test-audit` | PASS | 0 |

## Evidence Tails (last 12 lines per gate)

### check-workspace

```

warning: function `poly_add_assign` is never used
   --> crates/cgap-crypto/src/fips203.rs:324:8
    |
324 | pub fn poly_add_assign(r: &mut [i16; N], b: &[i16; N]) {
    |        ^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `cgap-crypto` (lib) generated 13 warnings (run `cargo fix --lib -p cgap-crypto` to apply 11 suggestions)
    Checking cgap-aoq v26.11.3 (/Users/sac/cgap/crates/cgap-aoq)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.91s
```

### test-cgap-arrow

```
test macro_manifest_walk ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 206.41s

     Running tests/mutation_kills.rs (target/debug/build/cgap-arrow/d31c6f7819c9efc1/out/mutation_kills-d31c6f7819c9efc1)
error: test failed, to rerun pass `-p cgap-arrow --test mutation_kills`

Caused by:
  could not execute process `/Users/sac/cgap/target/debug/build/cgap-arrow/d31c6f7819c9efc1/out/mutation_kills-d31c6f7819c9efc1` (never executed)

Caused by:
  No such file or directory (os error 2)
```

### test-cgap-parquet

```

thread 'accessor_scan_16mib_under_ceiling' (31095625) panicked at crates/cgap-parquet/tests/accessor_perf_court.rs:99:5:
accessor scan of 16 MiB took 984.768709ms; the resolved-slice cache is not holding
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    accessor_scan_16mib_under_ceiling

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.96s

error: test failed, to rerun pass `-p cgap-parquet --test accessor_perf_court`
```

### test-cgap-git

```
test court_namespace_projection ... ok
test court_too_many_entries_typed_err ... ok
test court_swapped_order_flips_to_typed_err ... ok
test court_byte_stability_two_passes ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests cgap_git

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### test-cgap-blake3

```
test compress_is_zero_alloc ... ok
test hash_wide_simd_64mib_is_zero_alloc ... ok
test hash_scalar_64mib_is_zero_alloc ... ok
test positive_control_counter_is_live ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.25s

   Doc-tests cgap_blake3

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### test-integration

```
test nested_export_struct_abi_walk_two_children ... ok
test varlen_u64_offsets_family_roundtrip ... ok
test c_abi_roundtrip_i64_u64_f32_f64_bitwise_and_double_release ... ok
test neon_vs_portable_mask_bitwise_equality ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

   Doc-tests cgap_integration

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### test-cgap-beam

```
 {init,do_boot,3,[]}]


note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    beam_kill_stress_100_iterations

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.62s

error: test failed, to rerun pass `-p cgap-beam --test kill_stress`
```

### audit-branchless

```
== disassembler: objdump (Apple LLVM version 17.0.0)
== SELFTEST
== positive control: FIRED (4 banned ops in branchy.o) — gate is not vacuous
== scope demo: clean branchless fixture -> 0 hits
== SELFTEST PASS
```

### cli-verify

```

warning: function `poly_add_assign` is never used
   --> /Users/sac/cgap/crates/cgap-crypto/src/fips203.rs:324:8
    |
324 | pub fn poly_add_assign(r: &mut [i16; N], b: &[i16; N]) {
    |        ^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

[verify_v26] FR-04 court using git version 2.51.2
[verify_v26] battery complete: 7 rows, 0 FAIL, skips=[FR-07]
{"tool":"verify_v26","rows":[{"id":"FR-01","gate":"size/align (Quad64 64B/64B; Buf 64B-aligned heap)","status":"PASS","evidence":"size_of(Quad64)=64 align_of(Quad64)=64 Buf 143/143 allocs 64B-aligned (sizes 1..=1000 step 7)"},{"id":"FR-02","gate":"layout law (derived from FR-01 + Quad64 wire round-trip ladder spot check)","status":"PASS","evidence":"Quad64 new->to_le_bytes(64B)->from_le_bytes round-trip equal; offsets 0x00/0x04/0x08/0x0C/0x10/0x18/0x1C/0x20 held by build-time asserts in layout.rs"},{"id":"FR-03","gate":"parquet RLE round-trip (rle_encode -> rle_decode)","status":"PASS","evidence":"values=10000 bits=8 encoded=10003B enc_fast=10003B decode_ok=true values_equal=true"},{"id":"FR-04","gate":"git pack write + in-crate parser read + real git fsck/verify-pack/cat-file court","status":"PASS","evidence":"blobs=8 pack_bytes=147 idx_bytes=1296 in_crate_parse=true git=git version 2.51.2 fsck_exit_ok=true verify-pack_exit_ok=true cat-file_exit_ok=true n_objects=8  writer_us=401"},{"id":"FR-06","gate":"blake3 known-answer spot check (empty + abc + 2KiB)","status":"PASS","evidence":"hash(\"\")=af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262 match=true hash(\"abc\")=6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85 match=true hash(<2048B>)=634f590a498b3e29165cd8bd32f30a99a2b0d8949a3d7ce35779b6253d8ca0d5 expected=634f590a498b3e29165cd8bd32f30a99a2b0d8949a3d7ce35779b6253d8ca0d5 match=true oracle=[b3sum(/opt/homebrew/bin/b3sum)=634f590a498b3e29165cd8bd32f30a99a2b0d8949a3d7ce35779b6253d8ca0d5 match=true]"},{"id":"FR-05","gate":"replay/determinism spot check (Quad64 wire re-encode stability)","status":"PASS","evidence":"to_le_bytes->from_le_bytes->to_le_bytes byte-identical (deterministic re-encode)"},{"id":"FR-07","gate":"BEAM interop","status":"SKIP","evidence":"requires a live BEAM (cgap-beam NIF harness); owned by the integration lane, not runnable in this in-process battery"}],"summary":{"total":7,"fail":0,"skips":["FR-07"],"timing_us":{"FR-01":201,"FR-02":18,"FR-03":1076,"FR-04":197995,"FR-06":10760,"FR-05":7}},"exit_ok":true}
```

### test-audit

```
test ocel2_slab::tests::ocel_mutate_court_catches_all_classes_and_restores_byte_exact ... ok
test ocel2_slab::tests::a05_power_cut_sweep_every_prefix_and_byte_fail_closed ... ok
test powl::tests::a20_stress_one_million_rows_powl_verify ... ok
test ocel2_slab::tests::a20_stress_one_million_rows_append_seal_reopen ... ok

test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.36s

   Doc-tests cgap_audit

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Baseline

Workspace-check baseline: `scripts/verify_baseline.txt` (0 lines).
PRE-EXISTING-FAIL = failure output matches baseline; FAIL = output differs (introduced).
Regenerate: `scripts/verify_v26.10.31.sh --regen-baseline` (see verify-README.md).
