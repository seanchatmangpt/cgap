//! BENCH-3 (lane BENCH-3-bn3-chain) — chained deduction pipeline bench.
//!
//! Business case (the 80/20): line-speed deductive reasoning — rules at
//! cache speed is the substrate's reason to exist. This target measures
//! the WHOLE chain end to end (24B statement scan -> RuleCompiler ->
//! unify_batch closure over the scanned set -> ConsequentProjector
//! project) and, via per-stage probes over the same corpus, attributes
//! the cost: scan vs compile vs unify vs project — which carries ~80%.
//! Companion target `bn3` measures stages in isolation; this one measures
//! the composed pipeline. Receipt:
//! docs/jira/v26.11.1/waves/bench-chains-bn3.md.
//!
//! Corpus: canonical SplitMix64 seed `0xBF58_476D_1CE4_E5B9` (charter
//! "Method discipline", verbatim; a non-canonical constant in bench code
//! is the known write-corruption class). Chicago asserts ride the benched
//! closure: scan closure-count, per-rule match counts vs O(N) raw-word
//! oracles (independent of the register machine), projected-row counts +
//! M12-7 popcounts, digest checksums vs a setup pass (determinism). Each
//! bench names the mutation class that must move its number (charter
//! "Anti-vacuity").

use cgap_bn3::arena::Slab;
use cgap_bn3::project::{ConsequentProjector, DeltaRows};
use cgap_bn3::sigil::{encode_term, BN3Statement, Sigil, BN3_STATEMENT_SIZE, PAYLOAD_MASK};
use cgap_bn3::unify::{
    unify_batch_into, unify_batch_probe, BodyAtom, CompiledRule, EvalScratch, JoinProbeScratch,
    MatchSet, Rule, RuleCompiler, StatementColumns, DEFAULT_CHUNK_ROWS,
};
use divan::counter::{BytesCount, ItemsCount};
use divan::{black_box, Bencher};

/// GAUGE: N=7 min/med/max — alloc column is MEASURED via AllocProfiler
/// (divan 0.1.21: `#[global_allocator]` is `#[cfg(test)]`-only upstream,
/// so the bench binary installs it itself — pattern mirrors
/// benches/parquet.rs:34-35). Without this install, tallies are never
/// collected and any "alloc column reads 0 B" claim would be vacuous.
#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

/// Canonical corpus seed (charter Method discipline, verbatim).
const CANONICAL_SEED: u64 = 0xBF58_476D_1CE4_E5B9;

/// Fixed projection epoch (spec 4.4 determinism contract: fixed input +
/// fixed rule set => byte-identical batch).
const CHAIN_EPOCH: u64 = 7;

/// Canonical SplitMix64 (same form as `bn3.rs` — the corruption-witness
/// constant rides the multiplier; the seed is the witness constant).
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

/// SlotConst term word over a masked 56-bit payload. Payloads are
/// pre-masked by callers, so an Err here is a bench bug: abort loudly.
fn const_word(payload: u64) -> u64 {
    encode_term(Sigil::SlotConst, payload & PAYLOAD_MASK).expect("const_word encode")
}

/// VarUni term word naming variable register `idx` (< 256 by caller law).
fn var_word(idx: u64) -> u64 {
    encode_term(Sigil::VarUni, idx).expect("var_word encode")
}

/// Predicate payload classes: PA = selector parents + join left side,
/// PB = join candidates, PC = bulk filler; PH0/PH1 = head constants of
/// the two rules. Distinct full words.
const PA_PAYLOAD: u64 = 0xA11;
const PB_PAYLOAD: u64 = 0xB22;
const PC_PAYLOAD: u64 = 0xC33;
const PH0_PAYLOAD: u64 = 0xD44;
const PH1_PAYLOAD: u64 = 0xE55;

/// Join-key domain: s and o payloads drawn from `[0, KEY_DOM)` (64 key
/// pairs) — same selectivity law as `bn3.rs`, pinned so the nested-loop
/// join's probe work stays O(N): parents = N/256 rows, candidates = 256
/// rows, probe pairs = parents x candidates = N.
const KEY_DOM: u64 = 8;

/// Row-band shape at capacity N: the first `a_rows = max(16, N/256)` rows
/// carry predicate PA (selector matches AND join parents), the next 256
/// rows carry PB (join candidates), the bulk carries PC. Identical law to
/// `bn3.rs::join_shape` so the two targets' numbers are comparable.
fn join_shape(n: usize) -> (usize, usize) {
    ((n / 256).max(16), 256usize)
}

