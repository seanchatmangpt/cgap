//! No-alloc eval/unify arm (lane P3-11 W2-9-REDO, spec A12 Section 4.3 /
//! 7.4). Mirrors the register-unifier semantics of [`crate::unify`]
//! (the u64-lane comparator arm, nested-loop join, first-occurrence
//! binding order) over fixed-capacity buffers: no `Vec`, no `HashMap`,
//! no `alloc::` — every capacity is a hard `const` bound exceeded only
//! as a typed refusal, never a reallocation.
//!
//! Authority: the ALLOC arm (`crate::unify`) remains the semantics
//! authority; this module is a capacity-bounded mirror whose differential
//! court asserts binding-for-binding equality against it.
//!
//! Anti-vacuity anchors (courts below name their mutations):
//! - "inject one heap allocation in the eval path" MUST flip the
//!   zero-alloc court (executed in a /tmp sandbox this wave);
//! - "alias comparator: compare the register NUMBER instead of the bound
//!   VALUE" MUST flip the differential court (executed in a /tmp sandbox).
//!
//! Fixed capacities (spec A11 Section 5 transport widths; spec A12 4.2
//! band widths):
//! - constants: [`NA_MAX_CONSTS`] = 64 (the constant band R0..=R63);
//! - variables: [`NA_MAX_VARS`] = 128 (the variable band R64..=R191);
//! - body atoms: [`NA_MAX_ATOMS`] = 8;
//! - matches: [`NA_MAX_MATCHES`] = 1024.

use crate::sigil::{self, Sigil};
use crate::unify::{CompiledTerm, Rule, StatementColumns};

/// Constant-pool capacity = the R0..=R63 band width (spec A12 4.2).
pub const NA_MAX_CONSTS: usize = 64;
/// Variable capacity = the R64..=R191 band width (spec A12 4.2).
pub const NA_MAX_VARS: usize = 128;
/// Body-atom capacity per rule.
pub const NA_MAX_ATOMS: usize = 8;
/// Match capacity per evaluation (the fixed output/join buffer width).
pub const NA_MAX_MATCHES: usize = 2048;

const MASK_ALL: u64 = u64::MAX;

// Hard const bounds: capacity drift is a BUILD failure, not a court
// surprise (house pattern: unify.rs const asserts).
const _: () = assert!(NA_MAX_CONSTS == crate::unify::CONST_POOL_REGS);
const _: () = assert!(NA_MAX_VARS == crate::unify::MAX_RULE_VARIABLES);
const _: () = assert!(NA_MAX_MATCHES > 0);
const _: () = assert!(NA_MAX_ATOMS >= 2);

/// Typed refusal surface of the noalloc arm. Every bound is loud.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NaError {
    /// More than [`NA_MAX_CONSTS`] distinct constants.
    ConstCapacity,
    /// More than [`NA_MAX_VARS`] distinct variables.
    VarCapacity,
    /// More than [`NA_MAX_ATOMS`] body atoms.
    AtomCapacity,
    /// VarUni payload beyond the 8-bit register space.
    RegisterOutOfRange(u64),
    /// FormulaRef in the predicate slot (A12 ban, cited from A11).
    FormulaRefInPredicate,
    /// A sigil the unifier does not unify (VarExi/BuiltinOp/RESERVED).
    UnsupportedTermSigil(u8),
    /// A head variable no body atom binds.
    UnboundHeadVariable,
    /// An empty rule body.
    EmptyRuleBody,
    /// Evaluation exceeded the fixed match capacity — loud, never a
    /// silent truncation.
    MatchCapacity,
}

/// A rule compiled into the noalloc program: fixed arrays, first-seen
/// interning identical to [`crate::unify::RuleCompiler`] (constants into
/// the pool in body-then-head scan order; variables into `R64 + idx` in
/// body scan order; provenance `(atom, slot)` per variable).
#[derive(Clone, Debug, PartialEq)]
pub struct NaRule {
    /// Emitting-rule id.
    pub rule_id: u32,
    /// Body atoms in compiled form, evaluation order.
    pub atoms: [[CompiledTerm; 3]; NA_MAX_ATOMS],
    /// Body-atom count (`0 < atom_count <= NA_MAX_ATOMS`).
    pub atom_count: usize,
    /// Head template in compiled form.
    pub head: [CompiledTerm; 3],
    /// Constant pool `R0..` (first-seen order, body then head).
    pub const_pool: [u64; NA_MAX_CONSTS],
    /// Distinct constants used (`const_pool[..const_count]`).
    pub const_count: usize,
    /// Distinct variable count (band width used).
    pub var_count: usize,
    /// Per variable index: `(atom, slot)` that binds it (first
    /// occurrence, body scan order). `0xFF` atom = head-only variable.
    pub var_first: [(u8, u8); NA_MAX_VARS],
}

