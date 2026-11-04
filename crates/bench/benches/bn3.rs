//! BENCH-BN3 (lane F2-13) — Divan membrane target `bn3`.
//!
//! Converts the A11-A13 perf TARGETs to MEASURED (docs/jira/v26.11.1/
//! bench-campaign-charter.md bench map row "A11-A13"): 24B statement
//! sequential-scan GB/s (A11), unify ns/pair (A12, metric per
//! doe-4-unifier-data-structure.md Section "Metric definition (primary)":
//! one PAIR = one (parent match, candidate row) probe of the hot loop,
//! pair count = sum over join atoms of cur_len x cand_len, ns/pair =
//! unify_batch_into wall time / pair count), builtin dispatch ns (A13).
//! Receipt: docs/jira/v26.11.1/waves/bench-bn3-receipt.md.
//!
//! Corpus: canonical SplitMix64 seeded `0xBF58_476D_1CE4_E5B9` (charter
//! "Method discipline", verbatim; a non-canonical constant in bench code
//! is the known write-corruption class). Chicago asserts ride the benched
//! closure: scan closure-count, unify binding-count vs an O(N) raw-word
//! histogram oracle (independent of the register machine), dispatch
//! eval-count + integer-oracle checksum. Each bench names the mutation
//! class that must move its number (charter "Anti-vacuity").

use cgap_bn3::arena::Slab;
use cgap_bn3::builtins::{
    dispatch, BuiltinContext, BuiltinPayload, OP_LOG_ASSERT, OP_MATH_GREATER_THAN, OP_MATH_MIN,
    OP_MATH_SUM, REGISTER_COUNT,
};
use cgap_bn3::sigil::{encode_term, BN3Statement, Sigil, BN3_STATEMENT_SIZE, PAYLOAD_MASK};
use cgap_bn3::unify::{
    unify_batch_into, BodyAtom, EvalScratch, MatchSet, Rule, RuleCompiler, StatementColumns,
    DEFAULT_CHUNK_ROWS,
};
use divan::counter::BytesCount;
use divan::{black_box, Bencher};

/// Canonical corpus seed (charter Method discipline, verbatim).
const CANONICAL_SEED: u64 = 0xBF58_476D_1CE4_E5B9;

/// Canonical SplitMix64 (same form as `const_time_bench.rs`/`ffor.rs` —
/// the corruption-witness constant rides the multiplier; seed is the
/// canonical witness constant itself).
struct SplitMix(u64);

impl SplitMix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// SlotConst term word over a masked 56-bit payload. `expect` is corpus
/// mint: the payload is masked into domain before this call, so an Err
/// here is a bench bug and must abort loudly, never coerce.
fn const_word(payload: u64) -> u64 {
    encode_term(Sigil::SlotConst, payload & PAYLOAD_MASK).expect("const_word encode")
}

/// VarUni term word naming variable register `idx` (< 256 by caller law).
fn var_word(idx: u64) -> u64 {
    encode_term(Sigil::VarUni, idx).expect("var_word encode")
}

// ---------------------------------------------------------------------------
// (1) A11 surface: 24B statement sequential scan. TARGET > 1.0 GB/s.
// Mutation class the assert kills: a skipped-row / dropped-tail-chunk scan
// (count < N) flips the closure-count assert.
// ---------------------------------------------------------------------------

/// Fixed slab of `n` statements; s/p/o payloads from the canonical stream.
fn seed_slab(n: usize) -> Slab {
    let mut sm = SplitMix(CANONICAL_SEED);
    let mut slab = Slab::fixed(n as u64).expect("Slab::fixed");
    for _ in 0..n {
        let s = const_word(sm.next());
        let p = const_word(sm.next());
        let o = const_word(sm.next());
        slab.push(BN3Statement::from_words(s, p, o))
            .expect("Slab::push at declared capacity");
    }
    slab
}