/// Pair count per doe-4's primary metric (`sum over join atoms of
/// cur_len x cand_len`, chunked at [`DEFAULT_CHUNK_ROWS`]) — same walk as
/// `bn3.rs::join_pair_count`; setup-computed, in-band witness.
fn join_pair_count(p_words: &[u64], pa: u64, pb: u64, chunk: usize) -> u64 {
    let n = p_words.len();
    let cand = p_words.iter().filter(|&&w| w == pb).count() as u64;
    let mut pairs = 0u64;
    let mut start = 0usize;
    while start < n {
        let end = (start + chunk).min(n);
        let cur = p_words[start..end].iter().filter(|&&w| w == pa).count() as u64;
        pairs += cur * cand;
        start = end;
    }
    pairs
}

/// Raw-word oracle over the SLAB (not the scan columns — the oracle path
/// shares zero code with the scan stage): exact counts of (a) the 1-atom
/// selector matches and (b) the 2-atom join by direct u64 equality
/// (hash join on the (s,o) word pair). NO unify code in the oracle path,
/// so the bench assert is a differential check, not a tautology.
fn oracle_counts(slab: &Slab, pa: u64, pb: u64) -> (usize, usize) {
    use std::collections::HashMap;
    let mut sel = 0usize;
    let mut hist: HashMap<(u64, u64), u64> = HashMap::new();
    for st in slab.statements() {
        if st.p == pa {
            sel += 1;
            *hist.entry((st.s, st.o)).or_insert(0) += 1;
        }
    }
    let mut join = 0usize;
    for st in slab.statements() {
        if st.p == pb {
            join += hist.get(&(st.s, st.o)).copied().unwrap_or(0) as usize;
        }
    }
    (sel, join)
}

/// The rule set: R_SEL — one-atom selector `(?x PA ?y) -> (?x PH0 ?y)`;
/// R_JOIN — two-atom join `(?x PA ?y) AND (?x PB ?y) -> (?x PH1 ?y)`.
/// Two rules so the compile stage is a real rule-SET compile, not a
/// single-rule special case.
fn rule_set() -> Vec<Rule> {
    let pa = const_word(PA_PAYLOAD);
    let pb = const_word(PB_PAYLOAD);
    vec![
        Rule {
            rule_id: 0xC0DE_0001,
            body: vec![BodyAtom::new(var_word(0), pa, var_word(1))],
            head: [var_word(0), const_word(PH0_PAYLOAD), var_word(1)],
        },
        Rule {
            rule_id: 0xC0DE_0002,
            body: vec![
                BodyAtom::new(var_word(0), pa, var_word(1)),
                BodyAtom::new(var_word(0), pb, var_word(1)),
            ],
            head: [var_word(0), const_word(PH1_PAYLOAD), var_word(1)],
        },
    ]
}

/// Fixed corpus: slab + rules + oracle expectations, built ONCE per bench
/// invocation (setup), never inside the timed closure. Every expected
/// count comes from the raw-word oracle, not from code under test.
struct ChainCorpus {
    n: usize,
    slab: Slab,
    rules: Vec<Rule>,
    expected_sel: usize,
    expected_join: usize,
}

fn build_corpus(n: usize) -> ChainCorpus {
    let mut sm = SplitMix(CANONICAL_SEED);
    let mut slab = Slab::fixed(n as u64).expect("Slab::fixed");
    let (a_rows, b_rows) = join_shape(n);
    let pa = const_word(PA_PAYLOAD);
    let pb = const_word(PB_PAYLOAD);
    let pc = const_word(PC_PAYLOAD);
    for i in 0..n {
        let s = const_word(sm.next() % KEY_DOM);
        let p = if i < a_rows {
            pa
        } else if i < a_rows + b_rows {
            pb
        } else {
            pc
        };
        let o = const_word(sm.next() % KEY_DOM);
        slab.push(BN3Statement::from_words(s, p, o))
            .expect("Slab::push at declared capacity");
    }
    assert_eq!(slab.len(), n, "seeded count");
    assert_eq!(slab.fixed_capacity(), Some(n), "fixed-capacity law");
    let (expected_sel, expected_join) = oracle_counts(&slab, pa, pb);
    let p_words: Vec<u64> = slab.statements().iter().map(|st| st.p).collect();
    let pairs = join_pair_count(&p_words, pa, pb, DEFAULT_CHUNK_ROWS);
    const MIN_ORACLE_COUNT: usize = 0;
    const MIN_PAIR_COUNT: u64 = 0;
    assert!(
        expected_sel > MIN_ORACLE_COUNT && expected_join > MIN_ORACLE_COUNT,
        "oracle empty at N={n}: bench would be vacuous"
    );
    assert!(
        pairs > MIN_PAIR_COUNT,
        "pair count empty at N={n}: bench would be vacuous"
    );
    eprintln!(
        "[chains-bn3] corpus N={n} parents={a_rows} cands={b_rows} pairs={pairs} \
         expected_sel={expected_sel} expected_join={expected_join}"
    );
    ChainCorpus {
        n,
        slab,
        rules: rule_set(),
        expected_sel,
        expected_join,
    }
}

