//! Extracted-triple ingestion: lexemes in, `SlotId` slots out, collision-
//! trapped (§5 producer obligation of lane A06, run over the verifier's own
//! corpus — the verifier inherits the producer's collision duty).

use cgap_bont::primitives::{
    slot_id, CollisionDetector, CollisionError, SlotId,
};
use std::fmt;
use std::vec::Vec;

/// One LLM-extracted fact before slotting: raw lexeme bytes (lexemes may be
/// non-UTF8; the verifier never assumes strings, mirroring A06 §1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtractedTriple<'a> {
    pub subject: &'a [u8],
    pub predicate: &'a [u8],
    pub object: &'a [u8],
    /// Extraction source index (provenance preserved in the grounded context).
    pub source: u32,
}

/// A slotted triple: every term is a collision-vetted `SlotId`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotTriple {
    pub subject: SlotId,
    pub predicate: SlotId,
    pub object: SlotId,
    pub source: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IngestError {
    /// Two DISTINCT lexemes truncate to the same slot (A06 §5). Fail-closed:
    /// the corpus is refused, naming both lexemes.
    Collision(CollisionError),
}

impl fmt::Display for IngestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IngestError::Collision(e) => write!(f, "ingest refused: {}", e),
        }
    }
}

/// The ingested corpus: slotted triples plus the lexeme registry needed to
/// name terms in typed refusals.
#[derive(Clone, Debug, Default)]
pub struct IngestedCorpus {
    pub triples: Vec<SlotTriple>,
    /// slot -> first lexeme that claimed it (for refusal naming).
    pub lexemes: std::collections::BTreeMap<u64, Vec<u8>>,
}

impl IngestedCorpus {
    /// Lexeme bytes for a slot (first claimant), if known.
    pub fn lexeme(&self, slot: SlotId) -> Option<&[u8]> {
        self.lexemes.get(&slot.0).map(|v| v.as_slice())
    }
}

/// Ingest extracted triples through the real `slot_id` law, collision-
/// trapped. Every term lexeme of every triple is fed through the
/// `CollisionDetector`; a distinct-lexeme collision refuses the whole
/// corpus (fail-closed, §5).
pub fn ingest(triples: &[ExtractedTriple<'_>]) -> Result<IngestedCorpus, IngestError> {
    let mut det = CollisionDetector::new();
    let mut lexemes = std::collections::BTreeMap::new();
    let mut slotted = Vec::with_capacity(triples.len());
    for t in triples {
        for lexeme in [t.subject, t.predicate, t.object] {
            det.insert(lexeme).map_err(IngestError::Collision)?;
            let s = slot_id(lexeme).0;
            lexemes.entry(s).or_insert_with(|| lexeme.to_vec());
        }
        slotted.push(SlotTriple {
            subject: slot_id(t.subject),
            predicate: slot_id(t.predicate),
            object: slot_id(t.object),
            source: t.source,
        });
    }
    Ok(IngestedCorpus { triples: slotted, lexemes })
}
