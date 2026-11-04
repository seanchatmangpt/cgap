# ZP-14 — CBMQS-CLI Standalone Crate Gap-Close Receipt

Date: 2026-11-04. Lane: GAP-CLOSE. Status: ALIVE (build, tests, and a real
emit/verify/tamper cycle re-run this session in /tmp).

## ZP-14 vs ZP-21 resolution

Resolution: the standalone crate won. The CBMQS RFC-0089 §8.4 certificate
emit/verify tool lives as its own std host crate `crates/cbmqs-cli`, not as
a noun inside the ggen CLI (`crates/cli`) and not inside the embedded
workspace surface. The RFC-0089 certificate kernel sits in `cgap-aoq`
behind the `cert` feature, default-off:

- `crates/cgap-aoq/Cargo.toml:9-16` — "ZP-21: cert feature gates the
  alloc-using RFC-0089 §8.4 certificate module (no new dependencies —
  digest substrate injected). `[features] cert = []`" — default = empty,
  preserving the no-dep/no-alloc embedded surface.
- `crates/cgap-aoq/src/lib.rs:30-37` — `#[cfg(feature = "cert")] pub mod
  rfc0089_certificate;`.
- The opt-in happens only at the consumer: `crates/cbmqs-cli/Cargo.toml:15`
  `cgap-aoq = { path = "../cgap-aoq", features = ["cert"] }`.
- Disclosure: the string "ZP-14" has zero occurrences in the cgap repo
  (verified by exhaustive grep this session); the lane narrative is
  session-only, cited here as PARTIAL. The on-disk artifacts above are the
  resolution's surviving evidence and are ALIVE as file facts.

## Envelope schema

Kernel side — `crates/cgap-aoq/src/rfc0089_certificate.rs:240-248`:
canonical JSON with fixed key order, no whitespace; `schema_version` is the
FIRST key (contract law); `canonical_form` / `seal_algorithm` live INSIDE
the sealed body so the seal binds the schema identity itself;
`timestamp_source` stays `"NONE"` (kernel law: no wall clock).

CLI side — `crates/cbmqs-cli/src/cert_io.rs`:

- `:12-14` `SCHEMA_VERSION = 1`, `CANONICAL_FORM = "cbmqs-certificate-v1"`,
  `SEAL_ALGORITHM = "BLAKE3-256-hex"`.
- `:78-97` `envelope_json` — "the file bytes ARE the sealed canonical
  string"; `:98-100` `seal_over` — "Seal = hex(BLAKE3-256 over the
  canonical envelope bytes)".
- `:33-40` typed refusal `RunSeqCollision { run_seq,
  presented_input_hash, derived_input_hash }`; Display at `:71-76`
  ("REFUSED RunSeqCollision: run_seq {n} presented input_hash ... but the
  §6-derived table carries ...").

## RunSeqCollision

Identity law verified in test: `crates/cbmqs-cli/tests/
certificate_tamper_court.rs` — `court_run_seq_collision_law_typed` PASS
(this session). The verify path checks schema-key refusal, then sidecar,
then RunSeqCollision, then seal — in that order
(`crates/cbmqs-cli/src/main.rs:282+` doc comment).

## C7 / C8 / C9 outcomes

The C7/C8/C9 labels themselves are session-only (no on-disk file names
them — PARTIAL, disclosed). The on-disk outcomes they map to, all ALIVE:

- Parity court: `tests/certificate_parity_court.rs`
  (1 test, PASS, 4.92s this session).
- Skip court: `tests/certificate_skip_court.rs` —
  `court_min_of_3_strip_fails_closed_no_artifact` PASS (exit-3 loud skip
  writes no artifact; `main.rs:243-248` "SKIP CertificateIncomplete:
  skipped gates {:?} -- no certificate written").
- Tamper court: `tests/certificate_tamper_court.rs` —
  `court_tamper_four_mutations_each_flip_verify_to_exit_2` PASS (this
  session).

## Honest DISQUALIFIED emit

- `crates/cbmqs-cli/src/main.rs:12-13` — "The seal binds the BODY, not the
  outcome: a DISQUALIFIED certificate verifies fine."
- This session, real run in /tmp (`target/debug/cbmqs-audit`):
  `--emit-certificate ... --run-seq 2 --evidence p.txt` -> exit 0, stdout
  `EMITTED ... (schema_version 1, canonical_form cbmqs-certificate-v1,
  seal_algorithm BLAKE3-256-hex, run_seq 2, overall Disqualified)`; the
  file itself carries `"overall":"DISQUALIFIED"` (grep-witnessed). The
  emit is honest: it never upgrades the verdict to pass the gate.

## Full emit/verify/tamper cycle (ALIVE — this session, /tmp dirs)

Commands run against the debug binary, exits observed:

1. `cbmqs-audit --emit-certificate $D/c.json --run-seq 1 --evidence
   p.txt` -> exit 0; wrote `c.json` + `c.json.seal`.
2. `cbmqs-audit --verify-certificate $D/c.json` -> exit 0, stdout
   `VERIFIED ... (run_seq 1, input_hash f67c044c00de09d98dcae5acb0e36d
   0c19cfaf4d18df341ddc4d0a9b403fc76b)`.
3. Tamper: one byte flipped in `c.json` ->
   exit 2, stdout `REFUSED canonical_form unknown:
   fixed-declaration-order-v1 (known: cbmqs-certificate-v1)` — typed,
   fail-closed, schema-key refusal firing before anything else.

## V1-vs-CG7 last-writer reconciliation

CG7's inline §6 law in `crates/cbmqs-cli/src/main.rs` is FINAL:
`fn g7_leg_b_from_receipt` (`main.rs:109-149`) parses the AB-14 receipt's
"50k span" line from
`/Users/sac/ggen/docs/jira/v26.11.3/receipts/ab-14-chain-scale.md`
(const at `main.rs:39-40`), requires the min-of-3 disclosure (`:130`),
computes `ms * 1000.0 / 50_000.0`, maps <= 25.0 -> Pass else Fail;
unparsable -> Skip. `scorecard_table` (`:151-160`) is "derived from the
scorecard's §6 source ... NOT a hand-encoded verdict table."

The earlier V1 form (`receipt_parse.rs`) is superseded and no longer
exists: `crates/cbmqs-cli/src/` contains only `main.rs` and `cert_io.rs`
(ls-witnessed this session); repo-wide grep for `receipt_parse` returns
nothing. Whichever lane wrote last, the surviving §6 implementation is the
inline main.rs law; any competing receipt-parsing file is deleted, and
this receipt treats main.rs as the single authority.

## Build/test receipts (ALIVE — this session)

- `cargo build -p cbmqs-cli` -> exit 0 ("Finished dev profile").
- `cargo test -p cbmqs-cli` -> exit 0: 0 unit + parity 1 + skip 1 + tamper
  2 = 4 passed, 0 failed across 4 test binaries.

## Standing summary

- Build, courts, cycle, DISQUALIFIED emit, schema, RunSeqCollision:
  ALIVE (commands + exits above, this session).
- ZP-14/C7/C8/C9 lane labels and the ZP-14/21 deliberation narrative:
  PARTIAL (session-only; on-disk artifacts corroborate the outcome).
- Nothing BLOCKED; `publish = false` on the crate
  (`Cargo.toml:7`), so no registry surface is claimed.
