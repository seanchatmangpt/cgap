# R4 — ggen receipt / portable receipt system (Explore brief)

Lane: R4. Source: `/Users/sac/ggen`. Claims cited file:line.

## Three deliberately coexisting receipt mechanisms (never merged)

1. **BLAKE3 chain receipt** — `.ggen-v2/receipt.json` + `receipt-log.jsonl`,
   `ggen_engine::sync::write_receipt` (crates/ggen-engine/src/sync.rs:245, 3434-3730).
2. **Portable receipt envelope** — SHA-256, cross-engine, RFC-GPACK-001 §54-56,
   `.ggen-v2/receipt-portable.json`, `write_portable_envelope`
   (crates/ggen-engine/src/portable_receipt.rs:500-539); `replay.status=PASS` only via the
   replay court (replay_court.rs:132-202).
3. **Ed25519-signed install receipts** — `.ggen/receipts/pack-*.json`,
   `ggen_config::receipt::Receipt` (crates/ggen-config/src/receipt/receipt_impl.rs:10-24).

Plus: O(1) streaming receipt-verification dialect (receipt_dialect.rs:1-30); loop-level
exact-SHA evidence receipts under receipts/ (e.g. v26.9.22/W0-baseline.json:3 binds base_sha
to exact commit).

## Portable envelope (RFC-GPACK-001)

- Rule: "The envelope REPORTS standing, it does not confer it" (§57; portable_receipt.rs:9-13).
- Fail-closed UNKNOWN: unknowable fields present as "UNKNOWN", never omitted, never invented
  (:15-19, tested :641-649).
- Fields (:219-257): schema/spec consts (:506-507); engine {name, version} (:508-511);
  toolchain (observed rustc/cargo --verbose, [FM-CHAIN-018] on failure :323-338);
  environment (os/arch + variables_sha256 over bounded allowlist, secrets excluded :351-399);
  subject {pack, version, pack_digest} (:119-129); dependencies[] exact transitive closure,
  scope always ["SEMANTICS","LAW","PROJECTION"] (:131-147); composition.resolved_packs[] ALL
  resolved packs sorted so "iteration order cannot hide an unrelated pack" (:149-168);
  graph.canonical_digest (:170-177); admission {gates_attempted, refusals} — "a gate that
  never executed is never reported passed" (:179-188); consequences[] {target, operation,
  sha256}, MANAGED_WRITE/MANAGED_INJECT only, re-hashed off disk post-write, [FM-CHAIN-015]
  fail-closed (:190-199, :272-310); work_order optional (:45-48); replay.status always
  "UNKNOWN" at manufacture (:201-207); standing ALIVE/PARTIAL_ALIVE/REFUSED:<CODE> never
  collapsed (:50-83); generation never synthesized from timestamps (:249-256).

## Identity: exact-SHA pack digest (RFC §37/38)

`pack_digest_sha256` (crates/ggen-engine/src/pack.rs:1127-1195):
`PackDigest = SHA256("ggen-pack-v1\0" || E_1 || ... || E_n)`, `E_i = u64be(|path|) || path ||
u64be(|bytes|) || bytes`. Domain-separated, length-prefixed; files sorted by UTF-8 path bytes;
excludes .git and runtime caches; does NOT cover emitted consequences (:1140-1145). Known
vectors :1502-1530. Legacy receipt BLAKE3, envelope SHA-256 — "NOT byte-equivalent" (§56).

## Two replay courts

**Replay court** (replay_court.rs:130-202): ALIVE + every consequence re-hashes equally →
Pass{verified_at, digests}; absent → REPLAY_ABSENT; mismatch → REPLAY_MISMATCH; unreadable →
REPLAY_UNREADABLE; non-ALIVE → Unknown, never collapses. "No silent pass" (:18-19). Envelope
is Serialize-only; court reads separate Deserialize EnvelopeView — "the engine never reads
back its own claims as authority" (:83-87).

**Clean-state replay GALL-001** (replay.rs:261-311): refuses dry-run; requires portable
receipt [FM-CHAIN-016]; requires standing ALIVE + replay UNKNOWN ("ordinary manufacture cannot
self-promote"); subject must identify exactly one resolved pack [FM-CHAIN-017]; runtime
identity re-observed byte-equal; clears prior MANAGED_WRITE targets (refuses MANAGED_INJECT —
never destructively reconstructs); re-runs real sync; 13-field identity witness
{sha256, equal} per field; writes adjudicated receipt binding source/reconstructed/court
identities.

## Legacy BLAKE3 chain receipt

`SyncReceipt { record: ReceiptRecord, payload: ReceiptPayload }` (sync.rs:294-299). Payload
deny_unknown_fields — injected extra field fails deserialization (:306-311). Fields:
graph_hash, outputs (BLAKE3 off disk), packs, decisions, closure (input binding),
build_provenance, generation (monotonic epoch from log tail). write_receipt (:3656-3730+):
re-hashes every target off disk [FM-CHAIN-008]; chains onto previous head, genesis all-zeros;
"Never extend an unverified head" [FM-CHAIN-009] (:3693-3730); ts_ns fixed 0. Always written
even after partial failure (:1713-1738). ReceiptRecord/ChainRule/verify_chain via
graphlaw::receipt_chain seam (receipt_chain_seam.rs:1-66); signing key GGEN_SIGNING_KEY env or
.ggen/keys/signing.key (keys.rs:6,71,108-116).

## Ed25519 install receipts

Receipt {operation_id, timestamp, input_hashes, output_hashes, signature,
previous_receipt_hash} (receipt_impl.rs:26-44); signs canonical JSON w/ signature blanked
(:157-166); hash = SHA-256 canonical JSON (:131-136). Sabotage tests: empty/garbage signature
must fail ("returning Ok would be Fail-Open", :332-381). ReceiptManager keys .ggen/keys/*.pem,
receipts .ggen/receipts/ (ggen-cli/src/receipt_manager.rs:28-120). CLI: `ggen receipt verify
<path> [--public-key]` (cmds/receipt.rs:3-10). Post-commit hook generates commit-bound receipts
(scripts/ggen-receipt-generate.sh:1-50).

## Validation stack

- Published JSON Schema: crates/ggen-engine/schema/receipt-portable.schema.json (draft
  2020-12, additionalProperties:false, 14 required fields, digest patterns, scope enum).
- Rust lockstep test envelope_serialization_matches_published_schema (:651-749).
- Python court scripts/validate_receipt_schema.py:1-28 (exit 0 only if every court passes).
- E2E witnesses portable_receipt_e2e.rs (happy path :157-247; §57 falsifier: gate-refused sync
  emits envelope reporting REFUSED:GATE_VIOLATION with nothing written :250-268).
- Precedent: receipts must hash actual effect content, not op labels
  (crates/bcinr-pddl/tests/receipt_hashes_effects.rs:1-12).
