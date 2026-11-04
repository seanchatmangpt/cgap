# BR-3 follow-up: BLAKE3 ≥256 KiB throughput cliff — ROOT-CAUSED AND FIXED

Date: 2026-11-04 · Repo: /Users/sac/cgap @ b73f64cb (lane: DIAGNOSIS, BR-3 follow-up)

## Verdict

**FIXED.** The 4.4× cliff (1.98 → 0.45 GB/s at 256 KiB) was a **kernel-dispatch
boundary, not an algorithm cost**: `cgap_blake3::hash` routes any input above
the small-batch window (4..=64 KiB, `is_small_batch_eligible`,
crates/cgap-blake3/src/lib.rs:293-298) into the **pure-scalar sequential
per-chunk loop** — the NEON 4-way/x2 leaf kernels never run above 64 KiB.
~0.45 GB/s is exactly scalar throughput. No allocations anywhere: the
counting-alloc census (CountingAlloc over System, release build) shows
**0 allocs/call at 16 KB, 64 KB, 256 KB, 1 MB** — the per-chunk-heap-churn
hypothesis is REFUTED.

## Fix

`crates/cgap-blake3/src/lib.rs`, in `hash` (after the small-batch branch):

```rust
if len > 64 * 1024 {
    return hash_many_simd(data); // dual-group x2 NEON leaf kernel, same zero-alloc stack Merkle merge
}
```

Bounded: one dispatch branch, no redesign, zero-alloc preserved
(`hash_many_simd_x2` uses the identical `merge_into_stack`/`finalize_root`
as the scalar loop). A/B'd against the alternative (`hash_wide_simd`
64-chunk windows): x2 routing measured flat ~1.97-1.99 GB/s vs 1.37 GB/s
for wide_simd at 256 KB/1 MB — x2 kept.

## Alloc census (release probe, SplitMix64 corpus seed, min-of-5 timing)

| len | allocs/call | before GB/s | after GB/s |
|---|---|---|---|
| 16 KB | 0 | 0.89 | 1.986 |
| 64 KB | 0 | 0.88 | 1.976 |
| 256 KB | 0 | 0.338 | **1.958** (4.05×; bench 131 µs) |
| 1 MB | 0 | 0.415 | **1.978** (4.75×) |

Bench curve re-run (`cargo bench -p cgap-crypto --bench primitives -- blake3_sum32`, divan, release-gated):
64 B 137.6 ns · 256 B 556.8 ns · 1 KB 2.187 µs · 4 KB 3.041 µs · 16 KB 8.207 µs (1.996 GB/s) ·
64 KB 32.7 µs (2.004) · **256 KB 131 µs (2.001 GB/s, was 0.45)** · **1 MB 524.7 µs (2.00 GB/s, was 0.45)**.
Curve is flat 2.0 GB/s from 16 KB through 1 MB — cliff GONE.

## Gates (all run this session, exact subject b73f64cb+fix)

- `cargo test -p cgap-blake3`: **12/12 suites ok, 0 failed** (all 11 vector
  courts incl. official vectors 0..8192 with the 4096 B cross-language pin
  case, multi_chunk, small_batch, scale_stress parity, zero_alloc courts green).
- `cargo bench -p cgap-crypto --bench primitives -- blake3_sum32`: re-run, table above.
- `cargo check --workspace`: exit 0.
- Byte-identity: unchanged outputs asserted by official_vectors + parity courts.

## Cross-repo finding (read-only, arcus twin)

`/Users/sac/arcus/internal/arcus/adapter/blake3.go` (`blake3Hash`, :162-218) is
**uniformly scalar** — no SIMD small-batch path at all, hence **no boundary and
no cliff** (uniform scalar throughput at every size; Go BLAKE3 is
byte-identical to Rust per the IPC-2 interop courts). The cliff is
cgap-Rust-specific; the frozen Go file needs no change.

## Status

ALIVE. Probe crate `/tmp/blake3probe` (lane scratch, not committed).
Dirty files in `crates/cgap-ffi/`, `scripts/mutation_corpus/` belong to other
lanes — untouched, unstaged.
