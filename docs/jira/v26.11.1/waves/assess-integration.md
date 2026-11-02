# A-4 assessment receipt: cgap-integration

Standing: **ALIVE**. Purpose: cross-crate integration harness — 5-line
src/lib.rs is a doc comment only; the crate IS its tests/ suites (4 files),
driving real cgap-arrow/parquet/git/blake3/audit collaborators with
final-state assertions (Chicago, no mocks). Dependency surface: cgap-arrow
(lib dep — currently unused in lib code, cargo emits an unused-dep warning),
dev-deps add audit, blake3, parquet(std), git. Publish state: guarded
`publish = false` (structural, same class as cgap-court/cgap-beam per
p3-recon-manifests.md; P3-10 p3-pub-integration-wasm.md recorded
dry-run exit 101 and honored the guard, NOT repaired). No
p3-pub-aux-final.md landed as of this receipt — guard work in-flight.

## Commands (this session)

- `cargo build -p cgap-integration` -> exit 0 (1 warning: unused cgap-arrow
  lib dependency).
- `cargo test -p cgap-integration` -> exit 0; 20/20 pass:
  admission_e2e 2, cross_crate 6, error_paths 8, membranes 4, lib 0.
