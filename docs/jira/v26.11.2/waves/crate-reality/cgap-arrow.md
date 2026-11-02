# Crate Reality Receipt: cgap-arrow

Lane: REAL-cgap-arrow · 2026-11-01 · repo /Users/sac/cgap (shared tree untouched; no git commands run)

## Verdict: REAL

## 1. Test run (command + exit)

```
CARGO_TARGET_DIR=/tmp/real-arrow-target cargo test -p cgap-arrow   # exit 0
```

Duration: unit suite 12.13s; total wall ~40s cold build + run.

| Suite | Result |
|---|---|
| unittests src/lib.rs | 134 passed, 0 failed |
| tests/mutation_kills.rs | 1 passed, 0 failed (35 filtered) |
| tests/property.rs (generated from tests/corpus) | 1 passed |
| tests/validation.rs (generated) | 6 passed |
| Doc-tests | 0 |

Total 142 passed / 0 failed. Build root /tmp/real-arrow-target deleted after run (lease law).

## 2. rdflib manifest drift-check

Individual `p:manifest-cgap-arrow a p:CrateManifest` in
`~/ggen-marketplace/packs/cgap-pack/ontology.ttl` (canonical Arrow domain graph;
repo `ontology/cgap.ttl` carries no crate facts by design — one graph, one source of truth).

rdflib parse: 1224 triples. Graph subject vs `crates/cgap-arrow/Cargo.toml` + workspace:

| field | graph | Cargo.toml | match |
|---|---|---|---|
| crateName | cgap-arrow | cgap-arrow | yes |
| crateDescription | "Freestanding Apache Arrow columnar substrate: branch-free kernel inner-loop bodies, itemized latches/dispatch (no_std, zero dependencies)." | identical | yes |
| cratePublish | true | true | yes |
| crateVersion | 26.11.2 | workspace 26.11.2 | yes |

DRIFT: NONE.

## 3. Substance

- 28 src files; non-generated LOC: 8,945 (excludes generated_layout.rs, generated_version.rs)
- pub fn count (src): 176
- #[test] count: 134 in src + 43 in tests/ = 177 declared; 142 executed by `cargo test` (corpus-generated suites report 1 entry each)
- Zero deps (confirmed: `dependencies` map empty in Cargo.toml)
- tests/: mutation_kills.rs, property.rs, validation.rs (corpus-derived)

## JSON footer

```json
{"lane":"REAL-cgap-arrow","files_written":["/Users/sac/cgap/docs/jira/v26.11.2/waves/crate-reality/cgap-arrow.md"],
"commands":[["CARGO_TARGET_DIR=/tmp/real-arrow-target cargo test -p cgap-arrow","exit 0 (142 passed)"],
["rdflib manifest drift check","exit 0, DRIFT NONE"],["shutil.rmtree /tmp/real-arrow-target","done"]],
"courts_run":["cargo test -p cgap-arrow (142 tests incl. mutation_kills court)"],
"standing":"REAL","divergences":["ontology/cgap.ttl carries no p:CrateManifest individuals — canonical manifest graph is ~/ggen-marketplace/packs/cgap-pack/ontology.ttl (by that file's own design comment); drift-check performed against it"]}
```
