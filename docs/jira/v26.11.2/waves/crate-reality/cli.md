# Lane REAL-cli receipt — crates/cli

Date: 2026-11-01. Verdict: **ALIVE**.

## 1. cargo test (own workspace, CARGO_TARGET_DIR=/tmp/real-cli-target)

Total: **42 passed, 0 failed** across 8 suites:

| Suite | Passed |
|---|---|
| src/lib.rs unittests | 12 |
| src/main.rs | 0 |
| bins (doctor/status/verify_v26) | 0 each |
| tests/cli_new_verbs.rs | 23 |
| tests/mutation_kills.rs | 6 |
| tests/zero_code_cli.rs | 1 (`real_cli_refuses_unknown_surface`) |
| doc-tests | 0 |

Exit 0. mutation_kills (anti-vacuity) green: 6/6.

## 2. Real binary execution (bin name `cgap`, per Cargo.toml `[[bin]]`)

`/tmp/real-cli-target/debug/cgap --help` -> exit 0; full 16-command usage
(beam, court, audit, bn3, p28, gate, crypto, arrow, bench, self, parquet,
git, bont, gpu, align, help) + --format/--select/--introspect flags.

`/tmp/real-cli-target/debug/cgap self version` -> stdout `"26.11.2"`, exit 0.
Build root deleted after run (rmtree, verified absent).

## 3. custom_handlers.rs stub sweep

`grep -rn 'todo!\|unimplemented!' src/custom_handlers.rs src/custom_handlers/`
-> **0 matches** (grep exit 1). 4 `fn` items, all real bodies.

## 4. rdflib p:CrateManifest drift-check

cgap-pack ontology.ttl (`~/ggen-marketplace/packs/cgap-pack/ontology.ttl`)
holds 22 p:CrateManifest rows; **no `cgap-cli` row exists** — crates/cli is a
standalone workspace crate, outside the cgap-pack manifest facts. This is a
loud MISSING in the manifest-facts graph, not byte-drift: no fact to compare
means verdict = MISSING-COVERAGE for cgap-cli (not DRIFT). Suggest g1k add a
cgap-cli manifest fact (name cgap-cli, version 26.11.2, publish=true per
release-gates table) if the pack is meant to cover the CLI.

## Commands + exits

- cargo test (own workspace) -> exit 0, 42/42
- cgap --help -> exit 0
- cgap self version -> exit 0, stdout `"26.11.2"`
- grep todo!/unimplemented! custom_handlers -> 0 matches (exit 1)
- rdflib CrateManifest query -> exit 0, 22 rows, 0 cgap-cli

JSON footer:

```json
{"lane":"REAL-cli","files_written":["docs/jira/v26.11.2/waves/crate-reality/cli.md"],
"commands":[["cargo test (crates/cli own ws)","exit 0, 42 pass 0 fail"],
["cgap --help","exit 0"],["cgap self version","exit 0 -> 26.11.2"],
["grep stubs custom_handlers","0 matches"],["rdflib CrateManifest","22 rows, 0 cgap-cli"]],
"courts_run":["mutation_kills 6/6","zero_code_cli real-binary refusal"],
"standing":"ALIVE",
"divergences":["no p:CrateManifest row for cgap-cli in cgap-pack ontology.ttl (loud MISSING, not drift)"]}
```
