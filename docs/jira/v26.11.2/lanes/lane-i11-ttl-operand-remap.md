# Lane I-11 — TTL Operand Remap (AUM → annualGrossRevenue)

Date: 2026-11-02. Owned file: `docs/jira/v26.10.31/19-ontology-ttl.md`
only. Receipt lives here (v26.11.2 lanes dir; the v26.10.31 dir has no
`lanes/`); the TTL file itself carries the in-file AMENDMENT/CLOSED/
HISTORY blocks per ERRATA-never-rewrite.

## W1 Subject

`docs/jira/v26.10.31/19-ontology-ttl.md`, policy
`avec:SovereignInvariantPolicy_v1`.

## W2 Diff (before/after)

- Former :212 (`odrl:lt` branch): `odrl:leftOperand
  avec:assetUnderManagement` → `odrl:leftOperand
  avec:annualGrossRevenue`, with dated `# AMENDMENT 2026-11-02 (lane
  I-11)` comment above each replacement (in-TTL Turtle comments).
- Former :223 (`odrl:gte` branch): same remap, plus comment pinning
  the ≥ boundary (duty at exactly $50,000,000.00).
- Safe-harbor `rdfs:comment` rewritten (was "AUM below $50M"); old
  text preserved verbatim in the in-file HISTORY block.
- Added `avec:annualGrossRevenue` operand definition (odrl:LeftOperand,
  owl:NamedIndividual) citing instrument §2.4 / §2.2.2 / §2.2.3.
- Added in-file CLOSED note (E-08) + AMENDMENT block + HISTORY block
  (HTML comments, spec-doc convention in this file).
- No other lines touched. Old line numbers 212/223 were re-anchored by
  the insertion of the AMENDMENT block (remapped lines now at ~221/~237
  in the file).

## W3 Commands / exits (this session)

- `grep -n "assetUnderManagement" 19-ontology-ttl.md` → exit 0; 4 hits,
  ALL inside AMENDMENT/HISTORY comment blocks (lines 208/222/238/274);
  zero active operand occurrences remain.
- `grep -c annualGrossRevenue` → 4 (2 operands, 2 comment/definition
  references). `awk length>100` sweep: only pre-existing lines 43 and
  249 exceed 100 chars; no line I wrote does.
- Full re-read of the amended region (sed 206-300) confirms on-disk
  bytes match intent.

## W4 Court (textual): five aggregation duties → operand mapping

Compliance doc duties (34-p28-sil-compliance.md:107-129, amendment
note) each map to the new operand `avec:annualGrossRevenue`:

1. **Group Aggregation Level classification (§2.2.2)** → combined
   gross receipts/balance-sheet figures attributed per member →
   evaluated on `avec:annualGrossRevenue` (safe-harbor rdfs:comment
   now reads "measured at the Group Aggregation Level per instrument
   section 2.2.2"; operand definition states combined figures).
2. **Per-member regularization trigger** → each member's own
   `avec:annualGrossRevenue` vs $50,000,000.00 (`odrl:gte` branch
   fires per member; duty `avec:regularize` unchanged).
3. **Partition ban (meters/deployments/executions)** → stated in the
   operand definition's rdfs:comment (§2.2.3 anti-avoidance imports
   §3.3 spoliation invariants).
4. **Strictest-reading-pending-arbitration (§7.2)** → consistent with
   the `lt`/`gte` pair: the two branches are exhaustive and
   non-overlapping, so the strictest reading is exactly the `gte`
   branch at the boundary.
5. **§2.2.3 restructure-with-receipts-intact dispute test** →
   anti-avoidance invariant recorded in the operand definition.

**Boundary case** ($50,000,000.00 exactly, 34-p28:102-104): maps to
the `odrl:gte` duty branch — NOT safe harbor. The `odrl:lt` harbor
branch is strictly-below; `gte` + `lt` on the same
`avec:annualGrossRevenue` operand at the same
`"50000000"^^xsd:decimal` operand value implement the instrument §2.4
"comparison is ≥" semantics exactly. An entity at exactly
$50,000,000.00 satisfies `gte`, fails `lt`, owes `avec:regularize`.

## W5 Anti-vacuity / mutation statement

Textual lane; the court is a quote-mapping (W4), not an executable.
Mutation check performed by grep: reverting either operand value would
leave an active `assetUnderManagement` occurrence outside comment
blocks — the grep in W3 (4 hits, all commented) is the flip witness.
No executable gate exists for this file in the workspace (the TTL is
documentation; the SHACL validation receipt cited in the compliance
doc predates this amendment and is a fence, not a gate I can re-run
against prose).

## W6 Fences (must-not-regress)

- `gate0` / SHACL validate_ontology.sh consumers of this TTL: the
  remap changes ontology individuals; if a SHACL shape pins
  `avec:assetUnderManagement` elsewhere in this file it would
  regress — grep shows none (only comment hits). No code in the
  workspace greps for `assetUnderManagement` (compliance doc :177
  reference is prose, other lane's file, untouched).
- Boundary semantics fence: `odrl:lt`/`odrl:gte` operators and
  `"50000000"^^xsd:decimal` values untouched — the $50M-exactly
  REFUSAL case is preserved, not weakened.

## W7 Standing

**ALIVE** for the file edit (grep + re-read exits above, this
session). The five-duty mapping is a textual court (W4) — verified by
quotation, marked as such. No executable court exists for a spec-doc
TTL; nothing rounded up.

## Divergences

- Dispatch said receipt at "the dir your dispatch names"; dispatch
  named no dir, and v26.10.31 has no `lanes/` — receipt placed in the
  campaign lanes dir per repo convention.
- The compliance doc's remap instruction names "annual gross revenue"
  but no IRI; I minted `avec:annualGrossRevenue` under the file's
  existing `avec:` prefix (https://avec.chatman.dev/ontology/26.10.31#),
  consistent with sibling operands like `avec:settlementLatencyNs`.
- v26.10.31 contains pre-existing >100-char lines (43, 249) outside my
  edit; left untouched (never-rewrite).
