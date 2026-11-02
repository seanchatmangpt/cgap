# Crate-Reality Receipt: cgap-bont

Lane: REAL-cgap-bont (reality wave). Date: 2026-11-01. Repo: /Users/sac/cgap (canonical checkout; no git writes; /Users/sac/stardf untouched).

## Verdict: REAL

`cargo test -p cgap-bont` runs and passes against the exact on-disk crate.

## Commands + exits (this session, exact subjects)

| command | exit | result |
|---|---|---|
| `CARGO_TARGET_DIR=/tmp/real-bont-target cargo test -p cgap-bont` | 0 | 35 passed; 0 failed; 0 ignored (unit 35 + doctests 0); 3.39s |
| rdflib drift-check of pass witness + gate 110 SPARQL | 0 | see below |
| `python3 -c "shutil.rmtree('/tmp/real-bont-target')"` | 0 | build-root lease deleted |

## Substance

| file | LOC | pub fns | tests |
|---|---|---|---|
| src/lib.rs | 4 | 0 | 0 |
| src/oabi.rs | 747 | 8 | 6 |
| src/primitives.rs | 998 | 7 | 9 |
| src/shacl_compiler.rs | 3142 | 10 | 20 |
| total | 4891 | 25 pub fns | 35 tests |

106 fn/struct/enum items in shacl_compiler.rs; zero-dep except cgap-blake3
(path dep). Real collaborators (golden tables, sandbox flip courts named
`*_m*_flips`), no mocks.

## rdflib p:CrateManifest drift-check

Method: rdflib in-process SPARQL over
`packs/cgap-pack/witnesses/pass/110_manifest_fact_coverage.ttl` (correct
namespace `https://seanchatmangpt.github.io/packs/cgap-pack#`), compared via
tomllib against `crates/cgap-bont/Cargo.toml` + root `[workspace.package]`.

| fact | witness | Cargo.toml | drift |
|---|---|---|---|
| crateVersion | 26.11.2 | 26.11.2 | none |
| cratePublish | true | true | none |
| crateDescription | "cgap-bont manifest." | full B-ONT description | **DRIFT** |
| ontology.ttl (§8) | full B-ONT description | full B-ONT description | none |

Finding: the gate-110 PASS witness file carries a placeholder description
("cgap-bont manifest.") that drifts from the real manifest; the canonical
pack `ontology.ttl` §8 matches the real description byte-for-byte, and
`generated/manifest-facts.txt` projects the real description. Gate 110 itself
only requires crateName AND crateVersion, so it does not catch description
drift — gate_rows_pass_witness = 0 (gate satisfied), and the G1K-1 receipt's
fail-witness flip `[("cgap-gpu",)]` covers the version channel only. Low
severity (canonical graph is accurate; drift is witness-file-only), recorded
as a divergence, not fixed (out-of-lane file).

## Divergences

1. Witness-file description placeholder drift (above) — canonical graph clean.
2. First rdflib query returned 0 rows via the wrong prefix
   (`http://example.org/p#`); rerun with the file's actual prefix. Recorded
   so nobody reads the zero-row result as gate failure.

## JSON receipt

```json
{"lane":"REAL-cgap-bont","files_written":["/Users/sac/cgap/docs/jira/v26.11.2/waves/crate-reality/cgap-bont.md"],"commands":["CARGO_TARGET_DIR=/tmp/real-bont-target cargo test -p cgap-bont (exit 0, 35 passed)","rdflib drift-check pass witness vs Cargo.toml (exit 0)","shutil.rmtree /tmp/real-bont-target (exit 0)"],"courts_run":["cargo test -p cgap-bont 35-test suite incl. *_flips mutation courts"],"standing":"REAL — tests run+pass, substance 4891 LOC / 25 pub fns / 35 tests, manifest facts match canonical graph","divergences":["gate-110 pass-witness file has placeholder crateDescription (canonical ontology.ttl accurate)","wrong-prefix first query returned 0 rows, rerun with actual prefix"]}
```
