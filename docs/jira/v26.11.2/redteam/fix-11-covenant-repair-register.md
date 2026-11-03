# FIX-11 — Covenant Repair Register (P28-SIL v2.1.0)

Lane: FIX-11 (hardening wave, v26.11.2). Consolidates the BREACH-CONFIRMED
license-posture findings from RED-05, RED-06, RED-08 into ONE repair
register. Sole written file: this one. License text NOT edited (law: the
instrument carries its own ERRATA pattern — LICENSE-P28-SIL.md:303-314 —
each repair below is drafted as a rider, never a silent rewrite).

Inputs (read this session):
- `LICENSE-P28-SIL.md` (333 lines post-GAP-01/GAP-01b, URN line 3; line
  numbers below re-verified post-amendment, see ERRATA at EOF)
- `redteam/red-05-declaratory-judgment-complaint.md` (BREACH-CONFIRMED
  verdict table, :209-223)
- `redteam/red-06-asc450-audit-defusal.md` (defusal points D1-D5, :47-139)
- `redteam/red-08-joint-defense-consortium.md` (W4 neutralization model)
- `Cargo.toml:2-3,46`; `LICENSE.md:1-10` (assent chain)

Row format: attacked clause (quoted, path:line) / attack (one line, cites
red-NN) / repair that defeats it / triage (D = drafting-only,
S = structurally conflicting, i.e. unfixable without changing the deal).

---

## R1 — Covenant vs condition restructure (corporate-size triggers)

- **Clause**: "Licensor grants Deployer ... a perpetual, worldwide,
  non-exclusive, fully paid-up, royalty-free license ... provided
  strictly that Dj notin E_inst" — LICENSE-P28-SIL.md:96.
  Triggers at :105-108 (12b-2 filer status, IRC 448(c), EU 2013/34/EU
  3(4), FAR/CAS) condition the grant on the licensee's corporate form.
- **Attack**: corporate-size conditioning is covenant territory, not a
  copyright condition — breach yields contract damages only, no
  infringement claim; and §3.1 itself promises not to bring 17 U.S.C.
  actions, so the money machine eats itself (red-05 Part B3, verdict
  row :213).
- **Repair**: restructure the Institutional triggers as copyright
  CONDITIONS on the scope of the grant in the curated-bare-essentials
  framing: "the license to compile, execute, mutate, and distribute is
  GRANTED ONLY to non-Institutional Deployers; an Institutional Deployer
  acting without a countersigned Commercial Execution Lease acts outside
  the grant entirely (no license = direct infringement, 17 U.S.C.
  501), and the covenant not to sue (3.1) is inapplicable."
  State expressly which acts (copy, distribute, adapt) cease to be
  licensed on crossing 2.2, and pair with a cure path (4.3) so the
  condition reads as scope-defining, not punitive.
- **Triage**: D (drafting) — but see R2 for the retroactivity half,
  which is S.

## R2 — Retroactive repricing conflict (§2.1 vs §2.3)

- **Clause**: "a perpetual, worldwide, non-exclusive, fully paid-up,
  royalty-free license" — LICENSE-P28-SIL.md:96; vs "The exact instant
  Dj in E_inst holds, all past, present, and subsequent Executions are
  bound by Section 4" — LICENSE-P28-SIL.md:130-132.
- **Attack**: a perpetual, fully-paid grant cannot be retroactively
  converted into a metered one; retroactive imposition of a new monetary
  obligation on past free usage is unilateral modification without
  consideration (red-05 Part C.7, verdict row :214).
- **Repair**: split §2.3 into prospective-only effect: "Executions
  COMMENCING after the Institutional Transition Instant are bound by
  Section 4; Executions completed before the Instant remain licensed
  under 2.1 (grandfathered), provided the Deployer executes a
  Commercial Lease within [90] days of the Instant for ongoing use."
  This preserves the revenue model forward while killing the
  no-consideration retroactive-modification attack.
- **Triage**: S — the current deal's value proposition (back-epoch
  regularization fees per COMPENDIUM:415-422, cited red-06:82-84)
  depends on retroactive repricing; prospective-only concedes the
  back-epoch revenue line.

## R3 — Evidence presumptions §3.3 / §3.4 vs FRE 902

- **Clause**: "triggers an irrebuttable adverse inference, calculating
  commercial balance obligations at the maximum theoretical compute
  capacity" — LICENSE-P28-SIL.md:163; Eq. :165 (L_assessed, with
  undefined Tariff_commercial); "Receipts ... constitute conclusive,
  self-authenticating business records under FRE 803(6), 902(13),
  902(14). Deployer is equitably and irrevocably estopped" — :171.
