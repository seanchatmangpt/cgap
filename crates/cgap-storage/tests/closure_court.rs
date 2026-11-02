//! p1-closure courts (Chicago: real collaborators, assert final state).
//! Corpus: splitmix64-seeded from the canonical corpus seed
//! 0xA20C_0FFE_E5A7_E000. Graph shape: disjoint chains (seeded edge
//! order shuffle) - bounded closure; chain length 50 (N=1k: 20 chains,
//! N=10k: 200 chains).
//! Courts: 1 oracle set-equality, 2 bitplane parity, 3 anti-vacuity
//! mutation (drop second atom -> count MUST change else VACUOUS).
use cgap_storage::closure::{
    mutant_base_only_count, naive_closure, pair_key, semi_naive_closure,
    semi_naive_closure_bitplane_count, CORPUS_SEED,
};
use cgap_storage::dred::splitmix64;

fn chains_corpus(n_nodes: usize, chain_len: usize) -> (Vec<u32>, Vec<u32>) {
    let mut st = CORPUS_SEED;
    let n_chains = n_nodes / chain_len;
    let mut xs: Vec<u32> = Vec::new();
    let mut zs: Vec<u32> = Vec::new();
    for c in 0..n_chains {
        for i in 0..chain_len - 1 {
            let a = (c * chain_len + i) as u32;
            let b = a + 1;
            xs.push(a);
            zs.push(b);
        }
    }
    // Seeded Fisher-Yates shuffle of edge emission order (splitmix64).
    let mut order: Vec<usize> = (0..xs.len()).collect();
    for i in (1..xs.len()).rev() {
        let v = splitmix64(&mut st) as usize;
        let j = v % (i + 1);
        order.swap(i, j);
    }
    order.iter().map(|&i| (xs[i], zs[i])).unzip()
}

#[test]
fn court1_oracle_set_equality() {
    for n in [1000usize, 10_000usize] {
        let (ex, ez) = chains_corpus(n, 50);
        let expect = naive_closure(&ex, &ez);
        let (cx, cz, rounds) = semi_naive_closure(&ex, &ez);
        let got: std::collections::BTreeSet<u64> =
            cx.iter().zip(&cz).map(|(&a, &b)| pair_key(a, b)).collect();
        assert_eq!(got, expect, "set inequality at N={n}");
        assert!(rounds > 0);
        assert_eq!(got.len(), cx.len());
    }
    // Small deterministic sanity: 3-chain closure = 3 pairs.
    let (cx, cz, _) = semi_naive_closure(&[0, 1], &[1, 2]);
    assert_eq!(cx.len(), 3);
    assert_eq!(cz[0], 1);
    assert_eq!(cz[1], 2);
    assert_eq!(cz[2], 2);
}

#[test]
fn court2_bitplane_parity() {
    // Tiny corpus only: bitplane contains() is O(slots) per probe, so
    // feasibility caps this court at a 40-node / 180-pair closure.
    let (ex, ez) = chains_corpus(40, 10);
    let full = semi_naive_closure(&ex, &ez).0.len();
    let bp: usize =
        semi_naive_closure_bitplane_count::<1024>(&ex, &ez)
            .expect("capacity 1024 slots sufficient for 40-node closure");
    assert_eq!(full, bp, "packed-hash vs bitplane count divergence");
    // Loud capacity refusal: tiny set must type-err, not silently drop.
    let tiny = semi_naive_closure_bitplane_count::<4>(&ex, &ez);
    assert!(tiny.is_err());
}

#[test]
fn court3_mutation_drop_second_atom() {
    for n in [1000usize, 10_000usize] {
        let (ex, ez) = base_corpus_with_chain(n, 50);
        let full = semi_naive_closure(&ex, &ez).0.len();
        let mutant = mutant_base_only_count(&ex, &ez);
        assert_ne!(full, mutant, "VACUOUS: join contributed nothing at N={n}");
        assert!(full > mutant);
    }
}

fn base_corpus_with_chain(n_nodes: usize, chain_len: usize) -> (Vec<u32>, Vec<u32>) {
    chains_corpus(n_nodes, chain_len)
}