/// Sequential 24B/row scan of a fixed slab: one wrapping XOR-fold checksum
/// over the full s/p/o words. The `count` assert is the Chicago final-state
/// assert; the checksum is black_boxed so the fold is not optimized away.
#[divan::bench(consts = [65_536, 1_048_576])]
fn statement_scan_24b<const N: usize>(b: Bencher) {
    let slab = seed_slab(N);
    assert_eq!(slab.len(), N, "seeded count");
    assert_eq!(slab.fixed_capacity(), Some(N), "fixed-capacity law");
    let bytes = N * BN3_STATEMENT_SIZE;
    eprintln!("[bn3-bench] scan N={N} bytes={bytes}");
    b.counter(BytesCount::new(bytes)).bench_local(|| {
        let stmts = black_box(slab.statements());
        let mut sum = 0u64;
        let mut count = 0usize;
        for st in stmts {
            sum = sum.wrapping_add(st.s ^ st.p ^ st.o);
            count += 1;
        }
        assert_eq!(
            count, N,
            "scan closure count (skipped-row mutation flips this)"
        );
        black_box(sum);
    });
}

// ---------------------------------------------------------------------------
// (2) A12 surface: unify ns/pair over the real comparator. TARGET: sub-µs.
// Mutation classes the asserts kill: the binding-alias comparator class
// (doe-4 SHIM-DOE4) and any wrong-value binding order diverge from the
// raw-word oracle; a skipped probe changes the asserted pair count path.
// ---------------------------------------------------------------------------

/// Predicate payload constants: PA = atom-0 match class, PB = atom-1
/// candidate class, PC = bulk filler. Distinct full words.
const PA_PAYLOAD: u64 = 0xA11;
const PB_PAYLOAD: u64 = 0xB22;
const PC_PAYLOAD: u64 = 0xC33;

/// Join-key domain: s and o payloads are drawn from `[0, 8)` (64 keys), so
/// the deterministic corpus carries a healthy match count at both swept
/// capacities (expected ~64 matches at N=1024, ~1024 at N=65536).
const KEY_DOM: u64 = 8;

/// Row layout at capacity N: the first `a_rows = max(16, N/256)` rows carry
/// predicate PA (atom-0 matches = parents), the next `b_rows = 256` rows
/// carry PB (atom-1 candidates), the bulk carries PC. Selectivity is
/// pinned so the nested-loop join's `cur x cand` probe work stays
/// instrument-scale (~65k pairs at N=65536) while matches stay plentiful.
fn join_shape(n: usize) -> (usize, usize) {
    let a_rows = (n / 256).max(16);
    let b_rows = 256usize;
    (a_rows, b_rows)
}

/// SoA columns over the deterministic corpus: s/o payload words in
/// `[0, KEY_DOM)`, predicate class per `join_shape`.
fn seed_join_corpus(n: usize) -> (Vec<u64>, Vec<u64>, Vec<u64>, u64, u64) {
    let (a_rows, b_rows) = join_shape(n);
    let pa = const_word(PA_PAYLOAD);
    let pb = const_word(PB_PAYLOAD);
    let pc = const_word(PC_PAYLOAD);
    let mut sm = SplitMix(CANONICAL_SEED);
    let mut s_col = Vec::with_capacity(n);
    let mut p_col = Vec::with_capacity(n);
    let mut o_col = Vec::with_capacity(n);
    for i in 0..n {
        s_col.push(const_word(sm.next() % KEY_DOM));
        p_col.push(if i < a_rows {
            pa
        } else if i < a_rows + b_rows {
            pb
        } else {
            pc
        });
        o_col.push(const_word(sm.next() % KEY_DOM));
    }
    (s_col, p_col, o_col, pa, pb)
}

/// Raw-word oracle: exact match count of the 2-atom join by direct u64
/// equality over the corpus — NO unify code in the oracle path, so the
/// bench assert is a differential check (register machine vs plain
/// nested-loop semantics), not a tautology.
fn oracle_match_count(s_col: &[u64], p_col: &[u64], o_col: &[u64], pa: u64, pb: u64) -> u64 {
    use std::collections::HashMap;
    let mut hist: HashMap<(u64, u64), u64> = HashMap::new();
    for i in 0..s_col.len() {
        if p_col[i] == pa {
            *hist.entry((s_col[i], o_col[i])).or_insert(0) += 1;
        }
    }
    let mut total = 0u64;
    for i in 0..s_col.len() {
        if p_col[i] == pb {
            total += hist.get(&(s_col[i], o_col[i])).copied().unwrap_or(0);
        }
    }
    total
}

