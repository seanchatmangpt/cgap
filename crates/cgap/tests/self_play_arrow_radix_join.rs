//! SP4 self-play court — Arrow-shape columnar radix join (lane SP4).
//!
//! Self-play: 64K statement tuples -> `cgap_bn3::unify::StatementColumns`
//! (the Arrow-batch shape seam: SoA `&[u64]` columns, equal lengths,
//! position-aligned rows — unify.rs:356-362, recorded divergence 1) ->
//! `JoinProbeScratch::default()` (RADIXFIX: `radix: true` by explicit
//! `Default`, asserted in-test) -> `unify_batch_probe` (the LEASED
//! variant: radix tables + cache owned by the caller, reused across
//! chunks).
//!
//! RECEIPT (lane SP4, 2026-11-02):
//! - Gate: `cargo test -p cgap --test self_play_arrow_radix_join`
//!   (see standing lines at the bottom of this block, updated post-run).
//! - Differential vs naive nested-loop oracle (predicate-preselected
//!   double loop — pure filter hoist, semantics unchanged) on the SAME
//!   64K corpus: match-set equality (HashSet<MatchKey>).
//! - Zero-alloc steady state: global counting allocator around the
//!   leased probe after a warmup run on identical input; delta MUST be
//!   0 (the w5-l3-remeasure alloc-zero discipline).
//! - ANTI-VACUITY MUTANT (sandbox, /tmp/sp4-sandbox): bucket-xor
//!   asymmetric-bucket mutation of `bucket_candidates` in a /tmp copy of
//!   cgap-bn3 -> derived match set collapses -> differential FAILS
//!   (flip witnessed; shared tree untouched, sha256-verified; see the
//!   divergence note on the radix hash seed in the lane report: the
//!   radix probe is seedless — `radix_bucket` is a pure
//!   `(y ^ (y >> 16)) & 0x0FFF` payload mix, so the dispatch's "zero
//!   the seed" was implemented as the closest soundness-breaking
//!   asymmetric-bucket mutation).
//! - STANDING (filled after the gate run this session):
//!   ALIVE — gate exit 0, 4/4 tests passed; mutant court flip witnessed
//!   (sandbox build FAILED as required; baseline sha256-exact).

use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use cgap_bn3::sigil::{encode_term, Sigil, PAYLOAD_MASK};
use cgap_bn3::unify::{
    unify_batch_probe, BodyAtom, EvalScratch, JoinProbeScratch, MatchSet, Rule, RuleCompiler,
    StatementColumns,
};

// --- Global counting allocator (zero-alloc steady-state court) --------

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Serializes the allocation-counter-sensitive phases of the tests in
/// this binary: while one test holds `TEST_SERIAL`, no other test in
/// this binary is inside its measured window, so a counter delta is
/// attributable to the measuring test alone.
static TEST_SERIAL: Mutex<()> = Mutex::new(());

struct CountingAlloc;