/// Warm workspace: the production zero-alloc hot-path shape — columns,
/// scratch, match sets, delta rows and canonical-byte buffers are
/// caller-owned and reused across iterations (capacity retained). Built
/// prewarmed: the constructor runs one full scan so every timed scan
/// starts from retained capacity, never a cold grow.
struct Workspace {
    col_s: Vec<u64>,
    col_p: Vec<u64>,
    col_o: Vec<u64>,
    scratch: EvalScratch,
    probe: JoinProbeScratch,
    out_sel: MatchSet,
    out_join: MatchSet,
    rows_sel: DeltaRows,
    rows_join: DeltaRows,
    canon_sel: Vec<u8>,
    canon_join: Vec<u8>,
    projector: ConsequentProjector,
}

impl Workspace {
    fn new(corpus: &ChainCorpus) -> Self {
        let mut ws = Workspace {
            col_s: Vec::with_capacity(corpus.n),
            col_p: Vec::with_capacity(corpus.n),
            col_o: Vec::with_capacity(corpus.n),
            scratch: EvalScratch::default(),
            probe: JoinProbeScratch::default(),
            out_sel: MatchSet::default(),
            out_join: MatchSet::default(),
            rows_sel: DeltaRows::default(),
            rows_join: DeltaRows::default(),
            canon_sel: Vec::new(),
            canon_join: Vec::new(),
            projector: ConsequentProjector::new(),
        };
        for st in corpus.slab.statements() {
            ws.col_s.push(st.s);
            ws.col_p.push(st.p);
            ws.col_o.push(st.o);
        }
        assert_eq!(ws.col_s.len(), corpus.n, "prewarm scan");
        ws
    }
}

/// Stage 1 — SCAN: sequential 24B/row walk of the fixed slab into SoA
/// columns (the AoS->SoA ingest the unifier reads). Mutation class the
/// assert kills: a skipped-row / dropped-tail-chunk scan (count < N
/// flips the closure-count assert).
fn scan_stage(corpus: &ChainCorpus, ws: &mut Workspace) -> usize {
    let stmts = black_box(corpus.slab.statements());
    ws.col_s.clear();
    ws.col_p.clear();
    ws.col_o.clear();
    let mut count = 0usize;
    for st in stmts {
        ws.col_s.push(st.s);
        ws.col_p.push(st.p);
        ws.col_o.push(st.o);
        count += 1;
    }
    assert_eq!(
        count, corpus.n,
        "scan closure count (skipped-row mutation flips this)"
    );
    count
}

/// Stage 2 — COMPILE: [`RuleCompiler`] over the two-rule set. Mutation
/// class: deleting the join's second body atom flips the atoms.len()
/// assert (2 -> 1) and moves compile ns (cost scales with atom count).
fn compile_stage(corpus: &ChainCorpus) -> (CompiledRule, CompiledRule) {
    let c_sel = RuleCompiler::compile(&corpus.rules[0]).expect("selector rule compiles");
    let c_join = RuleCompiler::compile(&corpus.rules[1]).expect("join rule compiles");
    assert_eq!(c_sel.atoms.len(), 1, "selector compiles to one body atom");
    assert_eq!(c_sel.var_count, 2, "selector binds two variables");
    assert_eq!(c_join.atoms.len(), 2, "join compiles to two body atoms");
    assert_eq!(c_join.var_count, 2, "join binds exactly two variables");
    (c_sel, c_join)
}

