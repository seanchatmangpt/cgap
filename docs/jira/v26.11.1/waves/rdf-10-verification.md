# RDF-10 Verification-Wedge Lane Receipt

Lane: RDF-10-verification (lane-runner) - 2026-11-01 - /Users/sac/cgap

## Wedge statement

OST reaches on-device LLM-output verification via Samsung hardware; cgap
reaches the same endpoint via receipt-checkable derivation: a hallucination
is caught not by trusting the model but by re-deriving every emitted fact
through the fixed-width cgap-bont SlotId law (A06: SlotId =
u64::from_le_bytes(BLAKE3("bont\x01" ++ lexeme)[0..8])), checking the
slotted corpus against typed integrity rules, and sealing the grounded
context with the A16 chatman keyed sealer so post-hoc tampering is typed-loud
(`SealError::Tampered` with both roots named). The OST frame is cited from
the dispatch; no OST/Samsung frame document exists on disk (divergence D3
below).

## API surface

- `crates/cgap-verification/src/ingest.rs` - `ExtractedTriple`,
  `ingest(&[ExtractedTriple]) -> Result<IngestedCorpus, IngestError>`:
  every term lexeme through the real `cgap_bont::primitives::slot_id` +
  `CollisionDetector` (A06 §5 producer obligation, inherited by the
  verifier); collision refuses the whole corpus.
- `crates/cgap-verification/src/rules.rs` - rules as data (`Rule{name,
  kind: RuleKind}`): built-in constructors `domain`, `range`,
  `no_self_loop`, `no_orphan_objects`, plus `custom`; `RuleId(SlotId)`;
  `evaluate(rule, corpus) -> Vec<Violation>` (codes 1=domain, 2=range,
  3=self-loop, 4=orphan).
- `crates/cgap-verification/src/run.rs` - `run_verification(triples, rules,
  mode) -> Result<GroundedReceipt, IngestError>` (per-rule pass/fail +
  canonical grounded-context bytes, magic `CGVR` v0x01, sorted triples,
  sorted rules, violations against canonical triple order);
  `refusals(receipt, triples) -> Vec<Refusal>` naming rule lexeme + slot +
  constraint; `seal_receipt`.
- `crates/cgap-verification/src/seal.rs` - keyed commitment size-unbounded:
  `seal_grounded(key, ctx) -> SealedContext`, `verify_sealed(key, ctx,
  sealed)`; leaf_i = `cgap_crypto::chatman::keyed_digest(key, chunk_i)`
  (chunks of `KEYED_MAX_INPUT_LEN` = 1024), root =
  `chatman::admissions_root(leaves)`. Wrong key, any byte flip, or length
  change is typed-loud.
- `RuleMode::Disabled` exists ONLY for the anti-vacuity court.

## Courts (all real collaborators, no mocks)

- `court_hallucinated_triple_refusal_names_the_rule` - PASS. A triple
  outside the actor domain yields exactly one typed refusal naming
  `rule/actor-domain` (lexeme + 0x-slot) and constraint "domain"; the clean
  corpus yields zero; the hallucination moves the seal root.
- `court_seal_round_trip_and_tamper_flip` - PASS. Seal -> verify identical;
  wrong key, byte flips at offsets 0/17/mid/last -> `SealError::Tampered`;
  truncation -> `SealError::LengthMismatch`.
- `court_seal_multi_chunk` - PASS. >1024 B grounded context: >=2 keyed
  leaves, round-trips, mid-byte flip -> Tampered.
- `court_rule_evaluation_is_load_bearing` - PASS. `Disabled` seals the
  hallucinated corpus clean (violations_total == 0); `Enabled` fires
  (== 1) and changes the context bytes.
- `court_grounded_context_determinism` - PASS. Identical corpus+rules ->
  byte-identical context; reversed ingestion order -> identical; one added
  triple -> different.
- `court_ingestion_slot_law` - PASS. Terms derive through the real
  `slot_id`; the registry names slots back to lexemes; one lexeme per slot
  across the corpus (CollisionDetector exercised on every term). The
  distinct-lexeme 64-bit Collision refusal path is inherited from the A06
  producer courts (a real collision is infeasible to mint in-process; the
  refusal arm still executes in A06's own tests).
- `court_custom_rule_as_data_and_orphan_exemption` - PASS. A data-only
  custom rule catches the self-loop; orphan exemptions honored; dangling
  object -> orphan violation.

Gate: `CARGO_TARGET_DIR=/tmp/r10m-final-target cargo test --manifest-path
/Users/sac/cgap/crates/cgap-verification/Cargo.toml` -> exit 0, 7 passed.

## Anti-vacuity shim court (/tmp sandbox, sha256 fail-closed)

Sandbox /tmp/r10m-sb/cgap-verification (sha256 baseline run.rs
cc91f4f8b429, seal.rs 72073a5dbe9e, courts.rs 2c7a7845f137; sandbox run.rs
byte-identical to baseline pre-mutation). Mutation (SHIM-COURT):
`RuleMode::Enabled => evaluate(rule, corpus)` replaced with
`Enabled => Vec::new()` in the sandbox copy only. Result: cargo test exit
101, courts `court_hallucinated_triple_refusal_names_the_rule`,
`court_rule_evaluation_is_load_bearing`,
`court_custom_rule_as_data_and_orphan_exemption` FAILED - the gate cannot
pass with rule evaluation disabled; it is not vacuous. Shared-tree files
verified byte-identical post-court (restore check True); sandbox deleted.

## Divergences

- D1: The dispatch lease says "workspace member"; the lane contract makes
  the root Cargo.toml read-only, so the crate carries its own `[workspace]`
  table (crates/cli pattern) and builds via --manifest-path. Coordinator
  one-liner at integration: add "crates/cgap-verification" to root members
  and drop the crate-local [workspace] table.
- D2: Shim-court lesson (recorded for the fleet): a sandbox crate with the
  SAME package name+version built into a SHARED CARGO_TARGET_DIR poisons
  subsequent real-tree runs (cargo reused the mutated rlib; the real gate
  read as 3 failures with disabled-mode behavior). Shim courts must use a
  dedicated target dir; the re-run in a clean dir restores exit 0.
- D3: The OST/Samsung frame is cited from the dispatch text only; no OST
  frame document exists under docs/jira/v26.11.1/ to anchor it.
- D4: `refusals()` re-derives the corpus by re-ingesting (real path, no
  cache); it drops violations whose index is out of corpus bounds (typed
  defensive check, exercised only implicitly).
