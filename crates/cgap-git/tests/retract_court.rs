//! Lane GIT-11 court: revert(commit-delta) -> cascade retraction over the
//! coordinate frame, cross-checked against the REAL DRed engine
//! (`crates/cgap-storage/src/dred.rs`, included verbatim below — disclosed
//! #[path]-style reuse; cgap-git's Cargo.toml is not this lane's file, and
//! dred.rs is the real engine per the B05 reality fence: 248M facts/s
//! honest min full-cascade, 839-944M facts/s keep-mask path ALONE — units
//! never conflated).
//!
//! Chicago discipline: real collaborators (the included dred source IS the
//! shipped engine), assert final state (survivor set-equality, orphan
//! counts), no mocks. Every court names its anti-vacuity mutation.

#[path = "../src/retract.rs"]
#[allow(dead_code)]
mod retract;

/// The REAL DRed engine, included verbatim (std test crate; dred.rs is
/// std-prelude-shaped and cannot compile inside no_std cgap-git proper).
#[path = "../../cgap-storage/src/dred.rs"]
#[allow(dead_code)]
mod dred_real;

use retract::{
    build_csr, cascade_oracle_rounds, compact_lanes, count_orphans, keep_mask_swar,
    project_delta_roots, revert_cascade, revert_cascade_mutant_one_step, splitmix64, RetractError,
    CORPUS_SEED,
};

/// Canonical chain corpus over the coordinate frame: facts 0..n form a
/// parent chain (f-1 -> f) plus chord edges every 8 facts; `derived_deps`
/// bit set iff the fact has a lineage parent; ~10% deleted roots
/// (Bernoulli, splitmix64); 25% alt-supported.
struct Frame {
    facts: usize,
    derived_deps: Vec<u64>,
    del_mask: Vec<u64>,
    alt_support: Vec<u32>,
    edges: Vec<(u32, u32)>,
}

fn build_frame(facts: usize, seed: u64) -> Frame {
    let words = facts.div_ceil(64);
    let mut st = seed;
    let mut derived_deps = vec![0u64; words];
    let mut del_mask = vec![0u64; words];
    let mut alt_support = vec![0u32; facts];
    let mut edges = Vec::new();
    for f in 0..facts {
        if f > 0 {
            edges.push(((f - 1) as u32, f as u32));
            derived_deps[f / 64] |= 1u64 << (f % 64);
        }
        if f > 0 && f % 8 == 0 {
            edges.push((0, f as u32));
        }
    }
    for f in 0..facts {
        if splitmix64(&mut st) % 100 < 10 {
            del_mask[f / 64] |= 1u64 << (f % 64);
        }
        if splitmix64(&mut st) % 100 < 25 {
            alt_support[f] = 1 + (splitmix64(&mut st) % 7) as u32;
        }
    }
    Frame {
        facts,
        derived_deps,
        del_mask,
        alt_support,
        edges,
    }
}

fn run_cascade(fr: &Frame) -> Vec<u32> {
    let words = fr.derived_deps.len();
    let mut counts = vec![0u32; fr.facts + 1];
    let mut fill = vec![0u32; fr.facts];
    let mut adj = vec![0u32; fr.edges.len()];
    build_csr(&fr.edges, fr.facts, &mut counts, &mut fill, &mut adj).expect("csr");
    let mut purged = vec![0u64; words];
    let mut keep = vec![0u64; words];
    let mut worklist = vec![0u32; fr.facts];
    let mut out = vec![0u32; words * 64 + 1];
    let n = revert_cascade(
        &fr.derived_deps,
        &fr.del_mask,
        &fr.alt_support,
        fr.facts,
        &counts,
        &adj,
        &mut purged,
        &mut keep,
        &mut worklist,
        0,
        &mut out,
    )
    .expect("cascade");
    out.truncate(n as usize);
    out
}

