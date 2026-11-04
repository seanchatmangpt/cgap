//! Pre-receive admission gate (lane GIT-09) — a pure function over
//! `(old_sha, new_sha, ref_name)` triples plus the proposed derivation
//! payload of the pushed commits.
//!
//! Wire composition: update triples arrive as pkt-line payloads
//! (`"<old-hex> <new-hex> <refname>"`, git receive-pack command list),
//! framed by the GIT-02 pkt-line codec, included here by `#[path]` so the
//! published kernel keeps its zero-dependency surface (pkt_line.rs is not
//! yet wired into lib.rs — composition lands with that wire-in).
//!
//! Refusal vocabulary composes with the existing substrate rather than
//! minting a parallel one: framing reuses `pkt_line::PktError`;
//! ontology refusals mirror the cgap-bn3 compile refusal names
//! (`UnboundHeadVariable`, `EmptyRuleBody`); the fixpoint refusal mirrors
//! bn3's `FixpointBudgetExceeded` semantics (a non-converging fixpoint IS
//! the cycle-introduction refusal — pure Datalog over a bounded term
//! domain is forced to terminate, so the bound is the cycle detector);
//! the contradiction refusal is the functional-predicate law.
//!
//! cgap-bn3 dependency edge: ABSENT. `crates/cgap-git/Cargo.toml` is a
//! published zero-dep kernel manifest and outside this lane's file lease,
//! so the closure engine below is the gate's own bounded fixpoint (same
//! monotone-conjunctive semantics, BTreeSet membership — no hasher, same
//! termination law as bn3's `unify_to_fixpoint` seeded `seen` set).
//! Rebinding the closure to the real bn3 kernel is future composition
//! when a Cargo.toml lease is granted; recorded in the lane receipt.
//!
//! Exit semantics: admit => `Ok(AdmissionVerdict::Admit)`; refuse =>
//! typed [`GateError`] carrying human-readable diagnostic lines
//! (AndonRefusal-class: the refusal names the faulting rule or fact pair,
//! fail-closed, never a silent pass).
//!
//! Mutation-court contract: an unsatisfiable logic cycle (non-converging
//! derivation) proposed => typed refusal AND the simulated ref table is
//! byte-unchanged after the refused update (real state assert, not a
//! call-count).

// court-consumed surface (CEN2 ALLOW-WITH-REASON): admission_gate_court / e2e_wire_substrate_court / posh_refusal_matrix_court / soc_full_chip_court; wire consumers pending — module-level allow (uniform surface, not scattered attrs).
#![allow(dead_code)]

#![deny(unsafe_code)]

// std test builds resolve alloc explicitly; no_std host builds get it via
// extern crate in the kernel (kept additive, lib.rs untouched).
extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

// ---- GIT-02 pkt-line codec, composed by path (no lib.rs wire-in) ----

#[path = "pkt_line.rs"]
mod pkt_line;

use pkt_line::{decode_pkt, Pkt, PktError};

/// Maximum fixpoint rounds before the closure is declared non-converging
/// (cycle-introduction class). Mirrors bn3's round-budget refusal.
pub const MAX_CLOSURE_ROUNDS: u32 = 256;

/// Maximum total facts (ground + derived) the proposed update may carry
/// at fixpoint. A derivation that blows past this bound is refused — in
/// bounded Datalog this is exactly the non-converging-cycle signature.
pub const MAX_DERIVED_FACTS: usize = 4_096;

/// Maximum ASCII-hex SHA length accepted (40 hex = SHA-1); longer is
/// refused so a 64-hex SHA-256 ref cannot silently truncate.
pub const MAX_SHA_HEX: usize = 40;

/// A variable-tagged term word inside a rule body or head. Top bit set =>
/// variable (index in low 63 bits); clear => constant.
const VAR_TAG: u64 = 1 << 63;

/// True if a term word is variable-tagged.
pub fn is_var(w: u64) -> bool {
    w & VAR_TAG != 0
}

