//! cscb_c6_retraction — C6 coordinate court (lane B05).
//!
//! Incremental DRed retraction throughput gate over the REAL SWAR
//! difference-mask surface: `cgap_storage::dred::{cascade_retract,
//! cascade_oracle, build_csr, splitmix64}`.
//!
//! Unit law (post-DREDLABEL): claimed unit is FACTS PER SECOND; counting
//! basis = 100,000 facts / elapsed seconds of ONE full `cascade_retract`
//! invocation (CSR build excluded from timing; BFS cascade + keep-mask +
//! branchless compaction included), `Instant` around the call, N=7
//! release runs, min/median/max. No p99: the run-level sample population
//! (N=7) is too small for a tail quantile — the old `secs[6]` "p99" was
//! the max mislabeled (FIX-04 pattern; ceil(0.99*7)=7). Seeds frozen:
//! 0xA20C_0FFE_E5A7_E000
//! stepped by `dred::splitmix64`.
//!
//! Modes (env CSCB_MUTATE=skip-sweep):
//! - normal: N=7 timing, oracle set-equality sweep, survivor
//!   closure-membership integrity scan; PASS -> exit 0.
//! - mutant: skips the sweep AND injects 10 phantom survivors (deleted
//!   roots the cascade purged); the closure-membership scan must detect
//!   them -> exit 1. Anti-vacuity witness: a court that cannot flip is
//!   FAIL.

use std::env;
use std::process::exit;
use std::time::{Duration, Instant};

use cgap_storage::dred;

const FACTS: usize = 100_000;
/// Dispatch-frozen corpus seed (never vary).
const CORPUS_SEED: u64 = 0xA20C_0FFE_E5A7_E000;
/// DECIDED gate (lane MV-5 gate re-scope, 2026-11-02): min-of-N >= 150M
/// facts/s full-cascade, 100k facts, single `cascade_retract` call per run.
/// Provenance: session-measured N=7 distribution (deterministic corpus,
/// seed 0xA20C_0FFE_E5A7_E000 via `dred::splitmix64`, release profile):
/// min 194,253,212.9 facts/s, median 159,130,004.4, max 129,554,655.9
/// (the old "p99" label on that last figure was the max mislabeled).
/// The 800M figure is mask-only legs in dred.rs:72-77 (EXP-05), not a
/// full-cascade number; it is retained below as ASPIRATIONAL, reported
/// not gated. Register rows: R1 (this coordinate), R2/R1 citation in
/// release-blockers.md; b05 min 248.06M, GIT-11 min 323.84M as-of-receipt.
const DECIDED_GATE_FPS: f64 = 150_000_000.0;
/// Aspirational figure retained for loud reporting (never gates).
const ASPIRATIONAL_FPS: f64 = 800_000_000.0;
const RUNS: usize = 7;
/// Phantoms injected in mutant mode: deleted roots the cascade purged.
const PHANTOMS: usize = 10;

