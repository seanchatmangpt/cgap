# af-3-simd-profile

AF-3 implementation lane (AB-12 follow-on), v26.11.3. Subject:
`hash_wide_simd` (crates/cgap-blake3/src/lib.rs) measured ~430 MiB/s vs
scalar 1768.8 MiB/s @64KiB (4x slower, AB-12). Host: aarch64 (Apple
Silicon), release profile.

## Profile instrument

`crates/bench/src/bin/af3_simd_profile.rs` (public API only, min-of-N
Instant timing, parity asserts before timing; `AF3_ROUNDS` env).

## Root cause (measured)

The inversion is a LEAF-KERNEL MISMATCH, not a parent-kernel or
cache-line cost:

- At 64 KiB, scalar `hash` is NOT scalar: `is_small_batch_eligible`
  routes 4..=64 whole chunks through `hash_many_simd` -> the NEON 4-way
  leaf kernel. That is where 1768.8 MiB/s comes from (lib.rs:214).
- `hash_wide_simd` hashes its 64 leaves with SEQUENTIAL SCALAR
  `hash_chunk` — the slow path (383 MiB/s leaf-only at 64 KiB, measured).
- NEON parents are innocent: isolated 64-CV parent reduction measured
  NEON 4-way 55 ns/parent vs scalar chain 79 ns/parent (wide 0.69x,
  i.e. wide parents FASTER).
- Scalar `hash` degrades above 64 KiB (1760 -> ~376 MiB/s at 128 KiB+)
  because its small-batch fast path tops out at exactly 64 whole chunks.

## Crossover sweep (pre-fix, min of 11 rounds)

| size | scalar MiB/s | wide MiB/s | winner |
|---|---|---|---|
| 64 KiB | 1760.6 | 414.5 | scalar (4.25x) |
| 128 KiB | 405.9 | 383.1 | scalar |
| 256 KiB..32 MiB | ~375 | ~380 | ~parity |
| 64 MiB | 222.1 | 402.2 | wide (0.55x) |

Pre-fix the only wide-wins size was 64 MiB (and ~parity 256 KiB+).

## Recommendation: FIX the wide kernel (implemented)

Not PIN-scalar (the wide path wins everywhere once its leaves use the
NEON kernel the repo already owns) and not DOCUMENT-only (the fix is a
leaf-kernel swap, not a rewrite). Type: FIX, landed.

Diff (lib.rs hash_wide_simd leaf phase): the 64-chunk window now batches
leaves through `multi_chunk::compress_chunks_4way` (NEON 4-way, scalar
fallback on non-aarch64) with a scalar clamped-end fallback for partial
trailing windows. The window is always 64 FULL chunks when engaged
(chunk_idx + 64 <= num_chunks); a partial LAST chunk (len not 1 KiB
aligned) falls back per chunk.

### Post-fix sweep (min of 11 rounds)

| size | scalar MiB/s | wide MiB/s | ratio | winner |
|---|---|---|---|---|
| 64 KiB | 1634.0 | 1134.7 | 1.44x | scalar |
| 128 KiB | 375.3 | 1131.6 | 0.33x | wide |
| 256 KiB | 375.5 | 1132.5 | 0.33x | wide |
| 512 KiB | 376.2 | 1132.5 | 0.33x | wide |
| 1 MiB | 406.1 | 1132.2 | 0.36x | wide |
| 8 MiB | 375.9 | 1132.3 | 0.33x | wide |
| 64 MiB | 335.8 | 1128.4 | 0.30x | wide |

Wide: 414 -> 1134 MiB/s at 64 KiB (2.7x); wins 3x everywhere >=128 KiB.
Wide now beats scalar at every size >= 128 KiB; at exactly 64 KiB scalar
(`hash`'s x2 dual-group driver, ~1.63 GiB/s) still leads the wide path's
4-way driver (~1.13 GiB/s) — residual, not the AB-12 inversion. The x2
driver (`compress_chunks_4way_x2`, measured 1.78 vs 1.22 GiB/s 4-way in
the CTQ-06 wave-6 A/B) is the identified follow-on if the 64 KiB point
matters.

## Flip court (kernels-no-std law)

`scripts/af3_court.sh` + /tmp helpers (af3_probe.rs, af3_manifest_patch.py,
af3_mutate.py). Sandbox copy of cgap-blake3 in /tmp, standalone manifest
patch, sha256-exact subject verification, shared tree never mutated
(sha256 re-verified after court).

- AFTER arm (fix present): parity (wide==hash @64KiB + ragged) + perf
  bar >=1000 MiB/s -> PASS
- BEFORE arm (SHIM-COURT mutation: leaf loop swapped back to scalar
  hash_chunk): probe FAILS the 1000 MiB bar -> flip proven, court not
  vacuous.
- Result: COURT PASS, AFTER=PASS BEFORE=FAIL, exit 0.

## Verification ladder (commands + exits this session)

- `cargo test -p cgap-blake3 --release` x2: 9/9 targets ok, 0 failures
  (both runs; includes the 1k-random wide-vs-scalar equivalence court,
  the chunk-tree Merkle law, official vectors, small-batch vectors,
  zero-alloc, thread differential).
- `cargo build --workspace` exit 0.
- FR-06 gate: `blake3_throughput --gate` -> 1.12 GiB/s >= LSL 1.0,
  exit 0 (pre-fix this bin measured ~0.37 GiB/s and would FAIL the LSL;
  the fix brings the gate back to passing).
- Court: `bash scripts/af3_court.sh` exit 0.

## Defect caught by courts mid-lane

First fix attempt sliced `data[start..start+4096]` unclamped and the
1k-random parity court panicked on a ragged 130574-byte input
(range-end-index). Fixed with the scalar clamped-end fallback; suite
green. This was a real bug the Chicago court caught pre-landing.

## Standing

- Kernel fix + profile bin + court script: ALIVE (commands above, this
  session, exact subject sha256
  f518f20b18ca1c34ce926ea393ee459e88a0260e3b33522d8f5f6cda63a9a658).
- 64 KiB residual (x2-vs-4-way driver gap): UNVERIFIED-as-fix candidate,
  measured as documented; no code change made.
- cgap-arrow's parallel blake3 copy NOT touched (out of lane).
