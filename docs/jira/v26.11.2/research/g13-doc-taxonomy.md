# R13 — ggen docs/book taxonomy (Explore brief)

Lane: R13. Source: `/Users/sac/ggen/docs/`, `/Users/sac/ggen/book/`. Read-only; claims cited
file:line.

## 1. Governing standard (house style)

`docs/CORE_TEAM_DOCUMENTATION_STANDARD.md` is the normative style law:

- **Doc classes** (:30-45): `NORMATIVE`, `OPERATIONAL`, `REFERENCE`, `EVIDENCE`, `DECISION`,
  `PLANNING`, `HISTORICAL`, `GENERATED`; unclassifiable = `UNKNOWN`, must not be linked as
  authoritative.
- **Claim standing vocabulary** (:56-66): `ALIVE`, `PARTIAL_ALIVE`, `BLOCKED`,
  `BUILD_BROKEN`, `UNKNOWN`, `UNSUPPORTED`. Banned prose: "works", "complete",
  "production-ready", "verified" without standing + scope + revision + falsifier.
- **Required YAML header** (:70-82): `class / standing / scope / source_revision /
  last_verified / owner / falsifier`.
- **Authority hierarchy** (:84-96): machine law > admitted ontologies > claims ledgers >
  normative docs > operational guides > reference > evidence > planning/historical. Conflicts
  are not reconciled rhetorically; lower authority points to the owner.
- **Operational guide structure** (:104-120): fixed 8-section order — Outcome → Boundary →
  Admission → Procedure → Receipt → Replay → Failure map → Rollback.
- **Verification ladder** (:131-136): unit → integration → e2e → chaos → stress → benchmark →
  verifier report; ALIVE only at the named rung.
- **Language rules** (:146-152): direct system language ("The verifier exits 0 at revision X"),
  no superlatives, no machine-specific absolute paths in commands.
- **PR receipt requirement** (:160-170): doc PRs report revisions, claims added/retired,
  falsifiers, unresolved UNKNOWN/BLOCKED items.
- Core doctrine equation across docs: **A = μ(O*)**.

## 2. Doc taxonomy

