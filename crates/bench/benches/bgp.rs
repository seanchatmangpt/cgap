//! BENCH: BGP multi-hop columnar equi-joins (lane p1-bgp, v26.11.2).
//!
//! Business case (the 80/20): multi-hop graph pattern joins at columnar
//! speed -- the k-hop chain `(?x1 P1 ?x2)(?x2 P2 ?x3)...` over the
//! canonical splitmix64 corpus, measured per hop depth through the
//! cgap-bn3 public API (`RuleCompiler::compile` + `unify_batch_probe`
//! over SoA `StatementColumns`, warm scratch + probe lease = the
//! production zero-alloc shape). Units: JOINS/s (Divan items/s where
//! each item is one full-chain match; units are never cross-multiplied
//! -- this is a join-rate report, not a facts/s or GB/s claim).
//!
//! Corpus: SplitMix64 seed `0xBF58_476D_1CE4_E5B9` verbatim (the
//! corruption-witness constant rides the multiplier; a non-canonical
//! constant in bench code is the known write-corruption class). N in
//! {65_536, 262_144}; key domain scales with N to bound fanout.
//!
//! Chicago asserts ride the benched closure: the expected full-chain
//! match count comes from an indexed nested-loop ORACLE in setup, and
//! the timed closure asserts the kernel count equals it every sample
//! (a wrong join must move this bench to a panic). Anti-vacuity: the
//! mutation class is the hop-predicate detachment of
//! `cgap-bn3/tests/bgp_hop_courts.rs` -- it MUST flip the court; a
//! bench that survives it is VACUOUS.

#![forbid(unsafe_code)]

use cgap_bn3::sigil::{encode_term, Sigil};
use cgap_bn3::unify::{
    unify_batch_probe, BodyAtom, DEFAULT_CHUNK_ROWS, EvalScratch, JoinProbeScratch, MatchSet,
    Rule, RuleCompiler, StatementColumns,
};
use divan::counter::ItemsCount;
use divan::{black_box, Bencher};
use std::collections::HashMap;

/// Canonical corpus seed (charter Method discipline, verbatim).
const CANONICAL_SEED: u64 = 0xBF58_476D_1CE4_E5B9;

/// Corpus predicate payloads: hop-1/2/3 predicates + bulk filler.
const P1: u64 = 0x100;
const P2: u64 = 0x200;
const P3: u64 = 0x4B0;
const PF: u64 = 0xF00;

/// The RULE-side hop-2 predicate payload (mirrors the court's
/// anti-vacuity seam; corrupting this detaches rule from corpus and
/// MUST flip the court's nonempty assert).
const HOP2_RULE_PRED: u64 = P2;

/// Canonical splitmix64.
struct SplitMix(u64);

impl SplitMix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_490B_BADD_03);
        z ^ (z >> 31)
    }
}

/// Raw A11 SlotConst term word over payload `v`.
fn cst(v: u64) -> u64 {
    encode_term(Sigil::SlotConst, v).expect("in-range payload")
}

/// Raw A11 VarUni term word for variable payload `r`.
fn vr(r: u64) -> u64 {
    encode_term(Sigil::VarUni, r).expect("in-range register")
}

/// SoA corpus columns over term words.
struct BgpCorpus {
    s: Vec<u64>,
    p: Vec<u64>,
    o: Vec<u64>,
}

impl BgpCorpus {
    fn cols(&self) -> StatementColumns<'_> {
        StatementColumns::new(&self.s, &self.p, &self.o).expect("equal column lengths")
    }
}

/// Build the splitmix64 corpus at `n` rows, predicate class = draw % 4.
fn build_corpus(n: usize, key_dom: u64) -> BgpCorpus {
    let mut sm = SplitMix(CANONICAL_SEED);
    let mut c = BgpCorpus {
        s: Vec::with_capacity(n),
        p: Vec::with_capacity(n),
        o: Vec::with_capacity(n),
    };
    for _ in 0..n {
        let r = sm.next();
        let p = match r % 4 {
            0 => P1,
            1 => P2,
            2 => P3,
            _ => PF,
        };
        c.s.push(cst((r >> 8) % key_dom));
        c.p.push(cst(p));
        c.o.push(cst((r >> 34) % key_dom));
    }
    c
}