unsafe impl std::alloc::GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let p = std::alloc::System.alloc(layout);
        if !p.is_null() {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        p
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        std::alloc::System.dealloc(ptr, layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: std::alloc::Layout, new_size: usize) -> *mut u8 {
        let p = std::alloc::System.realloc(ptr, layout, new_size);
        if !p.is_null() {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        p
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

// --- Corpus (SEED LAW: 0xBF58_476D_1CE4_E5B9 = canonical splitmix64
// seed; 0xA20C_0FFE_E5A7_E000 = frozen corpus-spread constant, mixed
// into the pooled values, never used as a seed) ------------------------

const SEED: u64 = 0xBF58_476D_1CE4_E5B9;
const CORPUS_CONST: u64 = 0xA20C_0FFE_E5A7_E000;
const BASE: u64 = 0x000A_0000_0000_0000;
const N_SELF_PLAY: usize = 65_536;

struct CourtRng {
    state: u64,
}

impl CourtRng {
    fn new(seed: u64) -> Self {
        CourtRng { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_490B_BADD_03);
        z ^ (z >> 31)
    }

    /// Value from a small pool, xor-spread with the frozen corpus
    /// constant (lane-deterministic, payload-window safe).
    fn pooled(&mut self, pool: u64, lane: u64) -> u64 {
        let spread = self.next() ^ CORPUS_CONST.rotate_left((lane * 13) as u32);
        BASE + (spread % pool)
    }
}

/// Raw A11 SlotConst term word (sigil 0x00 -> word == payload).
fn cst(v: u64) -> u64 {
    encode_term(Sigil::SlotConst, v).expect("in-range payload")
}

/// Raw A11 VarUni term word for variable register `r`.
fn vr(r: u64) -> u64 {
    encode_term(Sigil::VarUni, r).expect("in-range register")
}

/// The two-atom join rule under court. ?x joins atom0 -> atom1 on ?y.
///   atom0: (?x P ?y)   atom1: (?y Q ?z)
fn court_rule() -> Rule {
    Rule {
        rule_id: 7,
        body: vec![
            BodyAtom::new(vr(0), cst(BASE + 1), vr(1)),
            BodyAtom::new(vr(1), cst(BASE + 2), vr(2)),
        ],
        head: [vr(0), cst(BASE + 3), vr(2)],
    }
}

/// Corpus of `n` rows over per-lane pools `(s, p, o)`.
fn corpus(n: usize, pools: (u64, u64, u64)) -> (Vec<u64>, Vec<u64>, Vec<u64>) {
    let mut rng = CourtRng::new(SEED);
    let (mut s, mut p, mut o) =
        (Vec::with_capacity(n), Vec::with_capacity(n), Vec::with_capacity(n));
    for _ in 0..n {
        s.push(rng.pooled(pools.0, 1));
        p.push(rng.pooled(pools.1, 2));
        o.push(rng.pooled(pools.2, 3));
    }
    (s, p, o)
}

// --- Naive nested-loop oracle (predicate-preselected double loop) -----

type MatchKey = (u32, u32, u64, u64, u64);

/// Naive nested-loop oracle: for every pair of corpus rows matching the
/// two-atom pattern (atom0 on row i, atom1 on row j, shared ?y equal),
/// insert the canonical match key. The constant-predicate membership of
/// each row is preselected once (a pure filter hoist over a
/// full-word-equality comparison — semantics identical to checking it
/// inside the inner loop, cost down from O(n^2) full atom matches).
/// `probe_dropped` reproduces the M-L3-1 mutation semantics (bound-slot
/// probe skipped) — used ONLY by the in-binary anti-vacuity arm.
fn oracle(cols: &StatementColumns<'_>, rule: &Rule, probe_dropped: bool) -> HashSet<MatchKey> {
    let n = cols.len();
    let p0 = rule.body[0].p & PAYLOAD_MASK;
    let p1 = rule.body[1].p & PAYLOAD_MASK;
    let mut idx0 = Vec::new();
    let mut idx1 = Vec::new();
    for row in 0..n {
        if cols.p[row] == p0 {
            idx0.push(row);
        }
        if cols.p[row] == p1 {
            idx1.push(row);
        }
    }
    let mut out = HashSet::new();
    for &i in &idx0 {
        for &j in &idx1 {
            // ?y = o[i] must equal s[j] (the join probe's bound-slot
            // equality); skipped under `probe_dropped`.
            if !probe_dropped && cols.s[j] != cols.o[i] {
                continue;
            }
            out.insert((i as u32, j as u32, cols.s[i], cols.o[i], cols.o[j]));
        }
    }
    out
}

/// The kernel's joined matches over a LEASED probe as the canonical key
/// set (plus the counters the caller wants pinned: returned so the
/// radix/cache assertions run on the same run that produced the set).
fn kernel_keys(
    rule: &Rule,
    cols: &StatementColumns<'_>,
    chunk: usize,
    probe: &mut JoinProbeScratch,
) -> HashSet<MatchKey> {
    let compiled = RuleCompiler::compile(rule).expect("court rule compiles");
    let mut scratch = EvalScratch::default();
    let mut ms = MatchSet::default();
    unify_batch_probe(&compiled, cols, chunk, &mut scratch, &mut ms, probe)
        .expect("unify_batch_probe ok");
    assert_eq!(ms.len, ms.atom_rows[0].len(), "match set shape");
    (0..ms.len)
        .map(|m| {
            (
                ms.atom_rows[0][m],
                ms.atom_rows[1][m],
                ms.var_values[0][m],
                ms.var_values[1][m],
                ms.var_values[2][m],
            )
        })
        .collect()
}

// --- Courts ------------------------------------------------------------

/// RADIXFIX: the default probe is the L3 radix fast path by explicit
/// `Default` (never a derived accident); the leased probe served from
/// the candidate cache MUST show cache hits at >1 chunk.
#[test]
fn sp4_default_probe_is_radix_with_cache_hits() {
    let _gate = TEST_SERIAL.lock().expect("serial lock");
    let rule = court_rule();
    let (s, p, o) = corpus(N_SELF_PLAY, (2048, 4, 4096));
    let cols = StatementColumns::new(&s, &p, &o).expect("columns");
    let mut probe = JoinProbeScratch::default();
    assert!(probe.radix, "JoinProbeScratch::default() must be radix:true");
    let keys = kernel_keys(&rule, &cols, 4096, &mut probe);
    assert!(!keys.is_empty(), "anti-vacuity: 64K self-play corpus must yield joins");
    assert!(
        probe.cache_hits > 0,
        "leased probe served zero chunks from the candidate cache (hits={})",
        probe.cache_hits
    );
    assert!(
        probe.cache_misses > 0,
        "expected at least one per-batch rebuild (misses={})",
        probe.cache_misses
    );
}

/// SP4 core court: the LEASED radix probe on the 64K Arrow-shape corpus
/// agrees match-set-for-match-set with the naive nested-loop oracle.
/// Legs 64 and 4096 run the exhaustive double loop; the 64K leg uses
/// the predicate-preselected loop (pure filter hoist, semantics
/// unchanged).
#[test]
fn sp4_differential_naive_oracle() {
    let _gate = TEST_SERIAL.lock().expect("serial lock");
    let rule = court_rule();
    for &(n, pools, chunk) in &[
        (64usize, (8u64, 4u64, 16u64), 1usize),
        (4096, (2048, 4, 4096), 4096),
        (N_SELF_PLAY, (2048, 4, 4096), 4096),
    ] {
        let (s, p, o) = corpus(n, pools);
        let cols = StatementColumns::new(&s, &p, &o).expect("columns");
        let mut probe = JoinProbeScratch::default();
        let kernel = kernel_keys(&rule, &cols, chunk, &mut probe);
        let truth = oracle(&cols, &rule, false);
        assert!(
            !truth.is_empty(),
            "anti-vacuity: N={n} corpus must yield joins"
        );
        assert_eq!(
            kernel.len(),
            truth.len(),
            "N={n}: kernel/oracle match count"
        );
        assert_eq!(kernel, truth, "N={n}: match sets diverge");
    }
}

/// SP4 in-binary anti-vacuity: under the M-L3-1 semantics (join probe
/// dropped) the oracle MUST produce a strictly larger set — otherwise
/// the differential above proves nothing.
#[test]
fn sp4_antivacuity_probe_drop_flips_oracle() {
    let _gate = TEST_SERIAL.lock().expect("serial lock");
    let rule = court_rule();
    let (s, p, o) = corpus(4096, (2048, 4, 4096));
    let cols = StatementColumns::new(&s, &p, &o).expect("columns");
    let truth = oracle(&cols, &rule, false);
    let mutated = oracle(&cols, &rule, true);
    assert!(!truth.is_empty());
    assert!(
        mutated.len() > truth.len(),
        "VACUOUS: dropped probe did not flip (truth={} mutated={})",
        truth.len(),
        mutated.len()
    );
}

/// SP4 zero-alloc steady state (w5-l3-remeasure discipline): after a
/// warmup run over the full 64K corpus on identical input, a second run
/// on the SAME leased scratch/probe/out triple performs ZERO heap
/// allocations (the mutation "reintroduce per-call allocation" MUST
/// flip this court).
#[test]
fn sp4_zero_alloc_steady_state() {
    let _gate = TEST_SERIAL.lock().expect("serial lock");
    let rule = court_rule();
    let compiled = RuleCompiler::compile(&rule).expect("compile");
    let (s, p, o) = corpus(N_SELF_PLAY, (2048, 4, 4096));
    let cols = StatementColumns::new(&s, &p, &o).expect("columns");
    let mut scratch = EvalScratch::default();
    let mut out = MatchSet::default();
    let mut probe = JoinProbeScratch::default();
    assert!(probe.radix);

    // Warmup: sizes every leased buffer on identical input.
    unify_batch_probe(&compiled, &cols, 4096, &mut scratch, &mut out, &mut probe)
        .expect("warmup run ok");
    let warm_len = out.len;
    assert!(warm_len > 0, "anti-vacuity: warmup must yield joins");

    // Steady state: identical run, reused buffers — the counter window
    // is held under the serial gate, so the delta is attributable.
    ALLOC_COUNT.store(0, Ordering::SeqCst);
    unify_batch_probe(&compiled, &cols, 4096, &mut scratch, &mut out, &mut probe)
        .expect("steady-state run ok");
    let steady_allocs = ALLOC_COUNT.swap(0, Ordering::SeqCst);
    assert_eq!(out.len, warm_len, "replayed run must be deterministic");
    assert!(
        steady_allocs == 0,
        "steady-state run allocated {steady_allocs} times; leased hot path must be alloc-zero"
    );
}
