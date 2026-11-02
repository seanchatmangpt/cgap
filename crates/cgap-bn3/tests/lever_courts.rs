//! L3 differential court (W2-12 shell, spec 07-court-corpus.md Section 2).
//!
//! Compares the real nested-loop join path (`unify_batch` over
//! `unify.rs:793-880`) against a naive raw-word hash-join oracle over a
//! CourtRng corpus at N in {1, 64, 4096} (100_003 lands with the W3
//! lever bench). Asserts match-for-match identical binding sets.
//!
//! Anti-vacuity: the court self-mutates its own oracle in-test with the
//! M-L3-1 semantics (join probe dropped: candidate matches regardless of
//! bound-slot equality) and requires the differential to FLIP — a green
//! court where the mutated oracle still agrees is VACUOUS and fails.
//! M-L3-2 (alias comparator, `unify.rs:560-563` `lane_eq_mask`) is the
//! W3 kernel mutation; this shell proves the assertion harness binds.
//!
//! SEED LAW: CourtRng seed is 0xBF58_476D_1CE4_E5B9 (canonical splitmix64
//! seed). 0xA20C_0FFE_E5A7_E000 is the bitslice corpus-spread constant,
//! never a seed.

use cgap_bn3::sigil::{encode_term, Sigil};
use cgap_bn3::unify::{BodyAtom, MatchSet, Rule, RuleCompiler, StatementColumns};
use std::collections::HashSet;

/// Canonical splitmix64 (SEED LAW: 0xBF58_476D_1CE4_E5B9).
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

    /// Value from a small pool, so joins are nontrivial (many candidate
    /// rows share a join key; selectivity comes from the corpus shape).
    fn pooled(&mut self, pool: u64) -> u64 {
        BASE + (self.next() % pool)
    }
}

const BASE: u64 = 0x000A_0000_0000_0000;
const SEED: u64 = 0xBF58_476D_1CE4_E5B9;

const S_POOL: u64 = 8;
const P_POOL: u64 = 4;
const O_POOL: u64 = 16;

/// Raw A11 SlotConst term word for corpus row value `v`.
fn cst(v: u64) -> u64 {
    encode_term(Sigil::SlotConst, v).expect("in-range payload")
}

/// Raw A11 VarUni term word for variable register `r`.
fn vr(r: u64) -> u64 {
    encode_term(Sigil::VarUni, r).expect("in-range register")
}

/// The two-atom join rule under court. Variable payloads 0,1,2 compile to
/// band indices 0,1,2 (first-seen order): ?x joins atom0->atom1.
///   atom0: (?x P ?y)   atom1: (?y Q ?z)
fn court_rule() -> Rule {
    Rule {
        rule_id: 7,
        body: vec![
            BodyAtom::new(vr(0), cst(0x000A_0000_0000_0000 + 1), vr(1)),
            BodyAtom::new(vr(1), cst(0x000A_0000_0000_0000 + 2), vr(2)),
        ],
        head: [vr(0), cst(0x000A_0000_0000_0000 + 3), vr(2)],
    }
}

/// One joined match as a canonical tuple (atom-0 row, atom-1 row,
/// bindings for ?x, ?y, ?z).
type MatchKey = (u32, u32, u64, u64, u64);

/// Naive raw-word oracle: nested double loop over corpus rows, pattern
/// match atom0 against row i, atom1 against row j, shared variable ?y
/// must agree. `probe_dropped` reproduces the M-L3-1 mutation semantics
/// (bound-slot probe skipped, prior binding kept) — used ONLY by the
/// anti-vacuity arm.
fn oracle(cols: &StatementColumns<'_>, rule: &Rule, probe_dropped: bool) -> HashSet<MatchKey> {
    let mut out = HashSet::new();
    let n = cols.len();
    for i in 0..n {
        let mut b0 = [None::<u64>; 3];
        if !atom_match(rule.body[0], cols, i, &mut b0, true) {
            continue;
        }
        for j in 0..n {
            let mut b1 = b0;
            if !atom_match(rule.body[1], cols, j, &mut b1, !probe_dropped) {
                continue;
            }
            out.insert((
                i as u32,
                j as u32,
                b1[0].expect("?x bound"),
                b1[1].expect("?y bound"),
                b1[2].expect("?z bound"),
            ));
        }
    }
    out
}

