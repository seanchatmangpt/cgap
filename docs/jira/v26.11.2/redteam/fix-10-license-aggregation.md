# FIX-10: License Aggregation Clause (redline for RED-04)

**Lane:** FIX-10 · **Date:** 2026-11-02 · **Standing:** ALIVE (deliverable
executed against on-disk referents this session) · **Mode:** redline text
only — this lane owns NO license file; the coordinator owns the edit.

**Subject under fix:** RED-04 BREACH-CONFIRMED
(`/Users/sac/cgap/docs/jira/v26.11.2/redteam/red-04-corporate-smurfing-loophole.md:146-157`):
the instrument has no revenue-aggregation clause; affiliate rings,
dispersed LLC trusts, and unconsolidated JVs escape the §2.2
classification; the $50M figure lives only in the compliance TTL layer
while LICENSE.md advertises it as an instrument term.

**Authority:** red-04 W6 required-fixes items 1-4
(red-04-corporate-smurfing-loophole.md:159-172). This document supplies
the redlined clause TEXT; it does not edit
`LICENSE-P28-SIL.md`, `LICENSE.md`, or
`docs/jira/v26.10.31/34-p28-sil-compliance.md`.

---

## Part 1 — Redlined replacement for §2.2

Base text quoted verbatim from
`/Users/sac/cgap/LICENSE-P28-SIL.md:101-108` (§2.2). Drafting convention:
`[-deleted-]` / `{+inserted+}`.

### 1a. Lead paragraph of §2.2 (line 103), redlined

> Deployer $\mathcal{D}_j$ (including all consolidated parent entities,
> joint-venture vehicles, and controlled special-purpose subsidiaries)
> {+acting severally or as a member of a Controlled Group as defined in
> §2.2.1,+} exits $\mathcal{U}_{\text{exempt}}$ and transitions into
> **Institutional Exploitation Status** ($\mathcal{D}_j \in
> \mathbf{E}_{\text{inst}}$) upon satisfying any of the following statutory
> functional boundaries {+measured, where $\mathcal{D}_j$ is a member of a
> Controlled Group, at the Group Aggregation Level per §2.2.2+}:

### 1b. New §2.2.1 — "Controlled Group" definition (inserted after §2.2 bullet list)

Text (as it would appear inserted as §2.2.1 in the instrument):

> 2.2.1. **"Controlled Group" ($\mathbf{G}_{\text{ctrl}}$):**
>
> The set of legal persons connected to Deployer $\mathcal{D}_j$ by any of
> the following, evaluated without regard to jurisdictional
> veil-formation formalities:
>
> * **Ownership prong:** a person holding, directly or indirectly through
>   one or more intermediaries, more than fifty percent (50%) of the
>   voting power or economic value of another person, including any
>   interest held through nominees, custodians, or voting agreements;
> * **Special-Purpose-Entity prong:** any bankruptcy-remote,
>   thin-equity, or ring-fenced vehicle (SPE, SPC, SPV, orphan trust, or
>   equivalent) whose equity is held, optioned, or economically
>   enjoymented by a person within an Ownership prong chain, whether or
>   not majority voting power is formally conveyed;
> * **Trust prong:** any trust, estate, or fiduciary arrangement of
>   which a person within an Ownership prong chain is grantor, trustee
>   with discretionary distribution power, or discretionary beneficiary
>   receiving more than a de minimis share of surplus;
> * **Joint-Venture prong:** any unincorporated or incorporated joint
>   venture, including a 50/50 or dispersed-interest JV that escapes
>   consolidation under ASC 810, whose net-results interest is held by
>   persons within a single Ownership prong chain in aggregate;
> * **Attribution prong:** stock, profit interest, or surplus held by a
>   family member, related grantor trust, or person acting in
>   coordination (within the meaning of IRS § 318(a)(3) constructive
>   ownership as modified herein) is attributed to the acquiring person.
>
> Membership in a Controlled Group is a question of economic reality,
> decided by these prongs, not by the entity's state-law form,
> consolidation status under ASC 810, or absence of a single
> majority holder.
>
> 2.2.2. **Group Aggregation Level:**
>
> Each statutory functional boundary of this §2.2 is evaluated with
> respect to the aggregate of the Controlled Group: the combined
> gross receipts, consolidated balance-sheet figures, market
> capitalization, regulatory reporting status, and contractor status of
> all members of $\mathbf{G}_{\text{ctrl}}$ are attributed to each
> member. If any member of $\mathbf{G}_{\text{ctrl}}$ satisfies a
> boundary at the Group Aggregation Level, every member satisfies it;
> every member transitions to Institutional Exploitation Status and
> $\mathcal{D}_j \in \mathbf{E}_{\text{inst}}$ holds group-wide,
> triggering §2.3 (Topological Transition Invariant) per member. No
> member may partition deployments, meters, or executions among members
> to remain below any boundary: measurement is at the group, never at
> the individual license holder. Where group members disagree as to
> classification, the strictest reading governs pending §7.2
> arbitration.
>
> 2.2.3. **Anti-Structural-Avoidance (extending §3.2):**
>
> Any corporate reorganization, incorporation of a new vehicle, transfer
> of operations or assets between members or to a new entity,
> trust formation, or coordinated multi-entity deployment arrangement
> undertaken with classification-avoidance as a principal purpose —
> that is, where a person's principal purpose is to cause any member to
> remain in $\mathcal{U}_{\text{exempt}}$ at the Group Aggregation
> Level — constitutes an **Act of Disruption or Circumvention Dialogue**
> within the meaning of §3.2, with all consequences of §3.3 (Statutory
> Evidentiary Preservation and Spoliation Invariants) attaching.
> Documentation evidencing such a principal purpose includes internal
> memoranda, entity-formation filings, tax-structure analyses, and
> advisor communications; discovery of a single such document raises the
> inference of a principal purpose which Deployer may rebut only by
> preponderant contemporaneous business evidence of non-tax,
> non-classification commercial purpose existing before the
> reorganization. Selection of a deployment vehicle AFTER Institutional
> Exploitation Status attaches does not restore $\mathcal{U}_{\text{exempt}}$
> for any member of the group.