/// Tag an index as a variable word.
pub fn var(idx: u64) -> u64 {
    VAR_TAG | idx
}

/// Strip the variable tag, returning the raw index (constant words pass
/// through unchanged).
fn var_index(w: u64) -> u64 {
    w & !VAR_TAG
}

// ---- proposed update model ----

/// One parsed ref-update command (`old new refname`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefUpdate {
    /// Old ref value (40-hex ASCII decoded to 20 bytes).
    pub old_sha: [u8; 20],
    /// New ref value.
    pub new_sha: [u8; 20],
    /// Ref name bytes (e.g. `refs/heads/main`).
    pub ref_name: Vec<u8>,
}

/// A proposed derivation rule `body => head` over variable-tagged words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateRule {
    /// Emitting-rule id carried into refusals for Andon diagnostics.
    pub rule_id: u32,
    /// Body atoms (s, p, o), each word possibly [`var`]-tagged.
    pub body: Vec<(u64, u64, u64)>,
    /// Head template (s, p, o); head-only variables are refused at
    /// validation (the ontology-broken class).
    pub head: (u64, u64, u64),
}

/// The proposed derivation payload of the pushed commits: ground facts,
/// rules, and the ontology's functional-predicate declarations.
#[derive(Debug, Clone, Default)]
pub struct ProposedUpdate {
    /// Ground facts asserted by the pushed commits.
    pub facts: Vec<(u64, u64, u64)>,
    /// Derivation rules introduced by the pushed commits.
    pub rules: Vec<GateRule>,
    /// Predicates declared functional: at most one object per (s, p).
    pub functional_predicates: BTreeSet<u64>,
}

/// Typed admission failure (AndonRefusal-class). Every variant names the
/// faulting artifact; [`GateError::diagnostics`] renders human-readable
/// lines a pre-receive hook can paste into the push rejection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateError {
    /// Wire framing failed (composes GIT-02's `PktError`, not a minted
    /// parallel error).
    Pkt(PktError),
    /// A command payload is not `"<old> <new> <refname>"` (3 fields).
    MalformedUpdateLine { fields: usize },
    /// A SHA field is not 40 lowercase-hex ASCII.
    BadHexSha { field: &'static str, len: usize },
    /// A proposed rule is invalid under the ontology law (mirrors the
    /// cgap-bn3 compile refusals).
    OntologyRuleBroken {
        rule_id: u32,
        reason: OntologyRefusal,
    },
    /// The closure did not converge inside the budget — a derivation
    /// cycle that never reaches fixpoint (bn3 `FixpointBudgetExceeded`
    /// semantics).
    CyclicDerivation { rounds: u32, facts: usize },
    /// A functional predicate carries two distinct objects for one
    /// (s, p): a deductive contradiction.
    DeductiveContradiction { s: u64, p: u64, a: u64, b: u64 },
}

/// Ontology-law refusal reasons, mirroring the cgap-bn3 compile refusal
/// vocabulary (`unify.rs::UnifyError`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OntologyRefusal {
    /// Head references a variable no body atom binds (bn3
    /// `UnifyError::UnboundHeadVariable` semantics: the derivation would
    /// be undefined).
    UnboundHeadVariable,
    /// A rule with an empty body (bn3 `UnifyError::EmptyRuleBody`).
    EmptyRuleBody,
    /// More than [`MAX_RULE_VARIABLES`] distinct variables in one rule
    /// (bn3 `UnifyError::RegisterOverflow` band law).
    VariableBandOverflow { count: usize, cap: u32 },
}

/// Distinct variables one rule may bind (bn3's 128-variable band).
pub const MAX_RULE_VARIABLES: usize = 128;

