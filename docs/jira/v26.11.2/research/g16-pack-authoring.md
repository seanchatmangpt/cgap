# R16 — marketplace pack authoring guide (Explore brief)

Lane: R16. Source: `/Users/sac/ggen-marketplace`. Claims cited file:line. cgap's O-ABI packs
(ontologies/fibo, ontologies/cloud) are cgap-side; cgap-pack in the marketplace is the worked
example.

## Canonical pack layout (packs/cgap-pack/)

```
packs/<name>/
├── pack.toml          # manifest — identity only
├── ontology.ttl       # semantic authority (or ontology/*.ttl)
├── templates/*.tmpl|tera|eex   # optional projection logic
├── gates/*.rq | *.py | *.sh    # optional refusal gates
├── gate-court.toml    # witness-pair court config (optional)
├── witnesses/{pass,fail}/<gate-stem>.ttl
├── targets.toml       # optional sidecar: [targets] languages
├── qualification/     # optional positive fixtures
└── qualification.toml # optional: [consumer] extra_ontologies
```

## Manifest (pack.toml)

- `[pack]` admits ONLY name, version, description, deprecated, superseded_by —
  `PACK_TABLE_KEYS` scripts/marketplace.py:87 (unknown keys → PACK_KEY_UNADMITTED, :366;
  engine deserializes deny-unknown-fields).
- name must equal directory name (PACK_DIRECTORY_IDENTITY :337-345); version SemVer (:36,
  :348); description non-empty (:352).
- `[capabilities]` (README "Pack capabilities"): provides/requires/types as URN sets;
  self-IRI invariant — provides must include `urn:ggen:pack:<pack-id>` exactly, globally
  unique (tests/capability_corpus_test.rs).
- Sidecar targets.toml: `[targets] languages = [...]`, `[a-z][a-z0-9_+-]*`, non-empty,
  dup-free (docs/reference/pack-contract.md:83-92; marketplace.py:303-323; refusals
  TARGETS_IN_MANIFEST / TARGETS_LANGUAGES).
- Lifecycle metadata lives in root lifecycle.toml, NOT pack.toml (loader denies unknown keys)
  — docs/reference/pack-lifecycle-registry.md.

## Templates

- Optional; non-scaffolding files under templates/ must be .tmpl/.tera/.eex (TEMPLATE_SUFFIXES
  marketplace.py:40; refusal TEMPLATE_EXTENSION :359-362).
- Projection rules, not a second ontology. Pattern: Tera front-matter `to:` + `sparql:` SELECT;
  values live in RDF (packs/cgap-pack/templates/cgap_h.tmpl:1-18 — ORDER BY-deterministic).
- Never commit generated output; change RDF → re-run ggen sync run (cgap-pack README header).

## Gates + witness court

- gates/ sources: .rq (SPARQL refusal = violation-row SELECTs), .py/.sh bounded verifiers
  (GATE_SOURCE_SUFFIXES marketplace.py:41; pack-contract.md:31-38).
- Gates are ORDER BY-deterministic violation-row SELECTs; a row = refusal (cgap-pack
  gates/010_subject_identity.rq; 5 gates 010-050: subject_identity, promise_vs_content,
  alignment_invariant, format_completeness, slot_continuity).
- Witness court: gate-court.toml schema `ggen.semantic-gate-witness-court/1`, case_key =
  "exact-stem", gate_dir/pass_dir/fail_dir, require_pass/require_fail (all 9 lines of
  cgap-pack/gate-court.toml). Enforcement: scripts/check_gate_witness_courts.py (schema,
  safe_dir containment, duplicate-stem, orphan/missing witness). CI:
  .github/workflows/gate-witness-courts.yml.
- Each gate needs a pass witness (0 rows) + fail witness (≥1 row) at exact stem — anti-vacuity.
  Fail-witness example: witnesses/fail/010_subject_identity.ttl (p:Subject deliberately
  missing p:subjectId).

## Receipt integration

Receipts are the consumer/publish boundary, not pack.toml content. Publish flow requires
`ggen receipt verify` when the consumer uses receipts (docs/how-to/publish-a-pack.md §7); PR
receipt binds exact base/head SHA, pack/version/profile/class, commands+exit codes, witnesses,
replay/receipt results, authority ceiling (§8). ggen.lock pins pack by content hash
(blake3:...). Consumer-side digest pinning: scripts/fetch_pack.py lock/verify/fetch with
packs.lock.json (sha256; REFUSED:DIGEST_MISMATCH, REFUSED:LOCK_CATALOG_DRIFT) —
docs/how-to/pin-and-verify-packs.md. Archive/catalog publish emits digest (sha256 of
deterministic tar.gz, marketplace.py:135,147), download_url, size_bytes (:147-161). Publish
runs on push to main only after PR CI on the same content (.github/workflows/publish.yml).
Court receipts land in receipts/ tree + ci/courts.json binds courts to pack paths + receipt
files. Qualification report JSON binds pack identity/profile, source identity, consequence
identity/count, court result (docs/reference/ggen-qualification-contract.md).

## Versioning / publish flow

1. Author under packs/<name>/; bump version when published behavior changes
   (docs/how-to/update-a-pack.md §2).
2. Local admission: `python3 scripts/marketplace.py validate` → `catalog` twice + cmp
   (determinism proof) → `fingerprint` (publish-a-pack.md §5).
3. Real manufacture + replay: ggen sync run twice, byte-compare fixed point; ggen receipt
   verify (§6-7).
4. Exact-subject PR with the §8 receipt contents.
5. CI: ci.yml (exact-head assertion, path classification via ci_plan.py, per-pack courts from
   ci/courts.json), gate-witness-courts.yml, qualification rail (isolated capsule per pack,
   double sync fixed point, source digest unchanged).
6. On merge: publish.yml builds deterministic per-pack .tar.gz, uploads to rolling packs
   GitHub Release, deploys catalog (schema v2) to Pages. Never hand-edit catalog —
   deterministic projection.

## Validation courts

- Structural admission: marketplace.py validate — typed REFUSED:* (validation-contract.md):
  manifest missing/invalid, name/dir mismatch, non-SemVer, no Turtle at root or ontology/**
  (ONTOLOGY_SOURCE_MISSING :354-356), template extension, gates shape, Diátaxis required docs
  (REQUIRED_DOCS :65-86), symlinks refused.
- Profile derivation: project (ggen.toml) > projection (templates) > semantic (RDF only).
- Gate witness court: check_gate_witness_courts.py + gate-court.toml.
- Runtime qualification: scripts/qualify_packs.py — isolated capsule, double sync fixed point;
  config admitted via scripts/admit-config.sh marketplace.toml first.
- "CI is orchestration/evidence, not a second competing acceptance implementation"
  (validation-contract.md).

## Ontology library vs packs (critical)

ontologies/ (incl. public/fibo/) is upstream source material, never validated or loaded by
pack qualification (ontologies/README.md "Scope and status"). To ship a pack: create
packs/<name>/ with pack.toml ([capabilities] self-URN in provides); copy needed .ttl into
ontology/ preserving filenames for provenance (or list in qualification.toml
[consumer] extra_ontologies — relative, inside pack, no traversal); add gates + pass/fail
witnesses + gate-court.toml (cgap-pack is the template); templates project to consumer
outputs; consumers reference the pack by path in ggen.toml [packs] and run ggen sync run.

Key docs: docs/reference/pack-contract.md, validation-contract.md,
ggen-qualification-contract.md, docs/how-to/publish-a-pack.md, docs/tutorials/first-pack.md,
packs/cgap-pack/ (canonical example), scripts/marketplace.py,
scripts/check_gate_witness_courts.py.