| Category | Location | Exemplar |
|---|---|---|
| Diátaxis | docs/tutorials/, how-to/, reference/, explanation/ | docs/DIATAXIS_BEST_PRACTICES.md:1-60 (tone/goal/anti-pattern per quadrant) |
| ADRs (DECISION) | docs/adr/ADR-NNN-Topic.md | ADR-010-Event-Sourcing.md — Status/Date/Context/Deciders; Problem→Decision→Rationale→Consequences |
| Campaign programs | docs/jira/vX.Y.Z/ with EPIC.md + TICKET-NNN-slug.md + README.md index | docs/jira/README.md (versioned index w/ Standing column + "Standing law"); docs/jira/v26.7.23/TICKET-002-...md:1-70 (Status/Parent/Source/Objective/Current state/Target state/**Projection classification** (template% vs custom%)/Inputs/Outputs/Template responsibility/Domain-data responsibility/Custom-code boundary/Exclusions) |
| SJIRA | docs/sjira/v26.10.8/ | single-ticket variant |
| Release receipts (EVIDENCE) | docs/vX_Y_Z_*_receipt.md | v26_10_11_release_receipt.md:1-80 — Identity table (branch/HEAD/state per subject) → Consequence (commit-hash-cited) → Verification Ladder (each rung session-reported vs witnessed) → Replay (cold checkout) sh block |
| Phase receipts | docs/v26_10_10_phase1_receipt.md:15-45 | receipt table Claim / Command-evidence / Result; DONE/NOT DONE/VERIFIED; explicit NOT-DONE retention |
| Runbooks (OPERATIONAL) | docs/v26_10_11_publish_runbook.md:10-30 | Preconditions → numbered HALT points before irreversible acts → ordered procedure → post-publish receipt filing |
| Release notes | docs/v26_10_11_release_notes.md:1-30 | Highlights (each bullet carries commit hash) → Breaking/Notable → Internal → Verification → annotated tag message in fenced block |
| Definition of Done (NORMATIVE) | docs/DEFINITION_OF_DONE.md:1-30 | "GOLDEN RULE: all five gates must pass"; gate table (command/timeout/duration/purpose); per-gate success criteria + failure repair + Andon level; JSON twins (DEFINITION_OF_DONE.json, PERFORMANCE_/TESTING_/SECURITY_/DOCUMENTATION_) |
| Audits | docs/audits/YYYY-MM-DD-slug-audit.md; ACCEPTED_ADVISORIES.md | |
| Security suite | docs/security/ (ARCHITECTURE, TESTING, INCIDENT_RESPONSE, SAFE_CODING, CHECKLIST) | |
| Gates | docs/gates/every-binding-has-template.md etc. | one file per enforcement gate, law-as-sentence names |
| Archive (HISTORICAL) | docs/archive/{2025_docs, 2026_docs_pre_ggen_engine, legacy_structure} | |
| Specs | docs/specs/<feature-slug>/ with fixtures | |
| Docs-as-data | docs/docs.tree.ttl:9-30 (every doc as prov:Entity/dcat:Distribution with spdx:checksum sha256 + sh:conforms) + docs.tree.json | |

## 3. book/ taxonomy

mdBook (book/book.toml), SUMMARY.md — **264 numbered patterns in named Fields** ("Field I —
The Whole Manufacturing System"), each `NNN-kebab-slug.md` (e.g.
`receipts/195-inspection-receipts.md`, `receipts/197-divergence-ledger.md`).

Fixed per-pattern template (book/src/receipts/195-inspection-receipts.md:1-60): blockquote
**Pattern NNN · Field** + **Standing rule** epigraph → Context → Problem → Forces →
**Governing law** (renders A = μ(O*)) → Therefore → Configuration (5 numbered steps: admit →
apply at owning layer → actuate via declared path → verify in clean consumer → preserve
receipt) → **Crate alignment**.

Receipt periphery: `book/schemas/inspection-receipt.schema.json:1-35` (required
subject/version/inputs/checks/outcome; outcome enum verified|refused|bounded|unsupported|
inconsistent); `book/.ggen-v2/receipt.json` + receipt-log.jsonl; `book/SOURCE_NOTES.md:1-40`
(provenance ledger distinguishing "as authored" vs "Independently rerun YYYY-MM-DD" with exact
commands); MANIFEST.sha256, BOOK_INVENTORY.json, GAP_REPORT.md. Pattern #197 = "Corrective-
Divergence Ledger": every actuation terminates in durable receipt or typed refusal.

## 4. Naming conventions

- Campaign docs: `vX_Y_Z_<slug>.md` (underscores) top-level; `vX.Y.Z` dirs under jira/.
  Receipts/runbooks/notes trio per release.
- ADRs `ADR-NNN-Title-Case-Slug.md`; tickets `TICKET-NNN-kebab-slug.md`; gates
  `every-X-has-Y.md`; book patterns `NNN-kebab-slug.md` by Field dir.
- Receipt IDs `GALL_CONFORM_001.md`, `LIVING_LSP_ALIVE_001.yaml` (SCREAMING_SNAKE + numeric).
- Header YAML exactly the six fields from §1.

## 5. Prescription for cgap v26.11.2 docs

1. Adopt the class + standing YAML header verbatim (six fields).
2. Release artifact triple: notes (commit-hash-cited bullets) → runbook (HALT points before
   irreversible acts) → release receipt (Identity / Consequence / Verification Ladder with
   per-rung witnessed-vs-reported / Replay commands).
3. Campaign work as EPIC.md + TICKET-NNN-slug.md with Projection classification + exclusions.
4. Receipt tables: Claim | Command/evidence | Result, DONE/NOT DONE/VERIFIED with explicit
   NOT-DONE retention.
5. Machine side: docs.tree checksum ledger + JSON Schema receipts with the five-value outcome
   enum.
6. Highest-value style carries-over: banned wording list + 8-section operational-guide order.
