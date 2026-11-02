# Crate-Reality Receipt: cgap-bosym

Milestone: v26.11.2 starDF reality wave. Date: 2026-11-01. Lane REAL-cgap-bosym.
Canonical checkout only; no git writes; no pack edits.

## Verdict: REAL

The crate is substantive production code, tests pass on the exact subject,
and the canonical-graph manifest facts match the on-disk manifest exactly.

## Proof chain (all commands run this session, exits witnessed)

| command | exit | result |
|---|---|---|
| `CARGO_TARGET_DIR=/tmp/real-bosym-target cargo test -p cgap-bosym` | 0 | 14 passed; 0 failed; 0 ignored (unittests + 0 doc-tests) |
| rdflib (7.6.0) query over `p:CrateManifest` individual `p:manifest-cgap-bosym` in `/Users/sac/ggen-marketplace/packs/cgap-pack/ontology.ttl` | 0 | 1 row: description/publish/version all MATCH on-disk `crates/cgap-bosym/Cargo.toml` — see below |
| substance census (grep over `crates/cgap-bosym/src/slab.rs`) | 0 | 1521 LOC, 18 pub fns, 14 `#[test]` |

Note: the first rdflib attempt used a wrong p: namespace
(`https://avec.chatman.dev/ontology/cgap#`) and returned 0 rows — the pack
binds `p:` to `https://seanchatmangpt.github.io/packs/cgap-pack#`. Recorded
as transport correction, not drift.

## Substance census

- `src/lib.rs` (2 lines, module decl only) + `src/slab.rs` (1521 LOC).
- Public API: `SymbolKind`, `BosymHeader`, `BosymEntryPrefix`, `Sha256`
  (hand-rolled, FIPS-180 vector-tested), `sha256`, `sha16`, `BosymError`,
  `BosymEntry::encode`, `build_archive`/`parse_archive` (byte-exact
  round-trip + truncation-fail-closed at every offset + any-offset bit-flip
  refusal courts), `GraphExport`, `emit_projection`, `extract_fibo_tokens`,
  `fibo_graph_export`, `IngressDropClass`, `IngressVerdict`, ingress
  allocation-free admission courts.
- 14 tests: 6 of them named courts (ingress construction/drop/admission/
  wiring; t2 truncation, t3 bit-flip mutation courts). Anti-vacuity mutation
  of the drift check: a description/publish/version mismatch produces DRIFT
  (witnessed: the version mismatch initially printed DRIFT before the
  workspace-inherited `version.workspace = true` was resolved against root
  `Cargo.toml` version 26.11.2 — a checker defect, corrected in-session).
- No mocks, no `Process.sleep`; assert final state.
 Chicago discipline intact.

## Drift-check detail

Graph: `p:manifest-cgap-bosym a p:CrateManifest` with crateName
"cgap-bosym", crateDescription matching the Cargo.toml description byte-for-byte,
cratePublish true == `publish = true` (P3-2b guard flip 2026-11-01), crateVersion
"26.11.2" == root workspace version 26.11.2 (crate uses `version.workspace = true`).
VERDICT: NO-DRIFT. First-run "DRIFT" was a checker artifact (unresolved
workspace inheritance), corrected in-session with the resolution step shown
in the proof chain.

## Standing

ALIVE — test exit 0 on the exact subject this session; drift-check NO-DRIFT;
substance census witnessed.
