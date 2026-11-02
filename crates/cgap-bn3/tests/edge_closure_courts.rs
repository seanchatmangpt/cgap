//! C03 — BN3 edge/closure edge courts (v26.11.2 lane C03).
//!
//! Chicago courts over the transitive-closure fixpoint + L3 radix join
//! probe surface (cgap-bn3 `unify.rs`). Real collaborators, final-state
//! asserts, no mocks.
//!
//! Courts:
//!   E1 zero-edge graph: fixpoint over an empty store derives nothing
//!      and converges round 1 (no FixpointBudgetExceeded).
//!   E2 single-node self-loop: the closure rule self-joins the loop row
//!      exactly once (x==y==z) and the fixpoint adds no derived fact.
//!   E3 saturated radix bucket: every join key lands in bucket 0
//!      (RADIX_BUCKETS=4096, hash `(y ^ y>>16) & 0x0FFF` over the
//!      56-bit payload window, unify.rs:850-854); the bucketed probe
//!      must still emit exactly the true join set (superset filter,
//!      full-word compare) — no phantom joins.
//!   E4 cascading retraction across multiple derivation paths.
//!   E5 zero-alloc steady state via the AE7 EnclaveTrackingAlloc
//!      (#[path]-included: cgap-core lib.rs does not export
//!      tracking_alloc; same pattern as
//!      crates/cgap/tests/alloc_boundary.rs:50-53).
//!
//! Every court names its anti-vacuity twin (the input mutation that
//! MUST flip the positive assertion), run in the same test fn.
//!
//! SEED LAW: no RNG — all corpora are deterministic literals; the two
//! campaign seeds are not used and never conflated.

use std::alloc::System;

use cgap_bn3::sigil::{encode_term, Sigil};
use cgap_bn3::unify::{
    unify_batch_probe, unify_to_fixpoint, BodyAtom, EvalScratch, JoinProbeScratch, MatchSet, Rule,
    RuleCompiler, StatementColumns, DEFAULT_CHUNK_ROWS, RADIX_BUCKETS,
};

// AE7 tracking allocator: cgap-core's lib.rs exports only dma_ring /
// frame / isomorphism — tracking_alloc.rs is NOT re-exported (recorded
// divergence in the lane receipt), so it is included by #[path], the
// landed pattern (crates/cgap/tests/alloc_boundary.rs:50-53). The module
// is standalone (core+std only, no cgap deps), so it compiles inside
// this cgap-bn3 test binary without any Cargo dependency change.
#[path = "../../cgap-core/src/tracking_alloc.rs"]
mod tracking_alloc;

use tracking_alloc::{last_breach, reset_breach, start_hot_path_budget, window_counts};

#[global_allocator]
static ALLOC: tracking_alloc::EnclaveTrackingAlloc<System> =
    tracking_alloc::EnclaveTrackingAlloc { inner: System };

// ---------------------------------------------------------------------
// Term-word builders and the shared closure rule
// ---------------------------------------------------------------------

/// A11 SlotConst term word for payload `v`.
fn cst(v: u64) -> u64 {
    encode_term(Sigil::SlotConst, v).expect("in-range payload")
}

/// A11 VarUni term word for variable register `r` (0,1,2 for this rule).
fn vr(r: u64) -> u64 {
    encode_term(Sigil::VarUni, r).expect("in-range register")
}

/// "edge" predicate term word.
const E_PRED: u64 = 0x000A_0000_0000_0001;

/// Transitive-closure rule: (?x E ?y), (?y E ?z) => (?x E ?z).
fn closure_rule() -> Rule {
    Rule {
        rule_id: 42,
        body: vec![
            BodyAtom::new(vr(0), cst(E_PRED), vr(1)),
            BodyAtom::new(vr(1), cst(E_PRED), vr(2)),
        ],
        head: [vr(0), cst(E_PRED), vr(2)],
    }
}

