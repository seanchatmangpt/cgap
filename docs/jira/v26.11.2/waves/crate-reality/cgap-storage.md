# Lane REAL-cgap-storage — reality receipt

Date: 2026-11-01. Subject: `/Users/sac/cgap/crates/cgap-storage` (canonical checkout, main).

## 1. cargo test

`CARGO_TARGET_DIR=/tmp/real-storage-target cargo test -p cgap-storage` — **exit 0**.

| suite | result |
|---|---|
| unittests src/lib.rs | 5 passed, 0 failed |
| tests/court_crc_identity.rs | 3 passed |
| tests/court_faults.rs | 5 passed |
| tests/court_flash.rs | 3 passed |
| tests/court_mutations.rs | 6 passed |
| tests/court_sweep.rs | 2 passed |
| doc-tests | 0 |

**24 passed / 0 failed** across 6 suites, zero ignored.

## 2. rdflib p:CrateManifest drift-check

Facts live in `~/ggen-marketplace/packs/cgap-pack/ontology.ttl`
(IRI base `https://seanchatmangpt.github.io/packs/cgap-pack#`), NOT `ontology/cgap.ttl`
(cgap.ttl has zero p:CrateManifest rows — divergence noted by the earlier wave receipt
`verification.md`).

Query: `?c a p:CrateManifest ; p:crateName "cgap-storage"` → 1 row:

- crateName `cgap-storage` — byte-match Cargo.toml `name` — MATCH
- crateVersion `26.11.2` — Cargo.toml `version.workspace = true`; workspace
  `[workspace.package] version = "26.11.2"` — MATCH
- crateDescription exact string byte-match of Cargo.toml `description` — MATCH
- cratePublish `true` — Cargo.toml `publish = true` — MATCH

**NO DRIFT** (4/4 fields).

Anti-vacuity (shim-court, /tmp sandbox copy, shared tree untouched):
first mutation (dep versions) did NOT flip the projected-field check — vacuous, discarded.
Second mutation (`WAL-framed` → `MUTATED-WAL-framed` in description) → drift-check
returned False = **FLIPPED**. Restore sha256-exact, RESTORED=YES.

## 3. Substance

- LOC: 1,336 total src (`atomic_commit.rs` 1,035; `dred.rs` 298; `lib.rs` 3)
- `pub fn`: 25 (18 atomic_commit + 7 dred)
- tests: 19 `#[test]` (5 lib inline + 14 integration across 5 court files) → 24 executed incl. multi-assert

## 4. Verdict

**REAL / ALIVE.** Test gate exit 0, drift-check clean, court non-vacuity proven by a
description-field mutation flip with sha256-exact restore. Build root
`/tmp/real-storage-target` and sandbox deleted.

## JSON

```json
{"lane":"REAL-cgap-storage","files_written":["docs/jira/v26.11.2/waves/crate-reality/cgap-storage.md"],
"commands":[["CARGO_TARGET_DIR=/tmp/real-storage-target cargo test -p cgap-storage",0],
["rdflib p:CrateManifest drift-check (cgap-pack ontology.ttl)",0],
["shim-court description mutation (sandbox)",0],["sandbox sha256 restore",0]],
"courts_run":["cargo test gate (exit 0, 24/24)","p:CrateManifest drift-check (4/4 MATCH, NO DRIFT)",
"description-mutation shim-court (FLIPPED, RESTORED=YES)"],
"standing":"REAL/ALIVE","divergences":["p:CrateManifest facts in ggen-marketplace cgap-pack ontology.ttl, not ontology/cgap.ttl"]}
```