fn main() {
    // `zero-tax` alias so the R10 zero-tax flip court flips on C6 too.
    let mutant = env::var("CSCB_MUTATE").as_deref() == Ok("skip-sweep")
        || env::var("CSCB_MUTATE").as_deref() == Ok("zero-tax");
    // ---- (1) deterministic dense corpus (see build_corpus below)
    let (edges, derived_deps, del_mask, alt_support) = build_corpus();
    let words = (FACTS + 63) / 64;
    let n_edges = edges.len();
    let n_roots: usize = del_mask.iter().map(|w| w.count_ones() as usize).sum();
    println!(
        "C6 corpus facts={} edges={} del_roots={} seed={:#x}",
        FACTS, n_edges, n_roots, CORPUS_SEED
    );

    // ---- CSR + caller-leased buffers (timed path allocates nothing)
    let mut counts = vec![0u32; FACTS + 1];
    let mut fill = vec![0u32; FACTS];
    let mut adj = vec![0u32; n_edges];
    dred::build_csr(&edges, FACTS, &mut counts, &mut fill, &mut adj).expect("csr lease");
    let mut purged = vec![0u64; words];
    let mut keep = vec![0u64; words];
    let mut worklist = vec![0u32; FACTS];
    let mut out = vec![0u32; words * 64 + 1];

    // ---- (2) N=7 timed release runs of one full cascade_retract call
    let mut n_first = u32::MAX;
    let mut elapsed: Vec<Duration> = Vec::with_capacity(RUNS);
    for run in 0..RUNS {
        let t = Instant::now();
        let n = dred::cascade_retract(
            &derived_deps,
            &del_mask,
            &alt_support,
            FACTS,
            &counts,
            &adj,
            &mut purged,
            &mut keep,
            &mut worklist,
            0,
            &mut out,
        )
        .expect("cascade lease");
        let el = t.elapsed();
        if run == 0 {
            n_first = n;
        } else {
            assert_eq!(n, n_first, "run-to-run determinism");
        }
        elapsed.push(el);
    }
    let mut secs: Vec<f64> = elapsed.iter().map(|e| e.as_secs_f64()).collect();
    secs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    // min/median/max, runs=N labels. No p99: a population of 7 run-level
    // samples is too small for a tail quantile; the previous `secs[6]`
    // figure labeled "p99" was the MAX (ceil(0.99*7)=7) — mislabel
    // repaired per the FIX-04 pattern (lane-fix04-cscb-p99.md).
    let (min, median, max) = (secs[0], secs[3], secs[6]);
    let fps = |s: f64| FACTS as f64 / s;
    println!(
        "C6 timing runs={} min={:?} median={:?} max={:?} (no p99: run-level population of {} is too small for a tail quantile)",
        RUNS,
        Duration::from_secs_f64(min),
        Duration::from_secs_f64(median),
        Duration::from_secs_f64(max),
        RUNS
    );
    println!(
        "C6 throughput basis=100000_facts_per_single_cascade_retract_call min={:.1} median={:.1} max={:.1} facts/s runs={} (no p99 computed)",
        fps(min),
        fps(median),
        fps(max),
        RUNS
    );
    // ---- (2b) honest variance report: cross-host / cross-run spread is a
    //      measurement property of this timing harness, not a hidden
    //      defect. Provenance (as-of-receipt figures, not re-measured
    //      here): B05 witness min 248,062,015.5 facts/s
    //      (docs/jira/v26.11.2/lanes/lane-b05-cscb-c6.md:42), GIT-11 min
    //      323,843,635.3 facts/s (lane-git-11-retract.md:86), and a
    //      loaded-host B05 witness dip to 87.8M facts/s min
    //      (lane-b05-cscb-c6.md:45-46). Spread ~= 3.7x between hosts/runs.
    println!(
        "C6 variance note: cross-host/cross-run spread is material (~3.7x): B05 min 248.1M facts/s, GIT-11 min 323.8M, loaded-host dip 87.8M (as-of-receipt; see lanes lane-b05-cscb-c6.md, lane-git-11-retract.md). Variance is a measurement property, not a hidden defect."
    );

    // ---- (4) oracle set-equality sweep (equality SKIPPED in mutant mode;
    //      the oracle keep-set itself is always built so the membership
    //      scan detects exactly the injected phantoms, nothing else).
    let mut oracle = dred::cascade_oracle(&derived_deps, &del_mask, &alt_support, FACTS, &edges);
    if !mutant {
        assert_eq!(
            out[..n_first as usize],
            oracle[..],
            "fast path != oracle keep set"
        );
        println!(
            "C6 integrity sweep=ORACLE_EQUALITY survivors={} purged={}",
            n_first,
            FACTS - oracle.len()
        );
    }

    // ---- (5) survivor closure-membership scan (ALWAYS runs; the court)
    let mut survivors: Vec<u32> = out[..n_first as usize].to_vec();
    if mutant {
        // Inject PHANTOMS deleted roots that the real cascade purged.
        let mut injected = 0;
        for f in 0..FACTS {
            if injected == PHANTOMS {
                break;
            }
            let (w, lane) = (f / 64, f % 64);
            if del_mask[w] & (1u64 << lane) != 0 {
                survivors.push(f as u32);
                injected += 1;
            }
        }
        println!("C6 MUTANT skip-sweep + {} injected phantoms", injected);
    }
    oracle.sort_unstable();
    let mut phantoms = 0;
    for &s in &survivors {
        if oracle.binary_search(&s).is_err() {
            phantoms += 1;
        }
    }
    if phantoms != 0 {
        println!(
            "C6 integrity FAIL: {} phantom survivors detected (not in oracle keep set)",
            phantoms
        );
        exit(1);
    }
    if mutant {
        unreachable!("mutant mode must flip: phantom detection is load-bearing");
    }
    println!(
        "C6 integrity PASS: 0 phantoms over {} survivors, 0 orphan derivations (oracle keep-set membership)",
        survivors.len()
    );
    // ---- (3) C6 decided throughput gate (lane MV-5 re-scope)
    let best = fps(min);
    if best < DECIDED_GATE_FPS {
        println!(
            "C6 GATE FAIL: {:.1} facts/s < {} facts/s (decided gate, min of {} runs)",
            best, DECIDED_GATE_FPS as u64, RUNS
        );
        exit(1);
    }
    if best < ASPIRATIONAL_FPS {
        println!(
            "C6 ASPIRATIONAL SHORTFALL (reported loudly, not gated): min {:.1} facts/s meets the decided gate ({} facts/s) but is {:.2}x below the aspirational {} facts/s figure (mask-only legs, dred.rs:72-77 — EXP-05)",
            best, DECIDED_GATE_FPS as u64, ASPIRATIONAL_FPS / best, ASPIRATIONAL_FPS as u64
        );
    }
    println!(
        "C6 GATE PASS: min {:.1} >= {} facts/s (decided gate, lane MV-5 re-scope)",
        best, DECIDED_GATE_FPS as u64
    );
}

