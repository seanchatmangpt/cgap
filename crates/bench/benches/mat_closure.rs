//! mat (materialized) closure facts/s/core bench (G3-MAT-BENCH).
//! harness=false, Instant method mirroring benches/closure.rs (1 warmup +
//! 3 reps, min taken; no sleeps — measured, not waited).
//!
//! Units: facts/s/core ONLY — derived closure facts per second per core,
//! single-threaded, same metric class as the published RDFox envelope
//! (2-6.1M facts/s/core). No ratio claims vs RDFox, no GB/s conversion
//! (unit law, 06-bench-protocol §7); measured-vs-published only.
//!
//! Corpus: chains shape, splitmix64-seeded from the canonical corpus seed
//! 0xA20C_0FFE_E5A7_E000 (crates/cgap-storage/src/closure.rs CORPUS_SEED;
//! never the 0xBF58_476D_1CE4_E5B9 multiplier). Fact-count buckets match
//! the closure bench workload shape.
//!
//! Anti-vacuity (in-bench fence): the full closure fact count is asserted
//! strictly greater than the base-only mutant count
//! (`cgap_storage::closure::mutant_base_only_count`) every bucket — if the
//! join contributed zero derived pairs, this bench is VACUOUS. The /tmp
//! sandbox mutant court is recorded in the receipt
//! (docs/jira/v26.11.2/waves/w5-g3-mat-closure.md).
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
    let base = cgap_storage::closure::mutant_base_only_count(&ex, &ez);
    let mut best = f64::INFINITY;
    let mut facts = 0usize;
    let mut rounds = 0usize;
    // 1 warmup + 3 reps, min taken.
    for _ in 0..4 {
        let t = Instant::now();
        let (cx, cz, r) = cgap_storage::closure::semi_naive_closure(&ex, &ez);
        let d = t.elapsed().as_secs_f64();
        facts = cx.len();
        rounds = r;
        assert_eq!(cx.len(), cz.len());
        if d < best {
            best = d;
        }
    }
    // Ratio fence / anti-vacuity: full closure must strictly dominate the
    // base-only mutant count, or the join derived nothing (VACUOUS).
    assert!(
        facts > base,
        "VACUOUS: closure facts ({facts}) <= base-only mutant ({base})"
    );
    let fps = facts as f64 / best;
    println!(
        "bucket N={n_nodes} chain_len={chain_len} base_facts={base} closure_facts={facts} rounds={rounds} best_secs={best:.6} facts_per_sec_core={fps:.0}"
    );
}

fn main() {
    // Fact-count buckets (RDFox-envelope comparison buckets, same shape as
    // benches/closure.rs):
    bench(65_536, 8);
    bench(262_144, 16);
    bench(1_048_576, 32);
}
