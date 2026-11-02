# R2 — ggen.toml manifest schema (Explore brief)

Lane: R2. Sources: `/Users/sac/ggen/crates/ggen-config/src/manifest/`,
`/Users/sac/ggen/crates/ggen-engine/src/`. Read-only; claims cited file:line.

## 0. Two schemas, not one

`ggen.toml` is parsed by one of two independently-defined, deliberately-incompatible typed
schemas, chosen by a structural classifier on raw TOML before any typed parse:

- **Declarative-rules**: `ggen_config::manifest::GgenManifest`
  (`crates/ggen-config/src/manifest/types.rs:173`). `[[generation.rules]]`, `[[packs]]`
  array-of-tables, rules-only `[law]`.
- **Frontmatter**: `ggen_engine::config::GgenConfig`
  (`crates/ggen-engine/src/config.rs:45`). `[templates].dir`-driven, `[packs]`
  table-of-tables, `Law{rules,shapes,gates,reflexive}`.

Both documented at `types.rs:159-170` and `config.rs:31-42`. Dispatch:
`ggen_config::classify_ggen_toml` (`crates/ggen-config/src/config_schema.rs:305`) →
`ggen_engine::schema_dispatch::load`. No cross-schema equivalence guard by design
(`types.rs:169-170`).

Classifier outcomes (`Malformed`/`DeclarativeRules`/`Ambiguous`/`Frontmatter`/`Unsupported`),
`config_schema.rs:77-115`:
- `[generation]` present or `[[packs]]` array-shaped → decisive DeclarativeRules
  (:400-414 — `[generation]` cannot exist under frontmatter's deny_unknown_fields).
- `[law].shapes` present → frontmatter marker (:417-424).
- `packs` array-vs-table is the hard discriminator; `name` key secondary (:427-447).
- `[project].version` presence/absence two-way discriminator (:387-397).

## 1. Declarative-rules schema (authoritative for codegen)

Root `types.rs:171-277`, `#[serde(deny_unknown_fields)]` (:172).

| Section | Required | Fields | Cite |
|---|---|---|---|
| `[project]` | yes | name (req), version (req), description, authors, license, repository | types.rs:331-355 |
| `[ontology]` | yes | source (req), imports, base_iri, prefixes (BTreeMap), standard_only | types.rs:358-379 |
| `[inference]` | default | rules, max_reasoning_timeout_ms (5000) | types.rs:382-392, 24-26 |
| `[[inference.rules]]` | — | name, description, construct (SPARQL CONSTRUCT), order, when (ASK) | types.rs:395-415 |
| `[generation]` | **yes** | rules (req), max_sparql_timeout_ms (5000), require_audit_trail, determinism_salt, output_dir "." , enable_llm/llm_provider/llm_model | types.rs:430-463, 19-31 |
| `[[generation.rules]]` | — | name, query, template, output_file ({{vars}}), skip_empty, mode, when | types.rs:468-494 |
| `[validation]` | default | shacl (legacy, refused `[FM-LAW-017]`), gates (SPARQL ASK/SELECT), validate_syntax, no_unsafe, strict_mode (**default true**), rules | types.rs:604-646, 607-615, 33-44 |
| `[[packs]]` | default [] | name (req), registry ("local"), path, version | types.rs:191-193, 52-69 |
| `[law]` | default | rules only — shapes deliberately not here | types.rs:150-157, 321-328 |
| `[rules]` | opt | n3, datalog paths | types.rs:284-293 |
| `[pack_sources]` | opt | per-entry source = "path"\|"git", location | types.rs:205-210, 297-315 |
| `[sync]`, `[output]` | opt | **untyped, unread passthrough** — no reader in workspace | types.rs:141-148, 213-217 |

Enums: `QuerySource` untagged {pack,output,file}/{file}/{inline} (types.rs:501-528);
`TemplateSource` untagged Pack/File/Inline/Git/Package (types.rs:533-578); `GenerationMode`
Create (default — first sync writes, later syncs skip to preserve hand-edits) / Overwrite /
Merge (types.rs:587-601).

## 2. `[packs]` pins

Declarative `[[packs]]` (types.rs:52-69): path pin = `registry="local"` + `path`, resolved via
`resolve_pack_root` (`crates/ggen-engine/src/generation_rules.rs:911-931`), `[FM-GEN-006]` if
undeclared/pathless; only local packs resolvable (:896-907). In-pack file lookup:
`<pack_root>/<output_dir>/<file>` from the pack's `package.toml` `[pack.outputs]` — this
resolution **fails open** with a warn log (generation_rules.rs:938-965). Version field "used
when registry != local" (types.rs:66-68) but declarative path has no git machinery.

Frontmatter `[packs]` (config.rs:131-180) — real pinning lives here:
- `Path { path, extra_ontologies, lock=true(default) }` (:142-161); path relative to the
  manifest dir (pack.rs:253-256); `../`-relative sibling refs legitimate — traversal check
  deliberately NOT applied to pack `path` (config.rs:315-327).
- `Git { git, version (required tag/semver), subdir }` (:168-179); cached under
  `<config_root>/.ggen-v2/git-packs/<name>/`; read-only resolution refuses cache
  miss/version mismatch `[FM-PACK-012]` (pack.rs:268-283).
- Content-hash pinning `ggen.lock`: `lock=true` participates; `lock=false` opts out for
  regenerated-projection ontologies (config.rs:149-161; pack.rs:78-83). Git packs always
  locked (pack.rs:79-81). Sync fails closed on hash mismatch `[FM-PACK-008]`
  (pack.rs:1336-1364); malformed lock `[FM-PACK-009]` (pack.rs:1335).
- `extra_ontologies` unioned into the pack graph, traversal-checked (unlike `path`), each joins
  the content hash (config.rs:134-148, 328-339).

Pack resolution codes (pack.rs:257-266): FM-PACK-001 dir missing, -002 pack.toml missing,
-003 invalid, -004 ontology.ttl missing, -005 zero templates, -021 bad dialects, -010 clone
fail, -011 checkout fail, -012 cache miss.

## 3. Frontmatter schema fields

config.rs:45-60: `[project]` (name only, :102-107), `[ontology]` (source req, :110-123),
`[packs]` (§2), `[templates]` (dir req; aggregate_modules false; consumer_mode false,
:189-217), `[law]` (rules; shapes legacy-refused `[FM-LAW-012]`; gates; reflexive default
false, :62-99). Third struct `config_lib::GgenConfig`
(crates/ggen-config/src/config_lib/schema.rs:13-90) is operational-tables-only root whose
section types GgenManifest reuses; its inference/generation fields deliberately retired
(:81-87).

## 4. Parsing & validation

- Parser: `ManifestParser` parse/parse_and_validate/parse_str
  (`crates/ggen-config/src/manifest/parser.rs:39,59,74`) via `star_toml::from_str` (:75).
- Engine loader: `GgenConfig::load` (config.rs:227-283) — `[FM-CONFIG-001/002/003]`
  (missing/parse/semantic, :228-258).
- Pure-data validation (validation.rs:97-330): E0011 inference CONSTRUCT missing ORDER BY
  (strict-gated, :114-131); E0013 generation SELECT missing ORDER BY (:223-242, :409-420);
  E0014 undeclared pack (:155-187); E0015 GROUP_CONCAT without separator (:189-215); E0010
  VALUES in external .rq forbidden (:384-405); duplicate inference order warn-only (:136-143).
- Filesystem validation fail-fast (validation.rs:349-355): ontology/query/template/gate/law
  paths exist (:357-478).
- Heuristics textual, case-insensitive; comment-string false positives acknowledged
  (validation.rs:547-584, :549-553).
- Engine Validate: relative paths, traversal-checked via `check_path(..., Some(false))`
  (config.rs:285-372).

## 5. Error codes

ConfigError::TomlParse (unknown key), ::Validation (validation.rs:88); E0010/E0011/E0013/
E0014/E0015/E0101 (validation.rs:400,119,229,170,204,303); FM-CONFIG-001..003 (config.rs:229);
FM-LAW-012/017 legacy SHACL refused at sync (config.rs:72-81, types.rs:607-615); FM-LAW-013
gate violation (config.rs:85-88); FM-GEN-006 (generation_rules.rs:915-930); FM-PACK-001..021,
-008/-009 (pack.rs:257-266, 1336, 1335).

Deliberate non-checks (validation.rs:68-86): no traversal rejection on ontology.source/imports
(real manifests use `../`); no non-empty PackRef::name check; no ValidationRule field checks.

## 6. Schema-drift contract + defaults

`GenerationConfig`/`GenerationRule`/`QuerySource`/`TemplateSource`/`GenerationMode` carry
`#[derive(JsonSchema)]`, tests compare `schema_for!` against `schema/ggen-manifest-schema.ttl`
(types.rs:419-429); engine side vs `schema/ggen-toml-schema.ttl` (config.rs:25-29).
Defaults: SPARQL/reasoning timeout 5000 ms, output_dir ".", strict_mode true, Create mode,
severity Error, lock true, reflexive false (types.rs:18-44; config.rs:159-187). BTreeMap for
prefixes/packs/pack_sources (determinism). Classify-only ≠ valid (config_schema.rs:56-74).
Env-var expansion in star_toml pre-parse (config.rs:260-262). strict_mode=false downgrades
E0011/E0013/E0015 to warn, bypassing Validator (validation.rs:55-66); flip test
validation.rs:773-834.

## 7. Doc caveats

1. `[sync]`/`[output]` = "accepted but not read" passthrough — never document as typed keys.
2. `validation.shacl` / `law.shapes` legacy-refused; direct authors to `validation.gates` /
   `law.gates`.
3. Present `[[packs]]` vs `[packs]` as two separate schema profiles, mutually exclusive.
4. Call out the fail-open warn on `package.toml` output-key resolution
   (generation_rules.rs:938-952).
