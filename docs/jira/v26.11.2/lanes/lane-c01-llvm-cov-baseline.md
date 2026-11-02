# LANE C01 — llvm-cov coverage baseline receipt

Date: 2026-11-02 · HEAD snapshot `9a730320` · branch main · sandbox lease `/tmp/c01-cov`

## Standing: ALIVE (for the covered subset), snapshot-only

Coverage is a SNAPSHOT of a tree with ~20 sweeper lanes in flight — not a
verdict on any crate's final state. All claims below are bounded by the
exclusion set in force at run time.

## Files written (lane-owned)

- `scripts/run_coverage.sh` (executable, `set -euo pipefail`)
- `target/coverage/{coverage_export.json, coverage_baseline.json, coverage-matrix.md, make_matrix.py}`
- `docs/jira/v26.11.2/coverage/{coverage-matrix.md, coverage_baseline.json}`
- this receipt

## Commands + exits (this session)

- `ls $NIGHTBIN` → llvm-profdata, llvm-cov present (llvm-tools-preview OK), exit 0
- Run 1: `bash scripts/run_coverage.sh` → EXIT=101 — unbounded divan
  `chains_bn3` bench runs full benchmarking under `cargo test --all-targets`;
  killed orphan PID 40510. Fix: script now hard-excludes `--exclude bench`.
- Run 2: EXIT=101 — `cgap-bn3` test `edge_closure_courts` E0515 compile error
  (in-flight lane edit). Excluded `cgap-bn3`.
- Run 3: EXIT=101 — `cgap-beam` `bridge_contract` panics: "dylib missing after
  build: /tmp/c01-cov/debug/libcgap_beam.dylib" — test hard-codes a target-dir
  path; coupled to default build root, breaks under any leased
  CARGO_TARGET_DIR. Excluded `cgap-beam`.
- Run 4: EXIT=101 — `cgap-crypto` AE8 guard court: shells to rustc against
  build-script artifacts recorded with mismatched paths (`/tmp/c01-cov` vs
  `/private/tmp/c01-cov` symlink asymmetry) → E0460 instead of the expected
  E0599/E0433. Environment-coupled under leased roots; flagged for owning
  lane; excluded `cgap-crypto`.
- Run 5: EXIT=101 — `cgap-git` lib E0433 `cgap_blake3` unresolved in
  `object_store.rs` (dep sits under `[dev-dependencies]` while the lib uses
  it — mid-flight lane inconsistency). Files unchanged past the ~10-min
  compile-freeze SLA window; I did NOT cross the file-ownership boundary
  (dispatch forbids writing other lanes' files) and instead excluded
  `cgap-git` + dependents `cgap-storage`, `cgap`, `integration`.
- Run 7: EXIT=101 — `cgap-core` `manifest_court` DoD-10: SPDX-shaped
  `license-file` comment added in-flight to cgap-core/Cargo.toml. Excluded
  `cgap-core`.
- Run 8 (FINAL): `COV_EXCLUDE=cgap-bn3,cgap-beam,cgap-crypto,cgap-git,cgap-storage,cgap,integration,cgap-core bash scripts/run_coverage.sh`
  → **EXIT=0**; 0 FAILED suites; 150 profraw files merged; export written.
- Post-fix re-export: corrected `-object` flag usage in `llvm-cov export`
  (bare positionals were silently acting as source filters — only 1 file
  survived the first export). 68 test binaries, exit 0, 6.4 MB export.
- Matrix: `python3 target/coverage/make_matrix.py` → exit 0, "matrix written".

## Loud exclusions (HEAD-state, all in-flight-lane coupling — none diagnosed as coverage defects)

| crate | reason |
|---|---|
| bench | divan benches run unbounded under `cargo test --all-targets` |
| cgap-bn3 | E0515 in `edge_closure_courts` (in-flight) |
| cgap-beam | `bridge_contract` dylib-path coupling to leased target dir |
| cgap-crypto | AE8 court E0460 path asymmetry under leased root |
| cgap-git (+cgap-storage, cgap, integration dependents) | lib E0433 `cgap_blake3` (dep under dev-deps while lib uses it) |
| cgap-core | DoD-10 manifest court vs in-flight license-file line |

15 crates remain in the snapshot; 10 contribute instrumented src (cgap-arrow,
cgap-blake3, cgap-gate, cgap-branchless, cgap-parquet, cgap-p28, cgap-ffi,
cgap-gpu, cgap-bont, cgap-bosym). cgap-audit/cgap-court/cgap-fibo/cgap-cloud
have no `src/` (test-only/data packs) → structural 0/0 rows, not
instrumentation holes.

## Non-vacuity (anti-vacuity mutation named and verified)

A 100%-everywhere export would mean broken instrumentation. The baseline
shows real count=0 code paths in instrumented src, including:
- `cgap-ffi/src/lib.rs` `cgap_version` (the C ABI export) — count=0
- `cgap-arrow/src/c_abi.rs` `release_slot_swap` / `release_array` — count=0
- `cgap-branchless/src/scan.rs` `swar_lt_harvest`, `prefix_or_u64x8` — count=0
- `cgap-bont/src/shacl_mask.rs` `brute_force` — count=0
No crate reaches 100% (max fn% = cgap-gate 98.7). Instrumentation confirmed
live: a real cold path exists and is cited.

## Artifact law (what coverage_baseline.json is)

`coverage_baseline.json` is a REDUCED per-crate summary (fn/line/region
cov+tot + 8 coldest functions per crate). The RAW llvm-cov export is
`target/coverage/coverage_export.json` (6.4 MB, lane-owned lease artifact —
note: it lives under `target/`, which is lease debris per repo law; the
durable copy of the reduced summary + matrix is under
`docs/jira/v26.11.2/coverage/`).

## Divergences / caveats

- Dispatch named `docs/jira/v26.11.2/coverage/coverage-matrix.md` as a task
  output though the file list only named the lane receipt; both written.
- cgap-bont fn% 62.4 is inflated as LOW by in-flight shacl_mask work (module
  files were being rewritten during the run); do not treat as final.
- Perf/coverage numbers here are snapshot values with the stated method
  (`RUSTFLAGS="-C instrument-coverage"`, dev profile, merged 150 profraws);
  they are not targets and will move as in-flight lanes land.
- Build root lease `/tmp/c01-cov` deleted at lane end (below).
