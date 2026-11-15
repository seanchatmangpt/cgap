# docs-diataxis-pack gate spec (all fail-closed; any row => FM-PACK-013 naming the gate)

| gate | file | refuses |
|---|---|---|
| 010 | gates/010_required_properties.rq | Page missing title/position/relPath/body/quadrant; Page missing its quadrant's property (Tutorial goal, HowTo task, Reference subject, Explanation topic) |
| 020 | gates/020_single_quadrant.rq | Page with >1 quadrant; quadrant not a ddx:Quadrant enum member |
| 030 | gates/030_nav_ordering.rq | duplicate position within a quadrant (ordering tie); authored ddx:Nav whose navOrder != quadrant.order*1000+position |
| 040 | gates/040_output_path_unique.rq | duplicate relPath; non-.md; absolute/`../`/reserved SUMMARY.md |
| 050 | gates/050_nonempty.rq | zero ddx:Page in graph (vacuity refusal); more than one ddx:Doc |

SUMMARY.md order = (Quadrant ddx:order, ddx:position); 030 makes that total.
ddx:order / ddx:requires are now declared in ontology.ttl (F5).
Vacuity law: zero Pages is a loud refusal. Gate 050 emits a row on an empty
graph (reference_gate.py: exit 2 for zero pages, mirrors the refusal; exit 1
for doc-not-singleton). Syntax discipline: no BIND of a variable already in
scope in the same group (FM-GRAPH-013), no STRENDS (REGEX used), gate 010 has
no BIND.
