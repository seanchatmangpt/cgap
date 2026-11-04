# Verify Latest: v26.10.31 (lane I20)

Generated: 2026-11-04T08:28:43Z · HEAD: `f561b5636fe27366810fdb448b58022d14567e85` · host: arm64

## Gate Summary

| Gate | Status | Exit |
|------|--------|------|
| `check-workspace` | PASS | 0 |
| `test-cgap-arrow` | PASS | 0 |
| `test-cgap-parquet` | PASS | 0 |
| `test-cgap-git` | FAIL | 101 |
| `test-cgap-blake3` | PASS | 0 |
| `test-integration` | PASS | 0 |
| `test-cgap-beam` | FAIL | 101 |
| `audit-branchless` | PASS | 0 |
| `cli-verify` | FAIL | 101 |
| `test-audit` | PASS | 0 |

## Evidence Tails (last 12 lines per gate)

### check-workspace

```
    |         help: remove this `mut`

warning: function `poly_add_assign` is never used
   --> crates/cgap-crypto/src/fips203.rs:324:8
    |
324 | pub fn poly_add_assign(r: &mut [i16; N], b: &[i16; N]) {
    |        ^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `cgap-crypto` (lib) generated 13 warnings (run `cargo fix --lib -p cgap-crypto` to apply 11 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
```

### test-cgap-arrow

```
test null_slots_carry_zero_bytes ... ok
test null_count_matches_pushed_none_count ... ok
test viewvec_descriptor_law_inline_and_foreign ... ok
test varlen_offset_ladder_u32_and_u64 ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests cgap_arrow

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### test-cgap-parquet

```

running 2 tests
test lib_links_and_core_api_runs_under_both_configs ... ok
test std_only::std_scan_surface_runs_end_to_end_under_std_feature ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests cgap_parquet

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### test-cgap-git

```
test case6_offsets_above_2p32_resolve_as_full_64bit ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/lineage_workflow.rs (target/debug/build/cgap-git/4624ae3028bbc31b/out/lineage_workflow-4624ae3028bbc31b)
error: test failed, to rerun pass `-p cgap-git --test lineage_workflow`

Caused by:
  could not execute process `/Users/sac/cgap/target/debug/build/cgap-git/4624ae3028bbc31b/out/lineage_workflow-4624ae3028bbc31b` (never executed)

Caused by:
  No such file or directory (os error 2)
```

### test-cgap-blake3

```
test hash_scalar_64mib_is_zero_alloc ... ok
test compress_is_zero_alloc ... ok
test hash_wide_simd_64mib_is_zero_alloc ... ok
test positive_control_counter_is_live ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.92s

   Doc-tests cgap_blake3

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### test-integration

```
test varlen_u64_offsets_family_roundtrip ... ok
test nested_export_struct_abi_walk_two_children ... ok
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

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.53s

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
error: package(s) `cgap-cli` not found in workspace `/Users/sac/cgap`
```

### test-audit

```
test ocel2_slab::tests::a14_dam_court_refusal_precedence_fixed_order ... ok
test ocel2_slab::tests::a05_power_cut_sweep_every_prefix_and_byte_fail_closed ... ok
test powl::tests::a20_stress_one_million_rows_powl_verify ... ok
test ocel2_slab::tests::a20_stress_one_million_rows_append_seal_reopen ... ok

test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.83s

   Doc-tests cgap_audit

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Baseline

Workspace-check baseline: `scripts/verify_baseline.txt` (0 lines).
PRE-EXISTING-FAIL = failure output matches baseline; FAIL = output differs (introduced).
Regenerate: `scripts/verify_v26.10.31.sh --regen-baseline` (see verify-README.md).
