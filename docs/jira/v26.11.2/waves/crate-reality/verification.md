# REAL-verification receipt — cgap-verification

Date: 2026-11-01. Repo: /Users/sac/cgap. Lane: REAL-verification.

## Test gate

- Command: `CARGO_TARGET_DIR=/tmp/real-verification-target cargo test --manifest-path crates/cgap-verification/Cargo.toml`
- Exit: 0. Courts: 7 passed, 0 failed (tests/courts.rs):
  court_ingestion_slot_law, court_rule_evaluation_is_load_bearing,
  court_hallucinated_triple_refusal_names_the_rule,
  court_custom_rule_as_data_and_orphan_exemption,
  court_seal_round_trip_and_tamper_flip, court_grounded_context_determinism,
  court_seal_multi_chunk. Lib unit tests: 0 (all logic covered via courts).

## Public surface

- ingest.rs: `ingest(&[ExtractedTriple]) -> Result<IngestedCorpus, IngestError>`
- rules.rs: `RuleKind` (data-driven rule body: predicate/domain/range/
  forbid_self_loop/no_orphans/orphan_exemptions); constructors
  `RuleKind::{domain, range, no_self_loop, no_orphan_objects, custom}`;
  `evaluate(&Rule, &IngestedCorpus) -> Vec<Violation>`
- run.rs: `run_verification(&[ExtractedTriple], &[Rule], RuleMode) -> Result<GroundedReceipt, IngestError>`;
  `refusals(&GroundedReceipt, &[ExtractedTriple]) -> Vec<Refusal>`;
  `seal_receipt(&GroundedReceipt, &[u8;32]) -> Result<SealedContext, SealError>`
- seal.rs: `seal_grounded`, `verify_sealed` (A16 cgap-crypto chatman keyed sealer)

## Verdict: REAL (substantive, not scaffold)

The rule engine is substantive: rules are pure data (`RuleKind` fields),
evaluation walks real SlotId triples over a real cgap-bont corpus, and
`RuleMode::Disabled` exists explicitly as the anti-vacuity mutation target
(court_rule_evaluation_is_load_bearing proves Disabled seals hallucinated
triples clean — the exact failure the wedge prevents). Seal path does a real
A16 keyed round-trip with a tamper-flip court. Minor wart: dead
`refusals_unused` local in run_verification (run.rs:180-181) — cosmetic.

## Drift-check

`ontology/cgap.ttl` has 9 triples and ZERO `p:CrateManifest` rows for any
crate (rdflib query exit 0, rows 0) — cgap-verification is ontology-absent,
not drifted. Loud gap, not a fake pass.

Build root /tmp/real-verification-target deleted (lease cleanup).
Shared tree untouched (read-only lane; receipt file only).
