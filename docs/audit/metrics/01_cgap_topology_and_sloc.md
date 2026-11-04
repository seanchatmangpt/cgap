# AU1 Lane 01 — CGAP Topology & SLOC (Phase 1, measured 2026-11-02)

All numbers from wc/find/grep/sed run on /Users/sac/cgap at HEAD.

## 1. Workspace members (root Cargo.toml)
- 24 members: 22 "crates/*" + 2 "ontology-packs/*" (ontology-packs/fibo, ontology-packs/cloud).
- Note: crates/cli has its own [workspace]; not a member (comment in root Cargo.toml).
- crates/cgap-court: 0 src files, 0 LOC (empty scaffold).

## 2. Workspace dependency EDGES (crate -> crates, grep of Cargo.toml [dependencies])
audit -> arrow, parquet
bench -> arrow, blake3, parquet, core, gate, bn3, p28, crypto, ffi, git, storage
cgap -> arrow, blake3, git, parquet, gate, p28, branchless, bn3, core, storage
cgap-beam -> arrow, parquet, git, blake3
cgap-bn3 -> blake3
cgap-bont -> blake3
cgap-crypto -> blake3
cgap-ffi -> arrow
cgap-gate -> branchless
cgap-gpu -> arrow
cgap-p28 -> blake3, crypto
cgap-storage -> blake3
cgap-verification -> bont, crypto
cli (own workspace) -> arrow, parquet, git, blake3, bont, bn3, gate, p28, crypto, audit, court, gpu, beam
wasm-footprint -> arrow, parquet
Leaves: arrow, blake3, bosym, branchless, core, court, git, parquet, integration

## 3. SLOC table (files | src-LOC | tests/ LOC | in-src cfg(test) LOC (approx) | class)
Crate | files | srcLOC | testsLOC | cfg(test)~ | class
audit | 4 | 3,719 | 0 | 3,143 | tooling
bench | 20 | 5,222 | 0 | 0 | tooling
cgap | 3 | 516 | 3,069 | 313 | facade
cgap-arrow | 30 | 10,124 | 2,202 | 9,906 | kernel
cgap-beam | 10 | 3,512 | 2 | 2,025 | facade
cgap-blake3 | 2 | 1,656 | 5 | 1,189 | kernel
cgap-bn3 | 7 | 10,039 | 3 | 914 | kernel
cgap-bont | 5 | 6,327 | 0 | 6,047 | storage
cgap-bosym | 2 | 1,523 | 0 | 605 | storage
cgap-branchless | 9 | 1,650 | 3 | 1,514 | kernel
cgap-core | 5 | 1,423 | 11 | 739 | storage-class
cgap-court | 0 | 0 | 0 | 0 | kernel (empty)
cgap-crypto | 3 | 1,184 | 2 | 1,181 | kernel
cgap-ffi | 5 | 6,520 | 5 | 1,367 | kernel
cgap-gate | 2 | 1,883 | 1 | 153 | kernel
cgap-git | 8 | 5,287 | 11 | 4,346 | kernel
cgap-gpu | 1 | 105 | 0 | 19 | tooling
cgap-p28 | 4 | 4,527 | 2 | 3,512 | kernel
cgap-p28 (tests/) | 2 | — | 619 | — | kernel
cgap-parquet | 23 | 15,452 | 13 | 3,930 | kernel
cgap-storage | 5 | 2,075 | 11 | 1,688 | storage
cgap-verification | 5 | 650 | 1 | 217 | tooling
cli (non-member) | 12 | 4,458 | 3 | 765 (tests/) | tooling
ontology-packs/fibo+cloud | — | 4,057 | (in total) | — | ontology
wasm-footprint | 1 | 505 | 1 | 127 | tooling

Totals: kernel src (arrow+blake3+bn3+bont+bosym+branchless+core+court+crypto+ffi+gate+git+p28+parquet+storage): 65,776 LOC src.
tests/ dir LOC overall: 22,206 (sum of testsLOC column).
scripts/*.py: 12,321 LOC. benches: 18,440 LOC. *.ttl: 20 LOC (ontology/cgap.ttl). cli: 4,458 src LOC.

## 4. External (third-party) deps per member crate: ALL ZERO
Grep of every member crate's [dependencies] for non-cgap/non-workspace entries
returned zero hits for all members. cli (own workspace) declares: blake3,
clap-noun-verb, clap-noun-verb-macros, grep... (see §6) — only non-member
crate with external deps.

## 5. Zero-dep verdicts (kernel crates, lane GS1-era state)
- cgap-arrow: 0 external deps. no_std at crates/cgap-arrow/src/lib.rs:18 (`#![no_std]`, line 7 comment "zero external dependencies").
- cgap-blake3: 0 external deps. no_std (src/lib.rs:1 doc comment "Zero-Dependency `#![no_std]`").
- cgap-git: 0 external deps. `#![no_std]` at crates/cgap-git/src/lib.rs:9.
- cgap-parquet: 0 external deps. `#![no_std]` at crates/cgap-parquet/src/lib.rs:10.
- cgap-gate: 0 external (only ws dep: cgap-branchless). no_std (src/lib.rs:3 doc).
- cgap-crypto: 0 external (ws dep cgap-blake3). no_std (src/lib.rs:3 doc).
- cgap-p28: 0 external (ws deps blake3+crypto). no_std (src/lib.rs:3 doc).
- cgap-ffi: 0 external (ws dep arrow). no_std gate doc at src/lib.rs:8.
- cgap-bn3: 0 external (ws dep blake3). no_std (src/lib.rs:2 doc).
- cgap-branchless: 0 deps total. `#![no_std]` src/lib.rs:19.
- cgap-bosym, cgap-core: 0 deps total; NO no_std attribute found in lib.rs.
- cgap-storage, cgap-bont: only cgap-blake3; NO no_std attribute found in lib.rs.
- cgap-court: empty (no src, no deps) — scaffold only.

## 6. cli external deps (non-member, own workspace)
blake3, clap-noun-verb, clap-noun-verb-macros, linkme, serde, serde_json.

## Method caveats
- In-src cfg(test) LOC: awk from `#[cfg(test)]` marker to EOF per file; approximate
  (marker may not be at a file boundary).
- testsLOC = wc -l over crate's tests/ dir only; excludes dev-dep-driven tests elsewhere.
- ontology-packs/fibo & cloud are Rust crates (src+tests), 4,057 .rs LOC; only 20 LOC of .ttl exists
  (ontology/cgap.ttl) — the "ontology-packs/*.ttl" class is effectively empty.
