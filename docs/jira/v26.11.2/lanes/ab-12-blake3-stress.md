Canonical location: docs/jira/v26.11.3/receipts/ab-12-blake3-stress.md

# AB-12 — BLAKE3 kernel scale stress

- **Lane**: STRESS/AB-12, v26.11.2 starDF campaign
- **Subject**: vendored BLAKE3 kernel — canonical published copy
  `crates/cgap-blake3/src/lib.rs` (crates.io `cgap-blake3` 26.11.3); vendored variant
  `crates/cgap-arrow/src/blake3.rs` (diffed this lane; differs from the
  cgap-blake3 copy by the GATE-2 no-panic windowing only)
- **Owned files**: `crates/cgap-blake3/tests/blake3_scale_stress.rs` (new),
  this receipt
- **Gate**: `cargo test --release -p cgap-blake3 --test blake3_scale_stress`
  — green ×2 this session (run 1 with `-- --nocapture`, run 2 plain), 4/4 each:
  `throughput_curve_and_gpu_dispatch_boundary`,
  `chunk_tree_chain_shapes_merkle_law`, `simd_vs_scalar_equivalence_1k_random`,
  `keyed_mode_stress_1k_keys`. Full `cargo test -p cgap-blake3` (debug, all
  suites incl. zero-alloc courts) green, 0 failures.
- **Location law**: kernel home chosen = cgap-blake3 (the published crate with
  its own tests/ dir). From cgap-blake3, `cgap-crypto::chatman::keyed_digest`
  is unreachable without a dev-dep cycle (cgap-crypto depends on cgap-blake3),
  so court 4 binds keyed mode inline over the kernel's public
  `compress_scalar` with the spec KEYED flag (1<<4), mirroring
  `crates/cgap-crypto/src/chatman.rs`'s own binding — and pins two b3sum
  (`--keyed`, Homebrew reference) KATs generated this lane:
  key `bytes 0x01..=0x20`, empty input →
  `b54fe6f9a432e254a10d31999015209281ec73f50e83e9fbbf5092b6ef8493df`;
  192 B (i%251) input →
  `76a1d92f4e271be7ae2798159a8fd049885d1cc5b2ec9ee181f7217dc4d66398`.

## Throughput curve (court 1; release, min-of-3, this host, aarch64 darwin)

- 64 B: 1.2 hash / 244.1 wide MiB/s — single-call latency noise (run-1 min
  was 45.8); treat 1 KiB+ as the curve.
- 1 KiB: 372.0 / 426.1
- 64 KiB: 1768.8 / 437.7
- 1 MiB: 425.5 / 440.7
- 16 MiB: 412.4 / 430.5
- 64 MiB: 391.0 / 424.1

## Boundary law (court 1 asserts)

- `should_dispatch_gpu` flips exactly at `GPU_CROSSOVER_BYTES` (16 MiB):
  false at 16 MiB − 1, true at 16 MiB and 64 MiB.
- Digest law continuous across the boundary: on a prefix-extended input
  crossing 16 MiB (16 MiB − 1 KiB vs 16 MiB of one (i%251) pattern), both
  kernels agree each side, and the two inputs digest differently — dispatch
  is a perf crossover, not a correctness edge.

## Chunk-tree / Merkle law (court 2)

- Independent reducer (test-local `tree_hash` over public `hash_chunk` +
  `compress_scalar` + PARENT/ROOT; shares no reduction code with the kernel)
  byte-identical to both `hash` and `hash_wide_simd` across 10 chain shapes
  (2, 3, 63, 64, 65, 255, 256, 511, 1000, 1024 chunks) — includes the
  63/64/65-chunk SIMD boundary and the named 1k×1KB chain (1024 chunks, 1 MiB).
- Loud gap: the kernel exposes NO incremental `update()` API, so "incremental
  update chains" are emulated as chunk-sequence concatenation vs one-shot
  hash of the same buffer; the Merkle shape law carries the equivalence.

## SIMD-vs-scalar (court 3)

- 1k random inputs 1025..=131072 B: `hash` == `hash_wide_simd` byte-identical,
  1000/1000 (NEON 4-way parent kernel live on this host).
- AVX2 8-way path: UNVERIFIED-on-host — target-gated out
  (`target_arch = "x86_64"` + `target_feature = "avx2"`); host is aarch64.

## Keyed mode (court 4)

- Inline binding over public `compress_scalar`, KEYED flag per spec; two
  b3sum reference KATs pinned (above); 1k distinct keys → 1k distinct
  digests (HashSet); same key twice → identical.
- `cgap-crypto::chatman::keyed_digest` itself: UNVERIFIED this lane
  (dev-dep cycle); its official-vector KAT coverage lives in cgap-crypto's
  own tests (chatman_courts.rs).

## Findings

1. `hash_wide_simd` is NOT faster than scalar `hash` on this host: ~430 MiB/s
   flat vs 1768.8 MiB/s scalar at 64 KiB. Both converge ~390–440 MiB/s above
   1 MiB. Candidate defect for a profiling lane (suspect: the wide path's
   per-chunk `data[start..end]` bounds-checked slicing; per-chunk hash work
   is identical in both paths, so the gap lives in reduction/slicing).
2. Scalar `hash` degrades above 64 KiB: 1768.8 → ~430 MiB/s at 1 MiB+ (the
   scalar Merkle stack merge loop dominates past the cache window).
3. Boundary pinned: predicate flips exactly at 16 MiB; digest law continuous.
4. AVX2 equivalence UNVERIFIED-on-host (gate name recorded); NEON ALIVE.

## Falsifier status

Each court has a named anti-vacuity property: court 1 — predicate flip +
cross-input digest non-equality; court 2 — independent reducer breaks on any
chunk-index/flag law change; court 3 — breaks on any NEON transpose lane
misalignment; court 4 — b3sum KATs break on any flag/CV-law change.

## Standing

ALIVE (this session; commands + exits above). Never ran git.
