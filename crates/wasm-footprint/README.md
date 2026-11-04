# wasm-footprint

`no_std` cdylib wrapper supplying the wasm link glue (panic handler +
global allocator) around the cgap-arrow / cgap-parquet kernels; it is
the wasm32-wasip1 footprint subject measured by
`scripts/footprint.sh`. Standalone workspace (own Cargo.lock/target/),
deliberately NOT a root-workspace member.

## What ships

- `src/lib.rs` — 505 LOC, single file.
- Release profile: `lto = "thin"`, `codegen-units = 1`,
  `panic = "abort"` (abort is what makes the wasm link succeed; the
  publish verify step mirrors it — see Cargo.toml comment).
- Dependency surface: cgap-arrow, cgap-parquet (registry, 26.11.5 per Cargo.toml);
  cgap-blake3 dev-dep.
- On-target: `cargo build --target wasm32-wasip1 --release` -> exit 0;
  artifact `target/wasm32-wasip1/release/wasm_footprint.wasm`
  (4,629 B), per `waves/assess-wasm-footprint.md`. Tests: 8/8 pass
  (`cargo test` -> exit 0; lib 3, `tests/no_std_core.rs` 5).

## Divergence (declared shape, not a regression)

Host builds FAIL BY DESIGN: bare `cargo build` -> exit 101 (panic=abort
cdylib vs precompiled std); host `--release` -> exit 101
(dyld_stub_binder link failure, arm64 cdylib). The host is not the
subject target. Any lane that builds this crate bare will read BLOCKED;
builds MUST pin `--target wasm32-wasip1`.

## Status

- Publish: `publish = true` (P3-10 guard flip, 2026-11-01, path deps
  converted to registry deps) but NOT yet on the registry:
  `curl -A cgap-publish-receipt
  https://crates.io/api/v1/crates/wasm-footprint` -> 404, witnessed
  this session.
- Standing: ALIVE on its declared subject (wasm32-wasip1 build exit 0,
  8/8 tests, per `waves/assess-wasm-footprint.md`, this session).

## See Also

- Assessment: `docs/jira/v26.11.1/waves/assess-wasm-footprint.md`
- Footprint gate: `scripts/footprint.sh`