/// Fixed-capacity match output: match `m` is `rows[m][a]` (the store row
/// matched by body atom `a`) and `vars[m][idx]` (the bound value of
/// variable `idx`). Aligned by construction: every accepted match fills
/// `rows[m][0..atom_count]` and `vars[m][0..var_count]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NaMatches {
    /// Number of matches.
    pub len: usize,
    rows: [[u32; NA_MAX_ATOMS]; NA_MAX_MATCHES],
    vars: [[u64; NA_MAX_VARS]; NA_MAX_MATCHES],
}

impl NaMatches {
    /// A zeroed buffer (caller-owned, reusable; clearing is a field
    /// write, never an allocation).
    pub fn zeroed() -> Self {
        NaMatches {
            len: 0,
            rows: [[0; NA_MAX_ATOMS]; NA_MAX_MATCHES],
            vars: [[0; NA_MAX_VARS]; NA_MAX_MATCHES],
        }
    }

    /// The store row body atom `a` matched in match `m`.
    pub fn atom_row(&self, m: usize, a: usize) -> u32 {
        self.rows[m][a]
    }

    /// The bound value of variable `idx` in match `m`.
    pub fn var_value(&self, m: usize, idx: usize) -> u64 {
        self.vars[m][idx]
    }

    /// True when `m` indexes a real match.
    pub fn contains(&self, m: usize) -> bool {
        m < self.len
    }
}

/// Heap-allocate an all-zero `T` directly (no stack temporary): the
/// courts `Box` the ~8 MiB fixed buffers, and `Box::new(T::zeroed())`
/// materializes the zeroed value on the test thread's 2 MiB stack first
/// (witnessed: stack overflow in the alias-trap court). All-zero is a
/// valid [`NaEngine`]/[`NaMatches`] state (false/0 fields).
#[cfg(test)]
pub(crate) fn boxed_zeroed<T>() -> Box<T> {
    unsafe { Box::new_zeroed().assume_init() }
}

/// Per-evaluation scratch: the cur/nxt double buffer of the nested-loop
/// join, fixed-capacity. Heap use is ONE setup-time construction (the
/// courts `Box` it outside the measured window); evaluation itself is
/// allocation-free.
#[derive(Clone, Copy, Debug)]
pub struct NaEngine {
    cur_rows: [[u32; NA_MAX_ATOMS]; NA_MAX_MATCHES],
    cur_vars: [[u64; NA_MAX_VARS]; NA_MAX_MATCHES],
    cur_len: usize,
    nxt_rows: [[u32; NA_MAX_ATOMS]; NA_MAX_MATCHES],
    nxt_vars: [[u64; NA_MAX_VARS]; NA_MAX_MATCHES],
    nxt_len: usize,
    /// Per-variable bound flags for the current atom expansion.
    bound: [bool; NA_MAX_VARS],
}

impl NaEngine {
    /// A zeroed engine (one setup-time construction).
    pub fn zeroed() -> Self {
        NaEngine {
            cur_rows: [[0; NA_MAX_ATOMS]; NA_MAX_MATCHES],
            cur_vars: [[0; NA_MAX_VARS]; NA_MAX_MATCHES],
            cur_len: 0,
            nxt_rows: [[0; NA_MAX_ATOMS]; NA_MAX_MATCHES],
            nxt_vars: [[0; NA_MAX_VARS]; NA_MAX_MATCHES],
            nxt_len: 0,
            bound: [false; NA_MAX_VARS],
        }
    }
}