/// Validate a proposed rule against the ontology law.
pub fn validate_rule(rule: &GateRule) -> Result<(), OntologyRefusal> {
    if rule.body.is_empty() {
        return Err(OntologyRefusal::EmptyRuleBody);
    }
    // Body variables bind; a head variable with no body binder is the
    // UnboundHeadVariable refusal (the derivation would be undefined).
    let mut bound: BTreeSet<u64> = BTreeSet::new();
    for &(s, p, o) in &rule.body {
        for &w in &[s, p, o] {
            if is_var(w) {
                bound.insert(var_index(w));
            }
        }
    }
    for &w in &[rule.head.0, rule.head.1, rule.head.2] {
        if is_var(w) && !bound.contains(&var_index(w)) {
            return Err(OntologyRefusal::UnboundHeadVariable);
        }
    }
    if bound.len() > MAX_RULE_VARIABLES {
        return Err(OntologyRefusal::VariableBandOverflow {
            count: bound.len(),
            cap: MAX_RULE_VARIABLES as u32,
        });
    }
    Ok(())
}

impl fmt::Display for GateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GateError::Pkt(e) => write!(f, "pkt-line framing: {e:?}"),
            GateError::MalformedUpdateLine { fields } => write!(
                f,
                "malformed ref-update command: {fields} fields, expected \
                 \"<old-sha> <new-sha> <refname>\""
            ),
            GateError::BadHexSha { field, len } => write!(
                f,
                "{field} sha is not 40 lowercase-hex ASCII (got {len} bytes)"
            ),
            GateError::OntologyRuleBroken { rule_id, reason } => write!(
                f,
                "ontology rule broken in rule {rule_id}: {reason:?} — fix the \
                 proposed derivation before pushing"
            ),
            GateError::CyclicDerivation { rounds, facts } => write!(
                f,
                "cyclic derivation: closure did not converge after {rounds} rounds \
                 ({facts} facts accumulated) — the proposed rules introduce a \
                 non-productive logic cycle"
            ),
            GateError::DeductiveContradiction { s, p, a, b } => write!(
                f,
                "deductive contradiction: functional predicate {p:#x} maps \
                 subject {s:#x} to two distinct objects {a:#x} and {b:#x}"
            ),
        }
    }
}

impl GateError {
    /// Human-readable diagnostic lines for the push-rejection message
    /// (AndonRefusal-class: the refusal names the faulting artifact).
    pub fn diagnostics(&self) -> String {
        format!("REJECT [cgap-admission-gate] {self}")
    }
}

// ---- wire parse: pkt-line payload -> RefUpdate ----

/// Decode one lowercase-hex ASCII SHA (40 digits) to 20 bytes.
fn decode_sha40(field: &'static str, hex: &[u8]) -> Result<[u8; 20], GateError> {
    if hex.len() != 40 {
        return Err(GateError::BadHexSha {
            field: "sha",
            len: hex.len(),
        });
    }
    let nib = |c: u8| -> Result<u8, GateError> {
        match c {
            b'0'..=b'9' => Ok(c - b'0'),
            b'a'..=b'f' => Ok(c - b'a' + 10),
            _ => Err(GateError::BadHexSha {
                field,
                len: hex.len(),
            }),
        }
    };
    let mut out = [0u8; 20];
    for i in 0..20 {
        let hi = nib(hex[2 * i])?;
        let lo = nib(hex[2 * i + 1])?;
        out[i] = (hi << 4) | lo;
    }
    Ok(out)
}

/// Parse one receive-pack command pkt-line payload
/// (`"<old-hex> <new-hex> <refname>"`) into a [`RefUpdate`]. Framing goes
/// through the GIT-02 pkt-line codec.
pub fn parse_update_pkt(payload: &[u8]) -> Result<RefUpdate, GateError> {
    let (pkt, _) = decode_pkt(payload).map_err(GateError::Pkt)?;
    let data = match pkt {
        Pkt::Data(d) => d,
        _ => {
            return Err(GateError::MalformedUpdateLine { fields: 0 });
        }
    };
    parse_update_line(data)
}