/// COURT 1 (retraction correctness, dual oracle): the git-frame fast path
/// must produce the IDENTICAL surviving set as (a) the local scalar
/// rounds oracle AND (b) the REAL dred::cascade_oracle over the same
/// corpus. Anti-vacuity mutation: a hand-derived 5-fact chain KAT with a
/// known survivor set; also post-retraction integrity must be exactly
/// ZERO orphan derivations (anti-phantom law).
#[test]
fn court_cascade_matches_dual_oracle() {
    // KAT: chain 0->1->2->3->4; revert delta roots {0}; no alt support:
    // the whole chain purges. With alt on fact 2: {0,1,3,4} purge, 2 and
    // its downstream survive.
    let words = 1usize;
    let mut dd = vec![0b1_1110u64; 1]; // facts 1..4 derived
    let mut dm = vec![0b0_0001u64; 1]; // root 0 deleted
    let mut alt = vec![0u32; 5];
    let edges: Vec<(u32, u32)> = vec![(0, 1), (1, 2), (2, 3), (3, 4)];
    let _ = (words, &mut dd, &mut dm);
    dd = vec![0b11110u64];
    dm = vec![0b00001u64];
    let mut counts = vec![0u32; 6];
    let mut fill = vec![0u32; 5];
    let mut adj = vec![0u32; edges.len()];
    build_csr(&edges, 5, &mut counts, &mut fill, &mut adj).expect("csr");
    let mut purged = vec![0u64; 1];
    let mut keep = vec![0u64; 1];
    let mut worklist = vec![0u32; 5];
    let mut out = vec![0u32; 65];
    let n = revert_cascade(
        &dd,
        &dm,
        &alt,
        5,
        &counts,
        &adj,
        &mut purged,
        &mut keep,
        &mut worklist,
        0,
        &mut out,
    )
    .expect("cascade");
    assert_eq!(
        &out[..n as usize],
        &[] as &[u32],
        "no-alt chain purges fully"
    );
    alt[2] = 3; // fact 2 has an alternative derivation
    let n = revert_cascade(
        &dd,
        &dm,
        &alt,
        5,
        &counts,
        &adj,
        &mut purged,
        &mut keep,
        &mut worklist,
        0,
        &mut out,
    )
    .expect("cascade");
    assert_eq!(
        &out[..n as usize],
        &[2u32, 3, 4][..],
        "alt-supported fact 2 rescues 2..4"
    );
    // Corpus-level dual-oracle parity at >= 3 entropy shapes.
    for facts in [64usize, 1000, 4096] {
        let fr = build_frame(facts, CORPUS_SEED ^ facts as u64);
        let fast = run_cascade(&fr);
        let mut lout = vec![0u32; fr.derived_deps.len() * 64 + 1];
        let ln = cascade_oracle_rounds(
            &fr.derived_deps,
            &fr.del_mask,
            &fr.alt_support,
            fr.facts,
            &fr.edges,
            &mut lout,
        );
        let real = dred_real::cascade_oracle(
            &fr.derived_deps,
            &fr.del_mask,
            &fr.alt_support,
            fr.facts,
            &fr.edges,
        );
        assert_eq!(
            &fast[..],
            &lout[..ln as usize],
            "fast vs local oracle, facts {facts}"
        );
        assert_eq!(
            fast, real,
            "fast vs REAL dred::cascade_oracle, facts {facts}"
        );
        let (orphans, first) = count_orphans(
            &keep_of(&fr),
            &fr.derived_deps,
            &fr.alt_support,
            &fr.edges,
            fr.facts,
        );
        assert_eq!(
            (orphans, first),
            (0, u32::MAX),
            "zero orphans post-retraction, facts {facts}"
        );
    }
}

