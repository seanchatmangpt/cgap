//! DREDP courts: pin dred keep_mask's alt_support rescue term against the
//! scalar oracle `oracle_surviving` and against the in-repo mutants.
//! The rescue loop stays per-lane (DREDP probe: a word-parallel SWAR swap
//! via swar_nonzero_u64 measured -11% facts/s; refused on the perf
//! fence -- see docs/jira/v26.11.2/repairs/repair-16-dred-swar-wire.md).
//!
//! Anti-vacuity mutation (named, court flips on it): drop the rescue OR
//! in dred.rs keep_mask (alt term removed) -- every fact with
//! alt_support != 0 is lost and `court_rescue_matches_oracle` fails on
//! the first rescued fact. Flip was witnessed in a /tmp sandbox with
//! sha256-exact restore (receipt: repair-16).

use cgap_storage::dred::{self, oracle_surviving};

const CORPUS_SEED: u64 = 0xD5E5_C0DE_D1CE_0001;

fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0xBF58_476D_1CE4_E5B9);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z ^ (z >> 31)
}

/// Build a corpus: masks word-parallel, alt_support u32 with edge values.
fn corpus(facts: usize, seed: u64) -> (Vec<u64>, Vec<u64>, Vec<u32>) {
    let words = (facts + 63) / 64;
    let mut st = seed;
    let mut derived = vec![0u64; words];
    let mut del = vec![0u64; words];
    let mut alt = vec![0u32; facts];
    for w in 0..words {
        derived[w] = splitmix(&mut st);
        del[w] = splitmix(&mut st);
    }
    // purge padding lanes so oracle/impl agree on the fact domain.
    if words * 64 > facts {
        let valid = (1u64 << (facts - (words - 1) * 64)) - 1;
        derived[words - 1] &= valid;
        del[words - 1] &= valid;
    }
    let edges = [0u32, 1, 0x7F, 0x80, 0xFF, 0xFFFF, 0x1_0000, u32::MAX];
    for f in 0..facts {
        let pick = splitmix(&mut st) % 10;
        alt[f] = if pick < 4 {
            0
        } else {
            edges[(splitmix(&mut st) % edges.len() as u64) as usize]
        };
    }
    (derived, del, alt)
}

/// Compact keep_mask to a sorted surviving fact index vector.
fn surviving(masks: &[u64], words: usize, facts: usize) -> Vec<u32> {
    let mut v = Vec::new();
    for w in 0..words {
        let mut bits = masks[w];
        while bits != 0 {
            let f = w * 64 + bits.trailing_zeros() as usize;
            if f < facts {
                v.push(f as u32);
            }
            bits &= bits - 1;
        }
    }
    v
}

/// Court 1: swapped keep_mask == scalar oracle, 40 corpora x 5 sizes.
#[test]
fn court_rescue_matches_oracle() {
    let mut st = CORPUS_SEED;
    for facts in [1usize, 7, 63, 64, 65, 200, 1000] {
        for _ in 0..40 {
            let (derived, del, alt) = corpus(facts, splitmix(&mut st));
            let words = derived.len();
            let mut keep = vec![0u64; words];
            dred::keep_mask(&derived, &del, &alt, facts, &mut keep);
            let expect = oracle_surviving(&derived, &del, &alt, facts);
            let got = surviving(&keep, words, facts);
            assert_eq!(got, expect, "facts {facts}");
        }
    }
}

/// Court 2: the rescue term is load-bearing -- dropping alt_support (all
/// zero) must change the keep set on the same corpus (the flip witness).
#[test]
fn court_rescue_term_is_load_bearing() {
    let mut st = CORPUS_SEED.wrapping_add(1);
    for facts in [64usize, 200, 1000] {
        let (derived, del, alt) = corpus(facts, splitmix(&mut st));
        let words = derived.len();
        let mut keep = vec![0u64; words];
        dred::keep_mask(&derived, &del, &alt, facts, &mut keep);
        let zeros = vec![0u32; facts];
        let mut keep0 = vec![0u64; words];
        dred::keep_mask(&derived, &del, &zeros, facts, &mut keep0);
        assert_ne!(
            keep, keep0,
            "rescue term flipped-to-zero was not load bearing (facts {facts})"
        );
    }
}

/// Court 3: in-repo mutants (alt term dropped / inverted) diverge from
/// the guarded retract on a rescue-heavy corpus.
#[test]
fn court_mutants_diverge() {
    let mut st = CORPUS_SEED.wrapping_add(2);
    for facts in [64usize, 200] {
        let (derived, del, alt) = corpus(facts, splitmix(&mut st));
        let words = derived.len();
        let mut keep = vec![0u64; words];
        let mut out = vec![0u32; words * 64 + 1];
        let mut out_u = vec![0u32; words * 64 + 1];
        let mut out_i = vec![0u32; words * 64 + 1];
        let n = dred::retract(&derived, &del, &alt, facts, &mut keep, 0, &mut out);
        let nu = dred::retract_unchecked(&derived, &del, &alt, facts, &mut keep, 0, &mut out_u);
        let ni = dred::retract_inverted(&derived, &del, &alt, facts, &mut keep, 0, &mut out_i);
        let a = &out[..n as usize];
        let b = &out_u[..nu as usize];
        let c = &out_i[..ni as usize];
        assert_ne!(a, b, "unchecked mutant did not diverge (facts {facts})");
        assert_ne!(a, c, "inverted mutant did not diverge (facts {facts})");
        // and the guarded path matches the scalar oracle exactly.
        let mut km = vec![0u64; words];
        dred::keep_mask(&derived, &del, &alt, facts, &mut km);
        let expect = oracle_surviving(&derived, &del, &alt, facts);
        assert_eq!(surviving(&km, words, facts), expect, "facts {facts}");
    }
}