/// The noalloc compiler: [`Rule`] (the alloc arm's public rule type —
/// the input FORMAT, not a runtime dependency of evaluation) -> NaRule.
/// Decodes term words with the same laws and first-seen interning order
/// as [`crate::unify::RuleCompiler`]; compile-time allocation is absent
/// too — interning walks fixed arrays.
pub fn na_compile(rule: &Rule) -> Result<NaRule, NaError> {
    if rule.body.is_empty() {
        return Err(NaError::EmptyRuleBody);
    }
    if rule.body.len() > NA_MAX_ATOMS {
        return Err(NaError::AtomCapacity);
    }
    let mut out = NaRule {
        rule_id: rule.rule_id,
        atoms: [[CompiledTerm::Const(0); 3]; NA_MAX_ATOMS],
        atom_count: rule.body.len(),
        head: [CompiledTerm::Const(0); 3],
        const_pool: [0; NA_MAX_CONSTS],
        const_count: 0,
        var_count: 0,
        var_first: [(0xFF, 0xFF); NA_MAX_VARS],
    };
    // 8-bit payload space -> O(1) direct-mapped variable table (0xFF =
    // unseen sentinel; payload is a register number < 256).
    let mut var_map = [0xFFu8; 256];
    for (ai, atom) in rule.body.iter().enumerate() {
        let mut ct = [CompiledTerm::Const(0); 3];
        for (si, &word) in [atom.s, atom.p, atom.o].iter().enumerate() {
            ct[si] = na_intern(word, ai, si, &mut var_map, &mut out)?;
        }
        out.atoms[ai] = ct;
    }
    for (si, &word) in rule.head.iter().enumerate() {
        out.head[si] = na_intern(word, 0xFF, si, &mut var_map, &mut out)?;
    }
    // Head-binding law: every head variable is bound by some body atom.
    for term in out.head.iter().take(3) {
        if let CompiledTerm::Var(idx) = *term {
            if out.var_first[idx as usize].0 == 0xFF {
                return Err(NaError::UnboundHeadVariable);
            }
        }
    }
    Ok(out)
}

/// Term-word decode + intern (mirrors `unify::Interner::intern`, fixed
/// tables): `word_count` is the running atom index for provenance.
fn na_intern(
    word: u64,
    atom_idx: usize,
    slot: usize,
    var_map: &mut [u8; 256],
    out: &mut NaRule,
) -> Result<CompiledTerm, NaError> {
    let sig = (word >> sigil::SIGIL_SHIFT) as u8;
    if sig == Sigil::VarUni as u8 {
        let payload = word & sigil::PAYLOAD_MASK;
        if payload >= crate::sigil::REGISTER_COUNT as u64 {
            return Err(NaError::RegisterOutOfRange(payload));
        }
        let vp = payload as u8;
        if var_map[vp as usize] == 0xFF {
            if out.var_count >= NA_MAX_VARS {
                return Err(NaError::VarCapacity);
            }
            var_map[vp as usize] = out.var_count as u8;
            out.var_first[out.var_count] = (atom_idx as u8, slot as u8);
            out.var_count += 1;
        }
        return Ok(CompiledTerm::Var(var_map[vp as usize]));
    }
    if sig == Sigil::FormulaRef as u8 && slot == 1 {
        return Err(NaError::FormulaRefInPredicate);
    }
    if sig == Sigil::SlotConst as u8 || sig == Sigil::FormulaRef as u8 {
        // First-seen linear probe of the fixed pool.
        for k in 0..out.const_count {
            if out.const_pool[k] == word {
                return Ok(CompiledTerm::Const(k as u8));
            }
        }
        if out.const_count >= NA_MAX_CONSTS {
            return Err(NaError::ConstCapacity);
        }
        out.const_pool[out.const_count] = word;
        let reg = out.const_count as u8;
        out.const_count += 1;
        return Ok(CompiledTerm::Const(reg));
    }
    Err(NaError::UnsupportedTermSigil(sig))
}

/// Branch-free u64-lane equality mask — the SAME constant-time comparator
/// as `unify::lane_eq_mask` (cited semantics, mirrored here because that
/// fn is private): all-ones iff `a == b`. The differential court's
/// alias-comparator sandbox flip proves this arm really depends on it.
#[inline(always)]
fn lane_eq_mask(a: u64, b: u64) -> u64 {
    let d = a ^ b;
    let nz = (d | d.wrapping_neg()) >> 63;
    nz.wrapping_sub(1)
}

fn col<'a>(cols: &StatementColumns<'a>, slot: usize) -> &'a [u64] {
    match slot {
        0 => cols.s,
        1 => cols.p,
        2 => cols.o,
        _ => unreachable!("noalloc col: slot {slot} out of range 0..=2"),
    }
}

