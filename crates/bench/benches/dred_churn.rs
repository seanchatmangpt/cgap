//! W4-K1 / G4-DRED churn bench (v26.11.2): SWAR retraction under 10/50/90%
//! churn over a splitmix64-seeded fact corpus, plus an entropy sweep
//! (0/50/100% selectivity) for the compaction path.
//!
//! Seeds (never conflate): splitmix64 multiplier `0xBF58_476D_1CE4_E5B9`
//! (inside `cgap_storage::dred::splitmix64`) and corpus spread seed
//! `0xA20C_0FFE_E5A7_E000` (this bench's CORPUS_SPREAD). Courts:
//! (a) /tmp-mutated dred.rs copy drops the alt-support OR term -> a
//!     fact with a surviving alternate proof is wrongly purged -> the
//!     oracle set-equality assert fires;
//! (b) /tmp-mutated dred_churn.rs copy moves DELETE_FRACTIONS -> the
//!     printed PURGE_COUNTS line moves.
//!
//! Method: wall-clock `Instant`, min-of-N rounds; the fence is a RATIO
//! (full-recompute oracle arm / SWAR retract arm at identical churn) —
//! absolute-ms claims are reported alongside, never alone. Facts/s are
//! [TARGET-v26.11.2]-class until measured this session.

use cgap_storage::dred;
use std::time::Instant;

/// Canonical corpus spread seed (SEED LAW; distinct from the splitmix64
/// multiplier constant).
const CORPUS_SPREAD: u64 = 0xA20C_0FFE_E5A7_E000;

/// Delete fractions applied per churn batch (percent).
const DELETE_FRACTIONS: [usize; 3] = [10, 50, 90];

/// Corpus size for churn batches.
const FACTS: usize = 1 << 18;

/// Entropy sweep levels for the compaction path.
const ENTROPY_LEVELS: [u32; 3] = [0, 50, 100];

/// Min-of-N timing rounds per arm.
const ROUNDS: u32 = 30;

/// Splitmix64-seeded corpus: (derived words, del words, alt_support).
/// 60% derived, 30% carry an alternate proof; delete probability = pct.
fn corpus(facts: usize, del_pct: u32, seed: u64) -> (Vec<u64>, Vec<u64>, Vec<u32>) {
    let words = (facts + 63) / 64;
    let mut st = seed;
    let mut derived = vec![0u64; words];
    let mut del = vec![0u64; words];
    let mut alt = vec![0u32; facts];
    for f in 0..facts {
        if dred::splitmix64(&mut st) % 100 < 60 {
            derived[f / 64] |= 1 << (f % 64);
        }
        if dred::splitmix64(&mut st) % 100 < del_pct as u64 {
            del[f / 64] |= 1 << (f % 64);
        }
        if dred::splitmix64(&mut st) % 100 < 30 {
            alt[f] = 1 + (dred::splitmix64(&mut st) % 7) as u32;
        }
    }
    (derived, del, alt)
}

/// One SWAR retract arm over the corpus; final-state assert: the fast path
/// must reproduce the full-recompute oracle set EXACTLY.
fn retract_arm(
    derived: &[u64],
    del: &[u64],
    alt: &[u32],
    facts: usize,
    scratch: &mut Vec<u64>,
    out: &mut Vec<u32>,
) -> (Vec<u32>, u64) {
    scratch.clear();
    scratch.resize(derived.len(), 0);
    out.clear();
    out.resize(facts + 1, 0);
    let t = Instant::now();
    let n = dred::retract(derived, del, alt, facts, scratch, 0, out);
    let ns = t.elapsed().as_nanos() as u64;
    (out[..n as usize].to_vec(), ns)
}

/// Full-recompute oracle arm at the same churn (fence baseline; sleep-free).
fn oracle_arm(derived: &[u64], del: &[u64], alt: &[u32], facts: usize) -> (Vec<u32>, u64) {
    let t = Instant::now();
    let v = dred::oracle_surviving(derived, del, alt, facts);
    (v, t.elapsed().as_nanos() as u64)
}

fn min_of_ns(f: impl FnMut() -> u64) -> u64 {
    let mut f = f;
    (0..ROUNDS).map(|_| f()).min().unwrap()
}

fn main() {
    println!(
        "G4-DRED churn bench: facts={} rounds={} spread={:#x}",
        FACTS, ROUNDS, CORPUS_SPREAD
    );
    // Entropy sweep for the compaction path (fixed 50% delete fraction).
    for &e in &ENTROPY_LEVELS {
        let (d, x, a) = corpus(FACTS, e, CORPUS_SPREAD ^ (e as u64));
        let (set, _ns) = retract_arm(&d, &x, &a, FACTS, &mut Vec::new(), &mut Vec::new());
        let (oset, _) = oracle_arm(&d, &x, &a, FACTS);
        assert_eq!(set, oset, "entropy {e}%: SWAR set != oracle set");
        let best = min_of_ns(|| {
            let mut sc = Vec::new();
            let mut ou = Vec::new();
            retract_arm(&d, &x, &a, FACTS, &mut sc, &mut ou).1
        });
        println!(
            "ENTROPY {e:3}%  compact {} ns/min  {:.0} facts/s",
            best,
            facts_per_s(FACTS, best)
        );
    }

    // Churn ladder with ratio fence + per-batch oracle set-equality.
    println!("fraction | facts/s | oracle asserts | ratio(oracle/retract)");
    let mut asserts = 0usize;
    for &frac in &DELETE_FRACTIONS {
        let (d, x, a) = corpus(FACTS, frac as u32, CORPUS_SPREAD ^ (frac as u64));
        let (set, _) = retract_arm(&d, &x, &a, FACTS, &mut Vec::new(), &mut Vec::new());
        let (oset, _) = oracle_arm(&d, &x, &a, FACTS);
        assert_eq!(set, oset, "churn {frac}%: SWAR set != oracle set");
        asserts += 1;
        println!("PURGE_COUNTS frac={frac}% purged={}", FACTS - set.len());
        let best_retract = min_of_ns(|| {
            let mut sc = Vec::new();
            let mut ou = Vec::new();
            retract_arm(&d, &x, &a, FACTS, &mut sc, &mut ou).1
        });
        let best_oracle = min_of_ns(|| oracle_arm(&d, &x, &a, FACTS).1);
        let ratio = best_oracle as f64 / best_retract as f64;
        let fps = facts_per_s(FACTS, best_retract);
        println!(
            "{frac:8}% | {:.0} | {} | {:.2} (retract {} ns, oracle {} ns)",
            fps, asserts, ratio, best_retract, best_oracle
        );
        assert!(
            ratio >= 1.0,
            "ratio fence failed: SWAR retract slower than full recompute at {frac}% churn"
        );
    }
    println!("BENCH-OK");
}

fn facts_per_s(facts: usize, ns: u64) -> f64 {
    facts as f64 / (ns as f64 / 1e9)
}
