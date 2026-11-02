---
paths:
  - "crates/cgap-arrow/**"
  - "crates/cgap-blake3/**"
  - "crates/cgap-git/**"
  - "crates/cgap-parquet/**"
---

# Kernel crates: zero-external-dep, no_std, alloc/panic-pure

`crates/cgap-{arrow,blake3,git,parquet}` are zero-external-dep `#![no_std]`
published kernels — the substrate every other crate compiles against.

- No external deps, no `std::` in hot paths; typed error returns, never
  panics on inputs. Symbol-audit expectation: **0 alloc/panic symbols**
  (the `nm` purity gate).
- Frozen seeds are law: `0xBF58_476D_1CE4_E5B9` and corpus constant
  `0xA20C_0FFE_E5A7_E000` are never varied; a seed variant is a defect.
- Behavioral edits land only after a /tmp sandbox mutation court
  (sha256-exact restore).

## no_std inventory (attribute-checked 2026-11-02)

Clean `#![no_std]`: arrow, blake3, git, parquet, gate, crypto, p28,
ffi (no_std exactly on `target_os = "none"` XIP builds; host builds
keep std by design). **bn3: LANDED** (W6-BN3NS) —
`crates/cgap-bn3/src/lib.rs:3` `#![cfg_attr(not(test), no_std)]`,
host tests stay std via cfg(test). AE1 sealed SpillSync (parquet
mmap_writer / ffi sovereign): spill adds no std hole.
Still std: cgap-core, cgap-bont, cgap-bosym, cgap-storage.

Witnessed: `docs/jira/v26.11.1/waves/r2-xip-noalloc.md` (no-alloc arm,
119/0 green, `cargo test -p cgap-bn3` exit 0).