/// SoA columns over (s, o) pairs with the fixed E predicate.
fn cols_of(pairs: &[(u64, u64)]) -> StatementColumns<'static> {
    // Leak-free for a court binary: the leaked backing store is the
    // corpus itself (single allocation per court, lives for the whole
    // run); alternatives would thread lifetimes through every helper.
    let mut s: Vec<u64> = Vec::new();
    let mut p: Vec<u64> = Vec::new();
    let mut o: Vec<u64> = Vec::new();
    for (a, b) in pairs {
        s.push(cst(*a));
        p.push(cst(E_PRED));
        o.push(cst(*b));
    }
    let bs: &'static mut [u64] = Box::leak(s.into_boxed_slice());
    let bp: &'static mut [u64] = Box::leak(p.into_boxed_slice());
    let bo: &'static mut [u64] = Box::leak(o.into_boxed_slice());
    StatementColumns::new(bs, bp, bo).expect("aligned columns")
}

// ---------------------------------------------------------------------
// Independent oracle: BFS closure over (s,o) payload pairs
// ---------------------------------------------------------------------

/// Independent transitive-closure oracle: BFS from every node over the
/// base edge list. A genuinely different algorithm from the kernel's
/// nested-loop join + fixpoint driver (the differential-court pattern
/// of bgp_hop_courts / lever_courts).
fn oracle_closure(pairs: &[(u64, u64)]) -> std::collections::BTreeSet<(u64, u64)> {
    let mut adj: std::collections::BTreeMap<u64, Vec<u64>> = Default::default();
    for (a, b) in pairs {
        adj.entry(*a).or_default().push(*b);
    }
    let mut out = std::collections::BTreeSet::new();
    for &start in adj.keys() {
        let mut seen: std::collections::BTreeSet<u64> = Default::default();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start);
        seen.insert(start);
        while let Some(node) = queue.pop_front() {
            for &nxt in adj.get(&node).into_iter().flatten() {
                out.insert((start, nxt));
                if seen.insert(nxt) {
                    queue.push_back(nxt);
                }
            }
        }
    }
    out
}

/// Run the closure rule to fixpoint and extract the DERIVED set (facts
/// beyond the base edges) as payload (s,o) pairs. Batches may re-emit
/// re-derivations across epochs; the BTreeSet dedupes.
fn derived_set(pairs: &[(u64, u64)]) -> std::collections::BTreeSet<(u64, u64)> {
    let cols = cols_of(pairs);
    let batches = unify_to_fixpoint(&[closure_rule()], &cols).expect("fixpoint converges");
    let base: std::collections::BTreeSet<(u64, u64)> = pairs.iter().copied().collect();
    let mut got = std::collections::BTreeSet::new();
    for b in &batches {
        for i in 0..b.rows.len() {
            assert_eq!(b.rows.p[i], cst(E_PRED), "derived predicate is E");
            got.insert((b.rows.s[i], b.rows.o[i]));
        }
    }
    // Strip the cst() sigil offset back to payload: derived words are
    // cst(payload) words built by the projector from head templates.
    let off = cst(0);
    got.iter()
        .map(|(s, o)| (s - off, o - off))
        .collect::<std::collections::BTreeSet<(u64, u64)>>()
        .difference(&base)
        .copied()
        .collect()
}

// ---------------------------------------------------------------------
// E1 — zero-edge graph
// ---------------------------------------------------------------------

/// E1 positive: fixpoint over an EMPTY store. Converges (Ok, no
/// FixpointBudgetExceeded) round 1 and derives nothing.
/// Anti-vacuity twin (same fn): adding a 2-edge chain a->b->c MUST
/// flip the assertion — the derived set becomes {a->c} (nonempty).
#[test]
fn e1_zero_edge_graph_derives_nothing_and_converges() {
    let empty: Vec<(u64, u64)> = Vec::new();
    let cols = cols_of(&empty);
    let batches =
        unify_to_fixpoint(&[closure_rule()], &cols).expect("empty store: fixpoint converges");
    // Round 1 emits one empty batch per rule, then converges.
    assert_eq!(batches.len(), 1, "one round, one batch, zero new rows");
    assert_eq!(batches[0].rule_id, 42);
    assert_eq!(batches[0].rows.len(), 0, "zero-edge graph: no derivations");

    // Twin (M-E1): same court, one flipped input — a 2-edge chain. The
    // positive assertion (zero derived) MUST flip to {a->c}.
    let chain = vec![(1, 2), (2, 3)];
    let got = derived_set(&chain);
    let want: std::collections::BTreeSet<(u64, u64)> = [(1u64, 3u64)].into_iter().collect();
    assert_eq!(got, want, "twin: chain derives (a->c) — court flips");
}

