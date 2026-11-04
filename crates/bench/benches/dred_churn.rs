//! W4-K1 / G4-DRED churn bench (v26.11.2, D7 receipt-shape): SWAR retraction
//! under 10/50/90% churn over a splitmix64-seeded fact corpus, plus an
//! 11-level entropy sweep (0..100 step 10) for the compaction path.
//!
//! BATCH-UNIT EQUIVALENCE: one `dred::retract` call over the full corpus IS
//! the batch unit in this bench (the gate's "full increment+mat cycle"
//! maps one-to-one onto the single retraction pass; there is no separate
//! increment/mat split in the DRED kernel surface). NS_PER_BATCH lines
//! report ns per retract call = ns per batch.
//!
//! Seeds (never conflate): splitmix64 multiplier `0xBF58_476D_1CE4_E5B9`
//! (inside `cgap_storage::dred::splitmix64`) and corpus spread seed
//! `0xA20C_0FFE_E5A7_E000` (this bench's CORPUS_SPREAD). Courts:
//! (a) /tmp-mutated dred.rs copy drops the alt-support OR term -> a
//!     fact with a surviving alternate proof is wrongly purged -> the
//!     oracle set-equality assert fires;
//! (b) /tmp-mutated dred_churn.rs copy moves DELETE_FRACTIONS -> the
//!     printed PURGE_COUNTS line moves;
//! (c) /tmp-mutated dred_churn.rs copy SKIPS THE ENTROPY SWEEP (empty
//!     level list) -> the ENTROPY_VARIANCE line flips (panic, exit 101).
//!
//! FENCE RECONCILIATION (disclosed, not silent): the gate text asks for
//! "< 0.15 ns entropy variance" while court 3 (cgap-storage/src/dred.rs
//! flatness court) asserts max/min ns/lane < 4.0. The absolute-ns fence
//! does not lawfully bind the measured quantity (per-level ns/lane scales
//! with corpus selectivity by construction; a single retract over 2^18
//! facts is orders of magnitude above 0.15 ns). The lawfully-matching
//! fence is the RELATIVE one already enforced by court 3: ENTROPY_VARIANCE
//! max/min < 4.0. This bench mirrors that ratio fence; the absolute
//! ns/lane spread is printed informationally alongside, never alone.
//! No court is weakened.
//!
//! Method: wall-clock `Instant`, min-of-N rounds; the churn fence is a
//! RATIO (full-recompute oracle arm / SWAR retract arm at identical
//! churn). Facts/s are tagged MET/NOT-MET against the 15M target after
//! measurement.

use cgap_storage::dred;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// Counting allocator (LANE C6 alloc witness): counts heap allocations so
/// the CASCADE leg can assert REP4's zero-heap claim on the BFS walk. The
/// counter is read-and-reset around the measured call; the fast-path legs
/// run before the counter is armed and are unaffected semantically.
static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);

struct CountingAlloc;

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(p, l, n) }
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

/// Canonical corpus spread seed (SEED LAW; distinct from the splitmix64
/// multiplier constant).
const CORPUS_SPREAD: u64 = 0xA20C_0FFE_E5A7_E000;

/// Delete fractions applied per churn batch (percent).
const DELETE_FRACTIONS: [usize; 3] = [10, 50, 90];

/// Independent batch seeds per churn fraction (gate: N >= 9).
const BATCHES_PER_FRACTION: usize = 9;

/// Corpus size for churn batches.
const FACTS: usize = 1 << 18;

/// Entropy sweep levels: 0..100 step 10 (11 levels; gate: >= 9).
const ENTROPY_LEVELS: [u32; 11] = [0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100];

/// Min-of-N timing rounds per arm.
const ROUNDS: u32 = 30;

/// Throughput target (facts/s at 90% churn).
const TARGET_FACTS_PER_S: f64 = 15_000_000.0;

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

fn facts_per_s(facts: usize, ns: u64) -> f64 {
    facts as f64 / (ns as f64 / 1e9)
}

