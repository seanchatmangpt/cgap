# W1-01 Ontology / ggen Projection Surface Audit

Lane W1-01, 2026-11-02, repo /Users/sac/cgap (branch main). Read-only
audit; `ggen sync run --dry-run` attempted, refused pre-write at the
resolve stage (FM-PACK-008) — zero bytes written to generated files.

## Surfaces inventoried

- Root graph: `ontology/cgap.ttl` (20 lines; 9 av: triples, single
  individual `av:project` with 5 CTQ literals).
- Root templates: `gen-templates/version_rs.tmpl` ->
  `crates/cgap-arrow/src/generated_version.rs`;
  `gen-templates/specialization_manifest_rs.tmpl` ->
  `crates/cgap-parquet/src/generated_compiled_manifest.rs`.
- CLI graph: `crates/cli/ontology/cgap-cli.ttl` (256 lines; 1 cnv:Cli,
  15 cnv:Noun, 22 cnv:Command, 22 cnv:CustomBehavior).
- CLI outputs: `src/generated_cli.rs` (22 `#[verb]` routes + 15 noun
  registrations), `src/generated_behavior.rs` (closed interpreter),
  `src/generated_boundaries.rs` (13 fns), `src/custom_handlers.rs`
  (4 local bodies + re-exports from handlers_kernel/reasoning/
  sovereign covering all 22 route names — verified via `pub use`,
  custom_handlers.rs:122-128).
- Locks: root `ggen.lock`, `crates/cli/ggen.lock`.

## Drift findings

D1 DEFECT — stale version attr. `ontology/cgap.ttl:14` pins
`av:version "26.10.29"` while the workspace is at 26.11.1
(`Cargo.toml:39`) and the CLI graph pins `cnv:version "26.11.1"`
(`crates/cli/ontology/cgap-cli.ttl:6`). Projected downstream:
`crates/cgap-arrow/src/generated_version.rs:4` exports
`VERSION = "26.10.29"`. One graph bump + regen closes it.

D2 DEFECT — lock/pack hash mismatch. `ggen sync run --dry-run`
(witnessed this session) refuses at resolve: `ggen.lock` has
`blake3:0a30698a…` but `../ggen-marketplace/packs/cgap-pack` hashes
to `blake3:f79ea484…` (FM-PACK-008). Root regen is BLOCKED until
re-lock; CLI crate was not exercised (its packs are outside the repo
under /Users/sac/ggen-marketplace, untouched per lane ban).

D3 DEFECT — generated output without in-repo individual. The
specialization manifest (generated_compiled_manifest.rs:11-23) claims
derivation from `av:BufferSlot` widths, but `ontology/cgap.ttl`
contains no `av:BufferSlot` individuals; they live only in the
external cgap-pack (hash-drifted per D2). In-repo graph cannot
replay this output alone.

D4 INTENT — include/ headers are hand-maintained C ABI, not
graph-projected: no gen-template targets include/, and
`include/cgap.h:2` self-identifies as the A19 Wave 1 ABI freeze.
No cnv:/av: individuals name them; no drift beyond D1's version
strings appearing nowhere in the headers.

D5 INTENT — `crates/cli/ggen.toml` declares `[templates] dir =
"gen-templates"` but `crates/cli/gen-templates/` does not exist; all
CLI rendering comes from the five clap-noun-verb packs. Stale config
key, harmless at current sync behavior.

D6 [TARGET-v26.11.2] — CLI graph version 26.11.1 vs campaign 26.11.2
bump is expected to move `cnv:version` (cgap-cli.ttl:6) in the
release wave; not a defect today.

## Non-drift (consistency verified)

- 15 nouns / 22 commands in cgap-cli.ttl map 1:1 onto 22 `#[verb]`
  routes and 15 noun registrations in generated_cli.rs; zero
  individuals-without-output, zero outputs-without-individuals.
- All 22 commands are `cnv:CustomBehavior` and resolve to
  custom_handlers re-exports (custom_handlers.rs:122-128).
- CLI `cnv:version` 26.11.1 == cli Cargo.toml version 26.11.1.

## Court status

No mutation court in this lane (read-only audit; mutation of the
shared graph is prohibited). Anti-vacuity for the audit itself: D2 is
a witnessed refusal — the gate fired, exit path captured below.

## Commands (this session)

- `cat ggen.toml; cat crates/cli/ggen.toml` — exit 0
- `ggen sync run --dry-run` (repo root) — refused, FM-PACK-008;
  NOTE: observed via pipe, `EXIT=0` printed is the tail rc, the
  ggen process itself errored (ERROR: CLI execution failed).
- greps/stat over ontology, templates, generated outputs — exit 0

## Standing

PARTIAL — full inventory complete with file:line citations; regen
replay (dry-run) BLOCKED by D2, so projection-identity (graph ==
output bytes) is UNVERIFIED for both roots.

```json
{"lane":"W1-01","files_written":["docs/jira/v26.11.2/waves/w1-01-ontology-audit.md"],
"commands":["cat ggen.toml / crates/cli/ggen.toml (0)",
"ggen sync run --dry-run (FM-PACK-008 refusal, pre-write)",
"grep/stat inventory sweeps (0)"],
"courts_run":[],"standing":"PARTIAL",
"divergences":["D1 stale av:version 26.10.29 vs workspace 26.11.1",
"D2 ggen.lock pack hash mismatch blocks root regen",
"D3 av:BufferSlot individuals only in external pack",
"D4 include/ hand-maintained (INTENT)",
"D5 cli gen-templates dir absent (INTENT)",
"D6 26.11.2 version bump pending (TARGET)"]}
```
