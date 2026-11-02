# CLI-2: real handler bodies behind the 17 new CLI verbs

## Lane
CLI-2 (lane-runner), repo /Users/sac/cgap, 2026-11-02.

## Files written (lease)
- `crates/cli/src/custom_handlers.rs` (stub block replaced with module
  registration + re-exports; the 4 original handlers untouched)
- `crates/cli/src/custom_handlers/handlers_kernel.rs` (NEW)
- `crates/cli/src/custom_handlers/handlers_reasoning.rs` (NEW)
- `crates/cli/src/custom_handlers/handlers_sovereign.rs` (NEW)
- `docs/jira/v26.11.1/waves/cli-2-handlers.md` (this receipt)

## Handler table (verb -> owner API -> smoke-run exit, this session)
Binary: /tmp/c2-target/debug/cgap (CARGO_TARGET_DIR=/tmp/c2-target).

| verb | owner API called | in-handler final-state asserts | smoke exit |
|---|---|---|witness|
|---|---|---|---|
| bont slot-lookup | cgap_bont::primitives::{slot_id, lex_hash, truncate64, CollisionDetector::insert} + cgap_blake3::hash independent re-derivation | slot == rederived, raw BLAKE3 path agrees, detector len 1 | 0 |
| bont pack-verify | slot_id x8 lexemes + chain digest via cgap_blake3, computed twice | every slot re-derived; chain digest byte-equal twice | 0 |
| bn3 unify | cgap_bn3::unify::{RuleCompiler::compile, unify_batch} + sigil::encode_term | 3 matches on 4 rows; var0 bound to subject words | 0 |
| bn3 eval | cgap_bn3::builtins::dispatch (OP_MATH_SUM/OP_LOG_TRACE/OP_LOG_COUNTERS) | R0=12, trace=[12], counters latch 3 | 0 |
| gate dam-ingest | cgap_gate::penstock::ingress over a hand-built 160 B A04 frame (pub consts) | CGAP_OK on valid; corrupt magic -> 0x43470001 | 0 |
| parquet pack-emit | cgap_parquet::footer::{FooterSealer::new/seal, verify_seal} | verify_seal(body=128, rows=3) | 0 |
| parquet pack-check | verify_seal + one-bit tamper | tamper -> typed SealError (flip witnessed) | 0 |
| p28 attest-verify | cgap_p28::attestation::{to_bytes, from_bytes, validate_intrinsic} + frozen URN | 80 B wire roundtrip; over-ceiling refused | 0 |
| crypto seal | cgap_crypto::chatman::{DeltaS, seal_unsealed, verify_unsealed, admissions_root} + lease::Lease | tick advanced exactly 1; 229 B wire roundtrip | 0 |
| crypto receipt-verify | UnsealedReceipt::from_bytes + verify_unsealed + one-bit tamper | decode/verify refuse the tampered wire copy | 0 |
| audit powl-stats | cgap_audit::powl::verify over real Ocel2ColumnarLog | Sequence(A,B,C) conforms; (2,1,3) log flips to violation | 0 |
| audit ocel-stats | Ocel2ColumnarLog::append + typed refusals | id-regression + reserved-activity-0 both typed-refused | 0 |
| git dag-verify | cgap_git::dag::{commit_blocks, lineage_proof, verify_lineage} | seq=2 proof verifies; wrong parent refused | 0 |
| arrow slab-stats | cgap_arrow::slab::ArrowRecordSlab push/seal/from_bytes | 8 pushes, 4160 B sealed, byte-exact reload | 0 |
| gpu info | cgap_gpu::MetalSharedBuffer::from_buf on live Buf | Some(4096 B) observed (Metal available) | 0 |
| beam info | cgap_beam (see divergence 1) | real harness counts (2 passed / 0 failed) | 0 |
| court verdict-run | `cargo test -p cgap-court --test court_verdict` subprocess | 14 passed / 0 failed, exit 0, 28 s | 0 |

## Verification (this session)
- `cargo build` (crates/cli, CARGO_TARGET_DIR=/tmp/c2-target): exit 0;
  zero warnings from handler code (3 manifest-level warnings pre-date
  this lane: unused clap-noun-verb patch + cgap-court missing lib target).
- 17/17 verbs smoke-run exit 0 with real measured output above.

## Divergences / disclosures
1. **beam info cannot link the crate**: cgap-beam is `crate-type =
   ["cdylib"]` (no rlib) and the CLI lib has `#![forbid(unsafe_code)]`,
   so neither dependency linking nor a `#[path]` source include compiles
   (both witnessed E0433/E0453). The handler runs the beam crate's own
   real SPSC ring tests as a subprocess instead
   (`cargo test -p cgap-beam spsc`: 2 passed / 0 failed, exit 0) — a real
   collaborator, no mocks. A one-line upstream fix is `crate-type =
   ["cdylib", "rlib"]` on cgap-beam (owner lane's call, not this lease).
2. cgap-court likewise has no lib target; `court verdict-run` therefore
   also goes through the real test harness subprocess (by design).
3. bn3 eval fix during smoke: OP_LOG_TRACE reads `regs[dst]`; the trace
   dispatch must target R0 (the sum's destination), not a fresh register.
4. The kernel module files live under `src/custom_handlers/` (module
   directory of the seam file), not `src/` — same lease, different dir.
5. Receipt-listing typo guard: third file is
   `crates/cli/src/custom_handlers/handlers_sovereign.rs` (NEW).
