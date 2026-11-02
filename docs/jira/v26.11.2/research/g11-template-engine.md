# R11 — ggen template/codegen engine (Explore brief)

Lane: R11. Source: `/Users/sac/ggen`. Claims cited file:line.

## Layout

Engine crates/ggen-engine/src/: sync.rs (4,600), template.rs (1,836), write.rs (1,573),
pack.rs, frontmatter_rdf.rs, canonical/mod.rs, generation_rules.rs. CLI's generated_commands.rs
is itself generated. Workspace 26.10.12.

## Template syntax (Hygen semantics, Tera body)

Template = `---` YAML frontmatter + Tera body (template.rs:3-4; parse :403-428). Frontmatter
closed key set — deny_unknown_fields (:41-43); codes [FM-TPL-008] unterminated block, [FM-TPL-012]
delimiter, unknown-key refusal with full key list echoed (:437).

25-field Frontmatter (:43-196), load-bearing:
- `to:` output path, Tera-renderable; `{{ var }}` triggers per-row fan-out (one file per SPARQL
  row; same-path collision FM-WRITE-008) (:44-45).
- `sparql:` named queries (bare→default, sequence→query_0.., map), BTreeMap (:53-54).
- `for_each:` explicit query governing projection cardinality (:60-61).
- `when:` ASK guard (:101-102); `construct:` CONSTRUCT feed (:63-64).
- inject/before/after/at_line/skip_if/unless_exists/force/backup — Hygen write semantics
  (:66-123; decision table write.rs:1-24, ambiguous states fail closed).
- `shape:` SHACL shapes validated against rendered output (:124-134); frontmatter itself
  SHACL-validated (frontmatter_rdf.rs:134-165, FM-TPL-023).
- `determinism: true` — enforced not declared: renders twice, refuses on byte mismatch
  (:135-139; enforcement sync.rs:1157-1200).
- `freeze_policy:` Never / Always (one-time scaffold) / Checksum (skip on BLAKE3 divergence
  vs freeze slot; :371-390, freeze_slots_dir :144-148).
- `rdf:`/`rdf_inline:`/`prefixes:`/`base:` per-template overlay graph, never mutating the
  shared project graph (:149-174).
- sh_before/sh_after shell hooks, denylist-refused (:110-119).

Frontmatter facts projected to RDF (fm: ns) and SHACL-validated (frontmatter_rdf.rs:73-112).

## RDF individuals drive generation (five stages)

1. Pack ontologies + consumer ontology.ttl unioned into one graph.
2. Stage-0 gates (gates/*.rq SPARQL SELECT): non-empty result refuses sync before
   Extract/Render/Write; filename-sorted order.
3. Extract: named SELECT queries against graph; when: ASK decides skip; rows become Tera
   context (sync.rs:1243, :2216-2330).
4. Render: rows → Tera; to: fan-out writes one file per row; row-varying paths on static to:
   refused (sync.rs:2445-2448).
5. ORDER BY in pack queries fixes output ordering (e.g. generated_cli.rs.tmpl:34
   ORDER BY ?noun ?verb).

## Determinism

- BTreeMap/BTreeSet everywhere (sync.rs:24, 233-315, 513); named SPARQL results BTreeMap
  (:2268, :2316); explicit .sort() on gates (:1980), pack mounts, globs (:1496, 2039, 2096,
  2665, 3969); gate-precedes-writes deterministic (:813-814, 875, 1987-1988, 2037-2039).
- determinism: true double-render (sync.rs:1190-1200).
- Receipts ts_ns fixed 0 (:15); receipt epoch monotonic (:322, FM-WRITE-012 stale refusal).
- Canonical serialization (canonical/mod.rs): RDFC-10 N-Quads lexicographic, recursively
  sorted-key compact JSON (single workspace canonicalizer in ggen-config, deliberately not
  re-implemented :17-19), BTree-ordered collections (:25-38); CanonicalForm enum with
  domain-separated BLAKE3 (template.rs:200-224).
- Fortune-5 caps: MAX_QUERY_RESULT_ROWS, MAX_PENDING_OUTPUTS, MAX_OUTPUT_BYTES,
  MAX_TOTAL_RENDERED_BYTES (sync.rs:247, 521, 1564).
- ggen.lock deterministic content-hash pack lockfile (pack.rs:1248-1265; :1394).

## Pack structure

Contract (docs/pack-authoring/reference.md:3-8): pack.toml required ([pack] name/version/
description only); ontology.ttl required (FM-PACK-004); templates/*.tmpl ≥1; gates/*.rq
optional. Auto-discovery: pack.toml carries NO per-template list — resolve_pack_dir
auto-discovers every templates/*.tmpl on disk (pack.rs; praxis-core-pack/pack.toml:4).
Lockfile participation per-pack opt-in (pack.rs:78-82).

## clap-noun-verb six-pack

| Stage | Pack | Output |
|---|---|---|
| Schema | clap-noun-verb-schema-pack | cnv:Cli/Noun/Command/Argument/Behavior/Boundary law, 16 fail-closed gates 010_required.rq…160_generated_symbols.rq; NO individuals (composing can't alter consumer surface) |
| Crate | crate-pack | Cargo.toml, main.rs, lib.rs, custom_handlers.rs scaffold |
| Routing | routing-pack | src/generated_cli.rs — typed routes from cnv:Argument/Behavior rows, closed BehaviorSpec dispatch |
| Behavior | behavior-pack | generated_behavior.rs — closed interpreter (static-json/echo/expression/fs-write/fs-list/refusal), no dynamic seam |
| Boundary | boundary-pack | generated_boundaries.rs — rooted fs adapter, symlink refusal, sorted reads, BLAKE3 receipts |
| Verification | verification-pack | compiled-binary tests + command-reference doc; tests exercise success/refusal/arg-rejection/unknown-command/fs actuation/BLAKE3 receipt binding without mocks |

## "Never hand-edit" — three distinct mechanisms

1. Generated-only artifacts carry `//! GENERATED` headers baked into template bodies
   (generated_cli.rs.tmpl:87; crates/ggen-cli/src/generated_commands.rs:1-4; verbs/sync.rs:4).
2. Generated-once scaffolding (cnv:CustomBehavior → custom_handlers.rs): unless_exists: true +
   skip_empty: true (custom_handlers.rs.tmpl:1-5); header "GENERATED ONCE... never
   overwritten... the sole consumer handler seam" (:17-25). Write decision: unless_exists &&
   exists → Skipped (write.rs:108-111). cnv:CustomBehavior admitted by closed gate
   gates/090_behavior_kind.rq (schema-pack/ontology.ttl:58-59).
3. Frozen artifacts: freeze_policy: checksum records BLAKE3 per write; human edit → next sync
   SKIPS (not clobbers) with named mismatch (write.rs:19-28). ggen.lock headed
   "# ggen.lock — generated by 'ggen sync'. Do not edit." (pack.rs:1394).
