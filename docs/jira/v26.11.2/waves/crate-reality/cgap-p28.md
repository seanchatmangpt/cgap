# Crate reality receipt: cgap-p28

Lane: REAL-cgap-p28 · 2026-11-01 · branch main · shared tree untouched (sandbox-only mutation)

## Verdict: PARTIAL — unit-test target does not compile (71 errors, cfg(test)-only)

- `cargo build -p cgap-p28` (non-test lib): **exit 0** — frozen pins compile clean,
  including `const _` offset/size asserts (`ATTESTATION_LEN == 80`,
  `offset_of!(P28LicenseAttestation, attestation_flags) == 6`, `LICENSE_MAGIC = b"P28S"`).
- `cargo test -p cgap-p28` (full): **exit 101 — compile-abort, not a failure count**.
  All 71 errors are E0425/E0433/macro-not-found for `Vec`/`String`/`format!`/`vec!`/`println!`
  inside `#[cfg(test)]` modules of `crates/cgap-p28/src/trap.rs` (33), `src/attestation.rs` (38),
  `src/asc450.rs` (1) — no_std crate test modules missing `use std::vec::Vec` /
  `use std::string::String` imports. **Zero errors in non-test cfg.**
- Integration test targets: **exit 0 — 8 passed, 0 failed**
  (`p28_invariant_courts` 6/6, `lever_courts` 2/2), including `layout_frozen_integration`
  (80 B layout + magic + offset table proven at runtime) and `bit_flip_sweep_640_integration` /
  `sweep_vacuity_probes` (anti-vacuity present and passing).
- 46 `#[test]` unit tests across src/ are **unexecutable until the cfg(test) imports land** —
  dispatch did not authorize source edits; fix is mechanical (~3 import lines in 2 files).

## Anti-vacuity court: FLIPPED

Mutation `ATTESTATION_LEN: 80 -> 81` in /tmp sandbox copy
(`/tmp/p28-sandbox`, workspace-de-parented Cargo.toml, symlinked cgap-blake3/cgap-crypto):
const-assert compile abort E0080 (`assert!(ATTESTATION_LEN == 80)` fails) — court flips
at compile time. Restore verified sha256-exact
(`f1e591023cc924c0`, MATCH) before sandbox deletion; shared tree sha unchanged.

## rdflib p:CrateManifest drift-check: [SKIP] subject missing

No `p:CrateManifest` subject exists in `crates/cgap-p28/` (no ontology dir) or
`ontology/` repo-wide (grep zero matches). Loud skip — exit-2 class, not a fake pass.

## Substance census

| file | LOC | pub fn | #[test] |
|---|---|---|---|
| src/attestation.rs | 1371 | 9 | 15 |
| src/asc450.rs | 2363 | 22 | 19 |
| src/trap.rs | 767 | 1 | 4 |
| src/lib.rs | 21 | 0 | 0 |
| tests/p28_invariant_courts.rs | — | — | 6 |
| tests/lever_courts.rs | — | — | 2 |
| **total** | **4522** | **32** | **46** |

## Commands + exits (this session)

- `cargo build -p cgap-p28` → 0
- `cargo test -p cgap-p28` → 101 (compile-abort, cfg(test) imports missing)
- `cargo test -p cgap-p28 --test p28_invariant_courts --test lever_courts` → 0 (8 passed)
- sandbox mutation run (`cargo test --test p28_invariant_courts` in /tmp/p28-ws) → 101 (E0080 flip)
- sha256 restore check → MATCH; target lease `/tmp/real-p28-target` + sandboxes removed

Standing: PARTIAL. Affects `just doctor` and any workspace-wide `cargo test` gate.
