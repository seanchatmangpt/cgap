# R20 — ggen perf-claims inventory (Explore brief)

Lane: R20. Source: `/Users/sac/ggen`. Claims cited file:line. Purpose: G6 anti-laundering
context + RDFox like-for-like framing.

## Headline

**ggen contains ZERO RDFox (or any triple-store incumbent) performance comparison.**
`grep -rni rdfox` hits only 4 irrelevant files. Any "like-for-like vs RDFox" framing in cgap
v26.11.2 is net-new claim territory — there is no incumbent number to launder forward.

## Claim inventory by cluster

| Cluster | Representative claim | Class |
|---|---|---|
| FUSION_THESIS / phd-thesis SLOs | 2.8B msgs/s, 12ms consensus, 100% determinism over 10K cycles (FUSION_THESIS.md:193-202; phd-thesis/chapters/abstract.tex:7, chapter08.tex:163-167, appendices/appendixF.tex:5-25) | CLAIMED-unreceipted (agent-swarm workload misattributed to codegen; P50/95/99 table but no method/date/platform/raw output) |
| benches receipt | 18ms/sig transpile, 650ms/project pipeline (benches/PERFORMANCE_RECEIPT_TEMPLATE.md:19-22, 187-190) | CLAIMED-unreceipted (named TEMPLATE; all tilde-ranges; env block says Linux container /home/user/ggen — foreign platform) |
| benches SLO readme | 245.32 ms/sig, 12.45 ms/sig, 4.23 ms/object, 650.18 ms/project (SCHEMA_LAYER_SLO_README.md:144-159) | CLAIMED-unreceipted locally (inherits same provenance) |
| docs/performance README | CLI startup ~10ms, template parse 1.2ms ±0.1, RDF insert 12ms ±2, SPARQL cached 0.5ms ±0.1 (docs/performance/README.md:145-152, 352-358) | MEASURED-with-partial-receipt (dated 2025-11-18, Darwin 24.5.0 Apple Silicon; cites perf_results JSON not present in tree) |
| docs/performance optimizations | "2-4x faster bulk generation" (:180); "3-5x speedup", "40-80% latency reduction" (OPTIMIZATION_RECOMMENDATIONS.md:103,354,431) | CLAIMED (explicit projections) |
| CHANGELOG ontology-core | 420-680ms load, 12-98ms SPARQL, 24MB (CHANGELOG.md:879-881) | CLAIMED-unreceipted ("typical") |
| CHANGELOG P2P/DHT | <200ms DHT, 2-3x parallel (CHANGELOG.md:1428-1510, 1785-1845) | CLAIMED (self-labeled targets; placeholder date [3.0.0] - 2025-01-XX :~1440) |
| analysis/ | REAL_GROQ SLOs (:125-130); "35-40% of tests are false positives", 563 .expect() (FALSE_POSITIVE_EXECUTIVE_SUMMARY.md) | Measured-style, receipt-absent; internal audit context |
| docs/aps/claims.toml | CLI JTBD standings w/ falsifier commands + commit hashes (195 lines) | MEASURED-with-receipt — but **zero performance claims** |

## Implications for v26.11.2

1. No inherited incumbent number exists — the RDFox comparison is first-party, freshly
   measured, held to its own receipt standard.
2. ggen's headline numbers are workload-mismatched (2.8B msgs/s = swarm bus, not RDF/query
   throughput) — do not migrate into RDF/query framing.
3. ggen's receipt-voiced documents are demonstrably unreliable (PERFORMANCE_RECEIPT_
   TEMPLATE.md carrying concrete numbers on a foreign platform); corroborated by the repo's
   own cheat-scan history (docs/v26_10_10_cheat_scan_triage.md, 464 findings retired
   2026-07-20; FAKE_DETECTION_STRATEGY.md) — the exact laundering class G6 blocks.
4. Defensible ggen-side baseline if ever needed: only docs/performance/README.md
   (dated, platformed, ±stddev) + fresh cargo bench of schema_layer_slo.rs / receipt_bench.rs
   on the same host; everything else CLAIMED-unreceipted.
5. Root README.md:11-21 contains unresolved `<<<<<<< GENERATED >>>>>>> MANUAL` merge-conflict
   markers — a live drift defect in the file a G6-style gate would read.

Key files: FUSION_THESIS.md, benches/PERFORMANCE_RECEIPT_TEMPLATE.md,
benches/SCHEMA_LAYER_SLO_README.md, docs/performance/README.md, CHANGELOG.md,
phd-thesis/appendices/appendixF.tex, phd-thesis/CODE_VERIFICATION_REPORT.md,
docs/aps/claims.toml, docs/v26_10_10_cheat_scan_triage.md,
analysis/FALSE_POSITIVE_EXECUTIVE_SUMMARY.md.
