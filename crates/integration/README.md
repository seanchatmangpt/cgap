# cgap-integration

Cross-crate integration test suites for the cgap substrate: C-ABI
roundtrip, alignment law, format-code table, varlen/views roundtrips.
The crate IS its tests — `src/lib.rs` is a 5-line doc comment only; the
four `tests/` files drive real cgap-arrow / parquet / git / blake3 /
audit collaborators with final-state assertions (Chicago, no mocks).

Note: the on-disk directory is `crates/integration` (package name
`cgap-integration`), not `crates/cgap-integration`.

## What ships

- `tests/admission_e2e.rs` (2), `tests/cross_crate.rs` (6),
  `tests/error_paths.rs` (8), `tests/membranes.rs` (4) — 20/20 pass,
  witnessed this session: `cargo test -p cgap-integration` -> exit 0,
  per `waves/assess-integration.md`.
- Dependency surface: cgap-arrow (lib), dev-deps add cgap-audit (path,
  not on crates.io — 404 witnessed 2026-11-01), cgap-blake3,
  cgap-parquet (std feature), cgap-git.

## Status

- Publish: `publish = true` in Cargo.toml (P3-10 guard flip,
  2026-11-01, registry-only dep surface) but NOT yet on the registry:
  `curl -A cgap-publish-receipt
  https://crates.io/api/v1/crates/cgap-integration` -> 404, witnessed
  this session. P3-10 (`waves/p3-pub-integration-wasm.md`) recorded
  dry-run exit 101 before the flip.
- Standing: ALIVE (build exit 0 + 20/20 tests, per
  `waves/assess-integration.md`, this session).

## See Also

- Assessment: `docs/jira/v26.11.1/waves/assess-integration.md`
- Publish receipt: `docs/jira/v26.11.1/waves/p3-pub-integration-wasm.md`
