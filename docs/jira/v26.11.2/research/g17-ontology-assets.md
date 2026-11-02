# R17 — Ontology assets survey (Explore brief)

Lane: R17. Sources: `/Users/sac/ggen`, `/Users/sac/ggen-marketplace`, plus cgap's own
`ontologies/` (the doc subjects). Read-only; claims cited file:line.

## 1. cgap's own O-ABI packs (doc subjects)

**`/Users/sac/cgap/ontologies/fibo`** — Lane A09, `cgap-fibo`
- README.md:1-9: "FIBO O-ABI pack... build-time-emitted binary ontology pack plus `#![no_std]`
  const/types facade." Ships `fibo_sec_dbt_v1.oabi` (496 B, byte-stable); `src/bin/emit-pack.rs`
  collision-vets through A06's real CollisionDetector and self-validates through A06's real
  loader (Chicago). Status UNPUBLISHED as of writing, `publish = true` flipped 2026-11-01;
  21 tests / 0 failures.
- Constants (src/lib.rs): `FIBO_RELEASE = "FBC-2024Q4"` (:134), 32-byte `PACK_DIGEST` (:144),
  `SLOT_ID_BITS=64` (:165), `LEX_HASH_LEN=32` (:169), `OABI_V1_RECORD_SIZE=48` (:173),
  `OABI_V1_HEADER_LEN=64` (:184), `CLASS_ID_BITS=32`, `ENUM_ID_BITS=16`; classes
  `CLASS_CORP_BOND`/`CLASS_BOND`/`CLASS_DEBT_INSTR`/`CLASS_ACCT_RECV`; attrs
  `ATTR_MATURITY_DATE`/`ATTR_COUPON_RATE` (:196-214). Provenance flag:
  `PACK_PROVENANCE_DISTRIBUTION_SHA256 = "UNVERIFIED:no-distribution-on-disk"` (:158).
- License URN pin `urn:avec:license:p28-sil:v2.1.0-sovereign` (README See Also).

**`/Users/sac/cgap/ontologies/cloud`** — Lane A10, `cgap-cloud`
- README.md:1-6: TOSCA/CIMI lexemes, same build/runtime split. Ships `tosca_cimi_v1.oabi`
  (1,552 B); tests `courts.rs`, `raw_consumer.rs`, `wire_parity.rs` — 13/0. Status
  **PUBLISHED** (crates.io `cgap-cloud` v26.11.1, live-witnessed 2026-11-02T03:43:38Z).
- Constants (src/lib.rs): `OABI_MAGIC = b"OABIv1\0\0"` (:51), `SlotId =
  u64::from_le_bytes(BLAKE3(b"bont\x01" ++ lexeme)[0..8])` (:14), `CLASS_COUNT=4` (:76),
  `CLASS_COMPUTE=0` (:79), same 48/64/32/16 wire geometry.

Existing cgap anchors: `docs/diataxis/reference/crate-index.md:31-32` maps
`ontologies/cloud → cgap-cloud (A10)`, `ontologies/fibo → cgap-fibo (A09)`;
`docs/REVENUE_GENERATION_COMPENDIUM.md:250` (Tier 4 vertical O-ABI packs), :352-353 (A09 ALIVE
1M-row stress 173 ms; A10 committed `tosca_cimi_v1.oabi`), :116 (`.oabi` SlotId vendor
lock-in).

## 2. FIBO assets in /Users/sac/ggen

- Full FIBO master mirror: `ontologies/industry/financial/fibo-master/` — modules FBC, BE, SEC,
  DER, LOAN, IND, BP, CAE, ACTUS, MD, DCO, FND + `AboutFIBOProd*.rdf`, `MetadataFIBO.rdf`,
  `ONTOLOGY_GUIDE.md`; flat extracts `fibo-{instruments,credit,derivatives,loan,risk}.rdf`,
  `fibo-master.zip`; inventory `FINANCIAL_ONTOLOGIES_INVENTORY.md`, `ONTOLOGIES_REPORT.md`.
- Catalog: `ontologies/catalog.ttl` — DCAT 2 catalog, 3 tiers (`gcat:Core/Cached/Referenced`,
  :24-34). FIBO entries Tier 2 Referenced: FND (:248-256, `fibo-fnd`), BE (:258-266, `fibo-be`),
  SEC (:268-276, `fibo-sec`), authority "EDMCouncil", capability `financial-instruments`.
- FIBO-derived: `ontology_catalogue/yawl/ontology/profiles/fibo-alignment.ttl`;
  `ontology_catalogue/ggen/specs/012-grand-unified-kgc-thesis/ontology/{fibo-subset.ttl,
  agent5-fibo-bpmn-extensions.ttl, bruce-lee-fibo-constructs.ttl, chapter-8-construct-fibo-bpmn.ttl,
  workflow-finance-bridge.ttl}`; fixtures `ontology_catalogue/ggen/tests/fixtures/ontologies/
  fibo_{loan_approval,account_opening}.ttl`; BusinessOS `fibo-deal-{examples,integration}.ttl`;
  `ontology_catalogue/cre/examples/fortune5-fibo-lnctrl/ontology/f5_line_control.ttl`; SHACL
  shapes `ontology_catalogue/chatmangpt/tests/shapes/fortune5-compliance-shacl-shapes.ttl`.