/// Pattern-match one body atom against corpus row `row`. `Const` slots
/// compare raw word equality; `Var` slots bind fresh or, when already
/// bound and `enforce`, must equal the candidate (the join probe's
/// semantics). Bindings hold full corpus words (what the kernel stores
/// in `MatchSet::var_values`).
fn atom_match(
    atom: BodyAtom,
    cols: &StatementColumns<'_>,
    row: usize,
    binds: &mut [Option<u64>; 3],
    enforce: bool,
) -> bool {
    let words = [atom.s, atom.p, atom.o];
    for (k, &word) in words.iter().enumerate() {
        let sig = (word >> cgap_bn3::sigil::SIGIL_SHIFT) as u8;
        let cand = [cols.s[row], cols.p[row], cols.o[row]][k];
        if sig == Sigil::VarUni as u8 {
            let idx = (word & cgap_bn3::sigil::PAYLOAD_MASK) as usize;
            match binds[idx] {
                Some(v) if enforce && v != cand => return false,
                Some(_) => {}
                None => binds[idx] = Some(cand),
            }
        } else if word != cand {
            return false;
        }
    }
    true
}

/// Extract the kernel's joined matches as the canonical key set.
fn kernel_keys(rule: &Rule, cols: &StatementColumns<'_>, chunk: usize) -> HashSet<MatchKey> {
    let compiled = RuleCompiler::compile(rule).expect("court rule compiles");
    let ms: MatchSet =
        cgap_bn3::unify::unify_batch(&compiled, cols, chunk).expect("unify_batch ok");
    assert_eq!(ms.len, ms.atom_rows[0].len());
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

fn corpus(n: usize) -> (Vec<u64>, Vec<u64>, Vec<u64>) {
    let mut rng = CourtRng::new(SEED);
    let (mut s, mut p, mut o) = (Vec::with_capacity(n), Vec::with_capacity(n), Vec::with_capacity(n));
    for _ in 0..n {
        s.push(rng.pooled(S_POOL));
        p.push(rng.pooled(P_POOL));
        o.push(rng.pooled(O_POOL));
    }
    (s, p, o)
}

#[test]
fn l3_differential_join_court() {
    let rule = court_rule();
    for &n in &[1usize, 64, 4096] {
        let (s, p, o) = corpus(n);
        let cols = StatementColumns::new(&s, &p, &o).expect("columns");
        for &chunk in &[1usize, 64, 4096] {
            let kernel = kernel_keys(&rule, &cols, chunk);
            let truth = oracle(&cols, &rule, false);
            assert_eq!(kernel.len(), truth.len(), "N={n} chunk={chunk}: match count");
            assert_eq!(kernel, truth, "N={n} chunk={chunk}: binding sets differ");
        }
    }
}

#[test]
fn l3_antivacuity_probe_drop_flips() {
    // Non-vacuity: under the M-L3-1 semantics (bound-slot probe dropped)
    // the oracle MUST produce a strictly larger set on the N=4096 corpus
    // — otherwise the differential above proves nothing.
    let rule = court_rule();
    let (s, p, o) = corpus(4096);
    let cols = StatementColumns::new(&s, &p, &o).expect("columns");
    let truth = oracle(&cols, &rule, false);
    let mutated = oracle(&cols, &rule, true);
    assert!(!truth.is_empty(), "corpus must yield joins");
    assert!(
        mutated.len() > truth.len(),
        "VACUOUS: dropped probe did not flip (truth={} mutated={})",
        truth.len(),
        mutated.len()
    );
}

// --- W3-L3 additions ---------------------------------------------------

use cgap_bn3::noalloc::{NaEngine, NaError, NaMatches, NA_MAX_ROWS};
use cgap_bn3::unify::{
    emit_join_masks, unify_batch_probe, CompiledRule, EvalScratch, JoinProbeScratch,
};


/// Heap-direct zeroed Box for the multi-MiB fixed buffers (the crate's
/// `boxed_zeroed` pattern; `Box::new(T::zeroed())` materializes the
/// zeroed value on the 2 MiB test stack first — witnessed overflow).
fn boxed_zeroed_na<T>() -> Box<T> {
    let v = vec![0u8; core::mem::size_of::<T>()];
    unsafe { Box::from_raw(Box::into_raw(v.into_boxed_slice()) as *mut T) }
}

/// Three-way match-for-match differential: alloc radix probe (default
/// `unify_batch_probe`) vs alloc nested probe (`radix: false`) at
/// N in {1, 64, 4096} — every chunk size. The nested probe is the
/// pre-L3 micro-architecture; identity of match SETS (HashSet) is the
/// pinned property; order identity is pinned by the in-crate M12-6
/// courts over the default (radix) path.
#[test]
fn l3_radix_vs_nested_differential() {
    let rule = court_rule();
    for &n in &[1usize, 64, 4096] {
        let (s, p, o) = corpus(n);
        let cols = StatementColumns::new(&s, &p, &o).expect("columns");
        for &chunk in &[1usize, 64, 4096] {
            let compiled = RuleCompiler::compile(&rule).expect("compile");
            let mut scratch = EvalScratch::default();
            let mut out_r = MatchSet::default();
            let mut out_n = MatchSet::default();
            let mut pr = JoinProbeScratch::default();
            let mut pn = JoinProbeScratch { radix: false, ..Default::default() };
            unify_batch_probe(&compiled, &cols, chunk, &mut scratch, &mut out_r, &mut pr)
                .expect("radix ok");
            unify_batch_probe(&compiled, &cols, chunk, &mut scratch, &mut out_n, &mut pn)
                .expect("nested ok");
            assert_eq!(out_r.len, out_n.len, "N={n} chunk={chunk}: radix/nested count");
            for m in 0..out_r.len {
                assert_eq!(out_r.atom_rows[0][m], out_n.atom_rows[0][m], "row0 m={m}");
                assert_eq!(out_r.atom_rows[1][m], out_n.atom_rows[1][m], "row1 m={m}");
                for idx in 0..compiled.var_count {
                    assert_eq!(out_r.var_values[idx][m], out_n.var_values[idx][m], "var m={m}");
                }
            }
        }
    }
}

/// Wide-pool corpus (semi-join selectivity knob) for the 100_003 leg.
fn corpus_wide(n: usize) -> (Vec<u64>, Vec<u64>, Vec<u64>) {
    let mut rng = CourtRng::new(SEED);
    let pools = (2048u64, 4u64, 4096u64);
    let (mut s, mut p, mut o) = (Vec::with_capacity(n), Vec::with_capacity(n), Vec::with_capacity(n));
    for _ in 0..n {
        s.push(rng.pooled(pools.0));
        p.push(rng.pooled(pools.1));
        o.push(rng.pooled(pools.2));
    }
    (s, p, o)
}

/// N=100_003 leg: radix vs nested at 100_003, wide pools so the match
/// count stays bounded; sequence-ordered identity (Vec, not HashSet).
#[test]
fn l3_radix_vs_nested_100k() {
    let rule = court_rule();
    let n = 100_003usize;
    let (s, p, o) = corpus_wide(n);
    let cols = StatementColumns::new(&s, &p, &o).expect("columns");
    let compiled = RuleCompiler::compile(&rule).expect("compile");
    let mut scratch = EvalScratch::default();
    let mut out_r = MatchSet::default();
    let mut out_n = MatchSet::default();
    let mut pr = JoinProbeScratch::default();
    let mut pn = JoinProbeScratch { radix: false, ..Default::default() };
    unify_batch_probe(&compiled, &cols, 4096, &mut scratch, &mut out_r, &mut pr)
        .expect("radix ok");
    unify_batch_probe(&compiled, &cols, 4096, &mut scratch, &mut out_n, &mut pn)
        .expect("nested ok");
    assert_eq!(out_r.len, out_n.len, "100_003: radix/nested count");
    for m in 0..out_r.len {
        assert_eq!(out_r.atom_rows[0][m], out_n.atom_rows[0][m], "row0 m={m}");
        assert_eq!(out_r.atom_rows[1][m], out_n.atom_rows[1][m], "row1 m={m}");
        for idx in 0..compiled.var_count {
            assert_eq!(out_r.var_values[idx][m], out_n.var_values[idx][m], "var m={m}");
        }
    }
}

/// Noalloc radix mirror differential: `na_unify_radix` (fixed-capacity
/// radix tables) vs the ALLOC arm (semantics authority, radix default)
/// at N in {1, 64, 4096} — match-for-match (rows + binding values).
#[test]
fn l3_noalloc_radix_differential() {
    let rule = court_rule();
    for &n in &[1usize, 64, 4096] {
        // Pools narrowed vs the alloc differential so the joined match
        // count stays inside the noalloc arm's NA_MAX_MATCHES (2048).
        let (s, p, o) = if n == 4096 {
            let mut rng = CourtRng::new(SEED);
            let mut t = (Vec::new(), Vec::new(), Vec::new());
            for _ in 0..n {
                t.0.push(rng.pooled(1024));
                t.1.push(rng.pooled(4));
                t.2.push(rng.pooled(1024));
            }
            t
        } else {
            corpus(n)
        };
        let cols = StatementColumns::new(&s, &p, &o).expect("columns");
        let compiled = RuleCompiler::compile(&rule).expect("compile");
        let na_rule = cgap_bn3::noalloc::na_compile(&rule).expect("na compile");
        let mut eng = boxed_zeroed_na::<NaEngine>();
        let mut out = boxed_zeroed_na::<NaMatches>();
        cgap_bn3::noalloc::na_unify_radix(&na_rule, &cols, &mut eng, &mut out)
            .expect("na_unify_radix ok");
        let mut scratch = EvalScratch::default();
        let mut probe = JoinProbeScratch::default();
        let mut ms = MatchSet::default();
        unify_batch_probe(&compiled, &cols, 4096, &mut scratch, &mut ms, &mut probe)
            .expect("alloc ok");
        if n > 1 {
            assert!(ms.len > 0, "N={n}: court must yield joins (anti-vacuity)");
        }
        assert_eq!(ms.len, out.len, "N={n}: alloc/noalloc match count");
        for m in 0..out.len {
            assert_eq!(ms.atom_rows[0][m], out.atom_row(m, 0), "atom0 m={m}");
            assert_eq!(ms.atom_rows[1][m], out.atom_row(m, 1), "atom1 m={m}");
            for idx in 0..compiled.var_count {
                assert_eq!(ms.var_values[idx][m], out.var_value(m, idx), "var m={m}");
            }
        }
    }
}

/// Loud refusal: a store wider than the fixed radix table capacity
/// refuses with `NaError::MatchCapacity` BEFORE any table write — never
/// a silent truncation.
#[test]
fn l3_noalloc_radix_capacity_refusal() {
    let rule = court_rule();
    let na_rule = cgap_bn3::noalloc::na_compile(&rule).expect("na compile");
    let n = NA_MAX_ROWS + 1;
    let mut s = Vec::with_capacity(n);
    let mut p = Vec::with_capacity(n);
    let mut o = Vec::with_capacity(n);
    for i in 0..n {
        s.push(cst(i as u64));
        p.push(cst(0x000A_0000_0000_0000 + 1));
        o.push(cst(i as u64));
    }
    let cols = StatementColumns::new(&s, &p, &o).expect("columns");
    let mut eng = boxed_zeroed_na::<NaEngine>();
    let mut out = boxed_zeroed_na::<NaMatches>();
    let err = cgap_bn3::noalloc::na_unify_radix(&na_rule, &cols, &mut eng, &mut out)
        .expect_err("loud refusal at capacity");
    assert_eq!(err, NaError::MatchCapacity, "typed refusal");
    assert_eq!(out.len, 0, "no partial state before refusal");
}

/// emit_join_masks court: the point-range semi-join mask equals the
/// naive per-row oracle (key payload in bound_keys AND every Const slot
/// full-word equal), popcount matches, and a nonempty corpus yields
/// nonzero masks. Anti-vacuity mutation (in-test, oracle side): drop
/// the Const arm from the oracle -> the asserted mask equality MUST
/// flip (a green court under the mutated oracle is VACUOUS, fails).
#[test]
fn l3_emit_join_masks_court() {
    // corpus of 100 rows; join atom: (Const p) (Var ?y key) (Const o)
    let (s, p, o) = corpus(100);
    let cols = StatementColumns::new(&s, &p, &o).expect("columns");
    let kp = cst(0x000A_0000_0000_0000 + 2);
    let co = cst(0x000A_0000_0000_0000 + 9);
    // atom: s = ?y (key lane), p = const, o = const — compiled form
    let rule = Rule {
        rule_id: 11,
        body: vec![BodyAtom::new(vr(0), kp, co)],
        head: [vr(0), kp, co],
    };
    let compiled = RuleCompiler::compile(&rule).expect("compile");
    let atom = &compiled.atoms[0];
    let bound_keys = {
        let mut keys: Vec<u64> = (0..100)
            .filter(|&i| cols.p[i] == kp)
            .map(|i| cols.s[i])
            .collect();
        keys.sort_unstable();
        keys.dedup();
        keys
    };
    assert!(!bound_keys.is_empty(), "court corpus must have join keys");
    let mut masks = vec![0u64; cols.len().div_ceil(64)];
    let cnt = emit_join_masks(&cols, atom, &compiled.const_pool, &bound_keys, &mut masks);
    let mut oracle = 0u64;
    let mut oracle_no_const = 0u64;
    for i in 0..100 {
        let key_hit = bound_keys
            .iter()
            .any(|&k| k & 0x00FF_FFFF_FFFF_FFFF == cols.s[i] & 0x00FF_FFFF_FFFF_FFFF);
        if !key_hit {
            continue;
        }
        let const_ok = cols.p[i] == kp && cols.o[i] == co;
        let only_key = true;
        if key_hit && const_ok {
            oracle += 1;
        }
        if key_hit && only_key {
            oracle_no_const += 1;
        }
    }
    let pop: u32 = masks.iter().map(|m| m.count_ones()).sum();
    assert_eq!(cnt as u32, pop, "returned popcount == mask popcount");
    assert_eq!(pop as u64, oracle, "mask == naive oracle (key AND const)");
    assert!(
        oracle_no_const > oracle,
        "VACUOUS: dropping the Const arm did not flip the oracle"
    );
}

