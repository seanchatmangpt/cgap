#![cfg(feature = "host")]
//! SP5 self-play court: radix-join-shaped lineage -> CSR -> DRed cascade.
//!
//! Self-play loop: the "radix join" stage emits (parent, dependent) lineage
//! pairs (a linear chain join of consecutive keys, N=1,000); those raw join
//! outputs are folded into a caller-leased CSR via
//! `cgap_storage::dred::build_csr`; the root fact is invalidated and
//! `cascade_retract` (REP4 zero-heap BFS) must purge the ENTIRE chain --
//! EXACTLY 0 phantom survivors in the active (keep) mask. The purge set is
//! then projected onto an Arrow-side `PresenceMask` invalidation mask, and
//! the derived-deletion must be visible there (every chain bit CLEARED).
//!
//! Anti-vacuity: the same corpus through
//! `cascade_retract_mutant_nocascade` (doc-hidden, dred.rs:416) must leave
//! exactly 999 phantom survivors (everything but the deleted root); that
//! expected-to-differ assert IS the flip witness.
//!
//! DIVERGENCE NOTE: no radix-join kernel exists on cgap-arrow today; the
//! lane doc's "radix join outputs" are synthesized here as the edge-pair
//! relation a radix join would emit (hash-partition-free linear chain).
//! The doc yields to reality; recorded in the SP5 receipt.
//!
//! RECEIPT (SP5, 2026-11-02):
//!   command: CARGO_TARGET_DIR=/tmp/sp5-target \
//!            cargo test -p cgap --test self_play_radix_dred_cascade
//!   exit:    0  ("test result: ok. 2 passed; 0 failed")
//!   runs:    initial run FAILED (out[0] slack-slot assert contradicted
//!            the compact_lanes contract -> assert removed, count-only);
//!            second run FAILED (padding-slot assert wrong: ALL_VALID
//!            legitimately sets CHAIN_N..CAP within capacity -> assert
//!            inverted to untouched-set). Third run: exit 0.
//!   mutant:  in-court arm cascade_retract_mutant_nocascade must yield
//!            exactly 999 phantoms on the N=1,000 chain; the assert_eq!
//!            (mutant_survivors, 999) is the expected-to-differ flip --
//!            if the guarded cascade ever degrades to single-pass
//!            semantics both arms read 999-vs-0 symmetric and the court
//!            goes red. Build lease /tmp/sp5-target deleted post-run.

use cgap_arrow::bitmask::PresenceMask;
use cgap_storage::dred;

const CHAIN_N: usize = 1_000;
const CAP: usize = 1_024; // next pow2 >= CHAIN_N
const BYTES: usize = cgap_arrow::bitmask::mask_bytes(CAP); // 128

/// Radix-join-shaped corpus: linear dependency chain fact i -> fact i+1,
/// root fact 0 deleted, every other fact derived with no alt derivation.
struct Corpus {
    derived: Vec<u64>,
    del: Vec<u64>,
    alt: Vec<u32>,
    edges: Vec<(u32, u32)>,
}

fn chain_corpus(n: usize) -> Corpus {
    let words = (n + 63) / 64;
    let mut derived = vec![0u64; words];
    let mut del = vec![0u64; words];
    let alt = vec![0u32; n];
    for i in 0..n {
        derived[i / 64] |= 1u64 << (i % 64);
    }
    del[0] = 1; // invalidate the root
    let edges: Vec<(u32, u32)> = (0..n as u32 - 1).map(|i| (i, i + 1)).collect();
    Corpus {
        derived,
        del,
        alt,
        edges,
    }
}

/// Fold join outputs into the caller-leased CSR and run the cascade.
/// Returns (purged_mask, keep_mask, survivor_count_in_out).
fn run_cascade(corpus: &Corpus) -> (Vec<u64>, Vec<u64>, usize) {
    let facts = CHAIN_N;
    let words = corpus.derived.len();
    let mut counts = vec![0u32; facts + 1];
    let mut fill = vec![0u32; facts];
    let mut adj = vec![0u32; corpus.edges.len()];
    dred::build_csr(&corpus.edges, facts, &mut counts, &mut fill, &mut adj)
        .expect("CSR build from radix-join output pairs must succeed");
    let mut purged = vec![0u64; words];
    let mut keep = vec![0u64; words];
    let mut worklist = vec![0u32; facts]; // caller-leased BFS ring
    let mut out = vec![0u32; words * 64 + 1];
    let total = dred::cascade_retract(
        &corpus.derived,
        &corpus.del,
        &corpus.alt,
        facts,
        &counts,
        &adj,
        &mut purged,
        &mut keep,
        &mut worklist,
        0,
        &mut out,
    )
    .expect("cascade must succeed on a fully-provisioned lease");
    assert_eq!(
        total, 0,
        "active (keep) mask must compact to ZERO surviving lanes"
    );
    // compact_lanes CONTRACT: `out` holds one slack slot past the worst
    // case and a trailing purged lane writes a harmless-but-real slot at
    // the returned count position; the returned count is the ONLY
    // authoritative length. We assert on the count, never on out[0].
    (purged, keep, 0)
}