/// Stage 3 — UNIFY: `unify_batch_into` per rule over the scanned columns
/// (warm scratch + match sets). Mutation classes: the binding-alias
/// comparator (doe-4 SHIM-DOE4 class) or a skipped join probe diverge
/// from the raw-word oracle and flip the count asserts.
fn unify_stage(
    corpus: &ChainCorpus,
    ws: &mut Workspace,
    c_sel: &CompiledRule,
    c_join: &CompiledRule,
) {
    let cols = StatementColumns::new(&ws.col_s, &ws.col_p, &ws.col_o).expect("columns");
    assert_eq!(
        cols.len(),
        corpus.n,
        "scanned column width (stale-scan mutation flips this)"
    );
    unify_batch_into(
        c_sel,
        &cols,
        DEFAULT_CHUNK_ROWS,
        &mut ws.scratch,
        &mut ws.out_sel,
    )
    .expect("unify selector");
    assert_eq!(
        ws.out_sel.len, corpus.expected_sel,
        "selector match count vs raw-word oracle"
    );
    unify_batch_into(
        c_join,
        &cols,
        DEFAULT_CHUNK_ROWS,
        &mut ws.scratch,
        &mut ws.out_join,
    )
    .expect("unify join");
    assert_eq!(
        ws.out_join.len, corpus.expected_join,
        "join match count vs raw-word oracle"
    );
    assert_eq!(
        ws.out_join.var_values[0].len(),
        corpus.expected_join,
        "var0 column alignment"
    );
    assert_eq!(
        ws.out_join.var_values[1].len(),
        corpus.expected_join,
        "var1 column alignment"
    );
}

/// SHA-fold of a digest into a checksum (row-content stability witness;
/// the primary differential asserts are the oracle counts).
fn digest_fold(d: &[u8; 32]) -> u64 {
    let mut ck = 0x243F_6A88_85A3_08D3u64;
    for &b in d {
        ck = ck.rotate_left(8) ^ u64::from(b);
    }
    ck
}

/// Stage 4 — PROJECT: `ConsequentProjector::emit_into` per rule over the
/// match sets (zero-alloc warm path: caller-owned rows + canon buffers).
/// Mutation class: emitting one row per TWO matches flips the row-count
/// and M12-7 popcount asserts.
fn project_stage(
    corpus: &ChainCorpus,
    ws: &mut Workspace,
    c_sel: &CompiledRule,
    c_join: &CompiledRule,
) -> (u64, u64) {
    let (d_sel, pop_sel) = ws
        .projector
        .emit_into(
            c_sel,
            CHAIN_EPOCH,
            &ws.out_sel,
            &mut ws.rows_sel,
            &mut ws.canon_sel,
        )
        .expect("project selector");
    assert_eq!(
        ws.rows_sel.len(),
        corpus.expected_sel,
        "projected selector rows == matches"
    );
    assert_eq!(
        pop_sel as usize, corpus.expected_sel,
        "M12-7 popcount (selector)"
    );
    let ck_sel = digest_fold(&d_sel);
    let (d_join, pop_join) = ws
        .projector
        .emit_into(
            c_join,
            CHAIN_EPOCH,
            &ws.out_join,
            &mut ws.rows_join,
            &mut ws.canon_join,
        )
        .expect("project join");
    assert_eq!(
        ws.rows_join.len(),
        corpus.expected_join,
        "projected join rows == matches"
    );
    assert_eq!(
        pop_join as usize, corpus.expected_join,
        "M12-7 popcount (join)"
    );
    let ck_join = digest_fold(&d_join);
    black_box((ck_sel, ck_join))
}

/// THE CHAIN — full deduction pipeline per timed iteration: scan ->
/// compile -> unify -> project. Counter = statement bytes through the
/// chain (24B/row), so the table reads as line-speed deduction GB/s.
/// Mutation class: no-op the scan stage (stale columns) -> the column
/// width assert and both oracle count asserts flip; ns collapses toward
/// compile+project.
#[divan::bench(consts = [65_536, 1_048_576])]
fn deduction_chain<const N: usize>(b: Bencher) {
    let corpus = build_corpus(N);
    let mut ws = Workspace::new(&corpus);
    // Setup pass: the full chain once -> deterministic checksum
    // expectations (M12-6 determinism witness; the primary differential
    // asserts are the live oracle counts inside the stages).
    scan_stage(&corpus, &mut ws);
    let (c_sel, c_join) = compile_stage(&corpus);
    unify_stage(&corpus, &mut ws, &c_sel, &c_join);
    let (ck_sel, ck_join) = project_stage(&corpus, &mut ws, &c_sel, &c_join);
    let bytes = N * BN3_STATEMENT_SIZE;
    eprintln!(
        "[chains-bn3] chain N={N} bytes={bytes} ck_sel=0x{ck_sel:016X} ck_join=0x{ck_join:016X}"
    );
    b.counter(BytesCount::new(bytes)).bench_local(|| {
        scan_stage(&corpus, &mut ws);
        let (c_sel, c_join) = compile_stage(&corpus);
        unify_stage(&corpus, &mut ws, &c_sel, &c_join);
        let (ck2_sel, ck2_join) = project_stage(&corpus, &mut ws, &c_sel, &c_join);
        assert_eq!(
            ck2_sel, ck_sel,
            "selector digest checksum drifted (determinism)"
        );
        assert_eq!(
            ck2_join, ck_join,
            "join digest checksum drifted (determinism)"
        );
        black_box((
            ws.out_sel.len,
            ws.out_join.len,
            ws.rows_sel.len(),
            ws.rows_join.len(),
        ));
    });
}