/// The k-hop BGP rule (variable payloads 0..=hops).
fn bgp_rule(hops: usize) -> Rule {
    let preds = [cst(P1), cst(HOP2_RULE_PRED), cst(P3)];
    let body: Vec<BodyAtom> = (0..hops)
        .map(|h| BodyAtom::new(vr(h as u64), preds[h], vr(h as u64 + 1)))
        .collect();
    Rule {
        rule_id: 0x00B6_0001,
        head: [vr(0), cst(0x999), vr(hops as u64)],
        body,
    }
}

fn oracle_count(cols: &StatementColumns<'_>, hops: usize) -> usize {
    let preds = [cst(P1), cst(HOP2_RULE_PRED), cst(P3)];
    // cur: chained-key word -> count of open partial matches.
    let mut cur: HashMap<u64, usize> = HashMap::new();
    for i in 0..cols.len() {
        if cols.p[i] == preds[0] {
            *cur.entry(cols.o[i]).or_insert(0) += 1;
        }
    }
    for h in 1..hops {
        // idx: hop-key -> histogram of chained-out keys (fanout).
        let mut idx: HashMap<u64, HashMap<u64, usize>> = HashMap::new();
        for j in 0..cols.len() {
            if cols.p[j] == preds[h] {
                let sk = cols.s[j] & 0x00FF_FFFF_FFFF_FFFF;
                let ok = cols.o[j] & 0x00FF_FFFF_FFFF_FFFF;
                *idx.entry(sk).or_default().entry(ok).or_insert(0) += 1;
                }
        }
        let mut next: HashMap<u64, usize> = HashMap::new();
        for (k, cnt) in &cur {
            if let Some(fan) = idx.get(k) {
                for (&ok, c) in fan {
                    *next.entry(ok).or_insert(0) += cnt * c;
                }
            }
        }
        cur = next;
    }
    cur.values().sum()
}

/// Fixed corpus + rule + oracle expectation, built ONCE per bench
/// invocation (setup), never inside the timed closure.
struct BgpBench {
    corpus: BgpCorpus,
    rule: Rule,
    expected: usize,
}

fn build_bench(n: usize, hops: usize) -> BgpBench {
    let key_dom = (n / 4 / 4).next_power_of_two() as u64;
    let corpus = build_corpus(n, key_dom);
    let rule = bgp_rule(hops);
    let expected = oracle_count(&corpus.cols(), hops);
    assert!(expected > 0, "oracle empty at N={n}: bench would be vacuous");
    eprintln!(
        "[bench-bgp] N={n} hops={hops} key_dom={key_dom} expected_matches={expected}"
    );
    BgpBench { corpus, rule, expected }
}

/// Timed closure: warm-scratch unify over the k-hop BGP, with the
/// oracle-count assert riding every sample. Units: joins/s (Divan
/// items/s, one item per full-chain match).
fn run_hop<const N: usize>(b: Bencher, hops: usize) {
    let bench = build_bench(N, hops);
    let cols = bench.corpus.cols();
    let compiled = RuleCompiler::compile(&bench.rule).expect("court rule compiles");
    let expected = bench.expected;
    let mut scratch = EvalScratch::default();
    let mut probe = JoinProbeScratch::default();
    b.counter(ItemsCount::new(expected as u64)).bench_local(move || {
        let mut out = MatchSet::default();
        unify_batch_probe(
            &compiled,
            black_box(&cols),
            DEFAULT_CHUNK_ROWS,
            &mut scratch,
            &mut out,
            &mut probe,
        )
        .expect("unify_batch_probe succeeds");
        assert_eq!(out.len, expected, "kernel vs oracle count, N={N} hops={hops}");
        black_box(out.len);
    });
}

/// 2-hop BGP equi-join chain.
#[divan::bench(consts = [65_536, 262_144])]
fn bgp_two_hop<const N: usize>(b: Bencher) {
    run_hop::<N>(b, 2);
}

/// 3-hop BGP equi-join chain.
#[divan::bench(consts = [65_536, 262_144])]
fn bgp_three_hop<const N: usize>(b: Bencher) {
    run_hop::<N>(b, 3);
}

fn main() {
    divan::main();
}
