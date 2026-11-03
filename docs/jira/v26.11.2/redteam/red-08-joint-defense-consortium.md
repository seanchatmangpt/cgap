# RED-08 — Joint Defense Consortium / Antitrust-Collusion Red Team

Lane: RED-08, v26.11.2 adversarial wave, repo /Users/sac/cgap (canonical checkout).
Contract: READ-ONLY over the whole repo; sole output this file. No git commands.

## W1. Subject

The P28-SIL licensing structure at `/Users/sac/cgap/LICENSE-P28-SIL.md` is attacked
with the OIN/Linux-Foundation-style joint defense consortium scenario: N peer
$50M+ Institutional Entities pool capital to fund IPR/PGR invalidation proceedings
against any P28 patents, and share circumvention-relevant information among
themselves, to neutralize the license's isolation strategy.

## W2. Referent census (target text: ABSENT)

Greps over `docs/` and repo root (2026-11-02, this session):

- "isolated room" — 0 hits repo-wide.
- "bilateral" — 0 hits in any license or doc referent.
- "NDA" / "non-disclosure" — 0 hits.
- "consortium" — 0 hits.
- "joint defense" — 0 hits.

The dispatch's named target ("isolated room" bilateral NDA / multi-enterprise
isolation strategy) has NO on-disk referent. Logged: **UNSUPPORTED** with zero
file:line evidence possible — the strategy text does not exist.

## W3. What actually exists (analyzed instead)

The only multi-party licensing instrument on disk is the P28-SIL license itself,
`/Users/sac/cgap/LICENSE-P28-SIL.md` (291 lines):

- §2.2 (`LICENSE-P28-SIL.md:101-108`): Institutional Entity triggers include
  Accelerated/Large Accelerated Filer (17 C.F.R. § 240.12b-2), IRC § 448(c),
  EU Directive 2013/34/EU Art. 3(4), CAS/FAR Part 19 — every member of a
  $50M+ peer consortium clears at least one trigger, so all members sit in
  Institutional Exploitation Status (§2.3, line 110-112).
- §3.1 (`LICENSE-P28-SIL.md:118-120`): Covenant Not to Sue — note the
  directionality: **Licensor covenants not to sue Deployer**. There is no
  reciprocal covenant by the Deployer not to challenge validity.
- §3.2 Circumvention Dialogue predicate (`LICENSE-P28-SIL.md:122-131`): the
  only suspension trigger is Circumvention Dialogue / Act of Disruption —
  communications "whose optimization target is ∂R_receipt/∂E_t ≡ 0 or
  H_query → ∅" or code modification that strips receipts.
- §3.3 spoliation/legal hold (lines 133-143) and §5 privilege stripping
  (lines 179-192) bind individual Deployers, not licensee-to-licensee
  information exchange.
- §4 ASC 450 accrual (lines 151-176) prices unlicensed Institutional
  Exploitation per receipt volume; §7.2 confines arbitration to "mathematical
  auditing of Execution Receipts" (line 228) and declares validity and the
  Covenant "non-arbitrable and final" — a court, not an arbitrator, would
  hear any invalidity fight.
- §8.1 non-decomposition (line 238-240) bars clause-by-clause reading but
  does not create any information-sharing restriction.
- ERRATA RIDER (lines 279-291): the "mod Σ_PQC" term of §1.5 is invented
  lore with zero code referent — already on record via cull-08.

## W4. Consortium neutralization model (honest verdict)

### W4.1 Capital-pool invalidation funding — NOT CAUGHT

