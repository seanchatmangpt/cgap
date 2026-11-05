//! mat (materialized) closure facts/s/core bench (G3-MAT-BENCH).
//! Divan-timed with AllocProfiler.
//!
//! Units: facts/s/core ONLY — derived closure facts per second per core,
//! single-threaded, same metric class as the published RDFox envelope
//! (2-6.1M facts/s/core).
//!
//! Corpus: chains shape, splitmix64-seeded from the canonical corpus seed
//! 0xA20C_0FFE_E5A7_E000.
//!
//! Anti-vacuity (in-bench fence): the full closure fact count is asserted
//! strictly greater than the base-only mutant count every iteration.

use divan::counter::ItemsCount;
use divan::{black_box, Bencher};

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn chains_corpus(n_nodes: usize, chain_len: usize) -> (Vec<u32>, Vec<u32>) {
    let mut st: u64 = 0xA20C_0FFE_E5A7_E000;
    let n_chains = n_nodes / chain_len;
    let (mut xs, mut zs) = (Vec::new(), Vec::new());
    for c in 0..n_chains {
        for i in 0..chain_len - 1 {
            xs.push((c * chain_len + i) as u32);
            zs.push((c * chain_len + i + 1) as u32);
        }
    }
    let mut order: Vec<usize> = (0..xs.len()).collect();
    for i in (1..xs.len()).rev() {
        let j = (splitmix64(&mut st) as usize) % (i + 1);
        order.swap(i, j);
    }
    order.iter().map(|&i| (xs[i], zs[i])).unzip()
}

#[divan::bench(args = [
    (65_536, 8),
    (262_144, 16),
    (1_048_576, 32),
])]
fn materialized_closure_core(b: Bencher, (n_nodes, chain_len): (usize, usize)) {
    let (ex, ez) = chains_corpus(n_nodes, chain_len);
    let base = cgap_storage::closure::mutant_base_only_count(&ex, &ez);
    let (cx_sample, _, _) = cgap_storage::closure::semi_naive_closure(&ex, &ez);
    let facts = cx_sample.len();

    b.counter(ItemsCount::new(facts as u64)).bench_local(|| {
        let (cx, cz, r) = cgap_storage::closure::semi_naive_closure(
            black_box(&ex),
            black_box(&ez),
        );
        assert_eq!(cx.len(), cz.len());
        assert!(
            cx.len() > base,
            "VACUOUS: closure facts ({}) <= base-only mutant ({base})",
            cx.len()
        );
        black_box((cx.len(), cz.len(), r));
    });
}

fn main() {
    divan::main();
}
