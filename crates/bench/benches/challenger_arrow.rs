//! Challenger Sales Benchmark: Arrow Predicate Filtering (Branchless SIMD vs Branchy Loop)
//!
//! Contrasts branchy vector filtering with CGAP's constant-time branchless predicate filtering.

use cgap_arrow::predicates::filter_quads_eq;
use divan::{black_box, Bencher};

fn main() {
    divan::main();
}

const QUADS: usize = 16384; // 16k quads = 64k u32s

fn generate_quad_data(quad_count: usize) -> Vec<u32> {
    let mut data = Vec::with_capacity(quad_count * 4);
    let mut x = 0x123456789ABCDEF0u64;
    for _ in 0..quad_count * 4 {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        data.push((x & 0x0F) as u32); // 16 distinct values -> ~6.25% match rate for any needle
    }
    data
}

// -----------------------------------------------------------------------------
// Incumbent Arm: Conventional Branchy Filter
// -----------------------------------------------------------------------------

#[divan::bench]
fn arrow_filter_incumbent_branchy(b: Bencher) {
    let data = generate_quad_data(QUADS);
    let needle = 7u32;
    let mut out = vec![0u32; data.len()];

    b.bench_local(|| {
        let mut count = 0usize;
        let mut i = 0;
        while i < data.len() {
            if data[i] == needle {
                out[count] = data[i];
                out[count + 1] = data[i + 1];
                out[count + 2] = data[i + 2];
                out[count + 3] = data[i + 3];
                count += 4;
            }
            i += 4;
        }
        black_box(count);
    });
}

// -----------------------------------------------------------------------------
// Challenger Arm: CGAP Branchless Predicate Filter
// -----------------------------------------------------------------------------

#[divan::bench]
fn arrow_filter_cgap_branchless(b: Bencher) {
    let data = generate_quad_data(QUADS);
    let needle = 7u32;
    let mut out = vec![0u32; data.len()];

    b.bench_local(|| {
        let matched = filter_quads_eq(
            black_box(&data),
            black_box(QUADS),
            black_box(needle),
            black_box(&mut out),
        );
        black_box(matched);
    });
}
