# ERRC-F — docs-diataxis-pack — receipt

Lane: ERRC-F (ERRC-F IMPLEMENT). Subject:
`/tmp/ddx-consumer` sandbox plus the new pack. Date: 2026-11-01.

## Grounding

The predecessor `readme-diataxis-pack` is README-shaped: one Project
singleton plus flat facts and fixed output paths. This lane generalizes it
into a multi-page corpus: a `ddx:Book` singleton plus ordered `ddx:Page`
entries, a quadrant enum (tutorial, how-to, reference, explanation),
dynamic per-page output paths, and SUMMARY.md navigation.

## Pack structure (all under packs/docs-diataxis-pack/)

- pack.toml: `[pack]` name/version/description only, per the house
  deny-unknown-fields law; directory name matches pack name.
- ontology.ttl: classes `ddx:Book`, `ddx:Page`; properties `ddx:quadrant`,
  `ddx:position`, `ddx:relPath`, `ddx:body`; namespace
  `https://seanchatmangpt.github.io/ggen-marketplace/docs-diataxis#`.
- gates/010_required_properties.rq: Book needs dcterms:title; every Page
  needs position, title, quadrant, relPath, body (FILTER NOT EXISTS, no
  OPTIONAL fallback). FM-PACK-013 typed refusal naming the gate.
- gates/020_structure_integrity.rq: quadrant enum membership, duplicate
  relPath (write collision), duplicate position within a quadrant
  (ordering tie), non-.md relPath, Book singleton.
- templates/page.md.tmpl: for_each over pages, dynamic `to: relPath`, one
  markdown file per Page.
- templates/SUMMARY.md.tmpl and templates/index.md.tmpl: quadrant-grouped
  navigation, ordered by quadrant then position.
- qualification/consumer.ttl: deterministic fixture (Book plus five Pages
  across all four quadrants).
- README.md: consumer-facing shape and gate law.

## Courts run (real ggen 26.10.11, all this session)

1. Generation court: /tmp/ddx-consumer (facts.ttl with Book plus five
   Pages, ggen.toml pinning the pack by absolute path). `ggen sync run`
   exit 0; seven files generated: five pages plus SUMMARY.md plus
   index.md, quadrant-grouped and position-ordered; page bodies project
   verbatim (multi-line markdown via triple-quoted Turtle).
2. Replay court: second `ggen sync run` exit 0, 14 "unchanged" entries,
   idempotent.
3. Court A (required-properties flip): sandbox copy /tmp/ddx-court-a with
   ddx:body stripped from ex:p3. `ggen sync run` exit 1, typed refusal
   FM-PACK-013, pack `docs-diataxis-pack` gate `010_required_properties.rq`,
   first row names subject ex:p3 and the missing ddx:body property.
4. Court B (structure-integrity flip): sandbox copy /tmp/ddx-court-b with
   quadrant "how-to" mutated to "blog". `ggen sync run` exit 1, typed
   refusal FM-PACK-013 naming `020_structure_integrity.rq`, subject ex:p3.
   Sandboxes are throwaway /tmp copies; the marketplace tree was never
   mutated.

## Marketplace house conformance

- `python3.11 scripts/marketplace.py validate` — exit 2 with
  REFUSED:MANIFEST_MISSING:smalt-core-pack and
  REFUSED:ONTOLOGY_SOURCE_MISSING:smalt-core-pack. Pre-existing: the
  refusal names only smalt-core-pack (a separate in-flight pack, not
  modified this session), and the same refusal appears when the command
  is scoped to `check docs-diataxis-pack`. Fail-closed aborts the
  catalog/fingerprint/qualification plane for every pack, so
  catalog-determinism (`catalog > a; catalog > b; cmp`), `fingerprint`,
  and `qualify_packs.py --pack docs-diataxis-pack` are all blocked by the
  same pre-existing refusal.
- `python3.11 -m pytest tests/ scripts/` — 4 collection errors, all
  `ModuleNotFoundError: No module named 'cryptography'`; pre-existing
  environment gap, unrelated to this pack.
- My pack produced no validate refusal naming docs-diataxis-pack, and the
  real ggen runtime loaded pack.toml under the deny-unknown-fields law
  and executed both gates against a real consumer graph.

## What remains

- smalt-core-pack repair (owner lane) unblocks the whole marketplace
  validate/catalog/qualification plane; rerun
  `marketplace.py validate` + catalog determinism + scoped qualification
  for docs-diataxis-pack then.
- Marketplace index registration/admission step, if house law requires a
  separate admission transition — recorded, not self-admitted. The pack
  is a new untracked directory (`?? packs/docs-diataxis-pack/`); the
  operator owns all git transitions.

## Standing

ALIVE for the pack itself: real ggen runtime, exit-0 generation witness,
idempotent replay, and two typed gate flips (both courts flip; no vacuous
gate). Marketplace-plane conformance is BLOCKED:pre-existing
smalt-core-pack defect, not session-introduced.

## Divergences

- House CLAUDE.md says "branch before editing; never write directly to
  main" — the lane contract forbids git writes (operator commits), so the
  pack landed as an untracked directory on the current checkout state.
- ggen SPARQL engine quirks (witnessed, shaped the gate source):
  FM-GRAPH-013 rejects a BIND target whose variable name appears anywhere
  else in the query text including comments and projection lists; STREND
  is unsupported (replaced with REGEX("[.]md$")).