---

## Part 2 — Reconciliation of the $50M figure

State of the figure today (re-read on disk this session):

| Document | What it says | Path:line |
|---|---|---|
| Instrument | NO dollar figure; statutory-test delegation | LICENSE-P28-SIL.md:94-108 |
| Compliance/TTL | lt 50M → harbor; gte → duty; $50M is duty | 34-p28-sil-compliance.md:75-79 |
| README | "$50M universal safe harbor" as instrument term | LICENSE.md:13 |

**Recommendation: import the figure INTO the instrument (Option A),
not remove the advertisement.** Reasons:

1. LICENSE.md:12-15 already names the figure publicly; removing it
   creates a doc-drift debt and weakens the marketing claim that the
   safe harbor is objective.
2. The instrument already delegates to statutory tests that are all
   dollar-denominated or near-equivalents (IRC § 448(c) ≈ $30M-31M
   inflation-adjusted; EU Large Undertaking ≈ balance-sheet/revenue
   thresholds). A pinned numeric floor closes the gap between the
   statutory tests and the TTL's machine-readable `odrl:gte/lt` rule,
   which otherwise classifies at $50M while the instrument's § 448(c)
   cross-reference triggers near ~$31M — an internal contradiction
   RED-04's meter exploits.
3. The compliance layer already pins exact boundary semantics
   (34-p28-sil-compliance.md:75-79); importing them verbatim keeps
   TTL, doc, and instrument consistent with zero re-derivation.

Redline: insert a new §2.4 (after §2.3, LICENSE-P28-SIL.md:110-112):

> 2.4. **Numeric Safe-Harbor Floor:**
>
> Notwithstanding the statutory boundaries of §2.2, a Deployer (or,
> per §2.2.2, any member of its Controlled Group) whose aggregate
> annual gross revenue for the prior completed fiscal year is strictly
> less than Fifty Million United States Dollars ($50,000,000.00) is
> presumed to remain within $\mathcal{U}_{\text{exempt}}$; a member at
> or above $50,000,000.00 is within Institutional Exploitation Status
> and owes the regularization duty of §4.3. The measure is annual
> gross revenue (audited figure, prior fiscal year), resolving the
> measure mismatch recorded at
> docs/jira/v26.10.31/34-p28-sil-compliance.md:60-71; the TTL
> `odrl:leftOperand` must be remapped from `avec:assetUnderManagement`
> to annual gross revenue in the same change. Boundary semantics:
> comparison is ≥ (duty branch at exactly $50,000,000.00), per
> 34-p28-sil-compliance.md:75-88.

Coordination note: because the measure pin (AUM vs gross revenue) is
flagged as a must-reconcile item in the compliance doc, landing §2.4
requires the coordinator to also land the TTL operand remap, or the
instrument and machine layer will diverge in the opposite direction.

---

## Part 3 — Closing analysis: each RED-04 attack structure trips a clause

