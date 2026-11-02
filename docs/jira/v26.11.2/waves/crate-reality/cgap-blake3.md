# Crate Reality Receipt: cgap-blake3

Lane: REAL-cgap-blake3 · 2026-11-01 · repo /Users/sac/cgap (main)

## Verdict: REAL-WITH-DRIFT

## 1. Test run

Command: `CARGO_TARGET_DIR=/tmp/real-blake3-target cargo test -p cgap-blake3` — **exit 0**.

| Suite | passed | failed | ignored |
|---|---|---|---|
| unittests src/lib.rs | 3 | 0 | 0 |
| multi_chunk_vectors | 7 | 0 | 1 |
| official_vectors | 6 | 0 | 0 |
| small_batch_vectors | 3 | 0 | 0 |
| thread_differential | 3 | 0 | 0 |
| zero_alloc | 5 | 0 | 0 |
| doctests | 0 | 0 | 0 |

Total: 27 passed, 0 failed, 1 ignored (28 `#[test]` markers on disk).

## 2. Pack-graph drift check (rdflib on ontology.ttl)

Source: `/Users/sac/ggen-marketplace/packs/cgap-pack/ontology.ttl`,
namespace `https://seanchatmangpt.github.io/packs/cgap-pack#`.

| Fact | graph | Cargo.toml | match |
|---|---|---|---|
| crateName | cgap-blake3 | cgap-blake3 | OK |
| crateVersion | 26.11.2 | workspace 26.11.2 | OK |
| cratePublish | true | publish = true | OK |
| crateDescription | "…zero-dependency no_std BLAKE3… NEON and 8-way AVX2…" | "…zero-dependency #![no_std] BLAKE3… NEON & 8-way AVX2…" | **DRIFT** |

**DRIFT-D1 (cosmetic, description token mismatch)**: graph says `no_std`, manifest
says `#![no_std]`; graph `and` vs manifest `&`. No semantic divergence
(publish/version/name agree). Fix is one pack-template token — pack edits out of
lane scope, not applied.

## 3. Substance

- Non-generated src LOC: 1,650 (src/lib.rs 1155 + src/multi_chunk.rs 495; no
  build.rs-generated src).
- Public API: 10 `pub fn` (compress_scalar, compress, hash_chunk, hash, hash_wide_simd,
  hash_many_simd, hash_many_simd_x2, reduce_level_simd, compress_chunks_4way,
  compress_chunks_4way_x2) + IV / MSG_PERMUTATION / domain-flag / crossover consts.
- Test count: 28 `#[test]` across 6 suites + 5 integration files (1,189 test LOC).

## 4. Commands + exits

- `cargo test -p cgap-blake3` → exit 0 (27 pass / 0 fail / 1 ignored)
- rdflib manifest query → exit 0 (4 facts extracted)
- No shared tree src edits; receipt file only.

## Standing: REAL-WITH-DRIFT

Substantive src + full test pass + graph facts match except one cosmetic
description-token DRIFT-D1.
