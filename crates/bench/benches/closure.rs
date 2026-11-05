//! closure pairs/s bench (p1-closure; harness=false, Instant-timed).
//! Units: pairs/s ONLY (never cross-multiplied). Chains corpus,
//! splitmix64-seeded from 0xA20C_0FFE_E5A7_E000. Status: measured with
//! this stated method (harness=false Instant, 1 warmup + 3 reps min);
//! not Divan-instrumented, so receipt labels stay [TARGET-v26.11.2]
//! until a Divan entry exists.
use std::time::Instant;

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

fn bench(n_nodes: usize, chain_len: usize) {
    let (ex, ez) = chains_corpus(n_nodes, chain_len);
    // Warmup + 3 reps, min taken (no-sleep law: measured, not waited).
    let mut best = f64::INFINITY;
    let mut pairs = 0usize;
    for _ in 0..4 {
        let t = Instant::now();
        let (cx, cz, _r) = cgap_storage::closure::semi_naive_closure(&ex, &ez);
        let d = t.elapsed().as_secs_f64();
        pairs = cx.len();
        assert_eq!(cx.len(), cz.len());
        if d < best {
            best = d;
        }
    }
    let pps = pairs as f64 / best;
    println!("N={n_nodes} chain_len={chain_len} pairs={pairs} best_secs={best:.6} pairs_per_sec={pps:.0}");
}

fn main() {
    if let Err(v) = cgap_aoq::profile_guard::assert_release() {
        eprintln!(
            "LOUD-SKIP {} : {v}",
            std::env::args().next().unwrap_or_default()
        );
        return;
    }
    bench(65_536, 8);
    bench(1_048_576, 32);
}