/// Re-run the fast path and return the keep mask (for orphan audits).
fn keep_of(fr: &Frame) -> Vec<u64> {
    let words = fr.derived_deps.len();
    let mut counts = vec![0u32; fr.facts + 1];
    let mut fill = vec![0u32; fr.facts];
    let mut adj = vec![0u32; fr.edges.len()];
    build_csr(&fr.edges, fr.facts, &mut counts, &mut fill, &mut adj).expect("csr");
    let mut purged = vec![0u64; words];
    let mut keep = vec![0u64; words];
    let mut worklist = vec![0u32; fr.facts];
    let mut out = vec![0u32; words * 64 + 1];
    let _ = revert_cascade(
        &fr.derived_deps,
        &fr.del_mask,
        &fr.alt_support,
        fr.facts,
        &counts,
        &adj,
        &mut purged,
        &mut keep,
        &mut worklist,
        0,
        &mut out,
    )
    .expect("cascade");
    keep
}

/// COURT 2 (ANTI-VACUITY MUTATION TWIN, exit-1-class typed Err): a
/// driver that suppresses ONE cascade step (`revert_cascade_mutant_one_step`)
/// must inject orphan derivations on the chain corpus, and the integrity
/// driver must return the typed `RetractError::OrphanDerivation`. The
/// guarded driver returns Ok on the same corpus. A court that cannot flip
/// is FAIL.
#[test]
fn court_mutant_one_step_flips_typed() {
    let facts = 10_000usize;
    let fr = build_frame(facts, CORPUS_SEED ^ 0x6111);
    // Guarded path: integrity Ok, zero orphans.
    let guarded = integrity_driver(&fr, false);
    assert!(
        guarded.is_ok(),
        "guarded driver must pass integrity: {guarded:?}"
    );
    // Mutant path: one suppressed cascade step -> injected orphans ->
    // typed Err (exit-1-class).
    let mutated = integrity_driver(&fr, true);
    match mutated {
        Err(RetractError::OrphanDerivation { orphans, first }) => {
            assert!(orphans > 0, "mutant must inject orphans");
            assert!(first != u32::MAX);
            std::println!(
                "MUTANT FLIP: {} orphan derivations injected (first coord {})",
                orphans,
                first
            );
        }
        other => panic!("mutant must yield typed OrphanDerivation, got {other:?}"),
    }
    // Raw orphan evidence independent of the driver.
    let words = fr.derived_deps.len();
    let mut counts = vec![0u32; facts + 1];
    let mut fill = vec![0u32; facts];
    let mut adj = vec![0u32; fr.edges.len()];
    build_csr(&fr.edges, facts, &mut counts, &mut fill, &mut adj).expect("csr");
    let mut purged = vec![0u64; words];
    let mut keep = vec![0u64; words];
    let mut worklist = vec![0u32; facts];
    let mut out = vec![0u32; words * 64 + 1];
    let _ = revert_cascade_mutant_one_step(
        &fr.derived_deps,
        &fr.del_mask,
        &fr.alt_support,
        facts,
        &counts,
        &adj,
        &mut purged,
        &mut keep,
        &mut worklist,
        0,
        &mut out,
    )
    .expect("mutant run");
    let (orphans, _) = count_orphans(&keep, &fr.derived_deps, &fr.alt_support, &fr.edges, facts);
    assert!(
        orphans > 0,
        "raw mutant keep set carries orphans: VACUOUS court"
    );
}

