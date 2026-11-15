# docs-diataxis-pack gate spec (all fail-closed; any row => FM-PACK-013 naming the gate)

| gate | file | refuses |
|---|---|---|
| 010 | gates/010_required_properties.rq | Page missing title/position/relPath/body/quadrant; Page missing its quadrant's property (Tutorial goal, HowTo task, Reference subject, Explanation topic) |
| 020 | gates/020_single_quadrant.rq | Page with >1 quadrant; quadrant not a ddx:Quadrant enum member |
| 030 | gates/030_nav_ordering.rq | duplicate position within a quadrant (ordering tie); authored ddx:Nav whose navOrder != quadrant.order*1000+position |
| 040 | gates/040_output_path_unique.rq | duplicate relPath; non-.md; absolute/`../`/reserved SUMMARY.md |

SUMMARY.md order = (Quadrant ddx:order, ddx:position); 030 makes that total.
Note: ddx:order and ddx:requires are used on enum individuals in ontology.ttl
without separate property declarations (add before registration).
Vacuity law: zero Pages in the consumer graph must be a loud refusal, not ADMIT
(reference_gate.py exits 2 on it; the SPARQL gates alone would admit an empty
graph, so the ggen consumer must add a Doc/Page-count check).
