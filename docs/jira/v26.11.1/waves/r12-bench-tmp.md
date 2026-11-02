# R12: bench_tmp purge (cgap-blake3 pre-publish package cleanup)

Lane: R12-bench-tmp | Date: 2026-11-01 | Repo: /Users/sac/cgap (main, no git writes)
Trigger: F3-7 flagged `crates/cgap-blake3/tests/bench_tmp.rs` (564 B, print-only, no
assertions, scratch-flavored name) as shipping in the cgap-blake3 package. R2 blocks
publish on this marker.

## Deleted content (deletion record; sha256 99adbfba2e76104cf6f637da8c4f8f87bf16df3c4067826ecf3ed75fa27f38bf)

```rust
use std::time::Instant;
#[test]
fn bench_tmp() {
    let n = 256 * 1024 * 1024;
    let mut data = vec![0u8; n];
    for i in (0..n).step_by(4096) { data[i] = (i % 251) as u8; }
    let mut best = f64::MAX;
    for _ in 0..5 {
        let t = Instant::now();
        let d = cgap_blake3::hash_wide_simd(&data);
        let el = t.elapsed().as_secs_f64();
        best = best.min(el);
        println!("hash_wide_simd {:.3}s {:02x?} {:.3} GiB/s", el, &d[..4], n as f64 / el / 1073741824.0);
    }
    println!("BEST {:.3} GiB/s", n as f64 / best / 1073741824.0);
}
```

Why purged: print-only scratch benchmark (no assertions -> zero test bits, 5x256 MiB
allocations -> pure suite cost); the real perf lane lives in crates/bench/benches/.

## Commands + exits (this session)

1. `shasum -a 256 crates/cgap-blake3/tests/bench_tmp.rs` -> exit 0,
   `99adbfba2e76104cf6f637da8c4f8f87bf16df3c4067826ecf3ed75fa27f38bf`, 564 B (pre-delete)
2. `python3 -c "import os; os.remove('.../crates/cgap-blake3/tests/bench_tmp.rs')"` -> exit 0;
   post-delete `ls crates/cgap-blake3/tests/` shows the 5 real suites
   (multi_chunk_vectors, official_vectors, small_batch_vectors, thread_differential, zero_alloc)
3. `CARGO_TARGET_DIR=/tmp/r12-target cargo test -p cgap-blake3` -> exit 0.
   7 test binaries, 27 passed / 0 failed / 1 ignored (log: /tmp/r12-test.log):
   - lib unit: 3 passed; multi_chunk 7 passed 1 ignored; official_vectors 6 passed;
     small_batch 3 passed; thread_differential 3 passed; zero_alloc 5 passed; doc-tests 0
4. `cargo package --list -p cgap-blake3 --allow-dirty` -> exit 0; grep bench_tmp -> no match
   (exit 1), file ABSENT from ship list. Remaining package test files are the 5 real suites.
5. `rm -rf /tmp/r12-target` (lane build-root lease cleanup)

## Standing

ALIVE: file deleted, suite green without it (27/0), ship list clean. R2 publish unblocked.
