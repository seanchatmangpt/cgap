# Lane Ra4 (Raise): court anti-vacuity coverage census

Standing: ALIVE (census + 3 courts hardened, each flip witnessed this session; gate results below with caveats)

## Method
Census of 236 files under crates/*/tests (.rs/.sh) and scripts/courts. A file
"names" a mutation if it matches case-insensitive regex
`anti-vacu|mutation|mutant|vacuous|vacuity|flip|MUTATE`. Result: 193 name one,
43 do not (command: inline python3 glob+regex, exit 0). The regex is a loose
proxy (a hit does not prove the named mutation flips; UNVERIFIED for the 193).

## Courts lacking a named mutation, ranked (cheapest/highest-value first)
| rank | file | lines | #[test] | status |
|---|---|---|---|---|
| 1 | crates/cgap-parquet/tests/status_dictionary_consumer_court.rs | 36 | 1 | DONE (this lane) |
| 2 | crates/cgap-crypto/tests/mlkem768_ntt_courts.rs | 37 | 1 | DONE (this lane) |
| 3 | crates/cgap-crypto/tests/mlkem768_kem_courts.rs | 35 | 1 | DONE (this lane) |
| 4 | crates/cgap-crypto/tests/mlkem768_kem_rt_courts.rs | 80 | 3 | open |
| 5 | crates/cgap-core/tests/manifest_court.rs | 67 | 3 | open |
| 6 | crates/cgap-aoq/tests/aoq_v3_golden_bytes_fixture.rs | 52 | 1 | open (golden bytes: flip one fixture byte) |
| 7 | crates/cgap-gate/tests/frame_abi_mirror_court.rs | 108 | 1 | open (ABI mirror: flip one offset) |
| 8 | crates/cgap-crypto/tests/fips203_cbd_court.rs | 126 | 4 | open |
| 9 | crates/cgap-arrow/tests/validate_minimal.rs | 101 | 4 | open |
| 10 | crates/cgap-crypto/tests/mlkem768_interop_court.rs | 95 | 1 | open |
| 11 | crates/cgap-core/tests/doc_offset_table_court.rs | 155 | 2 | open |
| 12 | crates/csct/tests/bn3_falsification_court.rs | 147 | 3 | open (name says falsification, no named mutation) |
| 13 | crates/wasm-footprint/tests/no_std_core.rs | 127 | 6 | open |
| 14 | crates/cgap-blake3/tests/small_batch_vectors.rs | 118 | 3 | open (KAT; vector flip) |
| 15 | crates/cgap-blake3/tests/blake3_vectors.rs | 174 | 2 | open |
| 16 | crates/cgap-blake3/tests/official_vectors.rs | 205 | 6 | open |
| 17 | crates/cgap-parquet/tests/rle_court.rs | 205 | 8 | open |
| 18 | crates/cgap-parquet/tests/codec_properties.rs | 291 | 4 | open |
| 19 | crates/cgap-ocel2/tests/roundtrip_court.rs | 164 | 1 | open (not root member) |
| 20 | crates/cgap-mcp/tests/courts.rs | 183 | 4 | open (not root member) |
| 21 | crates/cgap-ffi/tests/alloc_courts.rs | 195 | 4 | open |
| 22 | crates/wire-shm/tests/wire_courts_golden.rs | 408 | 2 | open |
| 23 | crates/integration/tests/admission_e2e.rs | 504 | 2 | open |
| 24 | crates/cgap-arrow/tests/validation.rs | 613 | 6 | open |
| 25 | crates/cgap-arrow/tests/property.rs | 439 | 1 | open |
| 26 | crates/cgap-aoq/tests/af30_cage_match.rs | 176 | 1 | open |
| 27 | crates/cgap-crypto/tests/mldsa65_kat_courts.rs | 2739 | 3 | open (expensive) |
| 28 | crates/cgap-aoq/tests/macro_clearing_stress.rs | 463 | 2 | open |
| 29 | crates/cgap-beam/tests/kill_stress.rs | 500 | 1 | open |
| 30 | scripts/courts/court_cross_platform_qualification.sh | 45 | 0 | open (script) |
| 31 | crates/cli/tests/zero_code_cli.rs | 69 | 1 | GENERATED, never hand-edit; fix via graph |
| - | benches/perf (not correctness courts; mutation N/A or low value): cgap-crypto mlkem768_bench, mldsa65_bench; cgap-blake3 percall_latency; cgap-aoq chain_bench, aoq1_bench_court, macro_density_bench, macro_transit_bench; cgap-git perf_baseline, ladder_timing_stress | - | - | deprioritized |
| - | helpers/fixtures: cgap-aoq rfc0089_certificate.rs (10 lines, 0 tests), cgap-core tests/common/mod.rs, cgap-arrow tests/data/golden_vectors.rs | - | - | not courts |

## Hardened (3, comment-only edits to test files)
Mutations run in /tmp/Ra4-sbx (tar copy of tree, excl. target/.git), target /tmp/target-Ra4.
Baseline: both crypto courts and the parquet court pass (exit 0).
1. mlkem768_ntt_courts.rs: fips203.rs `let zeta = ZETAS[k];` (first, in ntt) -> `^ 1`: ntt_c_reference_pins FAILED (rc 101).
2. mlkem768_kem_courts.rs: fips203.rs `basemul2(.., -z)` -> `z`: basemul_matches_schoolbook FAILED (rc 101). (zeta mutation also flipped it.)
3. status_dictionary_consumer_court.rs: cgap-arrow status_band.rs `"EPOCH_RETIRED"` -> `"EPOCH_RETIREX"`: court FAILED (rc 101); the crypto courts stayed green under it (isolation).
Restore: each file restored from in-memory original, sha256 asserted equal
(fips203 23dbd0a9e178..., status_band d335559a3220...), mtime bumped via utime; final sandbox re-run green.
Files written: the 3 test files (doc-comment lines only) + this receipt.

## Incident (mtime/shared-target law witnessed)
After the sandbox work, a run in the SHARED tree against the same
CARGO_TARGET_DIR failed basemul_matches_schoolbook with unmodified shared
fips203.rs (diff vs sandbox: identical). Cause: workspace-member fingerprint is
path-relative, so the sandbox's bumped-mtime artifact was reused. `cargo clean -p cgap-crypto`
then shared-tree rerun: both crypto courts pass. Lesson: never share a target dir between sandbox and
shared tree; here the failure was a stale artifact, not a code defect.

## Gates
- `python3 scripts/boundary_gate.py` (no args): exit 1, "no mode: one of --mutate, --shim-court-only, --deps, --strings, --self-check-deps required" (usage refusal, not a verdict).
- `python3 scripts/boundary_gate.py --deps`: exit 0 (E=0 runtime externals, 15 crates).
- `python3 scripts/boundary_gate.py --self-check-deps`: exit 0 (both arms flipped).
- `python3 scripts/zero_string_gate.py --self-check`: exit 1. Legs poison/clean/scan-mutation PASS; DEBRIS leg A FAIL: objdump on x86_64 Linux "file format not recognized" for a stub .o (host-tooling mismatch, env-caused; the gate is documented macOS aarch64). Not rounded up: gate = FAIL on this host, UNVERIFIED on target host.

## Fences
Diff is comment-only in 3 tests; fences run: the 3 courts (green post-clean), boundary --deps/--self-check-deps. Other modified files in git status belong to other lanes.

## Divergences
- Plan says boundary_gate.py plain; it requires a mode flag (exit 1 without).
- CLAUDE.md says gates exit 3 on loud skip; zero_string self-check exits 1 here.
- cargo --offline unusable (aead dev-dep not vendored); network fetch used.