/// Pair count per doe-4's primary metric: `sum over join atoms of
/// cur_len x cand_len`, walked here from the corpus against the SAME
/// chunk boundaries the kernel uses ([`DEFAULT_CHUNK_ROWS`]) — computed
/// once at setup, asserted each iteration alongside the match count.
fn join_pair_count(p_col: &[u64], pa: u64, pb: u64, chunk: usize) -> u64 {
    let n = p_col.len();
    let cand = p_col.iter().filter(|&&w| w == pb).count() as u64;
    let mut pairs = 0u64;
    let mut start = 0usize;
    while start < n {
        let end = (start + chunk).min(n);
        let cur = p_col[start..end].iter().filter(|&&w| w == pa).count() as u64;
        pairs += cur * cand;
        start = end;
    }
    pairs
}

/// The 2-atom join: `(var0, PA, var1) AND (var0, PB, var1)` — matches are
/// row PAIRS sharing both bound terms; the head re-projects the pair.
fn join_rule() -> (Rule, u64, u64) {
    let pa = const_word(PA_PAYLOAD);
    let pb = const_word(PB_PAYLOAD);
    let rule = Rule {
        rule_id: 0xBEEF,
        body: vec![
            BodyAtom::new(var_word(0), pa, var_word(1)),
            BodyAtom::new(var_word(0), pb, var_word(1)),
        ],
        head: [var_word(0), pa, var_word(1)],
    };
    (rule, pa, pb)
}

/// Unify ns/pair at const capacities [1024, 65536], warm-scratch hot path
/// (`unify_batch_into` + reused [`EvalScratch`]/[`MatchSet`] — the
/// production zero-alloc shape; Divan's alloc column must read 0 B).
#[divan::bench(consts = [1024, 65_536])]
fn unify_join_2atom_pairs<const N: usize>(b: Bencher) {
    let (s_col, p_col, o_col, pa, pb) = seed_join_corpus(N);
    let (rule, _, _) = join_rule();
    let compiled = RuleCompiler::compile(&rule).expect("join rule compiles");
    // Compiled-shape asserts: a compiler mutation that drops the join
    // variable band or an atom flips these before any timing happens.
    assert_eq!(compiled.atoms.len(), 2, "two body atoms compiled");
    assert_eq!(compiled.var_count, 2, "join binds exactly two variables");
    let oracle = oracle_match_count(&s_col, &p_col, &o_col, pa, pb);
    let pairs = join_pair_count(&p_col, pa, pb, DEFAULT_CHUNK_ROWS);
    assert!(
        oracle > 0,
        "join oracle empty at N={N}: bench would be vacuous"
    );
    assert!(
        pairs > 0,
        "pair count empty at N={N}: bench would be vacuous"
    );
    eprintln!(
        "[bn3-bench] unify N={N} parents={} cands={} pairs={pairs} oracle_matches={oracle}",
        join_shape(N).0,
        join_shape(N).1,
    );
    let cols = StatementColumns::new(&s_col, &p_col, &o_col).expect("columns");
    let mut scratch = EvalScratch::default();
    let mut out = MatchSet::default();
    b.bench_local(|| {
        unify_batch_into(
            black_box(&compiled),
            black_box(&cols),
            DEFAULT_CHUNK_ROWS,
            &mut scratch,
            &mut out,
        )
        .expect("unify_batch_into");
        // Binding-count assert: the raw-word oracle predicts the match
        // count exactly; a mutated comparator (binding-alias class,
        // doe-4 SHIM-DOE4) or wrong binding order diverges and flips it.
        assert_eq!(
            out.len, oracle as usize,
            "unify binding count vs raw-word oracle"
        );
        assert_eq!(
            out.var_values[0].len(),
            oracle as usize,
            "var0 column alignment"
        );
        assert_eq!(
            out.var_values[1].len(),
            oracle as usize,
            "var1 column alignment"
        );
        black_box((out.atom_rows[0].len(), out.atom_rows[1].len()));
    });
}