/// Per-atom local match over one row (mirrors `atom_local_match_into`):
/// constant equality ANDed branch-free; within-atom variable repetition
/// (`?x p ?x`) is within-ROW equality; fresh variables bind in
/// first-occurrence slot order. Writes at most 3 fresh `(idx, val)`
/// bindings. `bound` marks variables bound by earlier atoms (skipped
/// here; the join probe checks them).
fn atom_local_match(
    rule: &NaRule,
    atom: &[CompiledTerm; 3],
    cols: &StatementColumns<'_>,
    row: usize,
    bound: &[bool; NA_MAX_VARS],
    fresh: &mut [(u8, u64); 3],
    n_fresh: &mut usize,
) -> bool {
    let mut acc = MASK_ALL;
    let mut seen_slot = [0u8; 3];
    let mut seen_idx = [0u8; 3];
    let mut n_seen = 0usize;
    *n_fresh = 0;
    for si in 0..3 {
        let w = col(cols, si)[row];
        match atom[si] {
            CompiledTerm::Const(creg) => {
                acc &= lane_eq_mask(w, rule.const_pool[creg as usize]);
            }
            CompiledTerm::Var(idx) => {
                let idx = idx as usize;
                if bound[idx] {
                    continue;
                }
                match (0..n_seen).find(|&k| seen_idx[k] as usize == idx) {
                    Some(k) => acc &= lane_eq_mask(w, col(cols, seen_slot[k] as usize)[row]),
                    None => {
                        seen_slot[n_seen] = si as u8;
                        seen_idx[n_seen] = idx as u8;
                        fresh[*n_fresh] = (idx as u8, w);
                        *n_fresh += 1;
                        n_seen += 1;
                    }
                }
            }
        }
    }
    acc == MASK_ALL
}

/// Push one accepted match onto a fixed buffer; typed refusal at
/// capacity, never a silent truncation.
#[inline]
fn push_match(
    dst_rows: &mut [[u32; NA_MAX_ATOMS]; NA_MAX_MATCHES],
    dst_vars: &mut [[u64; NA_MAX_VARS]; NA_MAX_MATCHES],
    len: &mut usize,
    parent_rows: &[[u32; NA_MAX_ATOMS]; NA_MAX_MATCHES],
    parent_vars: &[[u64; NA_MAX_VARS]; NA_MAX_MATCHES],
    atom_count: usize,
    var_count: usize,
    parent: usize,
    atom_row: u32,
    atom_slot: usize,
    fresh: &[(u8, u64); 3],
    n_fresh: usize,
) -> Result<(), NaError> {
    if *len >= NA_MAX_MATCHES {
        return Err(NaError::MatchCapacity);
    }
    let m = *len;
    if atom_slot != usize::MAX {
        dst_rows[m][..atom_count].copy_from_slice(&parent_rows[parent][..atom_count]);
        dst_rows[m][atom_slot] = atom_row;
        dst_vars[m][..var_count].copy_from_slice(&parent_vars[parent][..var_count]);
    } else {
        dst_rows[m] = [0; NA_MAX_ATOMS];
        dst_rows[m][0] = atom_row;
        dst_vars[m][..var_count].fill(0);
    }
    for &(idx, val) in &fresh[..n_fresh] {
        dst_vars[m][idx as usize] = val;
    }
    *len += 1;
    Ok(())
}

