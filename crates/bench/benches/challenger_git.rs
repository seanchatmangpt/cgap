//! Challenger Sales Benchmark: Git Index Search (Constant-Time Ladder vs Branchy Binary Search)
//!
//! Evaluates timing determinism & execution stalls:
//! std::slice::binary_search vs CGAP's constant-time ladder search over 20-byte Git OIDs.

use cgap_git::ladder::ladder_search;
use divan::{black_box, Bencher};

fn main() {
    divan::main();
}

const OID_COUNT: usize = 4096;

fn generate_sorted_oids(n: usize) -> Vec<[u8; 20]> {
    let mut oids = Vec::with_capacity(n);
    let mut seed = 0x123456789ABCDEF0u64;
    for i in 0..n {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let mut oid = [0u8; 20];
        oid[0..8].copy_from_slice(&(i as u64).to_be_bytes());
        oid[8..16].copy_from_slice(&seed.to_be_bytes());
        oids.push(oid);
    }
    oids.sort();
    oids
}

// -----------------------------------------------------------------------------
// Incumbent Arm: Standard Library Binary Search (Branchy)
// -----------------------------------------------------------------------------

#[divan::bench]
fn git_oid_lookup_incumbent_binary_search(b: Bencher) {
    let oids = generate_sorted_oids(OID_COUNT);
    let target = oids[OID_COUNT / 2];

    b.bench_local(|| {
        let res = black_box(&oids).binary_search(black_box(&target));
        let _ = black_box(res);
    });
}

// -----------------------------------------------------------------------------
// Challenger Arm: CGAP Constant-Time Ladder Search
// -----------------------------------------------------------------------------

#[divan::bench]
fn git_oid_lookup_cgap_ladder_search(b: Bencher) {
    let oids = generate_sorted_oids(OID_COUNT);
    let target = oids[OID_COUNT / 2];

    b.bench_local(|| {
        let res = ladder_search(black_box(&oids), black_box(&target));
        black_box(res);
    });
}
