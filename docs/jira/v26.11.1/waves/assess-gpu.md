# A-4 assessment receipt: cgap-gpu

Standing: **ALIVE**. Purpose: GPU execution membrane — Metal Unified Memory
zero-copy buffer bridge on Apple Silicon: wraps 64-byte-aligned / 16 KiB
page-aligned cgap-arrow buffers (`Buf`, `mmu_arena::MmuBuf`) into
`MTLResourceStorageModeShared` MTLBuffers with zero bus copies. 105 LOC
(src/lib.rs, single file). Dependency surface: cgap-arrow (path, 26.11.1);
build.rs compiles metal_shim.m (ObjC, -fobjc-arc, macOS-gated) and links
Metal + Foundation frameworks. Tests: no tests/ dir; 1 inline
`#[cfg(test)]` test (zero-copy wrap roundtrip).

## Commands (this session)

- `cargo build -p cgap-gpu` -> exit 0
- `cargo test -p cgap-gpu` -> exit 0; 1 passed / 0 failed (+0 doc-tests).

## Publish state

PUBLISHED-26.11.1 — verified live via crates.io API this session:
`curl -sS -A cgap-publish-receipt https://crates.io/api/v1/crates/cgap-gpu`
-> exit 0; `num_versions: 1`, `max_version: 26.11.1`, created
2026-11-02T03:33:40Z. Prior lane history: P3-9 (p3-pub-beam-gpu.md) hit a
metadata-400 BLOCK before the guard-flip addendum supplied
description/license-file; the landing postdates that receipt.
