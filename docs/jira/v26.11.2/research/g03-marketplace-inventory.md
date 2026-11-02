# R3 — ggen-marketplace pack inventory (Explore brief)

Lane: R3. Source: `/Users/sac/ggen-marketplace` (HEAD 2664bcaf5, 2026-10-31). Claims cited.

## Headline facts

- **345 entries in packs/** (344 pack dirs + 1 stray markdown). Each has pack.toml
  `[pack] name/version`.
- Marketplace release **v26.10.8**, pinned ggen binary v26.8.11, canonical source
  `seanchatmangpt/ggen-marketplace` (marketplace.toml:5-38).
- `docs/reference/PACK-INDEX.md` is the generated per-pack registry (326 rows; header :4-6),
  projected via `scripts/gen_pack_index.py`. **Stale**: 19 pack dirs absent from the index,
  including **cgap-pack** (verified by set-diff vs os.listdir).
- Profile census (index): 160 projection / 120 project / 46 semantic (of 326). Lifecycle: 320
  active, 6 deprecated (lifecycle.toml:20-29 and deprecated blocks at :9,123,296,464,471,478).
- Prior audit receipt exists: `_GGEN_MARKETPLACE_INVENTORY_RECEIPT.md` (commit 42b031c) —
  reports 307 packs; itself stale vs 344 today.

## O-ABI / fibo / cloud clarification

**No fibo or cloud O-ABI packs in marketplace packs/**. The vertical O-ABI packs live in
cgap: `/Users/sac/cgap/ontologies/fibo/` (A09 cgap-fibo) and `/Users/sac/cgap/ontologies/cloud/`
(A10 cgap-cloud). FIBO source ontologies (286 .rdf files) vendored at
`ontologies/public/fibo/` (R17's survey says `olderntologies/public/fibo/` — path discrepancy,
verify before citing), consumed by-IRI by packs like `ma-case-study-pack`. Cloud-adjacent
marketplace packs: azure-terraform-pack, gcp-marketplace-saas-pack, kubernetes-workload-pack,
gh-terraform-pack, typer-azure-pack, xaas-* family, chicago-xaas-surface-pack.

## cgap ↔ marketplace consumption evidence

- cgap-pack declares consumer: "Consumed by the ~/cgap repo via `ggen sync run`"
  (packs/cgap-pack/pack.toml:11).
- cgap declares dependency: `[packs] cgap-pack = { path = "../ggen-marketplace/packs/cgap-pack" }`
  (/Users/sac/cgap/ggen.toml); lockfile /Users/sac/cgap/ggen.lock.
- cgap also consumes readme-diataxis-pack (docs/diataxis/marketplace-generated/ggen.lock pins
  packs.readme-diataxis-pack, blake3 5c00ad37…; 24 generated files).
- Root marketplace ggen.lock locks only mdbook-pattern-language-pack (dogfood only).
- 16 pack.tomls name a consumer in descriptions (grep "consumed by"/"consumers"); **no
  machine-readable consumer-count field** exists — consumption proven by prose, external
  ggen.toml path deps (cgap is the only confirmed external consumer), tests/, lockfiles.
- cgap receipts referencing the marketplace: /Users/sac/cgap/docs/RECEIPTS.md (R5 ggen sync
  replay x2, R6 marketplace check + courts, exit 0, v26.10.29 ledger).

## cgap-pack structure (the doc-relevant pack)

v26.10.29 (pack.toml:2-3). "Canonical Arrow columnar graph... freestanding, branchless,
zero-dependency Apache Arrow representation layer" (:4-11). Admits Arrow type families as
PhysicalLayout individuals, 64-byte aligned BufferSlots, ArrowSchema+ArrowArray C Data
Interface ABI structs with sourceText → include/cgap.h. Structure: ontology.ttl (11 ttl incl.
witness fixtures), gates/010_subject_identity.rq … 050_slot_continuity.rq (5 gates),
witnesses/{pass,fail}/ exact-stem pairs, gate-court.toml (ggen.semantic-gate-witness-court/1),
templates/ 4 Tera templates with disjoint targets (cgap_h.tmpl, cgap_format_codes_h.tmpl,
layout_gen_rs.tmpl, subject-count.txt.tmpl) — README.md:9-18, :24-32 (50 Arrow type
individuals: Null 1, primitive 32, varbinary 4, binary-view 2, nested 9, REE 1, dictionary 1).
**Gap**: cgap-pack missing from PACK-INDEX.md — regenerate before citing.

Other notables: clap-noun-verb family (11 packs; deprecated compat skeleton clap-noun-verb-pack
with 6 named successors, lifecycle.toml:20-29); largest semantic packs dfcm-pack v2.2.0 (1,471
files, 1,073 .rq), epistemic-sensor-factory-pack (2,099 files), ggen-ecosystem-ocel-pack (1,267);
largest template manufacturers platform-engineers-handbook v0.3.0 (295 tmpl), tcps-release-pack
(85 tmpl).

## Drift caveats before citing

1. PACK-INDEX.md says "326 packs: 320 active, 6 deprecated" (:6) but packs/ has 344 dirs —
   19 missing (incl. cgap-pack). Regenerate with scripts/gen_pack_index.py.
2. Inventory receipt (42b031c) reports 307 — predates latest additions.
3. Versions heterogeneous: many 0.1.0; calendar 26.x.y dominate newer; cgap-pack 26.10.29.
4. cgap-fibo / cgap-cloud are in the cgap repo, not the marketplace — cite the cgap side;
   marketplace supplies cgap-pack + readme-diataxis-pack to the cgap consumer.