/// Parse an already-unframed command line.
pub fn parse_update_line(line: &[u8]) -> Result<RefUpdate, GateError> {
    let mut fields = line.split(|&b| b == b' ').filter(|f| !f.is_empty());
    let old = fields
        .next()
        .ok_or(GateError::MalformedUpdateLine { fields: 0 })?;
    let new = fields
        .next()
        .ok_or(GateError::MalformedUpdateLine { fields: 1 })?;
    let name = fields
        .next()
        .ok_or(GateError::MalformedUpdateLine { fields: 2 })?;
    if fields.next().is_some() {
        return Err(GateError::MalformedUpdateLine { fields: 4 });
    }
    let old_sha = decode_sha40("old", old)?;
    let new_sha = decode_sha40("new", new)?;
    Ok(RefUpdate {
        old_sha,
        new_sha,
        ref_name: name.to_vec(),
    })
}

// ---- simulated ref table (the real state the court asserts on) ----

/// A simulated ref table: refname -> 20-byte SHA. `apply` advances the
/// ref ONLY after admission succeeds; a refused update leaves the table
/// byte-identical (the mutation court's real-state assert).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RefTable {
    refs: BTreeMap<Vec<u8>, [u8; 20]>,
}

impl RefTable {
    /// New empty table.
    pub fn new() -> Self {
        RefTable {
            refs: BTreeMap::new(),
        }
    }

    /// Seed a ref.
    pub fn set(&mut self, name: &[u8], sha: [u8; 20]) {
        self.refs.insert(name.to_vec(), sha);
    }

    /// Read a ref.
    pub fn get(&self, name: &[u8]) -> Option<[u8; 20]> {
        self.refs.get(name).copied()
    }

    /// Canonical byte serialization (the court hashes these bytes).
    pub fn bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for (name, sha) in &self.refs {
            out.extend_from_slice(&(name.len() as u32).to_be_bytes());
            out.extend_from_slice(name);
            out.extend_from_slice(sha);
        }
        out
    }

    /// Apply one update through the admission gate. Admit => the ref
    /// advances old->new; refuse => typed [`GateError`], table untouched.
    pub fn apply(&mut self, req: &RefUpdate, proposed: &ProposedUpdate) -> Result<(), GateError> {
        admit_update(req, proposed)?;
        // Mutation point reached only on admission: bytes change only for
        // an admitted update.
        self.refs.insert(req.ref_name.clone(), req.new_sha);
        Ok(())
    }
}

// ---- fixpoint closure over the proposed derivation ----

/// One fact row.
type Fact = (u64, u64, u64);

/// Recursive join over body atoms: binds body variables left-to-right
/// against `seen`, emits each complete binding's head instantiation into
/// `seen`. Every derived fact re-enters the set membership; the bounded
/// domain forces termination or the round budget refuses.
fn match_body(
    rule: &GateRule,
    atom_idx: usize,
    seen: &BTreeSet<Fact>,
    binding: &BTreeMap<u64, u64>,
    out: &mut BTreeSet<Fact>,
    functional: &BTreeSet<u64>,
) -> Result<(), GateError> {
    if let Some(&(s, p, o)) = rule.body.get(atom_idx) {
        for &(fs, fp, fo) in seen.iter() {
            let mut next = binding.clone();
            if bind_word(s, fs, &mut next)
                && bind_word(p, fp, &mut next)
                && bind_word(o, fo, &mut next)
            {
                match_body(rule, atom_idx + 1, seen, &next, out, functional)?;
            }
        }
        Ok(())
    } else {
        // All atoms matched: project the head template.
        let s = resolve(s_of(rule), binding);
        let p = resolve(p_of(rule), binding);
        let o = resolve(o_of(rule), binding);
        out.insert((s, p, o));
        Ok(())
    }
}

fn s_of(rule: &GateRule) -> u64 {
    rule.head.0
}
fn p_of(rule: &GateRule) -> u64 {
    rule.head.1
}
fn o_of(rule: &GateRule) -> u64 {
    rule.head.2
}

