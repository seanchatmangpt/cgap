//! REP4 cascade lineage court (defect FA5): `dred::retract` is a
//! single-pass bitmask filter that reads derived_deps as predicate bits
//! but never FOLLOWS them as lineage edges — root deletion of an
//! N=10,000 chain left 9,999 phantom survivors. This court pins the
//! repaired cascade semantics against the scalar fixpoint oracle.
//!
//! Anti-vacuity: (a) in-court mutant `cascade_retract_mutant_nocascade`
//! must yield exactly the 9,999 phantom survivors the defect describes
//! while the guarded path yields 0; (b) a /tmp sandbox mutation of
//! dred.rs dropping the BFS must flip the chain court (flip witness in
//! the REP4 receipt, sha256-exact restore).

use cgap_storage::dred::{self, CascadeError};

const CHAIN_N: usize = 10_000;

// Test-only corpus builder; tuple shape mirrors the court's fixture layout.
#[allow(clippy::type_complexity)]
fn chain_corpus(n: usize) -> (Vec<u64>, Vec<u64>, Vec<u32>, Vec<(u32, u32)>) {
    let words = n.div_ceil(64);
    let mut derived = vec![0u64; words];
    let mut del = vec![0u64; words];
    let alt = vec![0u32; n];
    derived[0] = 1;
    del[0] = 1;
    for i in 1..n {
        derived[i / 64] |= 1 << (i % 64);
    }
    let edges: Vec<(u32, u32)> = (0..n as u32 - 1).map(|i| (i, i + 1)).collect();
    (derived, del, alt, edges)
}

fn run_cascade(
    derived: &[u64],
    del: &[u64],
    alt: &[u32],
    facts: usize,
    edges: &[(u32, u32)],
) -> Vec<u32> {
    let words = derived.len();
    let mut counts = vec![0u32; facts + 1];
    let mut fill = vec![0u32; facts];
    let mut adj = vec![0u32; edges.len()];
    dred::build_csr(edges, facts, &mut counts, &mut fill, &mut adj).unwrap();
    let mut purged = vec![0u64; words];
    let mut keep = vec![0u64; words];
    let mut worklist = vec![0u32; facts];
    let mut out = vec![0u32; words * 64 + 1];
    let n = dred::cascade_retract(
        derived,
        del,
        alt,
        facts,
        &counts,
        &adj,
        &mut purged,
        &mut keep,
        &mut worklist,
        0,
        &mut out,
    )
    .unwrap();
    out[..n as usize].to_vec()
}

/// COURT 1 (FA5 chain, the headline defect): N=10,000 linear dependency
/// chain, root deleted, no alt anywhere. Guarded cascade: 0 survivors.
/// The in-court mutant (cascade pass dropped) leaves exactly 9,999
/// phantom survivors — the court must distinguish them (flip witness a).
#[test]
fn court_chain_root_delete_no_phantoms() {
    let (d, x, a, edges) = chain_corpus(CHAIN_N);
    let survivors = run_cascade(&d, &x, &a, CHAIN_N, &edges);
    assert_eq!(
        survivors.len(),
        0,
        "cascade left {} phantom survivors on the FA5 chain",
        survivors.len()
    );
    // Flip witness: the mutant (BFS dropped) reproduces the defect.
    let words = CHAIN_N.div_ceil(64);
    let mut counts = vec![0u32; CHAIN_N + 1];
    let mut fill = vec![0u32; CHAIN_N];
    let mut adj = vec![0u32; edges.len()];
    dred::build_csr(&edges, CHAIN_N, &mut counts, &mut fill, &mut adj).unwrap();
    let mut purged = vec![0u64; words];
    let mut keep = vec![0u64; words];
    let mut worklist = vec![0u32; CHAIN_N];
    let mut out = vec![0u32; words * 64 + 1];
    let m = dred::cascade_retract_mutant_nocascade(
        &d,
        &x,
        &a,
        CHAIN_N,
        &counts,
        &adj,
        &mut purged,
        &mut keep,
        &mut worklist,
        0,
        &mut out,
    )
    .unwrap();
    assert_eq!(
        m as usize, 9_999,
        "mutant no longer reproduces the FA5 phantom-survivor defect"
    );
}

/// COURT 2 (diamond + alt rescue): root 0 -> 1, 0 -> 2; 1 -> 3, 2 -> 3.
/// fact 2 carries an alternative derivation, so the purge must NOT pass
/// through it; fact 3 declares an alternative too (its derivation via the
/// alive branch 2), so purged = {0, 1}, surviving = {2, 3}.
#[test]
fn court_diamond_alt_rescue() {
    let facts = 4usize;
    let words = 1usize;
    let d = [0b1111u64]; // all derived
    let x = [0b0001u64]; // root deleted
    let mut a = [0u32; 4];
    a[2] = 3; // fact 2 has an alternative derivation
    a[3] = 1; // fact 3: alive derivation via the surviving branch 2
    let edges = [(0u32, 1u32), (0, 2), (1, 3), (2, 3)];
    let got = run_cascade(&d, &x, &a, facts, &edges);
    assert_eq!(got, vec![2u32, 3], "alt-supported branch must survive");
    let _ = (words, &mut a);
}

