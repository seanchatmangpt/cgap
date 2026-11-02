# REAL Receipt: cgap-ffi

Milestone: v26.11.2 starDF reality wave. Lane: REAL-cgap-ffi. Date: 2026-11-01.
Canonical checkout /Users/sac/cgap; no git writes; shared tree untouched.

## Verdict: REAL (ALIVE)

## Proof chain (commands + exits, this session)

| command | exit | result |
|---|---|---|
| `CARGO_TARGET_DIR=/tmp/real-ffi-target cargo test -p cgap-ffi` | 0 | 45 unit + 4 alloc_courts + 2 lever_courts + 5 zero_copy = 56 passed, 0 failed |
| `CARGO_TARGET_DIR=/tmp/real-ffi-target cargo build -p cgap-ffi --release` | 0 | cdylib built; 2.68s |
| `nm -gU /tmp/real-ffi-target/release/libcgap_ffi.dylib` | 0 | 11 exported T symbols (C-ABI census) |
| rdflib query over cgap-pack ontology.ttl `p:manifest-cgap-ffi` | 0 | version/publish/description vs Cargo.toml |

## C-ABI export census (nm over release cdylib)

11 exported symbols: `_cgap_admit_frame`, `_cgap_array_addr`,
`_cgap_batch_rows`, `_cgap_commit_parquet`, `_cgap_digest_of`,
`_cgap_eval_bn3`, `_cgap_i32_batch`, `_cgap_p28_invariant_check`,
`_cgap_schema_addr`, `_cgap_verify_p28`, `_cgap_version`. Cross-checked:
18 `#[no_mangle]` occurrences in src (some non-exported/cfg'd); the nm
census over the actual dylib is the authoritative count.

## rdflib p:CrateManifest drift-check

Graph facts (p:manifest-cgap-ffi): version 26.11.2, publish true,
description "C Data Interface cdylib membrane for cgap (Gate 3
wire-parity exports)". Cargo.toml (version inherited from root
workspace.package, Cargo.toml:42 = 26.11.2): publish true, description
identical. DRIFT: NONE — all three fields byte-equal.

## Substance

- LOC: 5961 total src (lib 353, oabi_lookup 365, xip_tables 734,
  sovereign 4509).
- 31 `extern "C"` signatures; 14 pub fns in src (lib.rs public re-exports
  the sovereign entrypoints).
- Tests: 56 across 4 targets (unit 45 incl. zero-alloc/determinism
  courts; alloc_courts 4; lever_courts 2 incl. l4_status_flip_witness;
  zero_copy 5 incl. 64-byte alignment + exact-once release path).
- Deps: cgap-arrow 26.11.1 (path) only.

## Standing

ALIVE — cargo test + release build + nm census + rdflib check all run
this session against the exact subject, exit 0 each.

```json
{"lane":"REAL-cgap-ffi","files_written":["docs/jira/v26.11.2/waves/crate-reality/cgap-ffi.md"],
"commands":[["cargo test -p cgap-ffi",0],["cargo build -p cgap-ffi --release",0],["nm -gU libcgap_ffi.dylib",0],["rdflib CrateManifest query",0]],
"courts_run":["cgap-ffi test suite (56 tests, 4 targets)"],
"standing":"ALIVE","divergences":["manifest-facts.txt generated file not present in cgap-pack (g1k-01 receipt cites it; regen needed there, not a cgap-ffi drift)"]}
```
