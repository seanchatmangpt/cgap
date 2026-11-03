# RED-04: Safe Harbor Abuse & Corporate Smurfing Loophole

**Lane:** RED-04 · **Date:** 2026-11-02 · **Standing:** ALIVE (analysis executed
against on-disk referents this session) · **Mode:** read-only red team

## W1 — Target claim under attack

"$50M Gross Revenue Safe Harbor" (dispatch premise) as an anti-smurfing
barrier in the P28-SIL license.

## W2 — On-disk referents

The **license instrument itself never states "$50M"**. Grepped the whole repo
for `$50`, `50M`, `gross revenue`, `Safe Harbor`, `Tier 2`,
`Regularization`:

- `/Users/sac/cgap/LICENSE-P28-SIL.md` (the instrument, 291 lines): zero
  occurrences of `$50M`, `Gross Revenue Safe Harbor`, or any dollar figure.
  The safe harbor is defined by **statutory cross-reference**, not a revenue
  number:
  - §2.1 (lines 94-99): perpetual royalty-free license "provided strictly
    that $\mathcal{D}_j \notin \mathbf{E}_{\text{inst}}$".
  - §2.2 (lines 101-108): four objective statutory tests — Exchange Act
    Rule 12b-2 (Accelerated/Large Accelerated Filer, line 105); IRC
    § 448(c) gross-receipts test (line 106); EU Directive 2013/34/EU
    Art. 3(4) Large Undertaking / Directive (EU) 2024/1760 (line 107); CAS
    48 C.F.R. § 9903.201-1 or FAR Part 19 Tier-1 Prime (line 108).
- The **$50M figure lives outside the instrument**:
  - `/Users/sac/cgap/docs/jira/v26.10.31/34-p28-sil-compliance.md:70-83` —
    boundary pinned: `odrl:lt` 50,000,000 USD → harbor; `odrl:gte`
    50,000,000 USD → regularization duty; `$50,000,000.00 exactly` is
    inside the duty branch.
  - `/Users/sac/cgap/LICENSE.md:13` — README asserts a "$50M universal safe
    harbor", a claim the instrument does not itself contain.
  - Code: `/Users/sac/cgap/crates/cgap-p28/src/asc450.rs` +
    `/Users/sac/cgap/docs/jira/v26.11.1/lanes/A18-asc450-proof.md:87` —
    `Asc450Meter::new(safe_harbor_ceiling_cents: u64)` is a **per-entity**
    accumulator; the ceiling is a runtime parameter, not an aggregation
    rule.

**Classification of the dispatch target:** the claim "$50M Gross Revenue
Safe Harbor" is **ALIVE but MISPLACED** — the threshold exists on disk in
the TTL/compliance layer (34-p28-sil-compliance.md:70-83) and as a code
parameter, but the license instrument's own operative text (§2.2) does not
use it; the instrument delegates to statutory tests. Any smurfing verdict
must therefore be run against BOTH the statutory tests (instrument) and the
$50M `gte/lt` rule (ontology layer). Note also
34-p28-sil-compliance.md:63-69: the threshold **measure** (AUM vs annual
gross revenue) is an unresolved must-reconcile item — the safe harbor is
ambiguous even for a single entity.

## W3 — Clause-by-clause smurfing attack

The instrument's aggregation surface is exactly one parenthetical, §2.2
(line 103), quoted verbatim:

> Deployer $\mathcal{D}_j$ **(including all consolidated parent entities,
> joint-venture vehicles, and controlled special-purpose subsidiaries)**
> exits $\mathcal{U}_{\text{exempt}}$ and transitions into **Institutional
> Exploitation Status** ... upon satisfying any of the following statutory
> functional boundaries:

and the Deployer definition, §1.6 (lines 82-84), quoted verbatim:

> Any natural person, sovereign state, corporate entity, cross-border
> holding conglomerate, distributed autonomous organization, or algorithmic
> agent matrix that directly or indirectly instantiates, provisions,
> finances, or **derives economic surplus** from an Execution.

There is **no revenue-aggregation clause anywhere in the instrument** — no
clause attributes a group's combined gross receipts to each member for
purposes of the §2.2 tests. The parenthetical expands WHO counts as a
Deployer; it does not combine WHOSE numbers.

### Structure 1: wholly-owned sub-$50M SPEs sheltering a $500M enterprise

**BREACH-CONFIRMED (partial — survives via per-entity testing).** Each SPE
is evaluated standalone against the four §2.2 tests: a deliberately
thin-equity SPE below every statutory trigger (not an
accelerated filer, under the § 448(c) receipts test, not an EU Large
Undertaking, not a FAR prime) stays in $\mathcal{U}_{\text{exempt}}$ with a
perpetual royalty-free license. The §2.2 parenthetical names "controlled
special-purpose subsidiaries" as Deployers but never says the parent's
$500M revenue attributes downward. Nothing in the instrument requires
combined-group testing.