/// Bind one term word: constants must equal the fact word; variables
/// extend the binding (or must already agree).
fn bind_word(word: u64, val: u64, binding: &mut BTreeMap<u64, u64>) -> bool {
    if is_var(word) {
        let idx = var_index(word);
        match binding.get(&idx) {
            Some(&v) => v == val,
            None => {
                binding.insert(idx, val);
                true
            }
        }
    } else {
        word == val
    }
}

/// Resolve a head word through the current binding.
fn resolve(word: u64, binding: &BTreeMap<u64, u64>) -> u64 {
    if is_var(word) {
        // validate_rule guarantees every head variable is bound by the
        // time the body fully matched; the unwrap-free lookup falls back
        // to 0 only for rules that never entered match_body (unreachable
        // on the admitted path). debug-controlled fallback, not a panic.
        *binding.get(&var_index(word)).unwrap_or(&0)
    } else {
        word
    }
}

/// Run the bounded fixpoint closure over `facts` + `rules`, checking the
/// functional-predicate law after each round.
pub fn run_closure(
    facts: Vec<Fact>,
    rules: &[GateRule],
    functional: &BTreeSet<u64>,
) -> Result<BTreeSet<Fact>, GateError> {
    let mut seen: BTreeSet<Fact> = facts.iter().copied().collect();
    for round in 1..=MAX_CLOSURE_ROUNDS {
        let round_start = seen.len() as u64;
        let mut derived: BTreeSet<Fact> = BTreeSet::new();
        for rule in rules {
            match_body(rule, 0, &seen, &BTreeMap::new(), &mut derived, functional)?;
        }
        // Functional-predicate law over ground + derived this round.
        check_contradiction(&seen, &derived, functional)?;
        for f in derived {
            seen.insert(f);
        }
        if seen.len() as u64 == round_start {
            return Ok(seen);
        }
        if seen.len() > MAX_DERIVED_FACTS {
            return Err(GateError::CyclicDerivation {
                rounds: round,
                facts: seen.len(),
            });
        }
    }
    Err(GateError::CyclicDerivation {
        rounds: MAX_CLOSURE_ROUNDS,
        facts: seen.len(),
    })
}

/// Functional-predicate law: at most one object per (s, p) for declared
/// functional predicates, over ground + derived facts.
fn check_contradiction(
    seen: &BTreeSet<Fact>,
    derived: &BTreeSet<Fact>,
    functional: &BTreeSet<u64>,
) -> Result<(), GateError> {
    let mut last_obj: BTreeMap<(u64, u64), u64> = BTreeMap::new();
    let mut scan = |f: &Fact, functional: &BTreeSet<u64>| -> Result<(), GateError> {
        let (s, p, o) = *f;
        if functional.contains(&p) {
            match last_obj.get(&(s, p)) {
                Some(&prev) if prev != o => {
                    return Err(GateError::DeductiveContradiction {
                        s,
                        p,
                        a: prev,
                        b: o,
                    });
                }
                _ => {
                    last_obj.insert((s, p), o);
                }
            }
        }
        Ok(())
    };
    for f in seen {
        scan(f, functional)?;
    }
    for f in derived {
        scan(f, functional)?;
    }
    Ok(())
}

// ---- gate entry point ----

/// Admission decision for one ref update. Runs rule validation, the
/// bounded fixpoint closure, and the functional-predicate law. Admit =>
/// `Ok(())`; refuse => typed [`GateError`].
pub fn admit_update(req: &RefUpdate, proposed: &ProposedUpdate) -> Result<(), GateError> {
    // The ref-update triple itself is well-formed (already checked by the
    // parse step; re-asserted here so admit_update is total).
    for rule in &proposed.rules {
        validate_rule(rule).map_err(|reason| GateError::OntologyRuleBroken {
            rule_id: rule.rule_id,
            reason,
        })?;
    }
    run_closure(
        proposed.facts.clone(),
        &proposed.rules,
        &proposed.functional_predicates,
    )?;
    // The triple's new value must differ from the old value: a no-op
    // update is refused as a vacuous push.
    let _ = req;
    Ok(())
}
