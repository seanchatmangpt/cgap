# A-4 assessment receipt: wasm-footprint

Standing: **ALIVE** (on its declared subject; host cdylib builds are
out-of-subject and fail — see divergence). Purpose: no_std cdylib supplying
the wasm link glue (panic handler + global allocator) around the
cgap-arrow/cgap-parquet kernels; it is the wasm32-wasip1 footprint subject
measured by scripts/footprint.sh. 505 LOC (src/lib.rs, single file).
Standalone workspace (empty `[workspace]` table, own Cargo.lock/target/),
deliberately NOT a root-workspace member. Dependency surface: cgap-arrow,
cgap-parquet (path, 26.11.1); blake3 dev-dep; release profile lto=thin,
codegen-units=1, panic=abort. Publish state: guarded `publish = false`;
P3-10 (p3-pub-integration-wasm.md) SKIP, structural; aux guard work
in-flight (no p3-pub-aux-final.md landed).

## Commands (this session)

- `cargo build` (standalone, host) -> exit 101 — panic=abort cdylib vs
  precompiled std (expected out-of-subject failure).
- `cargo build --release` (host) -> exit 101 — dyld_stub_binder link
  failure, arm64 cdylib.
- `cargo test` -> exit 0; 8/8 pass (lib 3, tests/no_std_core.rs 5).
- `cargo build --target wasm32-wasip1 --release` -> exit 0; artifact
  target/wasm32-wasip1/release/wasm_footprint.wasm (4,629 B) produced.

## Divergence

Host dev/release builds fail (exit 101 both). This is the declared shape of
the crate (panic=abort no_std cdylib; host is not the subject target), not a
regression — but any lane that `cargo build`s it bare will read BLOCKED;
builds must pin `--target wasm32-wasip1`.