/// Production-shaped driver: run retraction (optionally with the
/// suppressed-step MUTANT), then the anti-phantom integrity gate. Typed
/// Err on any orphan — this is the exit-1-class surface the court flips.
fn integrity_driver(fr: &Frame, suppress_one_cascade_step: bool) -> Result<Vec<u32>, RetractError> {
    let facts = fr.facts;
    let words = fr.derived_deps.len();
    let mut counts = vec![0u32; facts + 1];
    let mut fill = vec![0u32; facts];
    let mut adj = vec![0u32; fr.edges.len()];
    build_csr(&fr.edges, facts, &mut counts, &mut fill, &mut adj)?;
    let mut purged = vec![0u64; words];
    let mut keep = vec![0u64; words];
    let mut worklist = vec![0u32; facts];
    let mut out = vec![0u32; words * 64 + 1];
    let survivors = if suppress_one_cascade_step {
        revert_cascade_mutant_one_step(
            &fr.derived_deps,
            &fr.del_mask,
            &fr.alt_support,
            facts,
            &counts,
            &adj,
            &mut purged,
            &mut keep,
            &mut worklist,
            0,
            &mut out,
        )?
    } else {
        revert_cascade(
            &fr.derived_deps,
            &fr.del_mask,
            &fr.alt_support,
            facts,
            &counts,
            &adj,
            &mut purged,
            &mut keep,
            &mut worklist,
            0,
            &mut out,
        )?
    };
    let (orphans, first) =
        count_orphans(&keep, &fr.derived_deps, &fr.alt_support, &fr.edges, facts);
    if orphans > 0 {
        return Err(RetractError::OrphanDerivation { orphans, first });
    }
    Ok(out[..survivors as usize].to_vec())
}

/// COURT 3 (throughput, MEASURED AND REPORTED — no hard gate): N=15
/// release-profile min/median facts/s of the FULL cascade (CSR lookup +
/// BFS fixpoint + SWAR keep pass + branchless compaction; CSR build
/// excluded, matching the B05 counting basis) over a 100,000-fact chain
/// frame. Units: facts/s, basis = facts / elapsed of one revert_cascade
/// call. GATE POLICY (stated, per dispatch): an honest measured number
/// with no gate beats a tuned gate — B05 measured 248M facts/s honest min
/// against an 800M aspiration and the same engine shape here would make
/// any threshold either vacuous or tuned; the number is reported, not
/// gated. Determinism: survivor count identical across all runs.
#[test]
fn court_throughput_measured_reported() {
    // ZP-18 timing fence: the min/median facts/s throughput report is a
    // release-profile measurement; under dev the court loud-SKIPs instead
    // of reporting debug-code throughput.
    if let Err(v) = cgap_aoq::profile_guard::assert_release() {
        eprintln!("LOUD-SKIP court_throughput_measured_reported: {v}");
        return;
    }
    let facts = 100_000usize;
    let fr = build_frame(facts, CORPUS_SEED ^ 0xBEEF);
    let words = fr.derived_deps.len();
    let mut counts = vec![0u32; facts + 1];
    let mut fill = vec![0u32; facts];
    let mut adj = vec![0u32; fr.edges.len()];
    build_csr(&fr.edges, facts, &mut counts, &mut fill, &mut adj).expect("csr");
    let mut purged = vec![0u64; words];
    let mut keep = vec![0u64; words];
    let mut worklist = vec![0u32; facts];
    let mut out = vec![0u32; words * 64 + 1];
    // warm + determinism + correctness anchor
    let n0 = revert_cascade(
        &fr.derived_deps,
        &fr.del_mask,
        &fr.alt_support,
        facts,
        &counts,
        &adj,
        &mut purged,
        &mut keep,
        &mut worklist,
        0,
        &mut out,
    )
    .expect("cascade");
    let mut lout = vec![0u32; words * 64 + 1];
    let oracle_n = cascade_oracle_rounds(
        &fr.derived_deps,
        &fr.del_mask,
        &fr.alt_support,
        facts,
        &fr.edges,
        &mut lout,
    ) as usize;
    assert_eq!(n0 as usize, oracle_n, "timed path equals oracle count");
    let mut best = u128::MAX;
    let mut total = 0u128;
    let runs = 15u32;
    for _ in 0..runs {
        for w in purged.iter_mut() {
            *w = 0;
        }
        let t = std::time::Instant::now();
        let n = revert_cascade(
            &fr.derived_deps,
            &fr.del_mask,
            &fr.alt_support,
            facts,
            &counts,
            &adj,
            &mut purged,
            &mut keep,
            &mut worklist,
            0,
            &mut out,
        )
        .expect("cascade");
        let ns = t.elapsed().as_nanos();
        // ZP-18 anti-DCE fence on the measured timing value.
        cgap_aoq::profile_guard::fence(ns);
        assert_eq!(n, n0, "survivor count must be run-invariant");
        if ns < best {
            best = ns;
        }
        total += ns;
    }
    let mean = total / runs as u128;
    let fbest = facts as f64 / (best as f64 / 1e9);
    let fmean = facts as f64 / (mean as f64 / 1e9);
    std::println!(
        "THROUGHPUT basis=facts/elapsed(revert_cascade incl BFS+SWAR+compact, CSR build excluded) facts={} runs={}",
        facts, runs
    );
    std::println!(
        "THROUGHPUT min = {:.1} facts/s ({:.3} us)",
        fbest,
        best as f64 / 1e3
    );
    std::println!(
        "THROUGHPUT mean = {:.1} facts/s ({:.3} us)",
        fmean,
        mean as f64 / 1e3
    );
    std::println!("THROUGHPUT GATE POLICY: reported, NOT gated (honest number; B05 precedent)");
    // Anti-vacuity for THIS court: the measurement must be real work —
    // the timed call must purge a nonzero set (else the bench is empty).
    let survivors = n0 as usize;
    assert!(survivors < facts, "retraction must purge something on this corpus (survivors {survivors}/{facts}): VACUOUS bench");
    assert!(best < u128::MAX);
}

