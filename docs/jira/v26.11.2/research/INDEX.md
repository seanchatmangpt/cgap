# Research Index — ggen/ggen-marketplace Explore Wave (g01-g20)

Provenance: 20-lane read-only Explore fan-out over `~/ggen` +
`~/ggen-marketplace`, landed 2026-11-01. Every claim inside each brief is
file:line-cited to its source tree; citations are re-read at use time per
no-overclaiming law (a cite may drift if the source tree moves — verify
before relying on one).

## Index

| File | Lane | Hook |
|---|---|---|
| g01-ggen-architecture.md | R1 | ggen workspace architecture overview |
| g02-manifest-schema.md | R2 | ggen.toml manifest schema — two schemas, not one |
| g03-marketplace-inventory.md | R3 | ggen-marketplace pack inventory |
| g04-receipt-system.md | R4 | receipt system — three coexisting receipt mechanisms |
| g05-shacl-validation.md | R5 | SHACL validation layers |
| g06-sparql-rdf-kernel.md | R6 | RDF/SPARQL kernel prior-art |
| g07-bench-conventions.md | R7 | bench suite conventions / framework landscape |
| g08-court-mutation-patterns.md | R8 | court + mutation testing patterns |
| g09-nostd-patterns.md | R9 | no_std patterns — where `#![no_std]` lives |
| g10-ci-publish-mechanics.md | R10 | CI / publish mechanics |
| g11-template-engine.md | R11 | template/codegen engine layout |
| g12-lockfile-replay.md | R12 | ggen.lock shape and replay |
| g13-doc-taxonomy.md | R13 | docs/book taxonomy — governing house style |
| g14-cli-structure.md | R14 | CLI structure — two distinct CLI systems |
| g15-governance-skills.md | R15 | governance skills + constitution |
| g16-pack-authoring.md | R16 | marketplace pack authoring guide |
| g17-ontology-assets.md | R17 | ontology assets survey (incl. cgap O-ABI packs) |
| g18-testing-discipline.md | R18 | ggen testing discipline / test organization |
| g19-marketplace-governance.md | R19 | marketplace governance + registry identity |
| g20-perf-claims-inventory.md | R20 | perf-claims inventory |

## Known discrepancies — re-verify before citing

- **g03**: FIBO vendored path inside ggen-marketplace is flagged as a
  discrepancy inside the brief itself; confirm the path on disk before
  citing it downstream.
- **g19**: README pack-count drift flagged in the brief — the README's
  count and the observed pack count disagree; re-count on disk before
  citing either number.

## Court discipline

A court citing a brief must re-read the cited file:line at use time. A
path/count that fails re-verification is a divergence, not a silent cite.
