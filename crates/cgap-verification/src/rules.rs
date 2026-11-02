
//! Typed integrity rules as data - fixed-width discipline mirroring the
//! stardf spine's datalog integrity surface, evaluated over cgap-bont
//! `SlotId` triples directly.
//!
//! Every rule is data (serializable into the grounded context): a name plus
//! a [`RuleKind`]. Built-ins are expressed as `RuleKind` values (domain /
//! range conformance, no-orphan-objects, no-self-loop); [`RuleKind`] is
//! fully data-driven so new integrity laws land as bytes, not code.

use cgap_bont::primitives::{slot_id, SlotId};
use std::vec::Vec;

/// Rule identity: the `SlotId` of the rule's name lexeme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuleId(pub SlotId);

/// Rule evaluation mode. `Disabled` exists ONLY so the anti-vacuity court
/// can prove rule evaluation is load-bearing: with it, hallucinated triples
/// seal clean - exactly the failure the wedge exists to prevent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleMode {
    Enabled,
    Disabled,
}

/// A typed integrity rule: name + body, all bytes, no code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub name: Vec<u8>,
    pub kind: RuleKind,
}

impl Rule {
    /// [`RuleId`] derived through the same `slot_id` law as terms.
    pub fn id(&self) -> RuleId {
        RuleId(slot_id(&self.name))
    }
}

/// The rule body as data. Lexeme terms resolve through `slot_id` at
/// evaluation time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleKind {
    /// Predicate filter: `None` matches all predicates.
    pub predicate: Option<Vec<u8>>,
    /// If set, the subject must be one of these lexemes (domain conformance).
    pub domain: Option<Vec<Vec<u8>>>,
    /// If set, the object must be one of these lexemes (range conformance).
    pub range: Option<Vec<Vec<u8>>>,
    /// If true, forbid `subject == object` on matching triples (no-self-loop).
    pub forbid_self_loop: bool,
    /// If true, every matching-triple object must also occur as a subject
    /// somewhere in the corpus (no-orphan-objects).
    pub no_orphans: bool,
    /// Objects exempt from the orphan check.
    pub orphan_exemptions: Vec<Vec<u8>>,
}

impl RuleKind {
    /// Domain-conformance built-in: `s <p> ?o` requires `s` in `allowed`.
    pub fn domain(name: &[u8], predicate: &[u8], allowed: &[&[u8]]) -> Rule {
        Rule {
            name: name.to_vec(),
            kind: RuleKind {
                predicate: Some(predicate.to_vec()),
                domain: Some(allowed.iter().map(|l| l.to_vec()).collect()),
                range: None,
                forbid_self_loop: false,
                no_orphans: false,
                orphan_exemptions: Vec::new(),
            },
        }
    }

    /// Range-conformance built-in: `?s <p> o` requires `o` in `allowed`.
    pub fn range(name: &[u8], predicate: &[u8], allowed: &[&[u8]]) -> Rule {
        Rule {
            name: name.to_vec(),
            kind: RuleKind {
                predicate: Some(predicate.to_vec()),
                domain: None,
                range: Some(allowed.iter().map(|l| l.to_vec()).collect()),
                forbid_self_loop: false,
                no_orphans: false,
                orphan_exemptions: Vec::new(),
            },
        }
    }

    /// No-self-loop built-in: `s <p> o` with `s == o` is a violation.
    pub fn no_self_loop(name: &[u8], predicate: Option<&[u8]>) -> Rule {
        Rule {
            name: name.to_vec(),
            kind: RuleKind {
                predicate: predicate.map(|p| p.to_vec()),
                domain: None,
                range: None,
                forbid_self_loop: true,
                no_orphans: false,
                orphan_exemptions: Vec::new(),
            },
        }
    }

    /// No-orphan-objects built-in: every object of a matching triple must
    /// also occur as a subject in the corpus (or be exempted).
    pub fn no_orphan_objects(name: &[u8], predicate: Option<&[u8]>, exemptions: &[&[u8]]) -> Rule {
        Rule {
            name: name.to_vec(),
            kind: RuleKind {
                predicate: predicate.map(|p| p.to_vec()),
                domain: None,
                range: None,
                forbid_self_loop: false,
                no_orphans: true,
                orphan_exemptions: exemptions.iter().map(|l| l.to_vec()).collect(),
            },
        }
    }

    /// Fully custom rule as data: any combination of the constraint fields.
    pub fn custom(name: &[u8], kind: RuleKind) -> Rule {
        Rule { name: name.to_vec(), kind }
    }
}

/// One typed violation: which rule, which triple, which constraint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation {
    pub rule: RuleId,
    /// Index into the ingested triple list.
    pub triple_index: u32,
    /// Constraint code: 1 = domain, 2 = range, 3 = self-loop, 4 = orphan.
    pub code: u8,
}

impl Violation {
    /// Constraint code rendered; `run.rs` embeds this in the refusal reason.
    pub fn code_name(self_code: u8) -> &'static str {
        match self_code {
            1 => "domain",
            2 => "range",
            3 => "self-loop",
            4 => "orphan-object",
            _ => "unknown",
        }
    }
}

/// Evaluate one rule over the corpus. `slot_id` resolution of rule lexemes
/// goes through the real derivation (no caching, no bypass).
pub fn evaluate(rule: &Rule, corpus: &crate::ingest::IngestedCorpus) -> Vec<Violation> {
    let pred_slot = rule.kind.predicate.as_ref().map(|p| slot_id(p));
    let domain_slots: Option<Vec<SlotId>> = rule
        .kind
        .domain
        .as_ref()
        .map(|vs| vs.iter().map(|v| slot_id(v)).collect());
    let range_slots: Option<Vec<SlotId>> = rule
        .kind
        .range
        .as_ref()
        .map(|vs| vs.iter().map(|v| slot_id(v)).collect());
    let exempt_slots: Vec<SlotId> = rule
        .kind
        .orphan_exemptions
        .iter()
        .map(|v| slot_id(v))
        .collect();

    let mut violations = Vec::new();
    for (i, t) in corpus.triples.iter().enumerate() {
        if let Some(p) = pred_slot {
            if t.predicate != p {
                continue;
            }
        }
        let mut hit = |code: u8| {
            violations.push(Violation {
                rule: rule.id(),
                triple_index: i as u32,
                code,
            });
        };
        if let Some(dom) = &domain_slots {
            if !dom.contains(&t.subject) {
                hit(1);
            }
        }
        if let Some(rng) = &range_slots {
            if !rng.contains(&t.object) {
                hit(2);
            }
        }
        if rule.kind.forbid_self_loop && t.subject == t.object {
            hit(3);
        }
        if rule.kind.no_orphans && !exempt_slots.contains(&t.object) {
            let is_subject = corpus.triples.iter().any(|o| o.subject == t.object);
            if !is_subject {
                hit(4);
            }
        }
    }
    violations
}