// ---------------------------------------------------------------------
// E2 — single-node self-loop
// ---------------------------------------------------------------------

/// E2 positive: store {n->n}. The closure rule's self-join over the
/// loop row matches EXACTLY once with x==y==z==n (probe-level, radix
/// path on), and the fixpoint derives no NEW fact (n->n already in the
/// store; a self-loop adds no information).
/// Anti-vacuity twin (M-E2): flip the loop edge n->n into m->n — the
/// self-join match count MUST drop 1 -> 0 (x==y no longer holds).
#[test]
fn e2_self_loop_self_joins_once_and_derives_nothing() {
    const N: u64 = 0x000A_0000_0000_0100;
    let loop_edges = vec![(N, N)];

    // Probe-level: compile the rule, evaluate with a warm scratch lease.
    let cols = cols_of(&loop_edges);
    let compiled = RuleCompiler::compile(&closure_rule()).expect("rule compiles");
    let mut scratch = EvalScratch::default();
    let mut out = MatchSet::default();
    let mut probe = JoinProbeScratch::default();
    assert!(probe.radix, "L3 radix probe is the default path");
    unify_batch_probe(
        &compiled,
        &cols,
        DEFAULT_CHUNK_ROWS,
        &mut scratch,
        &mut out,
        &mut probe,
    )
    .expect("probe succeeds");
    assert_eq!(out.len, 1, "self-join matches exactly once");
    assert_eq!(out.atom_rows[0][0], out.atom_rows[1][0], "same row twice");
    assert_eq!(out.var_values[0][0], cst(N), "?x == n");
    assert_eq!(out.var_values[1][0], cst(N), "?y == n");
    assert_eq!(out.var_values[2][0], cst(N), "?z == n");

    // Fixpoint-level: no NEW derived facts.
    let got = derived_set(&loop_edges);
    assert!(got.is_empty(), "self-loop derives nothing new, got {got:?}");

    // Twin (M-E2): flip the edge n->n -> m->n. x==y broken: 0 matches.
    let flipped = vec![(0x000A_0000_0000_0200u64, N)];
    let cols2 = cols_of(&flipped);
    let mut out2 = MatchSet::default();
    unify_batch_probe(
        &compiled,
        &cols2,
        DEFAULT_CHUNK_ROWS,
        &mut scratch,
        &mut out2,
        &mut probe,
    )
    .expect("probe succeeds on flipped edge");
    assert_eq!(out2.len, 0, "twin: self-join lost — count 1 -> 0");
}

// ---------------------------------------------------------------------
// E3 — saturated radix bucket
// ---------------------------------------------------------------------

/// Court-side replication of unify.rs:850-854 radix_bucket (private in
/// the kernel; the formula is the documented ABI). Self-check below:
/// if the kernel's hash ever changes, this court fails loudly rather
/// than passing vacuously on a stale replica.
fn court_bucket(word: u64) -> usize {
    let y = word & 0x00FF_FFFF_FFFF_FFFF;
    ((y ^ (y >> 16)) & 0x0FFF) as usize
}

/// Payloads that all hash to radix bucket 0: p = (r<<16) | (r & 0xFFF)
/// gives (p ^ p>>16) & 0xFFF == 0, distinct for distinct r.
fn bucket0_payload(r: u64) -> u64 {
    (r << 16) | (r & 0xFFF)
}