// ---------------------------------------------------------------------------
// (3) A13 surface: builtin dispatch (hot arms) at 65,536 evaluations.
// Mutation classes the asserts kill: an arm swap (min->max), an inverted
// gt comparator, an always-failing assert arm, or a skipped evaluation
// each move the integer-oracle checksum or the eval_count closure count.
// ---------------------------------------------------------------------------

/// Evaluations per iteration (charter dispatch cell).
const EVALS: usize = 65_536;

/// `log:assert` payloads self-compare R[201] (always passes, writes 1).
const ASSERT_REG: u8 = 201;

/// Hot-arm payload table + integer oracle. Operand band R[0..64) is
/// read-only for this workload; every math result lands in R[64..192), so
/// each result is a function of the initial register file alone — the
/// expected checksum needs no dispatch call to compute (differential
/// oracle, not a replay of the code under test).
#[divan::bench]
fn builtin_dispatch_hot_arms_65k(b: Bencher) {
    let mut sm = SplitMix(CANONICAL_SEED);
    const OP_BAND: usize = 64;
    let mut regs0 = [0u64; REGISTER_COUNT];
    for r in regs0.iter_mut().take(OP_BAND) {
        // Two-operand sum stays under the 2^56 payload ceiling.
        *r = sm.next() % (1u64 << 54);
    }
    // Real 4 KiB slab (crypto arms would consume it; the hot arms don't —
    // passed so the dispatch collaborator is real, not stubbed).
    let slab: Vec<u8> = (0..4096).map(|_| sm.next() as u8).collect();

    const OPCODE_CYCLE: [u8; 4] = [
        OP_MATH_GREATER_THAN,
        OP_MATH_SUM,
        OP_MATH_MIN,
        OP_LOG_ASSERT,
    ];
    let mut payloads = Vec::with_capacity(EVALS);
    let mut expected = 0u64;
    for i in 0..EVALS {
        let op = OPCODE_CYCLE[i % 4];
        let (dst, a, bb) = if op == OP_LOG_ASSERT {
            (ASSERT_REG, ASSERT_REG, 0)
        } else {
            let dst = (OP_BAND as u64 + sm.next() % 128) as u8;
            let a = (sm.next() % OP_BAND as u64) as u8;
            let bb = (sm.next() % OP_BAND as u64) as u8;
            (dst, a, bb)
        };
        let (va, vb) = (regs0[a as usize], regs0[bb as usize]);
        let res = match op {
            OP_MATH_GREATER_THAN => u64::from(va > vb),
            OP_MATH_SUM => va.wrapping_add(vb),
            OP_MATH_MIN => {
                if va < vb {
                    va
                } else {
                    vb
                }
            }
            _ => 1, // log:assert self-compare succeeds -> result 1
        };
        expected = expected.wrapping_add(res);
        payloads.push(BuiltinPayload::new(op, dst, a, bb));
    }
    assert_eq!(payloads.len(), EVALS, "payload table size");
    eprintln!(
        "[bn3-bench] dispatch evals={EVALS} ops=gt,sum,min,assert \
         expected_checksum=0x{expected:016X}"
    );

    b.bench_local(|| {
        let mut regs = regs0;
        let mut ctx = BuiltinContext::default();
        let mut sum = 0u64;
        for &pl in &payloads {
            let r = dispatch(pl, &mut regs, &slab, &mut ctx).expect("hot-arm dispatch ok");
            sum = sum.wrapping_add(r);
        }
        // Closure count: BuiltinContext::eval_count increments on every
        // dispatch entry (the production counter, not a test double).
        assert_eq!(ctx.eval_count, EVALS as u64, "dispatch closure count");
        assert_eq!(sum, expected, "dispatch checksum vs integer oracle");
        black_box((sum, regs[OP_BAND]));
    });
}

fn main() {
    divan::main();
}