## 3. Cloud ontology assets in /Users/sac/ggen

- Federated Semantic Registry: `ontology/registry.toml` — 16 packs, 203 sources, 25 P0
  required incl. `nist-cloud-taxonomy`, `tosca`, `focus`, `cloudevents`.
- Pack 01-cloud-reference (`ontology/01-cloud-reference/pack.toml`; projection
  `packs/autofde-semantic-registry-pack/ontology/01-cloud-reference.ttl`): 10 sources — DMTF
  CIM/OVF/Redfish, NIST SP 800-145 (P0)/500-292/500-332/SP 800-201, OASIS CAMP, OGF OCCI,
  OASIS TOSCA 2.0 (P0). All `standing: UNKNOWN`,
  `transformation: registry-only-until-admitted` except NIST taxonomy + TOSCA
  (`profile-or-import-after-pinned-retrieval`).
- Pack 02-cloud-resources (`ontology/02-cloud-resources/pack.toml`): 36 sources — provider
  projections (AWS CloudFormation, Azure ARM, GCP, IBM, OCI, Alibaba, Tencent, Huawei, OVH),
  K8s OpenAPI/CRDs, Crossplane CRDs, Terraform/Pulumi/Helm/Steampipe schemas, CloudEvents,
  OpenAPI 3.2, Avro/AsyncAPI/protobuf/gRPC/GraphQL/OData.
- Registry TTL projections: `packs/autofde-semantic-registry-pack/ontology/0{1,2}-cloud-*.ttl`
  (namespace `reg: https://ggen.io/ontology/autofde-registry/v1#`).
- Integration test: `tests/integration/ontology_workflows_multi_cloud.rs`.

## 4. Namespaces in active use

- ggen core: `ggen: http://ggen.dev/ontology#`, `a2a: https://ggen.dev/ontology/a2a`
  (ontology.ttl:7-8). Catalog: `gcat: https://ggen.io/ontology/catalog/v1#`
  (catalog.ttl:7). Registry: `reg: https://ggen.io/ontology/autofde-registry/v1#`
  (01-cloud-reference.ttl:10). Marketplace per-pack e.g. `fin:
  https://ggen.io/ontology/financial-ledger#` (packs/industry-financial-ledger/ontology.ttl:1),
  `mfa: http://seanchatmangpt.github.io/packs/mfact#`. Standards:
  rdf/rdfs/owl/xsd/dct/dcat/prov/skos/qudt/spdx/ocel (catalog.ttl:60-215).

## 5. ggen root fixture files (not assets)

`data.ttl` (synthetic `ex:` bench fixture), `entities.ttl:1-15` (minimal example),
`types.ttl:1-5`, `graphs/` contains only `schema.ttl`. Test/bench artifacts, not O-ABI sources.

## 6. ggen-marketplace ontology-bearing packs (50+)

- `packs/industry-financial-ledger/ontology.ttl` — **explicitly declines FIBO**: "FIBO's full
  credit-index structure is not minimally expressible here without importing its
  finance-specific upper structure" (:11-13); minimal FINREC double-entry vocab
  (`fin:Account/JournalEntry/Posting`) with ETag-CAS concurrency law.
- `packs/mfact-pack/ontology.ttl` (multi-factor auth, `mfa:` ns).
- Curated umbrella: `/Users/sac/ggen-marketplace/ontologies/ontologies.ttl` — auto-generated
  `owl:imports` manifest over `autofde/`, `cns/`, `open-ontologies/`, `public/`. Largest
  ontology-bearing packs: `strata-valve`, `strata-stratus`,
  `portfolio-epistemic-observability`, `ggen-combinatorial-maximalism`,
  `fortune5-enterprise-architecture`.

## 7. Doc recommendations

1. `ontologies/fibo`: cite ggen's fibo-master FBC-2024Q4 mirror as upstream provenance
   (↔ `FIBO_RELEASE` constant); catalog.ttl Tier-2 entries as the fetch-on-demand counterpart
   to the compiled `.oabi`. Note 496-byte container, SlotId = low-64 BLAKE3, and the
   `UNVERIFIED:no-distribution-on-disk` provenance flag as the honest-status line.
2. `ontologies/cloud`: ggen's 01-cloud-reference/02-cloud-resources packs (NIST 800-145,
   TOSCA 2.0, DMTF CIM P0s) are the named upstream of `tosca_cimi_v1.oabi` lexemes; cite
   registry.toml P0 list as the admission contract.
3. Distinguish: cgap `.oabi` packs (binary, collision-vetted, published) vs ggen TTL registries
   (registry-only-until-admitted, standing UNKNOWN). Compendium Tier-4 framing (:250) bridges.
4. Marketplace has no FIBO pack; `industry-financial-ledger` is deliberately FIBO-free — do not
   cite as a FIBO source.
