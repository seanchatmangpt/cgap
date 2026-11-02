# R5 — ggen SHACL validation (Explore brief)

Lane: R5. Sources: `/Users/sac/ggen/crates/ggen-graph/`, `/Users/sac/ggen/crates/ggen-engine/`.
Read-only; claims cited file:line.

## 1. Layers

| Layer | Crate / file | Role |
|---|---|---|
| Library API (per-violation) | `crates/ggen-graph/src/shacl.rs` (224 lines) | `validate_shacl(data, shape_graphs) -> Vec<ShaclViolation>` |
| Engine trait + law gate | `crates/ggen-engine/src/graph.rs:785` (trait), `:1310` (GraphLawStore impl), `:1076` (oxigraph stub = typed refusal) | `validate_shacl(shapes_turtle) -> ShaclOutcome` |
| Kernel helper | `crates/ggen-engine/src/law_engine.rs:248` `shacl_check()` → graphlaw `Step::AdmitShacl`; graphlaw pinned 26.10.13 (`Cargo.toml:140`) |
| Legacy | `crates/ggen-graph/src/dialect.rs:69` `check_shacl()` (deprecated) — single pass/fail DialectResult |

`ggen-graph/src/lib.rs:29-36`: the two-validator split is deliberate; downstream should use
`ggen-graph::shacl::validate_shacl`.

## 2. Shape parsing + enforced subset

- Parser: oxigraph `RdfParser` (Turtle) — shacl.rs:9-12, 42-46; shapes+data into a fresh
  in-memory `oxigraph::store::Store`; constraints evaluated via SPARQL SELECTs (:48-67).
- Enforced subset at sync time (shacl.rs:6-7): `sh:NodeShape` (:83-87), `sh:targetClass`
  (:98-114, falls back to shape IRI), `sh:property`/`sh:path`/`sh:minCount` (:117-137). **No
  closed/pattern/datatype** in this path. Full SHACL semantics come from graphlaw via
  `shacl_check` (law_engine.rs:248-283); `LawError::NotAdmitted` → `ShaclOutcome
  { conforms, violations }` with lines "focus node {}: {msg} (source shape {})" (:266-272).
- Legacy check_shacl implements the same minCount/targetClass subset (dialect.rs:69-200).

## 3. Report shapes

- `ShaclOutcome` (graph.rs:680-686): `{ conforms: bool, violations: Vec<String> }` — flat
  Rust struct, NOT a W3C SHACL report graph (though vocab defines report IRIs,
  `crates/ggen-graph/src/vocab/shacl.rs:127-173`).
- `ShaclViolation` (shacl.rs:28-40): `{ focus, shape, path, message, severity }`; severity
  Violation/Warning/Info (:17-25).
- CLI JSON (verbs/handlers.rs:351-388): `"shapes_conform": true`, `"shapes_checked": N`.

## 4. Sync-time consumption paths

1. **Template frontmatter `shape:`** — primary. `sync.rs:2746 admit_shape_files()`: resolves
   each path (FM-TPL-014 on failure, sync.rs:2759), BLAKE3-hashes into receipt input closure
   (:2800-2805), concatenates into `combined_shapes_ttl` (:2807-2811), runs
   `validate_shacl` (:2816). Non-conformance = typed refusal **FM-TPL-025** (:2830-2841);
   no-SHACL engine = **FM-TPL-026** (:2819-2827). Doc comment (:2729-2745) records this closed
   the 2026-08-03 audit lead "shape: is hashed but never SHACL-validated"; motivating shape
   files: `templates/cli/subcommand/graphs/shapes/cli.shacl.ttl`,
   `templates/api/endpoint/graphs/shapes/api.shacl.ttl`. Called at sync.rs:1310, 1380, 1423.
2. **Manifest `[validation].shacl`** — existence-validated fail-closed
   (`crates/ggen-config/src/manifest/validation.rs:439-447`; types.rs:610-615). Note: shapes
   deliberately NOT duplicated as `law.shapes` on the manifest schema (types.rs:152-155);
   `[validation].gates` (SPARQL gate files) is "the shacl successor" (validation.rs:449).
3. **`ggen law validate` verb** — `[law].shapes` from ggen.toml, loaded by `build_law_engine`
   (handlers.rs:1210-1284), materializes rules then runs every shape + denial check; any
   violation is typed FM-LAW refusal (handlers.rs:1307-1332; verb routing generated in
   `verbs/law.rs:29-32`).

Additional consumers: LSP `frontmatter_shacl_analyzer.rs:40-50` → diagnostic
`GGEN_FM_SHACL_001`; `.agent-admissibility/policies/*.shacl.ttl` (3 policy shapes);
`packs/*/shapes/*.shacl.ttl`.

## 5. Frozen IRIs / namespaces

- W3C SHACL namespace pinned: `http://www.w3.org/ns/shacl#` — `Shacl::NAMESPACE`
  (`crates/ggen-graph/src/vocab/shacl.rs:6`), re-exported `vocab::SHACL`
  (vocab/mod.rs:57,59,61). ~40 `NamedNodeRef<'static>` constants (sh:NodeShape, sh:property,
  sh:path, sh:minCount/maxCount, sh:closed, sh:in, sh:hasValue, sh:conform, sh:result*,
  sh:focusNode, sh:severity, sh:Violation/Warning/Info) — vocab/shacl.rs:9-173. Frozen.
- ggen-owned frozen namespaces: `http://ggen.dev/dfg#` (vocab/dfg.rs:10),
  `http://ggen.dev/powl#` (vocab/powl.rs:13); owl/rdfs/rdf/skos/prov/xsd/ocel modules
  (vocab/mod.rs:10-61). Shape docs pin `https://ggen.io/ontology/boundary-refusal#`
  (`ontologies/shapes/boundary-refusal.shacl.ttl:8,27`) and `http://ggen.io/pack-dialect#`
  (`ontology/shapes/pack-dialects.shacl.ttl:2`).
- Admissibility policy ns: `https://ggen.dev/ns/admissibility#`
  (`.agent-admissibility/policies/{no-private-namespace,public-vocab}.shacl.ttl:3`).
- **cgap agreement**: cgap's `crates/cgap-bont/src/shacl_compiler.rs:252-255` pins the same
  canonical `http://www.w3.org/ns/shacl#` and `sh:NodeShape` IRI (v26.10.31 pins) — byte-identical
  to ggen's `Shacl::NAMESPACE`. ggen's pack-dialects.shacl.ttl:31-45 pins
  sh:closed/sh:in/sh:datatype/sh:targetClass/sh:targetObjectsOf; envelope-gate pre-check before
  SHACL runs (boundary-refusal registry L13, law_engine.rs:401-411).

## 6. Cautions

- ggen-graph validator = minCount/targetClass ONLY; do not describe as full SHACL — full
  semantics live in graphlaw 26.10.13 inside ggen-engine.
- `[validation].gates` (SPARQL gates) is the labeled successor (validation.rs:449).
- oxigraph engine refuses SHACL (`fm_law(2)`, graph.rs:1076-1082); SHACL requires the default
  GraphLaw engine (remediation text sync.rs:2821-2826 / FM-TPL-026).
- Generated `crates/ggen-engine/src/generated/refusals.rs:2` names
  `ontologies/shapes/boundary-refusal.shacl.ttl` as source of truth (L13 registry,
  SPARQL surface-prefix closure :13-24).