- **Attack**: parties cannot stipulate irrebuttable evidentiary
  presumptions or private litigation holds; FRE 902 authenticates,
  does not make conclusive; 803(6) remains contestable; and the
  ledger is lossy by design (§6.2 ring drops, :236), which undercuts
  "conclusive" (red-05 Part C.3, rows :215-216; red-06 D4, :106-118).
- **Repair**: replace stipulated presumptions with verification
  MECHANICS: (a) receipts are evidence of the recorded transitions,
  weight contested under FRE 803(6)/902 like any business record —
  delete "conclusive" and "irrevocably estopped"; (b) on litigation
  hold, Deployer preserves ledgers and the licensor gains the right to
  an independent third-party audit of the receipt ledger (a contractual
  audit right, lawful to stipulate); (c) replace the peak-capacity
  integral with the actual metered receipt volume times the R10 tariff
  schedule (see R10), with the lossy-gap marker (§6.2) admitted as the
  ledger's own known limitation — gaps priced at the measured pre-gap
  run rate, capped, not at peak theoretical capacity.
- **Triage**: D for (a)/(b); S for the peak-capacity multiplier — its
  deterrence value IS the deal's leverage, and replacing it with
  metered volume removes the spoliation doomsday mechanism red-05
  brands a penalty (row :215).

## R4 — ASC 450 / SOX "mandate" §4.1-4.2

- **Clause**: "Deployer stipulates that Commercial Exploitation ... is
  an operational liability that is probable and estimable under FASB
  ASC 450-20-25-2" — LICENSE-P28-SIL.md:179; SOX integration :188-191.
- **Attack**: ASC 450 is an accounting standard, not law; accrual
  requires an ASSERTED claim, and the instrument has no demand
  mechanism, no invoice procedure, no notice clause — unasserted and
  unestimable means zero reserve (red-06 D1 :47-67, D5 :126-131;
  red-05 verdict row :217).
- **Repair**: reframe as a contractual NOTICE-AND-DEMAND right, not a
  statutory claim: Licensor MAY serve a written Invoice Statement
  (defined artifact: receipt-ledger excerpt + tariff line items) and a
  cure window (e.g. 60 days to execute a lease per 4.3); accrual
  language becomes "Deployer acknowledges that an asserted, invoiced
  claim is a loss contingency it must evaluate under ASC 450-20-25-2"
  — an acknowledgment that a GAAP process exists, not a stipulation of
  its outcome. Drop the SOX 302/906 false-certification paragraph
  (§4.2) entirely; a private licensor has no private right under those
  statutes and their presence is attorney-intimidation that weakens
  the contract (red-06 D5).
- **Triage**: D for the notice/demand mechanics; S for deleting SOX
  leverage — but red-06 shows the SOX lever yields zero reserve anyway,
  so deletion costs little deal value.

## R5 — Privilege stripping §5.3

- **Clause**: "Communications ... generated by or shared with legal
  counsel for the purpose of planning, structuring, or assessing the
  risk of bypassing Execution Receipts fall outside the attorney-client
  privilege and work-product doctrine (Clark v. United States)" —
  LICENSE-P28-SIL.md:213-215.
- **Attack**: crime-fraud is a judicial finding after in-camera review,
  not a contract default; a private stipulation cannot strip privilege
  (red-05 Part C.5, verdict row :218; red-08 W5.3).
- **Repair**: delete the override; replace with a severable
  acknowledgment that privilege is governed by applicable law and that
  nothing in the instrument enlarges or diminishes it. The licensor's
  real protection is already §3.2 (circumvention acts void the
  covenant) — privilege law needs no contractual help, and the clause
  as drafted invites a §17200 / misuse count while protecting nothing.
- **Triage**: D — pure deletion, no deal value lost.

## R6 — Privity / SCA / CI / LLM tooling §8.2 / §9 + assent chain

- **Clause**: "Prompts, vector queries, or embedding searches submitted
  to language models ... constitute inchoate steps under 18 U.S.C.
  1519" — LICENSE-P28-SIL.md:268; §9 SCA/SBOM mandates :272-284; CI/CD
  fail-closed :278-284. Assent chain: Cargo.toml:2-3 (comment line
  "governed by LICENSE-P28-SIL.md") -> Cargo.toml:46
  `license-file = "LICENSE.md"` -> LICENSE.md:7 (pointer link).
- **Attack**: no privity — scanners, CI operators, and LLMs are
  non-parties with no capacity to contract; the comment-line assent
  chain is browsewrap, breakable in one step (Specht) (red-05 B1
  :87-102, C.6 :187-192, verdict rows :219, :221).
