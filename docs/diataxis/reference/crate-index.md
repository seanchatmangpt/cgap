# Crate index

All 22 crates in `/Users/sac/cgap/crates` plus the two ontology crates.
crates.io versions verified live 2026-11-01 via
`curl -A cgap-publish-receipt https://crates.io/api/v1/crates/<name>`.

| Directory | Package name | crates.io version | Status | Receipt pointer |
|---|---|---|---|---|
| crates/audit | cgap-audit | 26.11.1 | published | `docs/jira/v26.11.1/waves/` (campaign ledger) |

(Published v26.11.1; v26.11.4 moved cgap-aoq/cgap-arrow APIs — `wire-law-v3-landing.md`.)
| crates/bench | bench | — (name collision: crates.io `bench` 1.1.0 is an unrelated crate) | local-only | `docs/jira/v26.11.1/waves/bench-chains-*.md` |
| crates/cli | cgap-cli | not published | standalone ggen CLI | `docs/jira/v26.11.1/waves/doc-6-bench-cli.md` |
| crates/cgap | cgap | 26.11.1 | published (umbrella) | `docs/jira/v26.11.1/waves/v26111-campaign-ledger.md` |
| crates/cgap-arrow | cgap-arrow | 26.11.1 | published (kernel) | `docs/jira/v26.11.1/lanes/A01-arrow-slab.md` |
| crates/cgap-beam | cgap-beam | not published | PROPOSED | lane doc A14 (`lanes/` index) |
| crates/cgap-blake3 | cgap-blake3 | 26.11.1 | published (kernel) | lane doc A-Blake3 family |
| crates/cgap-bn3 | cgap-bn3 | not published | PROPOSED | `lanes/A11-bn3-arena.md` / `A13-bn3-builtins.md` |
| crates/cgap-bont | cgap-bont | 26.11.1 | published | `lanes/A06-bont-scalars.md` |
| crates/cgap-bosym | cgap-bosym | 26.11.1 | published | `lanes/A07-bosym-slab.md` |
| crates/cgap-core | cgap-core | not published | PROPOSED | `lanes/A04-triumvirate-isomorphism.md` |
| crates/cgap-court | cgap-court | not published | PROPOSED (court harness) | `tests/court_verdict.rs` |
| crates/cgap-crypto | cgap-crypto | 26.11.1 | published | `lanes/A16-chatman-pqc.md` |
| crates/cgap-ffi | cgap-ffi | not published | PROPOSED | `crates/cgap-ffi/src/sovereign.rs` header |
| crates/cgap-gate | cgap-gate | not published | PROPOSED | `lanes/A08-shacl-bitmask.md` consumer |
| crates/cgap-git | cgap-git | 26.11.1 | published (kernel) | `lanes/A03-git-merkle.md` |
| crates/cgap-gpu | cgap-gpu | 26.11.1 | published | campaign ledger |
| crates/cgap-p28 | cgap-p28 | not published | PROPOSED | `lanes/` A17 family, `a17-finish-receipt.md` |
| crates/cgap-parquet | cgap-parquet | 26.11.1 | published (kernel) | `lanes/A02-parquet-mmap.md` |
| crates/cgap-storage | cgap-storage | not published | PROPOSED | campaign ledger |
| crates/integration | cgap-integration | not published | PROPOSED (membrane) | `waves/` receipts |
| crates/wasm-footprint | wasm-footprint | not published | local-only | `crates/wasm-footprint/` |
| ontology-packs/cloud | cgap-cloud | not published | O-ABI pack (A10) | `ontology-packs/cloud/` |
| ontology-packs/fibo | cgap-fibo | not published | O-ABI pack (A09) | `ontology-packs/fibo/` |

Notes.

- Published set (crates.io, 26.10.29 baseline, re-verified 26.11.1):
  cgap, cgap-arrow, cgap-blake3, cgap-bont, cgap-bosym, cgap-crypto,
  cgap-git, cgap-gpu, cgap-parquet, cgap-audit.
- The umbrella `cgap` facade re-exports the five zero-dep kernels:
  cgap-arrow, cgap-blake3, cgap-git, cgap-parquet (+ cgap-crypto).
