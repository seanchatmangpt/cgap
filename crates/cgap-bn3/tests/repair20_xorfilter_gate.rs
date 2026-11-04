//! Repair-20 (BH7 XOR-filter pre-gate) exact occupancy-summary court.
//!
//! Gate: `unify.rs` probe loop tests `bucket_occupancy[b/64]` bit
//! `b%64` before fetching `bucket_head[b]`; bit=0 skips the chain walk.
//! Exactness: occupancy bits are accumulated during `bucket_candidates`
//! from the REAL chain build, so bit=0 implies the bucket is provably
//! empty -- no probabilistic fingerprint. The byte-fingerprint OR-fold
//! form was REJECTED as unsound: OR of byte sets is a union, and two
//! keys' byte union can contain a byte tuple no single key has, so a
//! fingerprint bit would not be membership-sound.
//!
//! Courts (this file):
//! 1. Differential radix-vs-nested, match-for-match on an N x chunk grid.
//! 2. Wide-pool 100_003 leg, sequence-ordered identity.
//! 3. Occupancy consistency (final state, post-return): bit b of word
//!    b/64 is set iff bucket_head[b] != u32::MAX.
//!
//! Anti-vacuity lives in the repair-20 receipt procedure: a /tmp sandbox
//! source mutant that drops the occupancy-bit set in bucket_candidates
//! must make court 1 FAIL. SEED LAW: CourtRng seed 0xBF58_476D_1CE4_E5B9
//! (canonical splitmix64); 0xA20C_0FFE_E5A7_E000 is a corpus constant,
//! never a seed.

use std::collections::HashSet;

use cgap_bn3::sigil::{encode_term, Sigil};
use cgap_bn3::unify::{
    unify_batch_probe, BodyAtom, EvalScratch, JoinProbeScratch, MatchSet, Rule, RuleCompiler,
    StatementColumns,
};

/// Canonical splitmix64 (SEED LAW).
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

    fn pooled(&mut self, pool: u64) -> u64 {
        BASE + (self.next() % pool)
    }
}

const BASE: u64 = 0x000A_0000_0000_0000;
const SEED: u64 = 0xBF58_476D_1CE4_E5B9;

fn cst(v: u64) -> u64 {
    encode_term(Sigil::SlotConst, v).expect("in-range payload")
}

fn vr(r: u64) -> u64 {
    encode_term(Sigil::VarUni, r).expect("in-range register")
}

/// Two-atom join rule: (?x P ?y) (?y Q ?z), head (?x P3 ?z).
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

/// Narrow-pool corpus (8/4/16): few distinct keys, many EMPTY buckets --
/// the occupancy gate's win case.
fn corpus(n: usize) -> (Vec<u64>, Vec<u64>, Vec<u64>) {
    let mut rng = CourtRng::new(SEED);
    let mut s = Vec::with_capacity(n);
    let mut p = Vec::with_capacity(n);
    let mut o = Vec::with_capacity(n);
    for _ in 0..n {
        s.push(rng.pooled(8));
        p.push(rng.pooled(4));
        o.push(rng.pooled(16));
    }
    (s, p, o)
}

/// Wide-pool corpus (2048/4/4096): bounded matches at large N.
fn corpus_wide(n: usize) -> (Vec<u64>, Vec<u64>, Vec<u64>) {
    let mut rng = CourtRng::new(SEED);
    let mut s = Vec::with_capacity(n);
    let mut p = Vec::with_capacity(n);
    let mut o = Vec::with_capacity(n);
    for _ in 0..n {
        s.push(rng.pooled(2048));
        p.push(rng.pooled(4));
        o.push(rng.pooled(4096));
    }
    (s, p, o)
}

/// One match as a canonical tuple.
type MatchKey = (u32, u32, u64, u64, u64);

fn run(cols: &StatementColumns<'_>, rule: &Rule, chunk: usize, radix: bool) -> HashSet<MatchKey> {
    let compiled = RuleCompiler::compile(rule).expect("compile");
    let mut scratch = EvalScratch::default();
    let mut out = MatchSet::default();
    let mut probe = JoinProbeScratch::default();
    probe.radix = radix;
    unify_batch_probe(&compiled, cols, chunk, &mut scratch, &mut out, &mut probe)
        .expect("probe ok");
    let mut set = HashSet::new();
    for m in 0..out.len {
        set.insert((
            out.atom_rows[0][m],
            out.atom_rows[1][m],
            out.var_values[0][m],
            out.var_values[1][m],
            out.var_values[2][m],
        ));
    }
    set
}

/// Court 1: gated radix vs ungated nested, match-set identity.
#[test]
fn repair20_radix_gate_vs_nested_differential() {
    let rule = court_rule();
    for &n in &[1usize, 64, 4096] {
        let (s, p, o) = corpus(n);
        let cols = StatementColumns::new(&s, &p, &o).expect("columns");
        for &chunk in &[1usize, 64, 4096] {
            let got_r = run(&cols, &rule, chunk, true);
            let got_n = run(&cols, &rule, chunk, false);
            assert_eq!(got_r.len(), got_n.len(), "N={n} chunk={chunk}: count");
            assert_eq!(got_r, got_n, "N={n} chunk={chunk}: match set");
        }
    }
}

/// Court 2: wide-pool 100_003 leg, match-set identity at scale.
#[test]
fn repair20_radix_gate_vs_nested_100k() {
    let rule = court_rule();
    let n = 100_003usize;
    let (s, p, o) = corpus_wide(n);
    let cols = StatementColumns::new(&s, &p, &o).expect("columns");
    let got_r = run(&cols, &rule, 4096, true);
    let got_n = run(&cols, &rule, 4096, false);
    assert_eq!(got_r.len(), got_n.len(), "100_003: count");
    assert_eq!(got_r, got_n, "100_003: match set");
}

/// Court 3: occupancy summary is exactly the head-array occupancy,
/// asserted on FINAL STATE after a real probe run.
#[test]
fn repair20_occupancy_summary_exact() {
    let rule = court_rule();
    for &n in &[64usize, 4096] {
        let (s, p, o) = corpus(n);
        let cols = StatementColumns::new(&s, &p, &o).expect("columns");
        let compiled = RuleCompiler::compile(&rule).expect("compile");
        let mut scratch = EvalScratch::default();
        let mut out = MatchSet::default();
        let mut probe = JoinProbeScratch::default();
        unify_batch_probe(&compiled, &cols, 64, &mut scratch, &mut out, &mut probe)
            .expect("probe ok");
        // Final state: summary rebuilt with the last chunk's chains.
        assert_eq!(
            probe.bucket_occupancy.len(),
            (probe.bucket_head.len() + 63) / 64,
            "N={n}: summary covers every bucket"
        );
        let mut occupied = 0usize;
        for b in 0..probe.bucket_head.len() {
            let bit = (probe.bucket_occupancy[b >> 6] >> (b & 63)) & 1;
            let head = probe.bucket_head[b];
            assert_eq!(
                bit == 1,
                head != u32::MAX,
                "N={n} bucket={b}: occupancy bit vs head"
            );
            if bit == 1 {
                occupied += 1;
            }
        }
        // Non-vacuity of this court itself: the narrow-pool corpus must
        // leave some buckets EMPTY (else the gate never skips and this
        // exactness assert carries no bits).
        assert!(
            occupied < probe.bucket_head.len(),
            "N={n}: corpus must leave empty buckets (occupied={occupied})"
        );
    }
}