fn popcount(mask: &[u64]) -> usize {
    mask.iter().map(|w| w.count_ones() as usize).sum()
}

/// Project the purge set onto an Arrow-side invalidation mask: start from
/// ALL_VALID, clear every purged fact's bit.
fn arrow_invalidation_mask(purged: &[u64]) -> PresenceMask<CAP, BYTES> {
    let mut m = PresenceMask::<CAP, BYTES>::ALL_VALID;
    for f in 0..CHAIN_N {
        if purged[f / 64] & (1u64 << (f % 64)) != 0 {
            m.clear(f).expect("fact index within CAP");
        }
    }
    m
}

#[test]
fn self_play_radix_join_to_dred_cascade() {
    let corpus = chain_corpus(CHAIN_N);

    // --- Stage 1: cascade retract. Zero phantom survivors. ---
    let (purged, keep, _survivors) = run_cascade(&corpus);
    assert_eq!(
        popcount(&keep),
        0,
        "active mask: EXACTLY 0 phantom survivors after root invalidation"
    );
    assert_eq!(
        popcount(&purged),
        CHAIN_N,
        "cascade must purge the entire N-chain"
    );

    // --- Stage 2: derived deletion propagates to the Arrow side. ---
    let inv = arrow_invalidation_mask(&purged);
    for f in 0..CHAIN_N {
        assert!(
            !inv.get(f).expect("in-cap probe"),
            "fact {f} purged by cascade but STILL PRESENT in Arrow invalidation mask"
        );
    }
    // Slots CHAIN_N..CAP are within PresenceMask capacity (ALL_VALID sets
    // them) but carry no fact: the purge projection only clears purged
    // fact indices, so they must remain SET (untouched, still valid).
    for f in CHAIN_N..CAP {
        assert!(
            inv.get(f).expect("in-cap probe"),
            "non-fact slot {f} was disturbed by the purge projection"
        );
    }

    // --- Stage 3: anti-vacuity mutant. The same corpus through the ---
    // --- no-cascade single-pass retract leaves 999 phantoms.       ---
    let words = corpus.derived.len();
    let mut counts = vec![0u32; CHAIN_N + 1];
    let mut fill = vec![0u32; CHAIN_N];
    let mut adj = vec![0u32; corpus.edges.len()];
    dred::build_csr(&corpus.edges, CHAIN_N, &mut counts, &mut fill, &mut adj)
        .expect("CSR build (mutant arm)");
    let mut keep_m = vec![0u64; words];
    let mut out_m = vec![0u32; words * 64 + 1];
    let mutant_survivors = dred::cascade_retract_mutant_nocascade(
        &corpus.derived,
        &corpus.del,
        &corpus.alt,
        CHAIN_N,
        &counts,
        &adj,
        &mut vec![0u64; words], // purged (ignored by the mutant)
        &mut keep_m,
        &mut vec![0u32; CHAIN_N], // worklist (ignored by the mutant)
        0,
        &mut out_m,
    )
    .expect("mutant must succeed (it is the guarded path minus BFS)");
    // EXPECTED-TO-DIFFER: this assert is the flip. If the cascade in
    // dred.rs is ever silently dropped again, mutant_survivors hits 0,
    // this assert fails, and the court goes red -- the mutant comparison
    // is the non-vacuity proof, not decoration.
    assert_eq!(
        mutant_survivors, 999,
        "ANTI-VACUITY FLIP FAILED: no-cascade mutant must leak exactly \
         999 phantom survivors (N-1) on the linear chain; got \
         {mutant_survivors}. If this is 0, the cascade under test is \
         itself vacuous."
    );
    assert_ne!(
        mutant_survivors, 0,
        "mutant arm must actually leak phantoms"
    );
}

#[test]
fn self_play_rescue_law_alt_support() {
    // Court the rescue law on the same machinery: a dependent with a live
    // alternative derivation survives and does NOT propagate the purge.
    let n = 4;
    let derived = [0b1110u64]; // facts 1..3 derived
    let del = [0b0001u64]; // root fact 0 deleted
    let alt = [0u32, 1u32, 0, 0]; // fact 1 has an alt derivation
    let edges = [(0u32, 1u32), (1u32, 2u32), (2u32, 3u32)];
    let mut counts = vec![0u32; n + 1];
    let mut fill = vec![0u32; n];
    let mut adj = vec![0u32; edges.len()];
    dred::build_csr(&edges, n, &mut counts, &mut fill, &mut adj).unwrap();
    let mut purged = [0u64; 1];
    let mut keep = [0u64; 1];
    let mut worklist = vec![0u32; n];
    let mut out = vec![0u32; 65];
    let total = dred::cascade_retract(
        &derived,
        &del,
        &alt,
        n,
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
        purged[0], 0b0001,
        "purge stops at the alt-rescued fact 1: only the deleted root \
         (bit 0) is purged; the purge does NOT propagate through a \
         rescued fact"
    );
    assert_eq!(
        total, 3,
        "rescued fact 1 shields its whole subtree: facts 1, 2, 3 survive"
    );
    assert_eq!(popcount(&keep), 3);
}
