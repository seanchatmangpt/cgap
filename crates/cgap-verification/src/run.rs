
//! The verification run: all rules over all ingested triples ->
//! [`GroundedReceipt`] (per-rule pass/fail + the canonical grounded-context
//! bytes), then the A16 chatman keyed sealer commits the context.

use crate::ingest::{ExtractedTriple, IngestedCorpus};
use crate::rules::{evaluate, Rule, RuleMode, Violation};
use crate::seal;
use std::fmt;
use std::vec::Vec;

/// Grounded-context byte format magic: `b"CGVR"` + version byte 0x01.
pub const GROUNDED_MAGIC: [u8; 4] = *b"CGVR";

/// One rule's outcome in the receipt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleResult {
    pub rule: crate::rules::RuleId,
    pub name: Vec<u8>,
    pub passed: bool,
    pub violations: Vec<Violation>,
}

/// The verification receipt: per-rule pass/fail plus the sealed-ready
/// grounded context bytes.
#[derive(Clone, Debug)]
pub struct GroundedReceipt {
    pub results: Vec<RuleResult>,
    pub grounded_context: Vec<u8>,
    pub violations_total: usize,
}

/// A typed verification refusal naming the failing rule (the wedge's
/// answer to a hallucinated triple).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    pub rule: crate::rules::RuleId,
    pub rule_name: Vec<u8>,
    pub triple_index: u32,
    /// Constraint name, e.g. "self-loop".
    pub constraint: &'static str,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "verification refused by rule {:?} (slot 0x{:016x}): triple {} violates {}",
            String::from_utf8_lossy(&self.rule_name),
            self.rule.0 .0,
            self.triple_index,
            self.constraint,
        )
    }
}

fn put_u32(out: &mut Vec<u8>, v: usize) {
    out.extend_from_slice(&(v as u32).to_le_bytes());
}

fn put_lexeme(out: &mut Vec<u8>, lexeme: &[u8]) {
    put_u32(out, lexeme.len());
    out.extend_from_slice(lexeme);
}

/// Canonical grounded-context encoding (layout in the receipt doc). Sorted
/// triples + sorted rule names make the bytes a pure function of
/// (corpus, rules, outcome): equal bytes = equal verification.
fn encode_context(corpus: &IngestedCorpus, rules: &[Rule], violations: &[Violation]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&GROUNDED_MAGIC);
    out.push(0x01);
    put_u32(&mut out, rules.len());
    put_u32(&mut out, corpus.triples.len());

    let mut sorted = corpus.triples.clone();
    sorted.sort_by(|a, b| {
        (a.subject.0, a.predicate.0, a.object.0, a.source)
            .cmp(&(b.subject.0, b.predicate.0, b.object.0, b.source))
    });
    for t in &sorted {
        out.extend_from_slice(&t.subject.0.to_le_bytes());
        out.extend_from_slice(&t.predicate.0.to_le_bytes());
        out.extend_from_slice(&t.object.0.to_le_bytes());
        out.extend_from_slice(&t.source.to_le_bytes());
    }

    let mut sorted_rules: Vec<&Rule> = rules.iter().collect();
    sorted_rules.sort_by(|a, b| a.name.cmp(&b.name));
    for r in &sorted_rules {
        put_lexeme(&mut out, &r.name);
        out.push(0x00); // kind_code reserved (data rules; no enum tag needed)
        match &r.kind.predicate {
            Some(p) => put_lexeme(&mut out, p),
            None => put_u32(&mut out, u32::MAX as usize),
        }
        // constraint flags: domain|range|self-loop|orphan
        // (per-set count words below use 0xFFFF_FFFF = "no set")
        let mut flags = 0u8;
        if r.kind.domain.is_some() {
            flags |= 1;
        }
        if r.kind.range.is_some() {
            flags |= 2;
        }
        if r.kind.forbid_self_loop {
            flags |= 4;
        }
        if r.kind.no_orphans {
            flags |= 8;
        }
        out.push(flags);
        for set in [&r.kind.domain, &r.kind.range, &Some(r.kind.orphan_exemptions.clone())] {
            match set {
                Some(lexemes) => {
                    put_u32(&mut out, lexemes.len());
                    for l in lexemes {
                        put_lexeme(&mut out, l);
                    }
                }
                None => put_u32(&mut out, u32::MAX as usize),
            }
        }
    }

    // Violations are recorded against the CANONICAL (sorted) triple order so
    // the context is ingestion-order independent.
    let mut canon = Vec::with_capacity(violations.len());
    for v in violations {
        let t = corpus
            .triples
            .get(v.triple_index as usize)
            .expect("violation index within corpus");
        let canonical = sorted
            .iter()
            .position(|s| s == t)
            .expect("violated triple present in corpus");
        canon.push((v.rule.0 .0, canonical as u32, v.code));
    }
    canon.sort();
    put_u32(&mut out, canon.len());
    for (rule_slot, canonical, code) in canon {
        out.extend_from_slice(&rule_slot.to_le_bytes());
        put_u32(&mut out, canonical as usize);
        out.push(code);
    }
    out
}

/// The full verification pipeline: ingest -> evaluate every rule over every
/// triple -> encode the grounded context.
///
/// With [`RuleMode::Enabled`], any violation yields `Ok` receipt with the
/// rule marked failed AND a typed [`Refusal`] in `refusals`. With
/// [`RuleMode::Disabled`], no rule runs: the receipt reports zero violations
/// and the context still seals - the anti-vacuity court's mutation target.
pub fn run_verification(
    triples: &[ExtractedTriple<'_>],
    rules: &[Rule],
    mode: RuleMode,
) -> Result<GroundedReceipt, crate::ingest::IngestError> {
    let corpus = crate::ingest::ingest(triples)?;
    let corpus = &corpus;
    let mut results = Vec::new();
    let mut all_violations = Vec::new();
    for rule in rules {
        let violations = match mode {
            RuleMode::Enabled => evaluate(rule, corpus),
            RuleMode::Disabled => Vec::new(),
        };
        let passed = violations.is_empty();
        all_violations.extend(violations.iter().cloned());
        results.push(RuleResult {
            rule: rule.id(),
            name: rule.name.clone(),
            passed,
            violations,
        });
    }
    let refusals_unused: Vec<Refusal> = Vec::new();
    let _ = refusals_unused;
    let grounded_context = encode_context(corpus, rules, &all_violations);
    Ok(GroundedReceipt {
        violations_total: all_violations.len(),
        grounded_context,
        results,
    })
}

/// Typed refusals derived from a receipt: one per violation, naming the
/// rule (slot + lexeme) and constraint.
pub fn refusals(receipt: &GroundedReceipt, triples: &[ExtractedTriple<'_>]) -> Vec<Refusal> {
    let corpus = match crate::ingest::ingest(triples) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for r in &receipt.results {
        for v in &r.violations {
            if v.triple_index as usize >= corpus.triples.len() {
                continue; // out-of-bounds violation index cannot name a triple
            }
            out.push(Refusal {
                rule: v.rule,
                rule_name: r.name.clone(),
                triple_index: v.triple_index,
                constraint: Violation::code_name(v.code),
            });
        }
    }
    out
}

/// Convenience: seal the receipt's grounded context under `key` (A16
/// chatman keyed sealer). See [`crate::seal`].
pub fn seal_receipt(
    receipt: &GroundedReceipt,
    key: &[u8; 32],
) -> Result<seal::SealedContext, seal::SealError> {
    seal::seal_grounded(key, &receipt.grounded_context)
}
