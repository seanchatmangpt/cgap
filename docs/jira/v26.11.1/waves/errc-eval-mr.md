# ERRC-C receipt — packs M–R evaluation (ggen-marketplace)

- lane: ERRC-C (court-auditor, read-only on ~/ggen-marketplace)
- subject: /Users/sac/ggen-marketplace/packs (m–r prefix, 74 packs)
- repo doctrine read: /Users/sac/ggen-marketplace/CLAUDE.md + AGENTS.md (AGENTS.md authoritative)
- only write: this receipt. No git writes; /tmp sandboxes only.

## Commands + exits (mine)

| command | exit |
|---|---|
| ls packs/ (enumerate) | 0 |
| per-pack pack.toml field dump (name/version/description, 74/74 have pack.toml, 74/74 ttl, 57/74 gates/) | 0 |
| python3 scripts/marketplace.py validate | 2 (REFUSED:MANIFEST_MISSING:smalt-core-pack — fail-closed, pre-existing, outside m–r scope) |
| marketplace.py catalog x2 + cmp | cmp 0 (deterministic) |
| marketplace.py fingerprint | 2 (same smalt-core-pack refusals) |
| python3 scripts/qualify_packs.py --report /tmp/pack-qualification.json | 2 (same refusals; no report written) |
| python3 scripts/check_gate_witness_courts.py | 0 — 47 configured packs all ALIVE with require_fail+require_pass |
| ggen --version | ggen 26.10.11 |
| rdflib gate evaluation (consolidation-court pack) | 0, all 4 gates 0 rows baseline |
| /tmp mutation court (consolidation-court pack) | gate 010 flips 0→20 rows; sha256 restore OK |

## Courts run (anti-vacuity)

1. Marketplace validate/fingerprint/qualify all REFUSE with subject-missing exit 2
   (smalt-core-pack lacks pack.toml+ontology) — the repo's own fail-closed courts flip
   on real missing-subject conditions; not vacuous.
2. Gate-witness court: 47 packs carry pass+fail witness cases (anti-vacuity built in).
   Among m–r packs covered: premature-actuation-survival-pack(10 cases),
   projection-matrix-compose(3), qri-consumer-binding-pack(11),
   qri-qualification-profile-pack(10), qualified-capability-ecology-pack(12), rust-pack(5).
3. Direct mutation court on pack-consolidation-court-pack (in /tmp sandbox, sha256
   fail-closed restore): stripping all pcc:canonical triples flips gate
   010_required_relation.rq from 0 to 20 violation rows → court proven non-vacuous.
   Gates 020/030/040 did not flip under this mutation and have no witness cases →
   flip status UNVERIFIED for those three.

## Findings — ERRC classification (m–r tranche)

ELIMINATE:
- smalt-core-pack: manifest+ontology missing entirely (validate/fingerprint/qualify
  all exit 2) — restore or retire the pack directory; it currently blocks the whole
  marketplace acceptance sequence for every pack.
- 17 of 74 m–r packs (mcpp, mfact, mmdio, neako-web-deckgl, ops-dashboard-legend,
  rmcp, osx-clnr etc.) are single-consumer semantic profiles at 0.x with no gates
  beyond structure — consolidate via pack-consolidation-court-pack HARD_MERGE/
  META_COMPOSE law rather than growing in place.
- read (REDUCE): 57/74 have gates dirs but only ~6 m–r packs have pass/fail witness
  courts — gate directories without witnesses are prose-with-file-extension.

RAISE:
- pack-consolidation-court-pack: proven non-vacuous gate 010; add pass/fail witnesses
  for gates 020/030/040 (the court's own law).
- readme-diataxis-pack: four-quadrant Diátaxis projection, end-to-end verified
  against a real consumer; strongest docs-manufacturing capital in the tranche.
- rust-doc-hdit-pack: code-surface→RDF→hypervector doc/claim verification — direct
  fit for cgap's "generated artifacts never hand-edit" law; courts/ dir present.

CREATE-gap:
- No m–r pack measures hop-ratio or projection-vs-handwritten ratio; the projection/
  hop-ratio instrument lens is unowned. prior-art-admission-pack owns anti-reinvention
  law (reuse→compose→extend→invent) but nothing measures it per wave.
- No pack runs a Divan/PARETO bench membrane for marketplace pack capital (bench
  membranes live in consumer repos only).

## Packs cgap should consume

1. rust-doc-hdit-pack — project crates/cgap-* code surfaces to RDF and diff docs vs
   code (enforces the generated-artifacts law mechanically).
2. readme-diataxis-pack — restructure READMEs/CLAUDE.md docs tree Diátaxis-style.
3. prior-art-admission-pack — machine-checkable anti-reinvention rows for each new
   cgap lane before construction.
4. otel-weaver-ocel-pack — cgap court/audit spans → OCEL events (feeds C02).
5. pack-consolidation-court-pack — record cgap-pack↔cgap repo drift rows as
   ConsolidationRelations instead of parallel README truths.

## Top-3 slice innovations

1. Extend check_gate_witness_courts.py config to require require_fail witnesses for
   every gates/*.rq in every m–r pack (currently ~6/74 covered) — converts all
   unproven gates into flip-proven courts or loud misses.
2. Wire rust-doc-hdit-pack's hypervector doc/code diff as a cgap gate (cgap already
   fails CI on hand-edited generated files; this proves doc claims against code).
3. Adopt consolidation court + prior-art-admission as pre-construction admission for
   the A01–A20 moonshot lanes: no new lane crate without a ConsolidationRelation row
   proving no existing pack/crate owns the semantics.

## Standing

ALIVE for the evaluation itself (all commands executed this session, exits cited).
Marketplace-wide acceptance is BLOCKED upstream by smalt-core-pack (exit 2) —
pre-existing, outside m–r scope. Particular gates UNVERIFIED as noted.

## Final JSON

```json
{"lane":"ERRC-C","files_written":["/Users/sac/cgap/docs/jira/v26.11.1/waves/errc-eval-mr.md"],"commands":["ls packs (0)","pack.toml census (0)","marketplace.py validate (2, REFUSED smalt-core-pack)","catalog x2+cmp (0)","fingerprint (2)","qualify_packs.py (2, no report)","check_gate_witness_courts.py (0, 47 ALIVE)","rdflib gate baseline (0)","mutation court flip 0→20 rows + sha256 restore OK"],"courts_run":["marketplace validate fail-closed flip (exit 2 on missing subject)","gate-witness court 47 packs ALIVE","consolidation-court gate 010 mutation flip in /tmp sandbox, byte-exact restore"],"standing":"ALIVE (evaluation); marketplace acceptance BLOCKED upstream by smalt-core-pack (pre-existing, out of scope)","divergences":["smalt-core-pack missing manifest+ontology blocks validate/fingerprint/qualify repo-wide (exit 2)","gates 020/030/040 of pack-consolidation-court-pack not flip-proven and not in witness-court config","only ~6/74 m-r packs have pass/fail witness courts; 57 have gates dirs"]}
```
