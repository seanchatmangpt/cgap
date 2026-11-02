//! Deterministic Verification Surface — the LLM-verification wedge (lane
//! RDF-10, v26.11.1 moonshot).
//!
//! The OST frame reaches on-device LLM-output verification via Samsung
//! hardware; cgap reaches the same endpoint via receipt-checkable
//! derivation: hallucination is caught not by trusting the model but by
//! re-deriving every emitted fact through the fixed-width SlotId law and
//! checking it against typed integrity rules, then sealing the grounded
//! context so any post-hoc tampering is detectable. `docs/jira/v26.11.1/
//! waves/rdf-10-verification.md` is the lane receipt.
//!
//! Pipeline (`run.rs` wires it):
//! 1. [`ingest`] — extracted triples in; every lexeme slotted via the A06
//!    `cgap_bont::primitives::slot_id` law, collision-trapped through the
//!    real `CollisionDetector` (§5 producer obligation).
//! 2. [`rules`] — typed integrity rules as data (domain/range conformance,
//!    no-orphan-objects, no-self-loop, plus fully custom rules as data).
//! 3. [`run`] — all rules over all ingested triples -> [`GroundedReceipt`]
//!    with per-rule pass/fail and the canonical grounded-context bytes.
//! 4. [`seal`] — the A16 `cgap-crypto` chatman keyed sealer commits the
//!    grounded context (keyed chunk leaves + Merkle root, size-unbounded).

pub mod ingest;
pub mod rules;
pub mod run;
pub mod seal;