Counterweight (why only partial): §1.6's "directly or indirectly ...
derives economic surplus" makes the PARENT a Deployer of subsidiary
executions, and the parent alone fails the §2.2 tests; §2.3 (line 112)
binds "all past, present, and subsequent Executions" once Institutional.
So a wholly-owned structure is caught **at the parent**, but the license
gives no mechanism to attribute parent status to the sub-threshold
children that actually run the executions.

### Structure 2: distributed LLC trusts / non-controlled affiliates

**BREACH-CONFIRMED (uncatched).** The instrument's only control hook is
"**controlled** special-purpose subsidiaries" (§2.2, line 103) and
"cross-border holding conglomerate" (§1.6, line 84) — neither term is
defined anywhere in the instrument. A ring of cross-holding affiliates
below statutory thresholds at every node, or an LLC trust where no member
holds majority control, satisfies the four tests at each node. §1.6's
"derives economic surplus" prong is the only possible catch, and for a
trust structure with dispersed surplus claims it is a litigation question,
not a license predicate — exactly the "subjective natural-language
interpretation" the preamble (line 15) claims to have "completely
superseded".

### Structure 3: SPE joint ventures

**BREACH-CONFIRMED (uncatched).** "Joint-venture vehicles" is named in the
§2.2 parenthetical, but a 50/50 JV where neither parent consolidates under
ASC 810 (variable-interest analysis can leave both unconsolidated) is
tested on its own numbers. No clause allocates JV revenue to parents for
classification.

### Structure 4: deployment slicing (per-entity meters)

**BREACH-CONFIRMED.** The code referent aggregates only within one
`Asc450Meter`: `/Users/sac/cgap/crates/cgap-p28/src/asc450.rs` —
`Asc450Meter::new(safe_harbor_ceiling_cents: u64)` is instantiated
per-entity (A18-asc450-proof.md:87-90). Successive SPEs each running their
own deployments each keep their own accumulator under the ceiling; nothing
in code or license aggregates meters across legal entities.

## W4 — The anti-avoidance gap (what each clause does NOT cover)

- **§3.2 Circumvention Dialogue (lines 122-131)** — the closest thing to an
  anti-avoidance clause — is scoped verbatim to communications "whose
  optimization target is" $\partial \mathbf{R}_{\text{receipt}} /
  \partial \mathfrak{E}_t \equiv 0$ and to modifying/forking/patching "to
  strip, alter, mock, or reroute state deltas or BLAKE3 receipt hashes."
  Corporate restructuring that leaves receipts fully intact is **not**
  Circumvention Dialogue. Drafting the SPE memo does not void the covenant.
- **§9.3 Constructive Non-Compliance (lines 266-268)** — tool-side failure
  to detect status is no defense; it says nothing about entity
  structuring.
- **§8.1 Non-Decomposition (lines 238-240)** — forbids parsing the
  instrument piecemeal; it is an interpretation rule, not an anti-smurfing
  rule.
- **§7.3 Severability + reformation (lines 230-232)** — reforms invalid
  clauses; does not import a group-revenue test that was never drafted.

## W5 — Honest verdict

**Does the license as written catch entity-splitting? NO — PARTIAL catch
only.** Wholly-owned SPEs are caught at the consolidated parent via §1.6
"derives economic surplus" + §2.3 retroactivity, but no clause attributes
parent metrics to sub-threshold children; genuinely non-controlled
structures (affiliate rings, dispersed LLC trusts, unconsolidated JVs)
escape the §2.2 tests entirely at every node. The instrument has no
revenue-aggregation clause and no anti-avoidance clause reaching corporate
restructuring. The $50M ceiling — wherever it is enforced (TTL
`odrl:gte/lt`, per-entity `Asc450Meter`) — is enforced **per Deployer,
never per group**. A $500M enterprise can license the substrate through
arbitrarily many sub-threshold controlled or non-controlled vehicles with
the receipts fully intact and §3.2 never triggered.

## W6 — Required fixes (for the drafter; this lane changed nothing)

1. Add a group-aggregation rule: classification metric tested on the
   "Controlled Group" (defined by >50% vote/value ownership chain, ASC 810
   consolidation, or IRS § 52/1563 analog), with each member jointly
   classified at the group level.
2. Define "controlled" and "cross-border holding conglomerate" (§1.6/§2.2).
3. Extend §3.2 or add an express anti-avoidance clause covering structural
   reorganization undertaken with classification-avoidance as a principal
   purpose.
4. Reconcile the measure (AUM vs gross revenue —
   34-p28-sil-compliance.md:63-69) and either import the $50M figure into
   the instrument or demote LICENSE.md:13's "$50M universal safe harbor"
   claim.

## W7 — Commands / evidence

- `grep -rn -iE "Gross Revenue|Safe Harbor|50M|Tier 2|Regularization" ...` —
  located instrument + threshold layers (exit 0).
- `Read LICENSE-P28-SIL.md` (291 lines) — all quoted clauses verbatim from
  this session's read.
- `grep` + `sed` on 34-p28-sil-compliance.md, LICENSE.md,
  A18-asc450-proof.md (exit 0).
- No mutations, no git commands, shared tree untouched; single file written
  (this one).