/// E3 positive: saturate ONE radix bucket. Every join-key candidate row
/// lands in bucket 0 (hub v's bucket), so the bucketed probe scans a
/// 150-distractor chain for each of the 100 seed matches and must emit
/// exactly the 100 true joins — the superset filter is sound, no
/// phantom joins, no lost joins.
/// Anti-vacuity twin (M-E3): flip ONE seed edge (s_0 -> v becomes
/// s_0 -> w, w out-of-bucket and non-joinable) — the derived set MUST
/// lose s_0->t (100 -> 99 joins; the court's positive assertion flips).
#[test]
fn e3_saturated_bucket_probe_emits_exactly_the_true_joins() {
    assert_eq!(RADIX_BUCKETS, 4096, "L3 bucket count (the REAL bound)");
    const SEEDS: u64 = 100;
    const DISTRACTORS: u64 = 150;

    // Hub v and target t: bucket 0, distinct, non-zero payloads.
    let v = bucket0_payload(300);
    let t = bucket0_payload(301);
    // Distractor join-key candidates d_j: same bucket as v, different
    // full words — the probe must filter them on full-word compare.
    let d = |j: u64| bucket0_payload(j);
    // Seed subjects s_i: NOT in bucket 0 (they are not join keys; this
    // keeps the saturation pinned to the key bucket, not accidental).
    let s = |i: u64| 0x000A_0000_0000_1000 + i * 0x1B31; // bucket != 0
                                                         // Self-check of the replica hash (non-vacuity of E3 itself).
    assert_eq!(court_bucket(v), 0, "hub in bucket 0");
    assert_eq!(court_bucket(t), 0, "target in bucket 0");
    for j in 1..=DISTRACTORS {
        assert_eq!(court_bucket(d(j)), 0, "distractor d{j} in bucket 0");
        assert_ne!(d(j), v, "distractor differs from hub");
    }
    for i in 1..=SEEDS {
        assert_ne!(court_bucket(s(i)), 0, "seed s{i} NOT in bucket 0");
    }

    // Corpus: seeds (s_i -> v), hub (v -> t), distractors (d_j -> u_j)
    // where u_j is arbitrary and non-joinable.
    let mut pairs: Vec<(u64, u64)> = Vec::new();
    for i in 1..=SEEDS {
        pairs.push((s(i), v));
    }
    pairs.push((v, t));
    let u = |j: u64| 0x000B_0000_0000_2000 + j * 0x9E41;
    for j in 1..=DISTRACTORS {
        pairs.push((d(j), u(j)));
    }

    // Probe-level: exactly SEEDS matches (each seed x the hub row).
    let cols = cols_of(&pairs);
    let compiled = RuleCompiler::compile(&closure_rule()).expect("rule compiles");
    let mut scratch = EvalScratch::default();
    let mut out = MatchSet::default();
    let mut probe = JoinProbeScratch::default();
    unify_batch_probe(
        &compiled,
        &cols,
        DEFAULT_CHUNK_ROWS,
        &mut scratch,
        &mut out,
        &mut probe,
    )
    .expect("probe succeeds");
    assert_eq!(out.len as u64, SEEDS, "probe: exactly the 100 true joins");

    // Fixpoint-level: derived set == {s_i -> t}, nothing else.
    let got = derived_set(&pairs);
    let want: std::collections::BTreeSet<(u64, u64)> = (1..=SEEDS).map(|i| (s(i), t)).collect();
    assert_eq!(got, want, "closure: exactly seeds->t, no phantom joins");
    assert_eq!(got.len() as u64, SEEDS);

    // Twin (M-E3): flip ONE seed edge out of the joinable/bucket shape.
    let mut flipped = pairs.clone();
    flipped[0] = (s(1), 0x000A_0000_0000_9000u64); // s_1 -> w (non-joinable)
    let got2 = derived_set(&flipped);
    assert_eq!(got2.len() as u64, SEEDS - 1, "twin: one join lost");
    assert!(
        !got2.contains(&(s(1), t)),
        "twin: s_1->t gone — positive assertion flipped"
    );
}

// ---------------------------------------------------------------------
// E4 — cascading retraction across multiple derivation paths
// ---------------------------------------------------------------------