**Structure 1 — wholly-owned sub-$50M SPEs sheltering a $500M enterprise.**
RED-04 found this caught only at the parent, with no mechanism attributing
parent status downward (red-04:75-93). Under the new text: the SPEs are
Controlled Group members via the Ownership prong (>50% value through
intermediaries, §2.2.1 bullet 1) and the SPE prong (thin-equity,
ring-fenced vehicles "whether or not majority voting power is formally
conveyed", §2.2.1 bullet 2); §2.2.2 attributes the $500M group revenue to
each member — "every member transitions to Institutional Exploitation
Status ... triggering §2.3 (Topological Transition Invariant) per member."
The per-entity `Asc450Meter` partition (red-04:117-124) is barred
explicitly: "No member may partition deployments, meters, or executions
among members to remain below any boundary" (§2.2.2). **TRIPPED.**

**Structure 2 — distributed LLC trusts / non-controlled affiliate rings.**
RED-04 found no definition of "controlled" and no catch for dispersed
control (red-04:95-107). Under the new text: the Trust prong reaches
dispersed-trust arrangements via trustee discretionary-power or
beneficiary-share tests without majority control (§2.2.1 bullet 3);
affiliate rings are caught because "Membership in a Controlled Group is a
question of economic reality ... decided by these prongs, not by ... the
entity's state-law form ... or absence of a single majority holder"
(§2.2.1 closing paragraph). The §318(a)(3)-style Attribution prong
(§2.2.1 bullet 5) pools constructively-owned stakes. **TRIPPED.**

**Structure 3 — SPE joint ventures.**
RED-04 found a 50/50 non-consolidated JV tested on its own numbers
(red-04:109-115). Under the new text: the Joint-Venture prong expressly
covers "any unincorporated or incorporated joint venture, including a
50/50 or dispersed-interest JV that escapes consolidation under ASC 810"
(§2.2.1 bullet 4); the parents' aggregate interest is tested at the Group
Aggregation Level (§2.2.2). **TRIPPED.**

**Structure 4 (§3.2 gap) — restructuring that leaves receipts intact.**
RED-04 found §3.2 scoped to receipt/receipt-hash attack
(red-04:126-134). Under the new text: §2.2.3 makes "any corporate
reorganization, incorporation of a new vehicle, transfer of operations or
assets ... undertaken with classification-avoidance as a principal
purpose" an Act of Disruption or Circumvention Dialogue, importing §3.3's
legal-hold and spoliation consequences; a single structuring memo raises
a rebuttable inference. **TRIPPED.**

**Structure 5 — $50M figure mismatch.** The instrument now states the
figure with the same ≥ boundary semantics as the TTL
(§2.4 redline), eliminating the LICENSE.md:13 advertisement-orphan.
**TRIPPED.**

Anti-vacuity note for the coordinator's landing court: after the
license edit lands, a court should mutate a sandbox copy of
LICENSE-P28-SIL.md by deleting §2.2.1 and re-running the
structure-trip checks above — each must flip from TRIPPED to ESCAPES
(or the court is vacuous). Sandbox law: /tmp copy, sha256 fail-closed
restore, mtime bump after restore.

---

## Commands / evidence (this session)

- `Read docs/jira/v26.11.2/redteam/red-04-corporate-smurfing-loophole.md`
  (183 lines, exit 0) — all attack structures and clause gaps re-read.
- `Read LICENSE-P28-SIL.md` (291 lines, exit 0) — §1.6:82-84, §2.1:94-99,
  §2.2:101-108, §2.3:110-112, §3.2:122-131 re-read verbatim before
  drafting redlines.
- `Read LICENSE.md` (19 lines, exit 0) — :13 advertisement re-read.
- `Read docs/jira/v26.10.31/34-p28-sil-compliance.md:55-109` (exit 0) —
  TTL boundaries :75-79, measure mismatch :60-71 re-read.
- `Write/Edit docs/jira/v26.11.2/redteam/fix-10-license-aggregation.md`
  in 4 chunks (<50 lines each) — exit 0, on-disk verified by tool state.
- No git commands; no license file edits (coordinator-owned); shared
  tree untouched except this one owned file.

---

## ERRATA (2026-11-02, lane X-02FIX) — appended only, no rewrite above

This block supersedes rows/line-refs in the body; the body above is
preserved verbatim as redline history.

1. The Part 2 table row "Instrument | NO dollar figure; statutory-test
   delegation | LICENSE-P28-SIL.md:94-108" (:123 above) is SUPERSEDED:
   GAP-01 landed §2.4 (Numeric Safe-Harbor Floor) into the instrument;
   the $50,000,000.00 figure now lives at LICENSE-P28-SIL.md:134-136.
2. The Part 2 table row "Compliance/TTL | ... | 34-p28-sil-compliance.md:75-79"
   (:124 above) and the body cites at :142, :158 (:60-71), :162 (:75-88),
   and :237 (":75-79, measure mismatch :60-71") carry pre-GAP-01b line
   numbers. Post-GAP-01b, in docs/jira/v26.10.31/34-p28-sil-compliance.md
   the measure-mismatch section is :66-88 (heading at :66) and the
   boundary-value section is :90-105 (heading at :90); both verified
   on disk this session (`awk` range read, exit 0).
3. LICENSE.md:13 advertisement was corrected this session (lane X-02FIX,
   R7): "$50M universal safe harbor" → "$50,000,000.00 annual gross
   revenue safe-harbor floor per LICENSE-P28-SIL.md §2.4 (group-aggregated
   per §2.2.1-2.2.3, v2.1.1)".
4. LICENSE-P28-SIL.md §2.4 internal cross-cites (:136) were corrected
   this session (lane X-02FIX, R2): :60-71 → :66-88, :75-88 → :90-105,
   with an inline dated note; the residual TTL `odrl:leftOperand` remap
   is lane I-11's in-flight surface and is NOT landed here.
