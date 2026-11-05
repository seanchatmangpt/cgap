//! BGP multi-hop columnar equi-join courts (lane p1-bgp, v26.11.2).
//!
//! The BGP surface: k-hop chain `(?x1 P1 ?x2)(?x2 P2 ?x3)...` evaluated
//! by the cgap-bn3 public columnar join API (`RuleCompiler::compile` +
//! `unify_batch_probe` over SoA `StatementColumns`), compared per hop
//! against an independent indexed nested-loop composition oracle over
//! the SAME corpus. Identical result SETS per hop (binding tuples),
//! identical match counts.
//!
//! Corpus: canonical SplitMix64 seed `0xBF58_476D_1CE4_E5B9` (SEED LAW:
//! the algorithm multiplier; 0xA20C_0FFE_E5A7_E000 is the bitslice
//! corpus-spread constant and is never a seed). Sizes 65_536 and
//! 262_144; the key domain scales with N so per-hop fanout stays
//! bounded and both sets materialize.
//!
//! Zero pointer derefs: the join path is index-based over SoA u64
//! columns -- no raw-pointer deref anywhere in the court (the file is
//! `#![forbid(unsafe_code)]`); unify.rs's only unsafe code is the
//! allocation-counting global allocator used by its zero-alloc court,
//! not the join path.
//!
//! Anti-vacuity: mutating one hop predicate in the RULE (only)
//! detaches it from the corpus -- both hop sets go empty and the
//! nonempty-set assert FIRES. Witnessed as the /tmp shim-court in the
//! lane receipt (sed of the HOP2_RULE_PRED line; sha256+mtime restore).

#![forbid(unsafe_code)]

use cgap_bn3::sigil::{encode_term, Sigil};
use cgap_bn3::unify::{
    unify_batch_probe, BodyAtom, CompiledRule, EvalScratch, JoinProbeScratch, MatchSet, Rule,
    RuleCompiler, StatementColumns, DEFAULT_CHUNK_ROWS,
};
use std::collections::{HashMap, HashSet};

/// Canonical SplitMix64 seed (SEED LAW, verbatim).
const SEED: u64 = 0xBF58_476D_1CE4_E5B9;

/// Corpus predicate payloads: hop-1/2/3 predicates + bulk filler.
const P1: u64 = 0x100;
const P2: u64 = 0x200;
const P3: u64 = 0x4B0;
const PF: u64 = 0xF00;

/// The RULE-side hop-2 predicate payload. Corpus rows carry the corpus
/// const P2; the rule atom uses THIS const, so corrupting this line
/// detaches rule from corpus. Mutation witness: sed to `P2 + 1` ->
/// both hop sets go empty -> the nonempty assert fires.
const HOP2_RULE_PRED: u64 = P2;

/// Canonical splitmix64.
struct SplitMix(u64);

impl SplitMix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x0094_D049_0BBA_DD03);
        z ^ (z >> 31)
    }
}

/// Raw A11 SlotConst term word over payload `v`.
fn cst(v: u64) -> u64 {
    encode_term(Sigil::SlotConst, v).expect("in-range payload")
}

/// Raw A11 VarUni term word for variable payload `r` (< 256 by law).
fn vr(r: u64) -> u64 {
    encode_term(Sigil::VarUni, r).expect("in-range register")
}

/// SoA corpus: s/p/o u64 columns over term words.
struct Corpus {
    s: Vec<u64>,
    p: Vec<u64>,
    o: Vec<u64>,
}

impl Corpus {
    fn cols(&self) -> StatementColumns<'_> {
        StatementColumns::new(&self.s, &self.p, &self.o).expect("equal column lengths")
    }
}

/// Build the splitmix64 corpus at `n` rows: predicate class = draw % 4
/// (P1/P2/P3/filler), s/o payloads drawn from `[0, key_dom)`, encoded
/// as term words. Deterministic under SEED.
fn build_corpus(n: usize, key_dom: u64) -> Corpus {
    let mut sm = SplitMix(SEED);
    let mut c = Corpus {
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
        let s = cst((r >> 8) % key_dom);
        let o = cst((r >> 34) % key_dom);
        c.s.push(s);
        c.p.push(cst(p));
        c.o.push(o);
    }
    c
}