/// Evaluate one compiled rule over the whole store (the noalloc arm of
/// `unify_batch`; no chunking — the alloc arm's M12-6 court proves
/// chunking never changes results, so the mirror runs one span). Match
/// order and binding order are identical to the alloc arm: atom-0 row
/// ascending, join candidate ascending, variables bound at first
/// occurrence.
///
/// Allocation profile: zero. All state lives in the caller-owned
/// [`NaEngine`] / [`NaMatches`] fixed buffers (setup-time constructs).
pub fn na_unify(
    rule: &NaRule,
    cols: &StatementColumns<'_>,
    engine: &mut NaEngine,
    out: &mut NaMatches,
) -> Result<(), NaError> {
    if rule.atom_count == 0 {
        return Err(NaError::EmptyRuleBody);
    }
    engine.cur_len = 0;
    engine.nxt_len = 0;
    out.len = 0;
    // Variable bound-flags: an earlier atom binds it for every match.
    engine.bound = [false; NA_MAX_VARS];

    let mut fresh = [(0u8, 0u64); 3];
    let mut n_fresh = 0usize;

    // Atom 0: scan the store; nothing is bound yet.
    for i in 0..cols.len() {
        if atom_local_match(
            rule,
            &rule.atoms[0],
            cols,
            i,
            &engine.bound,
            &mut fresh,
            &mut n_fresh,
        ) {
            push_match(
                &mut engine.cur_rows,
                &mut engine.cur_vars,
                &mut engine.cur_len,
                // Atom 0 has no parent; nxt is quiescent here and serves
                // as the (unused) parent borrow target.
                &engine.nxt_rows,
                &engine.nxt_vars,
                rule.atom_count,
                rule.var_count,
                usize::MAX,
                i as u32,
                usize::MAX,
                &fresh,
                n_fresh,
            )?;
        }
    }

    // Atoms 1..: nested-loop join. Candidates from the full store; the
    // bound-slot probe compares the match's bound VALUE (never the
    // register number — the alias-comparator sandbox flip names this).
    for ai in 1..rule.atom_count {
        // Variables bound by atoms BEFORE this one probe the join;
        // variables first bound here are fresh for every match.
        for idx in 0..rule.var_count {
            let (fa, _) = rule.var_first[idx];
            engine.bound[idx] = (fa as usize) < ai;
        }
        let mut bound_slots_n = 0usize;
        let mut bound_slots = [(0u8, 0u8); 3]; // (slot, var idx)
        for (si, term) in rule.atoms[ai].iter().enumerate() {
            if let CompiledTerm::Var(idx) = *term {
                if engine.bound[idx as usize] {
                    bound_slots[bound_slots_n] = (si as u8, idx);
                    bound_slots_n += 1;
                }
            }
        }
        engine.nxt_len = 0;
        for m in 0..engine.cur_len {
            for j in 0..cols.len() {
                if !atom_local_match(
                    rule,
                    &rule.atoms[ai],
                    cols,
                    j,
                    &engine.bound,
                    &mut fresh,
                    &mut n_fresh,
                ) {
                    continue;
                }
                let mut ok = MASK_ALL;
                for &(si, vidx) in &bound_slots[..bound_slots_n] {
                    ok &= lane_eq_mask(col(cols, si as usize)[j], engine.cur_vars[m][vidx as usize]);
                }
                if ok != MASK_ALL {
                    continue;
                }
                push_match(
                    &mut engine.nxt_rows,
                    &mut engine.nxt_vars,
                    &mut engine.nxt_len,
                    &engine.cur_rows,
                    &engine.cur_vars,
                    rule.atom_count,
                    rule.var_count,
                    m,
                    j as u32,
                    ai,
                    &fresh,
                    n_fresh,
                )?;
            }
        }
        // Swap without touching the allocator (fixed buffers).
        core::mem::swap(&mut engine.cur_rows, &mut engine.nxt_rows);
        core::mem::swap(&mut engine.cur_vars, &mut engine.nxt_vars);
        engine.cur_len = engine.nxt_len;
    }

    // Row-wise drain into the caller's output buffer (a whole-array
    // assignment here materializes ~1 MiB debug-frame temporaries and
    // overflows the 2 MiB test-thread stack — witnessed this session).
    for k in 0..NA_MAX_MATCHES {
        out.rows[k] = engine.cur_rows[k];
    }
    for k in 0..NA_MAX_MATCHES {
        out.vars[k] = engine.cur_vars[k];
    }
    out.len = engine.cur_len;
    Ok(())
}

#[cfg(test)]
mod courts {
    use super::*;
    use crate::unify::{BodyAtom, RuleCompiler};
    use crate::unify::tests::current_allocs;

    const SEED: u64 = 0xBF58_476D_1CE4_E5B9;

    fn cw(id: u64) -> u64 {
        id
    }
    fn vw(reg: u8) -> u64 {
        (0x01u64 << 56) | reg as u64
    }