/// COURT 3 (cycle + alt rescue): 0 -> 1 -> 2 -> 0, root 0 deleted.
/// Without alt: the whole cycle purges. With alt on fact 2: {2} survives,
/// {0, 1} still purge (alt breaks propagation THROUGH 2 only).
#[test]
fn court_cycle_alt_rescue() {
    let facts = 3usize;
    let words = 1usize;
    let d = [0b0111u64];
    let x = [0b0001u64];
    let no_alt = [0u32; 3];
    let edges = [(0u32, 1u32), (1, 2), (2, 0)];
    let got = run_cascade(&d, &x, &no_alt, facts, &edges);
    assert_eq!(got, Vec::<u32>::new(), "alt-free cycle purges completely");
    let mut alt = [0u32; 3];
    alt[2] = 1;
    let got = run_cascade(&d, &x, &alt, facts, &edges);
    assert_eq!(got, vec![2u32], "cycle node with alt derivation survives");
    let _ = (words,);
}

/// COURT 4 (convergence equivalence vs the scalar fixpoint oracle):
/// deterministic pseudo-random DAG/cyclic corpora at several sizes must
/// produce the IDENTICAL surviving set as `cascade_oracle`.
#[test]
fn court_oracle_equivalence() {
    let mut st: u64 = dred::CORPUS_SEED ^ 0x5EED;
    for case in 0..8u64 {
        let facts = 32usize + (case as usize) * 96;
        let words = facts.div_ceil(64);
        let mut derived = vec![0u64; words];
        let mut del = vec![0u64; words];
        let mut alt = vec![0u32; facts];
        let mut edges: Vec<(u32, u32)> = Vec::new();
        for f in 0..facts {
            if dred::splitmix64(&mut st) % 100 < 70 {
                derived[f / 64] |= 1 << (f % 64);
            }
            if dred::splitmix64(&mut st) % 100 < 10 {
                del[f / 64] |= 1 << (f % 64);
            }
            if dred::splitmix64(&mut st) % 100 < 30 {
                alt[f] = 1 + (dred::splitmix64(&mut st) % 7) as u32;
            }
            // sparse pseudorandom lineage edges (may form cycles)
            let fanout = dred::splitmix64(&mut st) % 3;
            for _ in 0..fanout {
                let dep = (dred::splitmix64(&mut st) as usize) % facts;
                edges.push((f as u32, dep as u32));
            }
        }
        let got = run_cascade(&derived, &del, &alt, facts, &edges);
        let want = dred::cascade_oracle(&derived, &del, &alt, facts, &edges);
        assert_eq!(
            got,
            want,
            "cascade != oracle at case {case} (facts {facts}, edges {})",
            edges.len()
        );
    }
}

/// COURT 5 (typed refusals): under-provisioned leases and out-of-range
/// edges must return typed CascadeError values, never silently drop.
#[test]
fn court_typed_refusals() {
    let facts = 4usize;
    let d = [0b1111u64];
    let x = [0b0001u64];
    let a = [0u32; 4];
    let edges = [(0u32, 1u32), (1, 2), (2, 3)];
    let mut counts = vec![0u32; facts + 1];
    let mut fill = vec![0u32; facts];
    let mut adj = vec![0u32; edges.len()];
    dred::build_csr(&edges, facts, &mut counts, &mut fill, &mut adj).unwrap();
    let mut purged = vec![0u64; 1];
    let mut keep = vec![0u64; 1];
    let mut out = vec![0u32; 65];
    // (a) worklist capacity < facts -> WorklistCapacity
    let mut short_wl = vec![0u32; facts - 1];
    let err = dred::cascade_retract(
        &d,
        &x,
        &a,
        facts,
        &counts,
        &adj,
        &mut purged,
        &mut keep,
        &mut short_wl,
        0,
        &mut out,
    );
    assert_eq!(
        err,
        Err(CascadeError::WorklistCapacity {
            capacity: facts - 1,
            facts
        }),
        "worklist under-provisioning must be a typed refusal"
    );
    // (b) edge endpoint out of range -> EdgeOutOfRange
    let bad_edges = [(0u32, 7u32)];
    let mut bad_counts = vec![0u32; facts + 1];
    let mut bad_fill = vec![0u32; facts];
    let mut bad_adj = vec![0u32; 1];
    let err = dred::build_csr(
        &bad_edges,
        facts,
        &mut bad_counts,
        &mut bad_fill,
        &mut bad_adj,
    );
    assert_eq!(
        err,
        Err(CascadeError::EdgeOutOfRange {
            edge_index: 0,
            endpoint: 7,
            facts
        }),
        "out-of-range edge must be a typed refusal"
    );
    // (c) adjacency lease too small -> AdjacencyLease
    let mut tiny_adj = vec![];
    let err = dred::build_csr(&edges, facts, &mut counts, &mut fill, &mut tiny_adj);
    assert_eq!(
        err,
        Err(CascadeError::AdjacencyLease {
            needed: edges.len(),
            provided: 0
        }),
        "adjacency under-provisioning must be a typed refusal"
    );
    // (d) counts lease too small -> CountsLease
    let mut tiny_counts = vec![];
    let mut f4 = vec![0u32; facts];
    let mut a4 = vec![0u32; edges.len()];
    let err = dred::build_csr(&edges, facts, &mut tiny_counts, &mut f4, &mut a4);
    assert_eq!(
        err,
        Err(CascadeError::CountsLease {
            needed: facts + 1,
            provided: 0
        }),
        "counts under-provisioning must be a typed refusal"
    );
}