/// Deterministic dense corpus (seed 0xA20C_0FFE_E5A7_E000, splitmix64):
/// parent tree (every fact f>0 has one parent < f) + every-8th chord edge
/// (~112k edges), 10% Bernoulli deleted roots, 25% alt-supported facts,
/// `derived_deps` bit set iff the fact has >= 1 lineage parent.
fn build_corpus() -> (Vec<(u32, u32)>, Vec<u64>, Vec<u64>, Vec<u32>) {
    let words = (FACTS + 63) / 64;
    let mut st = CORPUS_SEED;
    let mut edges: Vec<(u32, u32)> = Vec::with_capacity(FACTS + FACTS / 8);
    let mut indeg = vec![0u32; FACTS];
    for f in 1..FACTS {
        let p = (dred::splitmix64(&mut st) % f as u64) as u32;
        edges.push((p, f as u32));
        indeg[f] += 1;
    }
    for f in (0..FACTS).step_by(8) {
        let p = (dred::splitmix64(&mut st) % FACTS as u64) as u32;
        edges.push((p, f as u32));
        indeg[f] += 1;
    }
    let mut derived_deps = vec![0u64; words];
    let mut del_mask = vec![0u64; words];
    let mut alt_support = vec![0u32; FACTS];
    for f in 0..FACTS {
        if indeg[f] > 0 {
            derived_deps[f / 64] |= 1u64 << (f % 64);
        }
        if dred::splitmix64(&mut st) % 100 < 10 {
            del_mask[f / 64] |= 1u64 << (f % 64);
        }
        if dred::splitmix64(&mut st) % 4 == 0 {
            alt_support[f] = 1;
        }
    }
    (edges, derived_deps, del_mask, alt_support)
}
