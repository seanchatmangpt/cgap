# p1-blake3: x2 NEON kernel opt-level >= 1 divergence — root cause + fix

## Subject

`crates/cgap-blake3/src/multi_chunk.rs` — `compress_chunks_4way_x2_neon` finalize.
Published 26.11.2 tarball carries the defect; fix lands behind the frozen publish
(zero `cargo publish`, per operator order).

## Root cause

`vst1q_one(out: &[u32; 4], v)` took a **shared** reference and cast it to `*mut`
to store a NEON vector. Writing through a pointer derived from a shared
reference is undefined behavior. At opt-level 0 the UB is harmless in practice;
at opt-level >= 1 LLVM forwards the visible `[0u32; 4]` initializer to the
element loads `cpre_a[l]` / `cpre_b[l]`, so the finalize computed
`out[l][i+8] = s_high ^ 0` — the `^ cv_pre` term silently dropped for output
words 8..16 of both x2 groups. Words 0..8 (`s_low ^ s_high`) were unaffected,
matching the observed court signature exactly.

The 4-way kernel was already correct: it writes its finalize buffers through
`as_mut_ptr()` on `mut` arrays.

## Fix

Removed `vst1q_one` entirely; the x2 finalize now uses `mut` arrays +
`as_mut_ptr()` (the correct 4-way shape). One file changed. Commit left to the
coordinator (lane contract: no git writes).

## Commands + exits (this session, exact subject /Users/sac/cgap)

| Command | Exit |
|---|---|
| `cargo test -p cgap-blake3 --release --test multi_chunk_vectors` (BEFORE fix) | 101 (2 courts FAIL: `x2_kernel_equals_two_4way_kernels`, `x2_kernel_matches_scalar_and_4way`; reproduces the opt-level >= 1 signature) |
| `cargo test -p cgap-blake3 --release` (AFTER) | 0 — all suites pass |
| `cargo test -p cgap-blake3` dev (AFTER) | 0 — all suites pass |
| Anti-vacuity court: UB mutation re-applied in `/tmp/p1-blake3-court/crate` sandbox, `cargo test --release --test multi_chunk_vectors` | 101 (courts flip back to FAIL — non-vacuous; sandbox crate sha-differs from shared tree by design, mutation was sandbox-only) |
| Shared-tree sha256 after court: `4c48d998f12567402cd34087617680c47c51102a6d1c9d8d91a8b2b5704e200a` | fix intact |
| `cargo test -p cgap-blake3 --release --test multi_chunk_vectors bench_throughput -- --ignored --nocapture` | 0, `RESULT bytes=268435456 rounds=5 hash_many_simd=1.85 GiB/s hash_many_simd_x2=1.85 GiB/s hash_wide_simd=0.43 GiB/s` |

Golden vectors byte-exact: `official_vectors`, `multi_chunk_vectors`
(b3sum differential included), and `small_batch_vectors` pass in BOTH dev and
release profiles.

## Performance

Measured, not a target: x2 at 1.85 GiB/s is byte-identical work to before
(the fixed instruction stream is the same `vst1q_u32` store — the UB version's
fast path was only reachable by miscompiling away the `^ cv_pre` term, i.e. by
being wrong). No speed paid for correctness.

## Standing

ALIVE (all gates above ran this session against the exact subject, exit 0
post-fix; anti-vacuity court flips exit 101).

## See Also

`docs/jira/v26.11.1/waves/a02-finish-receipt.md:170-184` (mtime-bump restore
law — not triggered here: the shared tree was never mutated; the mutation
lived its whole life in a /tmp sandbox).

```json
{
  "lane": "p1-blake3",
  "files_written": [
    "crates/cgap-blake3/src/multi_chunk.rs",
    "docs/jira/v26.11.2/waves/p1-blake3-optlevel-ub-fix.md"
  ],
  "commands": [
    {"cmd": "cargo test -p cgap-blake3 --release --test multi_chunk_vectors", "exit": "101 before fix, 0 after"},
    {"cmd": "cargo test -p cgap-blake3 --release", "exit": 0},
    {"cmd": "cargo test -p cgap-blake3 (dev)", "exit": 0},
    {"cmd": "anti-vacuity /tmp sandbox mutation -> cargo test --release --test multi_chunk_vectors", "exit": 101},
    {"cmd": "cargo test -p cgap-blake3 --release --test multi_chunk_vectors bench_throughput -- --ignored --nocapture", "exit": 0}
  ],
  "courts_run": [
    "x2_kernel_equals_two_4way_kernels",
    "x2_kernel_matches_scalar_and_4way",
    "multi_chunk_matches_sequential_across_dispatch_boundaries",
    "x2_driver_matches_b3sum",
    "differential_vs_b3sum_corpus_25_and_odd_lengths",
    "register_resident_permute_4way_matches_scalar_multi_pattern",
    "official_vectors (dev + release)",
    "anti-vacuity UB-mutation sandbox court (flipped: FAIL 101 -> fixed PASS 0)"
  ],
  "standing": "ALIVE",
  "divergences": [
    "Published 26.11.2 tarball carries the defect; publish frozen by operator order — fix lands behind the frozen publish, consumers get it at next version.",
    "Anti-vacuity court ran in a standalone /tmp sandbox crate (literal Cargo.toml replacing workspace inheritance) rather than a tree revert, because the shared tree must never be mutated; the court flipped (exit 101) as required."
  ]
}
```