Section 3.1's covenant runs one way: Licensor promises not to sue Deployer;
nothing obligates Deployer to refrain from challenging patent validity. An
IPR/PGR petition is not "Circumvention Dialogue": §3.2's predicate is defined
over communications whose optimization target is ∂R/∂E ≡ 0 or H_query → ∅
(line 128), i.e. receipt-derivation optimization — a validity challenge in
favor of the license's own invariants is not that. A consortium-funded IPR
on the OST register patents (`docs/jira/v26.11.2/patent/claims.txt`,
US10,817,467 / US11,216,456 / US11,409,698 / US11,645,278) is therefore not
caught by any clause. §8.1's non-decomposition bar (line 238-240) is a
canon-of-construction instruction, not a covenant; it cannot convert a
validity attack into Circumvention Dialogue.

### W4.2 Information-sharing between licensees — NOT CAUGHT

There is no confidentiality clause, no NDA, no grantback, no licensee-to-
licensee restriction anywhere in the 291-line instrument. A joint-defense
common-interest arrangement among Deployers shares (a) receipt-ledger
analytics, (b) arbitration outcomes, (c) pooled invalidation prior art,
(d) circumvention-adjacent know-how. Clause sweep:

| Consortium act | Clause that would catch it | Verdict |
|---|---|---|
| Pool capital for IPR/PGR vs P28 patents | none exists | NOT CAUGHT |
| Share invalidation prior art among licensees | none exists | NOT CAUGHT |
| Share receipt-ledger analytics/arbitration outcomes | none exists | NOT CAUGHT |
| Joint circumvention engineering among members | §3.2 (lines 122-131) | CAUGHT — but only the engineering act, not the association |

W4.2 note: §3.2 catches each member's own circumvention communications, so a
consortium whose *charter* is circumvention engineering would suspend each
participating Deployer's covenant — but the consortium's mere formation,
funding, and prior-art pooling remain uncaptured.

### W4.3 The "isolated room" premise is UNSUPPORTED twice over

Twice over: the on-disk license neither implements nor could implement an
"isolated room" bilateral NDA strategy — no confidentiality clause, no
bilateral covenant, no information-sharing restriction exists to neutralize.
The dispatch premise is UNSUPPORTED; the analysis above is against the real
instrument.

## W5. Honest verdict

**The license as written does NOT prevent consortium neutralization.** The
only license-family defenses a joint-defense consortium would face are:

1. §3.2 Circumvention Dialogue predicate (`LICENSE-P28-SIL.md:122-131`) —
   catches only members' own receipt-stripping communications, not the
   consortium itself;
2. §3.3 spoliation legal hold (`LICENSE-P28-SIL.md:133-143`) — a retention
   drag on each member's own records, not a bar on sharing;
3. §5.3 crime-fraud privilege stripping (`LICENSE-P28-SIL.md:189-191`) —
   de-privileges only counsel communications aimed at receipt bypass, not
   validity-challenge work product (which is lawful, so the exception's
   predicate fails and privilege holds).

Conversely, §7.2's "non-arbitrable and final" declaration for validity
(`LICENSE-P28-SIL.md:228`) is self-serving prose — an arbitrator's refusal
to hear invalidity pushes the consortium's challenge to district court /
PTO, which is where the consortium wants it anyway.

**Errata duty for downstream lanes**: any receipt claiming the license
"isolates" enterprises bilaterally should cite this file as a
BREACH-CONFIRMED correction; the instrument contains no bilateral term.

## W6. Commands / exits

- grep census for "isolated room|bilateral|NDA|consortium|joint defense"
  over docs/ + root: 0 referent hits (exit 0 both greps, this session).
- Reads: LICENSE-P28-SIL.md (full 291 lines), claims.txt head -40 (exit 0),
  03-ost-design-around-register.md sweep (exit 0).

## W7. Standing

**PARTIAL.** (a) The dispatch's named target text is UNSUPPORTED (absent
from disk); (b) the real instrument (LICENSE-P28-SIL.md) was located,
analyzed, and cracked: it contains no confidentiality, no bilateral
covenant, no anti-consortium or information-sharing clause; the §3.1
covenant is one-directional and §3.2 catches only individual receipt-
stripping acts. Consortium neutralization is uncaptured by the license
as written.