/// The k-hop BGP rule: body `(?x_i PRED_i ?x_{i+1})` for i in 0..hops,
/// head `(?x1 H ?x_{hops+1})`. Variable payloads 0..=hops compile to
/// band indices 0..=hops (first-seen order).
fn bgp_rule(hops: usize) -> Rule {
    assert!((2..=3).contains(&hops), "court covers 2-hop and 3-hop");
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

/// A hop-k binding tuple: values of ?x1..?x_{k+1} as full corpus words.
type Bindings = Vec<u64>;

/// Independent ORACLE: indexed nested-loop composition. Hop-1 rows are
/// found by one linear scan (full-word predicate compare); each later
/// hop composes by nested loop over (current matches x candidate rows
/// for that hop's predicate), with the join condition a full-word
/// equality of the chained variable -- the candidate list comes from a
/// payload-keyed index so the court stays tractable, but the join
/// semantics is row-to-row word equality, independent of the kernel's
/// radix probe. Returns the per-hop match counts plus the final-hop
/// binding set.
fn oracle(cols: &StatementColumns<'_>, hops: usize) -> (Vec<usize>, HashSet<Bindings>) {
    let preds = [cst(P1), cst(HOP2_RULE_PRED), cst(P3)];
    // Hop 1: linear scan, bindings [x1 = s, x2 = o].
    let mut cur: Vec<Bindings> = Vec::new();
    for i in 0..cols.len() {
        if cols.p[i] == preds[0] {
            cur.push(vec![cols.s[i], cols.o[i]]);
        }
    }
    let mut counts = vec![cur.len()];
    // Candidate index per later hop: payload(s-word) -> rows.
    for pred in &preds[1..hops] {
        let mut idx: HashMap<u64, Vec<usize>> = HashMap::new();
        for j in 0..cols.len() {
            if cols.p[j] == *pred {
                idx.entry(jk(cols.s[j])).or_default().push(j);
            }
        }
        let mut next: Vec<Bindings> = Vec::new();
        for m in &cur {
            let key = jk(m[m.len() - 1]);
            if let Some(rows) = idx.get(&key) {
                for &j in rows {
                    let mut b = m.clone();
                    b.push(cols.o[j]);
                    next.push(b);
                }
            }
        }
        cur = next;
        counts.push(cur.len());
    }
    (counts, cur.into_iter().collect::<HashSet<_>>())
}

/// Join-key extraction: the 56-bit payload window of a term word (what
/// the corpus encodes; full words are payload-equal iff these are).
fn jk(w: u64) -> u64 {
    w & 0x00FF_FFFF_FFFF_FFFF
}

/// Kernel result set for the k-hop rule: binding tuples from
/// `MatchSet::var_values` (variable payload r compiles to band index
/// r for this rule shape).
fn kernel_set(ms: &MatchSet, hops: usize) -> HashSet<Bindings> {
    let n = ms.len;
    (0..n)
        .map(|m| {
            (0..=hops)
                .map(|v| ms.var_values[v][m])
                .collect::<Bindings>()
        })
        .collect()
}

/// Run the kernel: compile the k-hop rule, evaluate with a warm
/// scratch lease, return the MatchSet.
fn run_kernel(rule: &Rule, cols: &StatementColumns<'_>) -> (CompiledRule, MatchSet) {
    let compiled = RuleCompiler::compile(rule).expect("court rule compiles");
    let mut scratch = EvalScratch::default();
    let mut out = MatchSet::default();
    let mut probe = JoinProbeScratch::default();
    unify_batch_probe(
        &compiled,
        cols,
        DEFAULT_CHUNK_ROWS,
        &mut scratch,
        &mut out,
        &mut probe,
    )
    .expect("unify_batch_probe succeeds");
    (compiled, out)
}

/// One court: run the k-hop BGP over the N-row corpus, compare the
/// kernel per-hop match counts and final binding set against the
/// oracle. Vacuity guards: both sides nonempty, and corpus density
/// sanity (each hop predicate present).
fn court(hops: usize, n: usize) {
    let key_dom = key_dom_for(n);
    let corpus = build_corpus(n, key_dom);
    let cols = corpus.cols();
    let rule = bgp_rule(hops);
    let (counts, oset) = oracle(&cols, hops);
    let (_, ms) = run_kernel(&rule, &cols);
    // Vacuity: the corpus must actually contain every hop predicate.
    for &p in &[P1, HOP2_RULE_PRED, P3][..hops] {
        let want = cst(p);
        assert!(
            cols.p.contains(&want),
            "corpus lacks predicate payload {p:#x} at N={n}: court vacuous"
        );
    }
    assert!(counts[0] > 0, "oracle hop-1 empty at N={n}: vacuous court");
    assert!(
        !oset.is_empty(),
        "oracle final-hop set empty at N={n}: vacuous"
    );
    assert_eq!(
        ms.len,
        counts[hops - 1],
        "kernel vs oracle match count, hops={hops} N={n}"
    );
    let kset = kernel_set(&ms, hops);
    assert_eq!(
        kset, oset,
        "kernel vs oracle binding SET, hops={hops} N={n}"
    );
}

/// Key domain scaled by N so per-hop fanout stays bounded (see file
/// header): 4 rows per (predicate, key) class on average.
fn key_dom_for(n: usize) -> u64 {
    (n / 4 / 4).next_power_of_two() as u64
}

#[test]
fn bgp_two_hop_matches_oracle() {
    court(2, 65_536);
}

#[test]
fn bgp_three_hop_matches_oracle() {
    court(3, 65_536);
}

#[test]
#[ignore = "expensive debug run - run with --release"]
fn bgp_two_hop_matches_oracle_at_scale() {
    court(2, 262_144);
}

#[test]
#[ignore = "expensive debug run - run with --release"]
fn bgp_three_hop_matches_oracle_at_scale() {
    court(3, 262_144);
}