/// Stage probe — SCAN only (same corpus, same asserts). Mutation class:
/// skipped-row / dropped-tail scan -> closure-count assert flips.
#[divan::bench(consts = [65_536, 1_048_576])]
fn stage_scan_24b<const N: usize>(b: Bencher) {
    let corpus = build_corpus(N);
    let mut ws = Workspace::new(&corpus);
    let bytes = N * BN3_STATEMENT_SIZE;
    b.counter(BytesCount::new(bytes)).bench_local(|| {
        let count = scan_stage(&corpus, &mut ws);
        black_box(count);
    });
}

/// Stage probe — COMPILE only. Mutation class: drop the join's second
/// body atom -> atoms.len() assert flips, ns moves.
#[divan::bench(consts = [65_536, 1_048_576])]
fn stage_compile_rules<const N: usize>(b: Bencher) {
    let corpus = build_corpus(N);
    b.bench_local(|| {
        let (c_sel, c_join) = compile_stage(&corpus);
        black_box((
            c_sel.rule_id,
            c_join.rule_id,
            c_sel.var_count,
            c_join.var_count,
        ));
    });
}

/// Stage probe — UNIFY only (compile is setup; columns from the prewarm
/// scan). This is the M-L3-1 BASELINE leg: the pre-L3 nested-loop join
/// probe (`radix: false`, per-chunk candidate rebuild — the
/// micro-architecture the L3 lever replaces). Mutation class:
/// binding-alias comparator / skipped join probe -> raw-word oracle
/// asserts flip.
#[divan::bench(consts = [65_536, 1_048_576])]
fn stage_unify_join<const N: usize>(b: Bencher) {
    let corpus = build_corpus(N);
    let mut ws = Workspace::new(&corpus);
    let (c_sel, c_join) = compile_stage(&corpus);
    // M-L3-1 baseline: the retired nested-loop probe, leased warm.
    let mut probe_nested = JoinProbeScratch {
        radix: false,
        ..Default::default()
    };
    // M-L3-1 arm guard: this leg measures the RETIRED nested-loop probe.
    // If Default ever flips back to radix:true (the FA3 defect class),
    // this leg would silently measure radix-vs-radix again — fail early.
    assert!(
        !probe_nested.radix,
        "nested differential leg must run radix:false"
    );
    b.bench_local(|| {
        let cols = StatementColumns::new(&ws.col_s, &ws.col_p, &ws.col_o).expect("columns");
        unify_batch_probe(
            &c_sel,
            &cols,
            DEFAULT_CHUNK_ROWS,
            &mut ws.scratch,
            &mut ws.out_sel,
            &mut probe_nested,
        )
        .expect("unify selector (nested)");
        assert_eq!(ws.out_sel.len, corpus.expected_sel, "selector oracle count");
        unify_batch_probe(
            &c_join,
            &cols,
            DEFAULT_CHUNK_ROWS,
            &mut ws.scratch,
            &mut ws.out_join,
            &mut probe_nested,
        )
        .expect("unify join (nested)");
        assert_eq!(
            ws.out_join.len, corpus.expected_join,
            "join oracle count (nested)"
        );
        black_box((ws.out_sel.len, ws.out_join.len));
    });
}

/// Stage probe — PROJECT only (compile+unify are setup). Mutation class:
/// one row per TWO matches -> row-count + M12-7 popcount asserts flip.
#[divan::bench(consts = [65_536, 1_048_576])]
fn stage_project_consequents<const N: usize>(b: Bencher) {
    let corpus = build_corpus(N);
    let mut ws = Workspace::new(&corpus);
    let (c_sel, c_join) = compile_stage(&corpus);
    unify_stage(&corpus, &mut ws, &c_sel, &c_join);
    b.bench_local(|| {
        let (ck_sel, ck_join) = project_stage(&corpus, &mut ws, &c_sel, &c_join);
        black_box((ck_sel, ck_join));
    });
}

