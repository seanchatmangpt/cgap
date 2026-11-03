# pub-13-docsrs-plan

PUB-13 docs.rs build plan + crates.io metadata (2026-11-02, PLAN).
docs.rs builds x86_64-unknown-linux-gnu, +nightly, --all-features.
Workspace pins floating nightly; no #![feature] anywhere; no
categories/keywords/[badges]/metadata.docs.rs exist yet.

## 1. Failure analysis (default docs.rs job, per crate)

| crate | verdict | grounding |
|---|---|---|
| cgap | PASS | no_std + alloc; default host feature pulls core/storage, compile on linux |
| cgap-arrow | PASS | no_std, zero deps; std/nested-types x86_64-safe |
| cgap-blake3 | PASS | no_std; SIMD #[target_feature]-gated (codegen, not errors) |
| cgap-parquet | PASS | no_std; std feature (mmap scanner) x86_64-safe |
| cgap-git | PASS | no_std; std default; all-features safe |
| cgap-core | PASS | core/alloc codec |
| cgap-storage | PASS | no_std; dep cgap-blake3 |
| cgap-bn3 | PASS | cfg_attr(not(test), no_std); docs build is not test |
| cgap-bont / bosym | PASS | core/alloc only |
| cgap-gate | PASS | no_std; dep branchless no_std-safe |
| cgap-crypto | PASS | no_std; dep blake3 |
| cgap-p28 | PASS | no_std core-only on non-wasm |
| cgap-ffi | PASS* | cdylib+rlib; rustdoc skips cdylib link |
| cgap-audit | PASS* | same; portable SWAR fallbacks everywhere |
| cgap-integration | PASS | registry-only dep surface |
| wasm-footprint | PASS* | cdylib-only; rustdoc skips link |
| cgap-gpu | AT-RISK | linux build.rs macos-gated no-op, but lib declares extern "C" cgap_metal_*; compile passes (link-time), needs doc(cfg) honesty |
| publish=false set + cli/csct | n/a | docs.rs never sees them |

Net: ZERO hard failures. cgap-gpu is the only crate needing a
source-level change (doc cfg honesty). cdylib crates pass because
cargo rustdoc does not link cdylib artifacts.

## 2. riscv-target finding

No crate declares riscv-target as a build-dep; nothing needs it for
docs. Guard: a future riscv-target build-dep must be
target-gated ([target.'cfg(target_arch = "riscv32")'.
build-dependencies]) or the x86_64 docs build breaks.

## 3. Per-crate docs.rs config

Universal block (all publish=true crates, after [package]):

    [package.metadata.docs.rs]
    all-features = true
    rustc-args = ["--cfg", "docsrs"]

all-features is a no-op for featureless crates — keep uniform so
the block is copy-paste identical across 17 manifests. For
cgap-arrow/parquet/git it correctly documents the std-gated
modules. No default-target override anywhere (implicit x86_64 is
right). cgap-gpu: same config + source-level doc-cfg honesty
(dox cfg_attr(docsrs, feature(doc_cfg)) + doc(cfg(target_os =
"macos")) on the Metal surface); do NOT set a darwin target
(docs.rs cannot build darwin).

## 4. Categories + keywords (official slugs; <=3 cats, <=5 kw)

| crate | categories | keywords |
|---|---|---|
| cgap | no-std, data-structures, cryptography | columnar, arrow, parquet, git, umbrella |
| cgap-arrow | no-std, data-structures, encoding | columnar, arrow, branchless, no_std, zerocopy |
| cgap-blake3 | no-std, cryptography | blake3, hash, merkle, avx2, neon |
| cgap-parquet | no-std, encoding, parser-implementations | parquet, columnar, swar, bit-unpack, no_std |
| cgap-git | development-tools, encoding | git, packfile, sha1, crc32, no_std |
| cgap-core | no-std, data-structures | frame-header, codec, isomorphism, no_std |
| cgap-storage | no-std, database-implementations | wal, transaction, flash, embedded, no_std |
| cgap-bont | no-std, data-structures | ontology, shacl, bitmask, oabi, no_std |
| cgap-bosym | no-std, data-structures | interning, symbol, slab, no_std |
| cgap-bn3 | no-std, data-structures | datalog, unifier, sigil, no_std |
| cgap-gate | no-std, development-tools | guard, fail-closed, court, no_std |
| cgap-crypto | no-std, cryptography | blake3, keyed, seal, merkle, no_std |
| cgap-p28 | development-tools, no-std | compliance, attestation, asc-450, sil |
| cgap-ffi | api-bindings, development-tools::ffi | c-data-interface, cabi, wire-parity, ffi |
| cgap-audit | development-tools, development-tools::profiling | branch-audit, machine-code, audit |
| cgap-gpu | api-bindings, os:macos | metal, gpu, zero-copy, unified-memory |
| cgap-integration | development-tools, testing | integration, roundtrip, cabi, wire-parity |
| wasm-footprint | no-std, wasm | wasm, wasip1, footprint, no_std, cdylib |

license-file manifests are valid for registry provided LICENSE.md
ships in the package (all point at ../../LICENSE.md, which
cargo package includes).

## 5. Badge block

Cargo.toml [badges] is dead — crates.io ignores it. Badges live in
README.md:

    [![docs.rs](https://img.shields.io/docsrs/CRATE/latest)](https://docs.rs/CRATE)
    [![crates.io](https://img.shields.io/crates/v/CRATE.svg)](https://crates.io/crates/CRATE)
    [![License](https://img.shields.io/badge/license-P28--SIL%20v2.1.0-blue)](LICENSE.md)

## 6. Risks

1. rust-version unset workspace-wide while the toolchain comment
   cites a floor — declare rust-version before badge publication
   (pub-02 scope).
2. cargo rustdoc skipping cdylib links means ffi/audit/
   wasm-footprint docs builds are untested against the real link
   step — acceptable (docs build is the only contract), record it.
3. Sequencing: land the universal block in one commit (zero
   build-behavior change); cgap-gpu doc-cfg is the only
   source-touching item; README badges after first successful
   docs.rs build.

## See Also

audit/pub-02-package-dryrun.md · audit/pub-06-readme-plan.md ·
audit/pub-08-version-unification.md