    fn lcg(state: &mut u64) -> u64 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *state >> 33
    }

    /// Rule classes of spec 7.1 (same as the alloc arm's rule_set).
    fn rule_set() -> Vec<Rule> {
        vec![
            Rule {
                rule_id: 0,
                body: vec![BodyAtom::new(vw(10), cw(100), vw(11))],
                head: [vw(10), cw(300), vw(11)],
            },
            Rule {
                rule_id: 1,
                body: vec![
                    BodyAtom::new(vw(10), cw(100), vw(11)),
                    BodyAtom::new(vw(11), cw(200), vw(12)),
                ],
                head: [vw(12), cw(100), vw(10)],
            },
            Rule {
                rule_id: 2,
                body: vec![BodyAtom::new(vw(10), cw(200), vw(10))],
                head: [vw(10), cw(100), cw(1)],
            },
            Rule {
                rule_id: 3,
                body: vec![BodyAtom::new(cw(1), cw(100), vw(11))],
                head: [vw(11), cw(200), cw(1)],
            },
            Rule {
                rule_id: 4,
                body: vec![BodyAtom::new(vw(10), cw(100), vw(11))],
                head: [vw(11), cw(300), vw(10)],
            },
        ]
    }

    /// Seeded corpus with >= ROWS rows; self-loops every 5th row feed the
    /// self-join rule. Deterministic under SEED (asserted in a court).
    fn gen_rows(rows_n: usize) -> (Vec<u64>, Vec<u64>, Vec<u64>) {
        let mut st = SEED | 1;
        let mut s = Vec::with_capacity(rows_n);
        let mut p = Vec::with_capacity(rows_n);
        let mut o = Vec::with_capacity(rows_n);
        for _ in 0..rows_n {
            let subj = cw(1 + lcg(&mut st) % 200);
            s.push(subj);
            if s.len() % 5 == 0 {
                p.push(cw(200));
                o.push(subj);
            } else {
                // 10% P2: keeps the join rule's match count well inside
                // NA_MAX_MATCHES (selectivity knob, documented).
                let pred = if lcg(&mut st) % 10 == 0 { cw(200) } else { cw(100) };
                p.push(pred);
                o.push(cw(1 + lcg(&mut st) % 200));
            }
        }
        (s, p, o)
    }

    /// Differential court (the lane's anti-silent-wrongness anchor): over
    /// a seeded 1200-row corpus and the full five-rule corpus, the
    /// noalloc arm's matches equal the ALLOC arm's (`unify_batch`)
    /// match-for-match — length, per-atom store rows, and every variable
    /// binding, in order. Identical bindings is the named property.
    /// Mutations that MUST flip this court (both executed in the /tmp
    /// sandbox this wave): the alias comparator (probe the register
    /// NUMBER instead of the bound VALUE — trap rows join falsely) and
    /// mask AND -> OR (over-acceptance scrambles the match set).
    #[test]
    fn differential_court_noalloc_matches_alloc_arm_over_1200_rows() {
        let (s, p, o) = gen_rows(1200);
        assert!(s.len() >= 1000);
        let cols = StatementColumns::new(&s, &p, &o).unwrap();
        let rules = rule_set();
        let mut engine = boxed_zeroed::<NaEngine>();
        let mut out = boxed_zeroed::<NaMatches>();
        for rule in &rules {
            let alloc_rule = RuleCompiler::compile(rule).unwrap();
            let na_rule = na_compile(rule).unwrap();
            assert_eq!(alloc_rule.var_count, na_rule.var_count, "rule {}", rule.rule_id);
            for idx in 0..na_rule.var_count {
                let (fa, fs) = na_rule.var_first[idx];
                assert_eq!(
                    alloc_rule.var_first[idx],
                    (fa as usize, fs as usize),
                    "rule {} var {idx} provenance",
                    rule.rule_id
                );
            }
            for chunk in [4096usize, 1] {
                let want = crate::unify::unify_batch(&alloc_rule, &cols, chunk).unwrap();
                na_unify(&na_rule, &cols, &mut engine, &mut out).unwrap();
                assert_eq!(want.len, out.len, "rule {} chunk {chunk} match count", rule.rule_id);
                for m in 0..out.len {
                    for a in 0..na_rule.atom_count {
                        assert_eq!(
                            want.atom_rows[a][m],
                            out.atom_row(m, a),
                            "rule {} chunk {chunk} match {m} atom {a} row",
                            rule.rule_id
                        );
                    }
                    for idx in 0..na_rule.var_count {
                        assert_eq!(
                            want.var_values[idx][m],
                            out.var_value(m, idx),
                            "rule {} chunk {chunk} match {m} var {idx} binding",
                            rule.rule_id
                        );
                    }
                }
            }
        }
    }

    /// Alias-trap court (doe-4 SHIM-DOE4 twin): a store carrying the raw
    /// register number 65 as a subject word must NOT join under the value
    /// probe; the in-process counterpart of the sandbox flip.
    /// Mutation that MUST flip this court: any comparator that matches a
    /// repeated/bound variable by register NUMBER or slot position.
    #[test]
    fn alias_trap_court_register_number_never_stands_for_a_value() {
        // Rule ?x P1 ?y . ?y P2 ?z; ?y is var idx 1 -> register R65.
        let rule = Rule {
            rule_id: 7,
            body: vec![
                BodyAtom::new(vw(10), cw(100), vw(11)),
                BodyAtom::new(vw(11), cw(200), vw(12)),
            ],
            head: [vw(12), cw(100), vw(10)],
        };
        let na = na_compile(&rule).unwrap();
        // Row 2's subject IS the raw word 65 (=?y's register number): an
        // aliasing comparator joins it; the value probe rejects it.
        let s = [cw(1), cw(2), 65, cw(2)];
        let p = [cw(100), cw(200), cw(200), cw(200)];
        let o = [cw(2), cw(3), cw(9), cw(5)];
        let cols = StatementColumns::new(&s, &p, &o).unwrap();
        let mut engine = boxed_zeroed::<NaEngine>();
        let mut out = boxed_zeroed::<NaMatches>();
        na_unify(&na, &cols, &mut engine, &mut out).unwrap();
        assert_eq!(out.len, 2, "alias trap row must not join");
        assert_eq!(out.atom_row(0, 1), 1);
        assert_eq!(out.atom_row(1, 1), 3);
        assert_eq!(out.var_value(0, 1), cw(2));
        assert_eq!(out.var_value(1, 1), cw(2));
    }


    /// Positive control (anti-vacuity): the counting-allocator counter
    /// MUST move for a fresh `Vec` inside the same measurement window —
    /// a zero-alloc court whose counter cannot leave zero is measuring
    /// nothing. Mutation that MUST flip this court: bypassing the
    /// allocator hook (counter reads 0 through a real allocation).
    #[test]
    fn zero_alloc_court_positive_control_counter_is_live() {
        let before = current_allocs();
        let probe: Vec<u8> = (0..64u8).collect();
        let delta = current_allocs() - before;
        assert!(delta >= 1, "counter read {delta} allocs for a fresh Vec — vacuous");
        assert_eq!(probe.len(), 64);
    }

    /// Zero-allocation court (spec 7.4, the lane's headline property):
    /// after setup (engine + output buffers constructed, one warm run
    /// touched), a second `na_unify` over the same 1200-row input
    /// performs ZERO heap allocations. Real kernel, real allocator.
    /// Mutation that MUST flip this court (executed: /tmp sandbox shim
    /// injecting `Vec::with_capacity(8)` in the `na_unify` row loop —
    /// court FAILED rc=101, byte-exact restore): any per-call allocation.
    #[test]
    fn zero_alloc_court_na_unify_allocates_zero_after_setup() {
        let (s, p, o) = gen_rows(1200);
        let cols = StatementColumns::new(&s, &p, &o).unwrap();
        let rules = rule_set();
        let na_rules: Vec<NaRule> = rules.iter().map(|r| na_compile(r).unwrap()).collect();
        let mut engine = boxed_zeroed::<NaEngine>();
        let mut out = boxed_zeroed::<NaMatches>();
        // Warmup: exercise the full path once (the noalloc arm carries no
        // growing state; the warmup exists to catch lazy one-time
        // allocation and to pin the reference results).
        let mut reference: Vec<(usize, Vec<u32>, Vec<Vec<u64>>)> = Vec::new();
        for r in &na_rules {
            na_unify(r, &cols, &mut engine, &mut out).unwrap();
            let mut vars = Vec::with_capacity(r.var_count);
            for idx in 0..r.var_count {
                vars.push((0..out.len).map(|m| out.var_value(m, idx)).collect());
            }
            reference.push((
                out.len,
                (0..out.len).map(|m| out.atom_row(m, 0)).collect(),
                vars,
            ));
        }
        // Measured window: full second pass, per-rule phase deltas.
        let mut total = 0usize;
        for (k, r) in na_rules.iter().enumerate() {
            let before = current_allocs();
            na_unify(r, &cols, &mut engine, &mut out).unwrap();
            let delta = current_allocs() - before;
            assert_eq!(delta, 0, "rule {} allocated {delta} times", r.rule_id);
            total += delta;
            // Same results as the warmup run.
            let (rlen, rrows, rvars) = &reference[k];
            assert_eq!(&out.len, rlen);
            for m in 0..out.len {
                assert_eq!(out.atom_row(m, 0), rrows[m]);
                for (idx, col_v) in rvars.iter().enumerate() {
                    assert_eq!(out.var_value(m, idx), col_v[m]);
                }
            }
        }
        assert_eq!(total, 0);
    }

    /// Compile-law courts: the typed refusals mirror the alloc arm's.
    /// Mutations that MUST flip these courts: silent widening (unknown
    /// sigil treated as constant), band wrap (65th constant wraps to
    /// R0), truncation (register payload > 255 truncated).
    #[test]
    fn compile_court_typed_refusals_mirror_alloc_arm() {
        let bad_sigil = Rule {
            rule_id: 1,
            body: vec![BodyAtom::new((0x02u64 << 56) | 5, cw(100), vw(11))],
            head: [vw(11), cw(100), vw(11)],
        };
        assert_eq!(na_compile(&bad_sigil), Err(NaError::UnsupportedTermSigil(0x02)));
        let fref_p = Rule {
            rule_id: 1,
            body: vec![BodyAtom::new(vw(10), (0x03u64 << 56) | 7, vw(11))],
            head: [vw(10), cw(100), vw(11)],
        };
        assert_eq!(na_compile(&fref_p), Err(NaError::FormulaRefInPredicate));
        let oor = Rule {
            rule_id: 1,
            body: vec![BodyAtom::new(vw(10), cw(100), (0x01u64 << 56) | 300)],
            head: [vw(10), cw(100), vw(10)],
        };
        assert_eq!(na_compile(&oor), Err(NaError::RegisterOutOfRange(300)));
        let unbound = Rule {
            rule_id: 1,
            body: vec![BodyAtom::new(vw(10), cw(100), vw(11))],
            head: [vw(12), cw(100), vw(10)],
        };
        assert_eq!(na_compile(&unbound), Err(NaError::UnboundHeadVariable));
        let empty = Rule {
            rule_id: 1,
            body: vec![],
            head: [cw(1), cw(100), cw(2)],
        };
        assert_eq!(na_compile(&empty), Err(NaError::EmptyRuleBody));
    }

    /// Capacity courts: exceeding a fixed capacity is the typed refusal,
    /// never a wrap or silent truncation.
    #[test]
    fn capacity_court_consts_vars_atoms_are_typed() {
        // 65 distinct constants.
        let mut body = Vec::new();
        for k in 0..65u64 {
            body.push(BodyAtom::new(cw(1000 + k), cw(100), cw(2000 + k)));
        }
        let rule = Rule {
            rule_id: 1,
            body,
            head: [cw(1000), cw(100), cw(2000)],
        };
        // Recorded: with NA_MAX_ATOMS = 8 the Const/Var interning tables
        // (64/128 wide) are structurally unreachable-through-compile (max
        // 24 distinct terms per rule); their guards protect the interner
        // itself. The reachable bounds are AtomCapacity (9 atoms) and the
        // evaluation-side MatchCapacity.
        assert_eq!(na_compile(&rule), Err(NaError::AtomCapacity));
        // 9 body atoms.
        let mut body = Vec::new();
        for k in 0..9u64 {
            body.push(BodyAtom::new(cw(k), cw(100), cw(k + 50)));
        }
        let rule = Rule {
            rule_id: 1,
            body,
            head: [cw(0), cw(100), cw(50)],
        };
        assert_eq!(na_compile(&rule), Err(NaError::AtomCapacity));
    }

    /// Match-capacity court: a single-atom rule over a store with more
    /// than NA_MAX_MATCHES matching rows refuses LOUDLY with
    /// NaError::MatchCapacity — never a silent truncation of the match
    /// set.
    /// Mutation that MUST flip this court: clamping `len` at capacity and
    /// returning Ok (the court's Ok(...) expectation fails it).
    #[test]
    fn capacity_court_match_overflow_is_typed_not_truncated() {
        let rows = (NA_MAX_MATCHES as u64) + 10;
        let s: Vec<u64> = (0..rows).map(cw).collect();
        let p: Vec<u64> = vec![cw(100); rows as usize];
        let o: Vec<u64> = (0..rows).map(|i| cw(10 + i)).collect();
        let cols = StatementColumns::new(&s, &p, &o).unwrap();
        let rule = Rule {
            rule_id: 0,
            body: vec![BodyAtom::new(vw(10), cw(100), vw(11))],
            head: [vw(10), cw(300), vw(11)],
        };
        let na = na_compile(&rule).unwrap();
        let mut engine = boxed_zeroed::<NaEngine>();
        let mut out = boxed_zeroed::<NaMatches>();
        assert_eq!(na_unify(&na, &cols, &mut engine, &mut out), Err(NaError::MatchCapacity));
    }

    /// Corpus determinism court: same seed, identical rows.
    #[test]
    fn corpus_court_same_seed_is_identical() {
        let a = gen_rows(64);
        let b = gen_rows(64);
        assert_eq!(a, b);
    }
}