/// Stage probe — UNIFY with the L3 radix semi-join probe (warm leased
/// [`JoinProbeScratch`]). Mutation class (M-L3-1): dropping the radix
/// probe (reverting the join to the full-candidate nested loop) moves
/// this stage's ns by the probe share — the >=10x gate vs
/// `stage_unify_join`. The oracle-count asserts ride along, so the
/// alias comparator (M-L3-2) flips them too.
#[divan::bench(consts = [65_536, 1_048_576])]
fn stage_unify_join_radix<const N: usize>(b: Bencher) {
    let corpus = build_corpus(N);
    let mut ws = Workspace::new(&corpus);
    let (c_sel, c_join) = compile_stage(&corpus);
    // FA3 radix-default flip guard: the default-leased probe must be the
    // L3 radix fast path. If Default regresses to radix:false (derived
    // default), this leg would measure nested-vs-nested again (the
    // w5-l3-remeasure defect class) — fail early, never mis-measure.
    assert!(
        ws.probe.radix,
        "radix leg must run on the default radix:true probe"
    );
    // FA3 cache-counter probe (radix-arm liveness): one untimed probe
    // eval. The candidate cache is invalidated per batch and rebuilt on
    // the join atom's first chunk (miss), then served from cache on
    // every later chunk of the SAME batch (hits) — misses=1,
    // hits=(chunks-1). This is the class FA3 pinned; it can only be
    // observed on the radix arm (`use_cache` requires probe.radix).
    {
        let cols = StatementColumns::new(&ws.col_s, &ws.col_p, &ws.col_o).expect("columns");
        unify_batch_probe(
            &c_sel,
            &cols,
            DEFAULT_CHUNK_ROWS,
            &mut ws.scratch,
            &mut ws.out_sel,
            &mut ws.probe,
        )
        .expect("unify selector (probe)");
        unify_batch_probe(
            &c_join,
            &cols,
            DEFAULT_CHUNK_ROWS,
            &mut ws.scratch,
            &mut ws.out_join,
            &mut ws.probe,
        )
        .expect("unify join (probe)");
        let chunks = N.div_ceil(DEFAULT_CHUNK_ROWS);
        assert_eq!(
            (ws.probe.cache_misses, ws.probe.cache_hits),
            (1, (chunks - 1) as u64),
            "cache-counter probe class: expect misses=1, hits=chunks-1 \
             (radix candidate cache live); got misses={} hits={}",
            ws.probe.cache_misses,
            ws.probe.cache_hits
        );
        ws.probe.cache_misses = 0;
        ws.probe.cache_hits = 0;
    }
    b.bench_local(|| {
        let cols = StatementColumns::new(&ws.col_s, &ws.col_p, &ws.col_o).expect("columns");
        unify_batch_probe(
            &c_sel,
            &cols,
            DEFAULT_CHUNK_ROWS,
            &mut ws.scratch,
            &mut ws.out_sel,
            &mut ws.probe,
        )
        .expect("unify selector (radix)");
        assert_eq!(ws.out_sel.len, corpus.expected_sel, "selector oracle count");
        unify_batch_probe(
            &c_join,
            &cols,
            DEFAULT_CHUNK_ROWS,
            &mut ws.scratch,
            &mut ws.out_join,
            &mut ws.probe,
        )
        .expect("unify join (radix)");
        assert_eq!(
            ws.out_join.len, corpus.expected_join,
            "join oracle count (radix)"
        );
        black_box((ws.out_sel.len, ws.out_join.len));
    });
}

/// ---- W4-D8: recursive-closure leg (MW4 workload bridge) ----
///
/// Transitive-closure pattern over the unify API: chain corpus, one
/// recursive rule `edge(?x ?y) & edge(?y ?z) -> edge(?x ?z)`, iterated
/// unify+project+materialize passes to fixpoint, derived pairs counted.
/// This is the bridge between cgap-bn3's statement-scan class and
/// stardf-datalog's materialization class — the MW4 comparison's second
/// measured point. Differential court: fixpoint derived-set equality vs
/// a naive iterative oracle at N=65536 (setup assert, not timed).
/// Anti-vacuity mutation class: dropping a closure iteration bound
/// (CL_MAX_PASSES -> 1 in a /tmp sandbox) exits before fixpoint and the
/// derived-count assert fires.
///
/// SEED LAW: corpus rides [`CANONICAL_SEED`] (0xBF58_476D_1CE4_E5B9)
/// only; the corpus constant 0xA20C... is deliberately absent here.

/// Edge predicate payload for the closure corpus (distinct from PA/B/C).
const CL_EDGE_PAYLOAD: u64 = 0xED9;

/// Chain length per chain; chains = N / CL_CHAIN_LEN. A chain of length
/// L closes to L*(L-1)/2 derived pairs, so the closure is closed-form:
/// at L=8, derived = 3.5N (fits a pre-sized fixed slab exactly).
const CL_CHAIN_LEN: usize = 8;

/// Closure pass bound (fixpoint needs CL_CHAIN_LEN - 1 = 7 passes).
/// Mutation class above: 1 leaves the fixpoint unreachable.
const CL_MAX_PASSES: usize = 64;

