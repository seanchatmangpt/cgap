//! Challenger Sales Benchmark: Crypto Integrity Hashing & Verification
//!
//! Contrasts conventional scalar hashing (SHA-256 / scalar BLAKE3) with CGAP Wide-SIMD BLAKE3.

use cgap_blake3::{hash, hash_wide_simd};
use divan::counter::BytesCount;
use divan::{black_box, Bencher};

fn main() {
    divan::main();
}

fn make_buffer(len: usize) -> Vec<u8> {
    let mut buf = vec![0u8; len];
    for (i, b) in buf.iter_mut().enumerate() {
        *b = (i & 0xFF) as u8;
    }
    buf
}

// -----------------------------------------------------------------------------
// Incumbent Arm: Scalar Hash (Single Chunk / Sequential)
// -----------------------------------------------------------------------------

#[divan::bench(args = [65536, 1048576, 8388608])]
fn hash_incumbent_scalar(b: Bencher, len: usize) {
    let buf = make_buffer(len);

    b.counter(BytesCount::new(len))
        .bench_local(|| {
            let digest = hash(black_box(&buf));
            black_box(digest);
        });
}

// -----------------------------------------------------------------------------
// Challenger Arm: CGAP Wide-SIMD Tree Hashing
// -----------------------------------------------------------------------------

#[divan::bench(args = [65536, 1048576, 8388608])]
fn hash_cgap_wide_simd(b: Bencher, len: usize) {
    let buf = make_buffer(len);

    b.counter(BytesCount::new(len))
        .bench_local(|| {
            let digest = hash_wide_simd(black_box(&buf));
            black_box(digest);
        });
}