- **Repair**: two-part fix. (a) Assent: make Cargo.toml declare the
  license via license-file pointing DIRECTLY at LICENSE-P28-SIL.md,
  and add an affirmative assent gate — the Substrate emits a typed
  refusal (CGAP_ERR_LICENSE_UNASSENTED class) on first run until an
  environment variable / config file records the URN plus deployer
  identity (mirrors the existing fail-closed admission discipline of
  §6.3). Execution-implies-assent survives only with a manifested,
  logged assent record, not a comment line. (b) Third-party tooling:
  convert §8.2/§9 from commands TO non-parties into declarations about
  the LICENSEE's own obligations (licensee shall not instruct agents to
  bypass receipts; licensee shall not configure CI to strip receipt
  generators) — binding only the licensee, who has privity.
- **Triage**: (a) is D-plus-code (small gate in the existing refusal
  taxonomy); (b) is D. The misuse count (red-05 Count II) dissolves
  once the license stops conditioning the copyright on non-party
  behavior.

## R7 — One-directional covenant §3.1

- **Clause**: "Licensor covenants not to institute, prosecute, or
  maintain civil actions under 17 U.S.C. ... nor seek equitable
  injunctive relief" — LICENSE-P28-SIL.md:142-144. No reciprocal
  licensee covenant exists.
- **Attack**: nothing obligates the Deployer to refrain from
  challenging validity; consortium-funded IPR/PGR petitions are not
  Circumvention Dialogue and are uncaptured (red-08 W4.1).
- **Repair**: EITHER (i) make it reciprocal — Deployer covenants not
  to challenge the validity of the P28 patents or the invariants as a
  condition of the covenant-not-to-sue (a standard mutual-standstill
  structure), OR (ii) leave §3.1 one-way and formally acknowledge the
  IPR exposure in the campaign ledger: validity challenges are a
  residual risk priced into the lease tiers, defended by patent
  prosecution quality, not by contract.
- **Triage**: (i) is D drafting but weakens the license's take-it-
  or-leave-it posture (red-05 B2 unconscionability angle); (ii) is a
  business-acceptance decision. Both defeat the red-08 hole; choose
  per deal posture.

## R8 — Licensee confidentiality / grantback / anti-consortium

- **Clause**: none exists — grep census confirms zero NDA /
  confidentiality / bilateral / consortium terms in the 333-line
  instrument (red-08 W2, :16-26).
- **Attack**: licensees can form a joint-defense consortium, pool
  invalidation prior art, and share receipt-ledger analytics and
  arbitration outcomes with no clause catching any of it (red-08 W4.2
  table, :79-92).
- **Repair**: add a licensee confidentiality + grantback section:
  (a) licensees shall not disclose Commercial Lease pricing or
  regularization settlement terms (prevents benchmark-shopping and
  consortium price coordination); (b) circumvention-adjacent findings
  (any artifact whose production required Circumvention-Dialogue-class
  analysis) are Licensor property via grantback and may not be shared
  with other Deployers; (c) optionally a most-favored-nation price
  clause instead of (a) if the goal is consortium-proofing rather than
  secrecy — MFN removes the incentive to pool for pricing intel.
  Note antitrust counsel review: (c) is the competition-safe variant.
- **Triage**: D (new clause) — but it changes the license's public
  posture from "open instrument" to "confidential program," a deal-
  posture choice, hence flagged S-borderline.

## R9 — Arbitration scope §7.2

- **Clause**: "The arbitrator's jurisdiction is confined to
  mathematical auditing of Execution Receipts; the validity of the
  invariants, the statutory classifications, and the Covenant Not to
  Sue are non-arbitrable and final" — LICENSE-P28-SIL.md:252.
- **Attack**: one-sided split — Licensor keeps court adjudication of
  what matters to it and arbitrates only arithmetic; worse, the scope
  is too NARROW for enforcement: arguably no jurisdiction to award the
  unpriced regularization fee at all (red-06 D5 :132-134; red-05 C.2
  :153-160, verdict row :220).
- **Repair**: widen the arbitrable scope to the full Section 4 money
  chain: "disputes concerning the commercial balance, the Invoice
  Statement (R4), the tariff schedule (R10), and lease regularization
  under 4.3 are arbitrable; validity of patents and of the copyright
  grant itself remains with the courts." Delete "the statutory
  classifications ... are non-arbitrable and final" — arbitrability is
  for the court to decide (Prima Paint), and the self-serving
  declaration buys nothing while inviting the unconscionability attack.