/// Naive iterative oracle (pure std, zero cgap-bn3 code): relational
/// join loop to fixpoint over the known edge set. Differential check —
/// shares no code with the unify path.
fn closure_oracle(seed: &[(u64, u64)]) -> std::collections::HashSet<(u64, u64)> {
    let mut known: std::collections::HashSet<(u64, u64)> = seed.iter().copied().collect();
    loop {
        let mut new = Vec::new();
        let snapshot: Vec<(u64, u64)> = known.iter().copied().collect();
        for &(a, b) in &snapshot {
            for &(c, d) in &snapshot {
                if c == b && known.insert((a, d)) {
                    new.push((a, d));
                }
            }
        }
        if new.is_empty() {
            return known;
        }
    }
}

/// Closure corpus: seed chain edges + the recursive rule + the oracle
/// expectation, all setup-computed.
struct ClosureCorpus {
    /// Seed facts only (the pre-closure state, never mutated).
    seed_slab: Slab,
    /// Fixed total capacity: seed + expected derived (closed-form closure).
    cap: usize,
    compiled: CompiledRule,
    expected_seed: usize,
    expected_derived: usize,
}

fn build_closure_corpus(n: usize) -> ClosureCorpus {
    let chains = n / CL_CHAIN_LEN;
    const MIN_CHAINS: usize = 0;
    assert!(chains > MIN_CHAINS, "N too small for one chain");
    let edge = const_word(CL_EDGE_PAYLOAD);
    let per_chain = CL_CHAIN_LEN * (CL_CHAIN_LEN - 1) / 2 - (CL_CHAIN_LEN - 1);
    let expected_derived = chains * per_chain;
    // Chains are DISJOINT: node ids are contiguous but the boundary edge
    // of each chain (pos == CL-1 -> next chain) is dropped, so chain k
    // spans ids [k*CL, (k+1)*CL) in isolation and the closed-form
    // expectation `chains * per_chain` holds exactly. (MB3: the old
    // builder emitted the boundary edge i -> i+1 for every i, merging
    // all chains into one length-n chain whose closure demands
    // n(n-1)/2 pairs — the fixed work slab then refused typed with
    // ArenaCapacityExceeded mid-fixpoint.)
    let seeds = n - chains;
    let cap = n + expected_derived;
    let mut slab = Slab::fixed(cap as u64).expect("closure slab fixed");
    for i in 0..n {
        if i % CL_CHAIN_LEN == CL_CHAIN_LEN - 1 {
            continue;
        }
        let s = const_word(i as u64);
        let o = const_word(i as u64 + 1);
        slab.push(BN3Statement::from_words(s, edge, o))
            .expect("seed push");
    }
    let rule = Rule {
        rule_id: 0xC0DE_0003,
        body: vec![
            BodyAtom::new(var_word(0), edge, var_word(1)),
            BodyAtom::new(var_word(1), edge, var_word(2)),
        ],
        head: [var_word(0), edge, var_word(2)],
    };
    let compiled = RuleCompiler::compile(&rule).expect("closure rule compiles");
    assert_eq!(compiled.atoms.len(), 2, "closure body has two atoms");
    assert_eq!(compiled.var_count, 3, "closure binds three variables");
    ClosureCorpus {
        seed_slab: slab,
        cap,
        compiled,
        expected_seed: seeds,
        expected_derived,
    }
}

/// Warm closure workspace: columns + scratch + match set + delta rows +
/// canon buffer, reused across passes and iterations (zero-alloc hot path).
struct ClosureWs {
    col_s: Vec<u64>,
    col_p: Vec<u64>,
    col_o: Vec<u64>,
    scratch: EvalScratch,
    out: MatchSet,
    rows: DeltaRows,
    canon: Vec<u8>,
    projector: ConsequentProjector,
    /// Dedup set over (s, o) of known facts (p is constant).
    known: std::collections::HashSet<(u64, u64)>,
    /// Work slab rebuilt from the seed slab each iteration.
    work: Slab,
}

impl ClosureWs {
    fn new(corpus: &ClosureCorpus) -> Self {
        ClosureWs {
            col_s: Vec::with_capacity(corpus.cap),
            col_p: Vec::with_capacity(corpus.cap),
            col_o: Vec::with_capacity(corpus.cap),
            scratch: EvalScratch::default(),
            out: MatchSet::default(),
            rows: DeltaRows::default(),
            canon: Vec::new(),
            projector: ConsequentProjector::new(),
            known: std::collections::HashSet::new(),
            work: Slab::fixed(corpus.cap as u64).expect("work slab"),
        }
    }