/// COURT 4 (typed lease refusals + delta projection KAT): build_csr
/// refuses bad leases/out-of-range edges with typed errors; the
/// revert-delta projection sets exactly the requested bits.
#[test]
fn court_typed_refusals_and_delta_projection() {
    let edges = [(0u32, 1u32), (1u32, 2u32)];
    let mut counts = [0u32; 4];
    let mut fill = [0u32; 3];
    let mut adj = [0u32; 2];
    assert!(build_csr(&edges, 3, &mut counts, &mut fill, &mut adj).is_ok());
    let mut adj_short = [0u32; 1];
    assert_eq!(
        build_csr(&edges, 3, &mut [0u32; 4], &mut [0u32; 3], &mut adj_short),
        Err(RetractError::AdjacencyLease {
            needed: 2,
            provided: 1
        })
    );
    assert_eq!(
        build_csr(
            &[(0u32, 9u32)],
            3,
            &mut [0u32; 4],
            &mut [0u32; 3],
            &mut [0u32; 1]
        ),
        Err(RetractError::EdgeOutOfRange {
            edge_index: 0,
            endpoint: 9,
            facts: 3
        })
    );
    // Delta projection: reverting a delta of facts {1, 64} sets bits 1
    // and word-1 bit-0 exactly.
    let mut dm = vec![0u64; 2];
    project_delta_roots(&[1, 64, 1], &mut dm);
    assert_eq!(dm, vec![0b10u64, 1u64], "idempotent bit projection");
    // SWAR keep mask KAT: derived+deleted+no-alt purges; alt survives;
    // partial tail word padding purged.
    let kd = [0b0111u64];
    let km = [0b0011u64];
    let ka = [0u32, 5, 0];
    let mut kk = [0u64; 1];
    keep_mask_swar(&kd, &km, &ka, 3, &mut kk);
    // fact0: derived+del+noalt -> 0; fact1: del+alt -> 1; fact2: neither -> 1.
    assert_eq!(kk, [0b110u64]);
    let mut kpad = [0u64; 1];
    keep_mask_swar(&[u64::MAX], &[u64::MAX], &[1u32; 3], 3, &mut kpad);
    assert_eq!(kpad, [0b111u64], "padding lanes beyond facts purged");
    let _ = compact_lanes(&[0b1010u64], 0, &mut [0u32; 65]);
}