- **Triage**: D — the widened scope is strictly better for the
  licensor; the narrow carve-out was self-inflicted.

## R10 — Tariff-symbol definitions (UnitCost / Tariff_commercial)

- **Clause**: "L_accrued = ... sum UnitCost(Delta S_admitted, k)" —
  LICENSE-P28-SIL.md:181-183; "L_assessed = integral C_peak ·
  Tariff_commercial dt" — LICENSE-P28-SIL.md:165. UnitCost and
  Tariff_commercial are never defined anywhere in the instrument.
- **Attack**: a demand computed from undefined symbols is not
  "reasonably estimable" under ASC 450-20-25-2; management cannot
  estimate from a formula with unbound parameters (red-06 D2 :71-87).
- **Repair**: EITHER define the symbols in-instrument with a pinned
  tariff table (per-transition-class UnitCost schedule, frozen by URN
  suffix like the existing byte-pin discipline — e.g.
  `urn:avec:license:p28-sil:v2.1.1-sovereign#tariff`), OR remove the
  symbols and define L_accrued by reference to a countersigned
  Commercial Lease schedule. The marketing-only tier figures
  ($150k/$650k/$2.5M+, COMPENDIUM:405-411) must either be promoted
  into the instrument or stop being cited as if priced.
- **Triage**: D — mandatory regardless of deal posture; an instrument
  that prices nothing cannot collect anything.

---

## Triage summary

| Row | Repair | Triage | Deal cost |
|---|---|---|---|
| R1 | covenant->condition restructure | D (with R2 caveat) | low |
| R2 | prospective-only 2.3 | S | back-epoch revenue line |
| R3 | presumptions->verification mechanics | D (a)/(b); S (peak mult.) | deterrence |
| R4 | notice-and-demand; drop SOX para | D; SOX del. S-borderline | low |
| R5 | delete privilege override | D | none |
| R6 | assent gate + privity rebind | D + small code | low |
| R7 | reciprocal covenant or IPR acknowledgment | (i) D / (ii) business | posture-dependent |
| R8 | confidentiality + grantback | D, S-borderline (posture) | public-posture change |
| R9 | widen arbitration scope | D | strictly positive |
| R10 | define or remove tariff symbols | D | mandatory |

**Unfixable-without-changing-the-deal set**: R2 (retroactivity IS the
back-epoch revenue model itself), the peak-capacity multiplier inside
R3 (the spoliation doomsday mechanism is the leverage), and — posture —
R8's public-vs-confidential choice. Everything else is drafting-only
and strictly strengthens enforcement.

## Fence law note

Documentation-only lane; no code, gates, or courts touched; no fences to
run. No git commands. License text unedited (rider pattern per
LICENSE-P28-SIL.md:303-314 ERRATA precedent). All clause quotes were
re-read from LICENSE-P28-SIL.md, Cargo.toml, LICENSE.md, and the three
redteam inputs on disk this session.

## Standing

ALIVE — register written and grounded this session; every path:line
receipt verified against disk this session.

## ERRATA (2026-11-02, lane I-12 stale-cite refresh)

The GAP-01/GAP-01b amendment landed (REVISION RIDER v2.1.1 at
LICENSE-P28-SIL.md:316; TC-08 ERRATA RIDER now :303-314), shifting every
clause line number below §2.2. This register's path:line cites were
refreshed in place (living register) against the current 333-line
instrument; all quoted text re-verified verbatim this session. Old ->
new: :110-112 -> :130-132 (§2.3); :139/:141/:147 -> :163/:165/:171
(§3.3/§3.4); :155 + :166-167 -> :179 + :188-191 (§4.1/§4.2); :189-191
-> :213-215 (§5.3); :212 -> :236 (§6.2); :244 + :248-256 + :258-264 ->
:268 + :272-284 + :278-284 (§8.2/§9); :118-120 -> :142-144 (§3.1);
:228 -> :252 (§7.2); :157-159 + :141 -> :181-183 + :165 (R10 symbols);
:279-290 -> :303-314 (ERRATA rider). Unchanged: :96 (§2.1 grant) and
:105-108 (§2.2 triggers). Cargo.toml:2-3,46 and LICENSE.md:7 assent
chain cites re-verified unchanged. Flag: R10's clause quote
"L_assessed = integral C_peak . Tariff_commercial dt" is an ASCII
transliteration; the instrument (:165) carries a leading nabla
(`\nabla \mathbf{Tariff}_{commercial}`) the transliteration omits —
pre-existing, retained verbatim, flagged per dispatch law.