/// E4: a closure graph with a REDUNDANT derivation path. Base nodes
/// a,b,c,d,e,x; edges ab, bc, cd, de, plus the alternate route b->x->c.
///
///   G1 = {ab, bc, cd, de, bx, xc}          (full graph)
///   G2 = G1 minus {bc}                     (retract the direct edge)
///   G3 = G2 minus {xc}                     (retract the alternate leg)
///
/// Positive assertions (differential vs the BFS oracle, equality is the
/// no-phantom-survivors check):
///   A. D1 == closure(G1) - G1
///   B. retracting bc alone loses NOTHING: bc is re-derivable via x,
///      and every downstream fact (a->e etc.) survives.
///   C. D3 == {ax, ce} only: with both legs gone the cascade completes —
///      b->c and all downstream facts vanish. This is the twin of B.
#[test]
fn e4_cascading_retraction_alternate_path_survival_and_cascade() {
    const A: u64 = 0x11;
    const B: u64 = 0x22;
    const C: u64 = 0x33;
    const D: u64 = 0x44;
    const E: u64 = 0x55;
    const X: u64 = 0x66;

    let g1 = vec![(A, B), (B, C), (C, D), (D, E), (B, X), (X, C)];
    let g2: Vec<(u64, u64)> = g1.iter().copied().filter(|e| *e != (B, C)).collect();
    let g3: Vec<(u64, u64)> = g2.iter().copied().filter(|e| *e != (X, C)).collect();

    // A. Full graph: derived == oracle.
    let d1 = derived_set(&g1);
    let full = oracle_closure(&g1);
    let base1: std::collections::BTreeSet<(u64, u64)> = g1.iter().copied().collect();
    let want1: std::collections::BTreeSet<(u64, u64)> = full.difference(&base1).copied().collect();
    assert_eq!(d1, want1, "G1 closure == BFS oracle (no phantom, no loss)");
    assert_eq!(d1.len(), 9, "G1: ac ad ae ax bd be ce xd xe");
    assert!(d1.contains(&(A, E)), "downstream a->e derived");

    // B. Retract direct edge bc: NO downstream fact is lost. bc itself
    // flips from derived to re-derived (it was in D1 via the direct
    // edge? no — bc was always only derivable via x; in G1 bc was a
    // BASE edge, in G2 it becomes derived). Net: D2 == D1 ∪ {bc},
    // zero facts lost — every D1 fact survives.
    let d2 = derived_set(&g2);
    assert!(
        d1.is_subset(&d2),
        "retracting bc alone: zero D1 facts lost (all survive)"
    );
    assert_eq!(
        d2.difference(&d1).collect::<Vec<_>>(),
        vec![&(B, C)],
        "bc flips base->derived (re-derivable via alternate path x)"
    );
    assert!(d2.contains(&(B, C)), "b->c survives via alternate path x");
    assert!(d2.contains(&(A, E)), "downstream a->e survives retraction");

    // Twin of B (the cascade): retract xc too — now bc has no path, the
    // cascade wipes b->c AND every downstream fact; D3 == {ax, ce} exactly
    // (ax survives via ab,bx; ce via cd,de).
    let d3 = derived_set(&g3);
    let base3: std::collections::BTreeSet<(u64, u64)> = g3.iter().copied().collect();
    let want3: std::collections::BTreeSet<(u64, u64)> =
        oracle_closure(&g3).difference(&base3).copied().collect();
    assert_eq!(d3, want3, "G3 derived == oracle");
    assert_eq!(
        d3,
        [(A, X), (C, E)].into_iter().collect(),
        "cascade complete: only the detached branches remain (ax, ce)"
    );
    assert!(!d3.contains(&(B, C)), "b->c phantom GONE");
    assert!(!d3.contains(&(A, E)), "a->e phantom GONE");
}

// ---------------------------------------------------------------------
// E5 — zero-alloc steady state (AE7 tracking allocator, single driver)
// ---------------------------------------------------------------------