    /// Reset to the pre-closure seed state (per timed iteration).
    fn reset(&mut self, corpus: &ClosureCorpus) {
        self.work = Slab::fixed(corpus.cap as u64).expect("work slab");
        for &st in corpus.seed_slab.statements() {
            self.work.push(st).expect("work push");
        }
        self.known.clear();
        for st in self.work.statements() {
            self.known.insert((st.s, st.o));
        }
        assert_eq!(self.known.len(), corpus.expected_seed, "seed set size");
    }

    /// One closure pass: scan -> unify -> project -> materialize (push
    /// only NEW facts). Returns new-fact count.
    fn pass(&mut self, corpus: &ClosureCorpus) -> usize {
        self.col_s.clear();
        self.col_p.clear();
        self.col_o.clear();
        for st in self.work.statements() {
            self.col_s.push(st.s);
            self.col_p.push(st.p);
            self.col_o.push(st.o);
        }
        assert_eq!(self.col_s.len(), self.work.len(), "pass scan width");
        let cols = StatementColumns::new(&self.col_s, &self.col_p, &self.col_o).expect("columns");
        unify_batch_into(
            &corpus.compiled,
            &cols,
            DEFAULT_CHUNK_ROWS,
            &mut self.scratch,
            &mut self.out,
        )
        .expect("closure unify");
        let (_digest, pop) = self
            .projector
            .emit_into(
                &corpus.compiled,
                CHAIN_EPOCH,
                &self.out,
                &mut self.rows,
                &mut self.canon,
            )
            .expect("closure project");
        assert_eq!(self.rows.len(), self.out.len, "projected rows == matches");
        assert_eq!(pop as usize, self.out.len, "M12-7 popcount (closure)");
        let mut new = 0usize;
        for i in 0..self.rows.len() {
            let (s, p, o, _) = self.rows.row(i);
            if !self.known.insert((s, o)) {
                continue;
            }
            self.work
                .push(BN3Statement::from_words(s, p, o))
                .expect("derived push");
            new += 1;
        }
        new
    }

    /// Full fixpoint run: passes until no new facts. Asserts the final
    /// state (Chicago): derived count vs the closed-form oracle count.
    /// Mutation class: CL_MAX_PASSES drop -> this assert fires.
    fn run_to_fixpoint(&mut self, corpus: &ClosureCorpus) -> usize {
        self.reset(corpus);
        let mut passes = 0usize;
        loop {
            let new = self.pass(corpus);
            passes += 1;
            if new == 0 {
                break;
            }
            assert!(passes <= CL_MAX_PASSES, "closure pass bound blown");
        }
        let total = self.work.len() - corpus.expected_seed;
        assert_eq!(
            total, corpus.expected_derived,
            "closure fixpoint derived count vs closed-form oracle \
             (iteration-bound mutation flips this)"
        );
        assert_eq!(
            self.known.len(),
            corpus.expected_seed + corpus.expected_derived,
            "set size at fixpoint"
        );
        total
    }
}

/// THE LADDER — closure to fixpoint per timed iteration, counter =
/// derived pairs (pairs/s at fixpoint). Mutation class: see
/// [`ClosureWs::run_to_fixpoint`].
#[divan::bench(consts = [65_536, 262_144, 1_048_576])]
fn closure_ladder<const N: usize>(b: Bencher) {
    let corpus = build_closure_corpus(N);
    let mut ws = ClosureWs::new(&corpus);
    // Setup: one fixpoint run + the differential court at the smallest
    // N (naive oracle equality; NOT timed).
    let derived = ws.run_to_fixpoint(&corpus);
    if N == 65_536 {
        let seed: Vec<(u64, u64)> = corpus
            .seed_slab
            .statements()
            .iter()
            .map(|st| (st.s, st.o))
            .collect();
        let oracle = closure_oracle(&seed);
        let bench_set: std::collections::HashSet<(u64, u64)> = ws.known.iter().copied().collect();
        assert_eq!(
            bench_set, oracle,
            "differential court: fixpoint derived-set equality vs naive oracle"
        );
        assert!(
            oracle.len() > corpus.expected_seed,
            "oracle adds facts (non-vacuous)"
        );
    }
    eprintln!(
        "[chains-bn3] closure N={N} derived={derived} cap={}",
        corpus.cap
    );
    b.counter(ItemsCount::new(derived)).bench_local(|| {
        let d = ws.run_to_fixpoint(&corpus);
        assert_eq!(d, derived, "derived count drifted (determinism)");
        black_box((d, ws.work.len()));
    });
}

fn main() {
    divan::main();
}
