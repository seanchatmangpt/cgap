# RX-5 — RFC-0089 §8.4 CBMQS Qualification Certificate

Subject: `/Users/sac/cgap/crates/cgap-aoq/tests/rfc0089_certificate.rs`
Standing: ALIVE (ran + passed this session, commands + exits below)
Date: 2026-11-03

## Diff

New integration-test court file, self-contained (no lib changes):

- `crates/cgap-aoq/tests/rfc0089_certificate.rs` — §8.4 certificate
  machinery + 5 courts. Input surface: verdict table with RX-3's verdict
  shapes. RX-3's standalone scorecard artifact had NOT landed at time of
  work (no scorecard file in repo), so per the lane contract the table is
  synthesized from the landed per-gate courts' shapes
  (`Status::{Pass, Fail, Skip}` / `GateOutcome`, cbmqs_gates.rs) — the
  green table cites the landed receipt numbers (ab-6 86 ns, ab-9 0 B/op,
  ab-19 12.2M/s). If RX-3 lands later, `VerdictTable` is its composition
  seam: feed its rows into `issue()`.
- `docs/jira/v26.11.3/receipts/rx-5-certificate.md` — this receipt.

## What the certificate is (§8.4)

Canonical JSON, fixed key order, no whitespace:

```
{spec, run_id, timestamp_source, run_seq, input_hash, gates[], overall,
 evidence_receipts[]}
```

Sealed with `cgap_blake3::hash` over the canonical JSON, hex-encoded
(64-char seal). Kernel law honored: `timestamp_source: "NONE"` — no wall
clock anywhere; identity = `run_seq` (run-sequence number) + BLAKE3
`input_hash` of the verdict table (and `run_id` derives from the input
hash prefix).

## Laws under court

- §6 disqualification law: `issue()` is fail-closed — any SKIP-class
  gate returns typed `CertificateIncomplete { skipped_gates }` before
  anything is computed; no certificate issues. SKIP+FAIL mix also
  refuses (SKIP check precedes all).
- §8.4 tamper law: `verify(cert, seal)` recomputes the BLAKE3 seal over
  the certificate AS IT EXISTS NOW; any post-seal mutation changes the
  canonical JSON → typed `SealMismatch { expected, observed }`.
- DISQUALIFIED is a valid certificate (it states a fact): it issues,
  seals, and verifies — overall DISQUALIFIED, failing gate named in the
  body.

## Gates (exit records)

Command, run twice this session from `/Users/sac/cgap`:

```
cargo test -p cgap-aoq --test rfc0089_certificate
```

Run 1: `test result: ok. 5 passed; 0 failed; 0 ignored` — exit 0.
Run 2: `test result: ok. 5 passed; 0 failed; 0 ignored` — exit 0.

Courts (each names its anti-vacuity mutation):

1. `court_pass_certificate_on_green_run` — PASS certificate on the
   green 7-gate table: issues, seals, verifies clean; deterministic
   (same table + receipts → same seal); `run_seq` 7→8 changes the seal
   (run-seq identity is load-bearing, not decorative).
2. `court_disqualified_certificate_on_injected_failure` —
   sandbox-injected G5 Fail mutation flips overall to DISQUALIFIED;
   seal differs from the green-run seal at the same seq; cross-verify
   against the green seal is a typed mismatch; the injected failure
   reason appears in the canonical body.
3. `court_tamper_any_field_mutation_refused` — 7 post-seal field
   mutations witnessed (gate verdict flip, measured edit, overall flip,
   run_seq bump, evidence_receipts push, input_hash swap, spec change),
   each → `verify` typed refusal; altered seal also refused. Mutation
   count asserted load-bearing (`assert_eq!(witnessed, 7)`).
4. `court_skip_gate_fails_closed` — G6 → Skip → typed
   `CertificateIncomplete` naming `["G6"]`; SKIP+FAIL mix also refuses;
   refusal is idempotent.
5. `court_non_vacuity_canonical_json_and_identity` — canonical JSON
   deterministic; contains `"timestamp_source":"NONE"`, no
   `SystemTime`/`Instant`; body-seal binding witnessed.

## Verification ladder

- narrow: the 5 courts above (green x2).
- regression guard: file is an isolated integration test — no lib or
  shared-file edits, zero collision surface with other lanes.

## Notes / deviations

- §8.4 spec text was not found in-repo (no RFC-0089 document file);
  the certificate shape was built exactly to the lane contract's field
  list (spec/run_id/timestamp_source NONE/run-seq + content hashes/
  gates/overall/evidence_receipts → BLAKE3 → hex seal).
- Git: never run (lane law). Coordinator owns transitions.
- Write-corruption incident note: three long-file Writes corrupted
  before the clean fourth (known ~1-in-5 agent long-write failure,
  repo CLAUDE.md); on-disk bytes verified via grep + compile before
  trusting the file.