/// E5: AE7 hot-path zero-alloc windows over the edge vectors.
/// Sequencing law (tracking_alloc module doc): the interceptor state is
/// process-global, so all windows run inside ONE #[test] fn.
#[test]
fn e5_zero_alloc_steady_state_driver() {
    reset_breach();

    // Window 0 — positive control: an allocation under budget(0,0)
    // MUST breach (a window that cannot flip is VACUOUS). This is the
    // anti-vacuity twin for the whole of E5.
    {
        let _g = start_hot_path_budget("c03-positive-control", 0, 0);
        let v = vec![7u64; 4];
        std::hint::black_box(&v);
    }
    let b0 = last_breach();
    assert!(
        b0.is_some(),
        "positive control: budget(0,0) window must breach on a real alloc"
    );
    assert_eq!(b0.map(|b| b.site), Some("c03-positive-control"));
    reset_breach();

    // Window 1 — probe steady state on the E3 saturated-bucket corpus
    // (rule compiled + scratch/probe/matches leased and WARMED outside
    // the window; the documented allocation discipline: buffers are
    // owned once, reused across calls).
    let pairs = saturated_corpus();
    let cols = cols_of(&pairs);
    let compiled = RuleCompiler::compile(&closure_rule()).expect("rule compiles");
    let mut scratch = EvalScratch::default();
    let mut out = MatchSet::default();
    let mut probe = JoinProbeScratch::default();
    // Warmup: bounded poll on a real observable (no sleep) — loop the
    // probe under a counting window until it reports quiet; buffers
    // reach high-water within a few calls. Assert convergence by the
    // 10th call (the budget window can flip: positive control above).
    let mut warm_quiet = false;
    for _ in 0..10 {
        let g = start_hot_path_budget("c03-warmup", 0, 0);
        unify_batch_probe(
            &compiled,
            &cols,
            DEFAULT_CHUNK_ROWS,
            &mut scratch,
            &mut out,
            &mut probe,
        )
        .expect("warmup probe");
        warm_quiet = window_counts().0 == 0;
        drop(g);
        if warm_quiet {
            break;
        }
        reset_breach();
    }
    assert!(
        warm_quiet,
        "probe path must reach steady state (zero-alloc) by the 10th call"
    );
    {
        let _g = start_hot_path_budget("c03-probe-steady-state", 0, 0);
        for _ in 0..20 {
            unify_batch_probe(
                &compiled,
                &cols,
                DEFAULT_CHUNK_ROWS,
                &mut scratch,
                &mut out,
                &mut probe,
            )
            .expect("steady-state probe");
        }
        let (allocs, bytes) = window_counts();
        assert_eq!(
            (allocs, bytes),
            (0, 0),
            "probe steady state over the edge vectors is zero-alloc"
        );
    }
    assert!(last_breach().is_none(), "no breach in probe window");

    // Window 2 — honesty arm: the fixpoint driver is NOT zero-alloc
    // (the `seen` BTreeSet is O(store) node allocs; AE7 honesty, see
    // crates/cgap/tests/alloc_boundary.rs:24-27). Assert a measured
    // nonzero count under an unbounded window rather than faking 0.
    {
        let _g = start_hot_path_budget("c03-fixpoint-counts", usize::MAX, usize::MAX);
        let _ = unify_to_fixpoint(&[closure_rule()], &cols).expect("fixpoint converges");
        let (allocs, bytes) = window_counts();
        assert!(
            allocs > 0,
            "fixpoint allocates (AE7 honesty: not zero-alloc)"
        );
        assert!(bytes > 0);
    }
    assert!(last_breach().is_none());
}

/// The E3 corpus, shared by E3 and E5 (deterministic literals).
fn saturated_corpus() -> Vec<(u64, u64)> {
    const SEEDS: u64 = 100;
    const DISTRACTORS: u64 = 150;
    let v = bucket0_payload(300);
    let t = bucket0_payload(301);
    let d = |j: u64| bucket0_payload(j);
    let s = |i: u64| 0x000A_0000_0000_1000 + i * 0x1B31;
    let u = |j: u64| 0x000B_0000_0000_2000 + j * 0x9E41;
    let mut pairs: Vec<(u64, u64)> = (1..=SEEDS).map(|i| (s(i), v)).collect();
    pairs.push((v, t));
    pairs.extend((1..=DISTRACTORS).map(|j| (d(j), u(j))));
    pairs
}