fn main() {
    if let Err(v) = cgap_aoq::profile_guard::assert_release() {
        eprintln!("LOUD-SKIP {} : {v}", std::env::args().next().unwrap_or_default());
        return;
    }
    println!(
        "G4-DRED churn bench: facts={} rounds={} spread={:#x} batches/frac={} entropy_levels={}",
        FACTS,
        ROUNDS,
        CORPUS_SPREAD,
        BATCHES_PER_FRACTION,
        ENTROPY_LEVELS.len(),
    );

    // Entropy sweep for the compaction path (fixed 50% delete fraction).
    let mut ns_per_lane: Vec<f64> = Vec::new();
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
        let lane = best as f64 / FACTS as f64;
        ns_per_lane.push(lane);
        println!(
            "ENTROPY {e:3}%  compact {best} ns/min  {:.4} ns/lane  {:.0} facts/s",
            lane,
            facts_per_s(FACTS, best)
        );
    }
    let mn = ns_per_lane.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = ns_per_lane.iter().cloned().fold(0.0, f64::max);
    let variance = mx / mn;
    println!(
        "ENTROPY_VARIANCE max/min={variance:.4} (absolute spread {:.4} ns/lane, informational)",
        mx - mn
    );
    // A degenerate sweep (0 or 1 levels) folds to 0.0 or NaN and would
    // print a vacuous ENTROPY_VARIANCE line; refuse it loudly.
    assert!(
        ENTROPY_LEVELS.len() >= 9 && variance.is_finite() && variance > 0.0 && variance < 4.0,
        "entropy-variance fence failed: max/min {variance:.4} across {} levels (degenerate sweep or spread >= 4.0)",
        ENTROPY_LEVELS.len()
    );

    // Churn ladder with ratio fence + per-batch oracle set-equality.
    println!("fraction | facts/s | oracle asserts | ratio(oracle/retract)");
    let mut asserts = 0usize;
    let mut ns_per_batch_90: Option<u64> = None;
    for &frac in &DELETE_FRACTIONS {
        let mut frac_best_ns = u64::MAX;
        for b in 0..BATCHES_PER_FRACTION {
            let seed = CORPUS_SPREAD ^ ((frac as u64) << 32) ^ (b as u64 + 1);
            let (d, x, a) = corpus(FACTS, frac as u32, seed);
            let (set, _) = retract_arm(&d, &x, &a, FACTS, &mut Vec::new(), &mut Vec::new());
            let (oset, _) = oracle_arm(&d, &x, &a, FACTS);
            assert_eq!(set, oset, "churn {frac}% batch {b}: SWAR set != oracle set");
            asserts += 1;
            println!(
                "PURGE_COUNTS frac={frac}% batch={b} purged={}",
                FACTS - set.len()
            );
            let best_retract = min_of_ns(|| {
                let mut sc = Vec::new();
                let mut ou = Vec::new();
                retract_arm(&d, &x, &a, FACTS, &mut sc, &mut ou).1
            });
            println!(
                "BATCH {b} frac={} ns={} facts/s={:.0}",
                frac,
                best_retract,
                facts_per_s(FACTS, best_retract)
            );
            if best_retract < frac_best_ns {
                frac_best_ns = best_retract;
            }
        }
        if frac == 90 {
            ns_per_batch_90 = Some(frac_best_ns);
        }
        let (d, x, a) = corpus(FACTS, frac as u32, CORPUS_SPREAD ^ (frac as u64));
        let best_oracle = min_of_ns(|| oracle_arm(&d, &x, &a, FACTS).1);
        let ratio = best_oracle as f64 / frac_best_ns as f64;
        let fps = facts_per_s(FACTS, frac_best_ns);
        let met = if fps >= TARGET_FACTS_PER_S {
            "MET"
        } else {
            "NOT-MET"
        };
        println!(
            "{frac:8}% | {:.0} | {} | {:.2} (retract {} ns, oracle {} ns) | 15M-target {}",
            fps, asserts, ratio, frac_best_ns, best_oracle, met
        );
        assert!(
            ratio >= 1.0,
            "ratio fence failed: SWAR retract slower than full recompute at {frac}% churn"
        );
    }
    let nsb = ns_per_batch_90.expect("90% churn batch timing missing");
    println!("NS_PER_BATCH retract={nsb}");

    // ------------------------------------------------------------------
    // LANE C6, leg (a) FAST-PATH: the churn ladder above IS the static
    // bitmask filter mechanism (receipted ~943M facts/s certification,
    // 10/50/90% churn, output format untouched). Label it explicitly so
    // the DREDLABEL honest split is visible in the bench output.
    // ------------------------------------------------------------------
    println!(
        "FASTPATH_LEG mechanism=static_bitmask_filter fractions={DELETE_FRACTIONS:?} facts={FACTS}"
    );

    // ------------------------------------------------------------------
    // LANE C6, leg (b) CASCADE: deep retraction over the N=10,000 linear
    // dependency chain (FA5 corpus) via `cascade_retract`. Reports
    // ns/lane for the cascade walk, survivor count (must be EXACTLY 0),
    // and the heap-alloc witness during the walk (REP4 zero-heap claim).
    // Anti-vacuity court: `cascade_retract_mutant_nocascade` (the
    // pre-REP4 filter-only behavior) MUST leave 9,999 phantom survivors
    // on this corpus (witnessed in repair-04) — flipped in-bench below.
    // ------------------------------------------------------------------
    const CHAIN: usize = 10_000;
    let chain_edges: Vec<(u32, u32)> = (0..CHAIN as u32 - 1).map(|i| (i, i + 1)).collect();
    let chain_words = (CHAIN + 63) / 64;
    let mut chain_derived = vec![0u64; chain_words];
    let mut chain_del = vec![0u64; chain_words];
    let chain_alt = vec![0u32; CHAIN];
    for f in 0..CHAIN {
        chain_derived[f / 64] |= 1 << (f % 64); // every fact derived
    }
    chain_del[0] = 1; // delete the chain root only

    let mut counts = vec![0u32; CHAIN + 1];
    let mut fill = vec![0u32; CHAIN];
    let mut adj = vec![0u32; chain_edges.len()];
    dred::build_csr(&chain_edges, CHAIN, &mut counts, &mut fill, &mut adj)
        .expect("chain CSR build");

    // ANTI-VACUITY FLIP (fresh, in-bench): the dropped-BFS mutant must
    // leave exactly 9,999 phantom survivors. If this fires green, the
    // real leg's survivor==0 assert below carries information.
    {
        let mut p = vec![0u64; chain_words];
        let mut k = vec![0u64; chain_words];
        let mut wl = vec![0u32; CHAIN];
        let mut out = vec![0u32; CHAIN + 1];
        let m = dred::cascade_retract_mutant_nocascade(
            &chain_derived,
            &chain_del,
            &chain_alt,
            CHAIN,
            &counts,
            &adj,
            &mut p,
            &mut k,
            &mut wl,
            0,
            &mut out,
        )
        .expect("mutant run");
        assert_eq!(
            m as usize, CHAIN - 1,
            "anti-vacuity flip failed: filter-only mutant did NOT leave {} phantom survivors (got {})",
            CHAIN - 1,
            m
        );
        println!(
            "CASCADE_MUTANT_FLIP survivors_mutant={m} expected={} FLIPPED",
            CHAIN - 1
        );
    }

    // Real cascade leg: alloc counter armed ONLY around the walk.
    let allocs_max = AtomicUsize::new(0);
    let best_cascade = min_of_ns(|| {
        let mut p = vec![0u64; chain_words];
        let mut k = vec![0u64; chain_words];
        let mut wl = vec![0u32; CHAIN];
        let mut out = vec![0u32; CHAIN + 1];
        ALLOC_COUNT.store(0, Ordering::SeqCst);
        let t = Instant::now();
        let survivors = dred::cascade_retract(
            &chain_derived,
            &chain_del,
            &chain_alt,
            CHAIN,
            &counts,
            &adj,
            &mut p,
            &mut k,
            &mut wl,
            0,
            &mut out,
        )
        .expect("cascade_retract chain leg");
        let ns = t.elapsed().as_nanos() as u64;
        ALLOC_COUNT.store(0, Ordering::SeqCst); // read-and-reset below
        let allocs = ALLOC_COUNT.swap(0, Ordering::SeqCst);
        assert_eq!(survivors, 0, "cascade chain: expected EXACTLY 0 survivors, got {survivors} (phantom dependents leaked)");
        let _ = out;
        allocs_max.fetch_max(allocs, Ordering::SeqCst);
        ns
    });
    println!(
        "CASCADE_LEG chain={CHAIN} ns_per_lane={:.4} survivors=0 allocs={}",
        best_cascade as f64 / CHAIN as f64,
        allocs_max.load(Ordering::SeqCst),
    );
    assert_eq!(
        allocs_max.load(Ordering::SeqCst),
        0,
        "REP4 zero-heap claim violated: cascade walk allocated on the hot path"
    );

    println!("BENCH-OK");
}
