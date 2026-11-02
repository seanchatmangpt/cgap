# cgap-gpu

GPU execution membrane for the cgap sovereign substrate: Metal Unified
Memory zero-copy buffer bridge on Apple Silicon. Wraps 64-byte-aligned /
16 KiB page-aligned cgap-arrow buffers (`Buf`, `mmu_arena::MmuBuf`) into
`MTLResourceStorageModeShared` MTLBuffers with zero bus copies.

## What ships

- `src/lib.rs` — 105 LOC, single file.
- `build.rs` + `metal_shim.m` — compiles the ObjC shim
  (-fobjc-arc, macOS-gated) and links Metal + Foundation frameworks.
- Dependency surface: cgap-arrow (path, 26.11.1).
- Tests: no `tests/` dir; 1 inline `#[cfg(test)]` zero-copy wrap
  roundtrip — 1/1 pass, witnessed this session: `cargo test -p cgap-gpu`
  -> exit 0, per `waves/assess-gpu.md`.

## Status

- Publish: PUBLISHED-26.11.1, live-witnessed this session:
  `curl -A cgap-publish-receipt https://crates.io/api/v1/crates/cgap-gpu`
  -> exit 0; `num_versions: 1`, `max_version: 26.11.1`, created
  2026-11-02T03:33:40Z.
- Standing: ALIVE (build exit 0 + 1/1 test, per `waves/assess-gpu.md`,
  this session).

## See Also

- Assessment: `docs/jira/v26.11.1/waves/assess-gpu.md`
- Sibling membrane: `crates/cgap-beam`
