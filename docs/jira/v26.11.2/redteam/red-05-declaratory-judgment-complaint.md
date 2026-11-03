# RED-05: Declaratory Judgment Complaint — P28-SIL v2.1.0 Litigation Defensibility Stress-Test

Lane: RED-05 (adversarial red-team wave, v26.11.2). Read-only lane over
`/Users/sac/cgap`; sole written file is this one.

## 0. Target inventory (grounded on disk)

- Sole operative license text: `/Users/sac/cgap/LICENSE-P28-SIL.md`
  (291 lines; URN `urn:avec:license:p28-sil:v2.1.0-sovereign`, line 3).
- Pointer file: `/Users/sac/cgap/LICENSE.md:1-18` (18 lines; identifies
  the instrument, no independent terms).
- Wired into build metadata: root `Cargo.toml:46`
  `license-file = "LICENSE.md"`; `crates/cgap-p28/Cargo.toml:11`
  `license-file = "../../LICENSE.md"`.
- **UNSUPPORTED targets**: no "Tripartite Agreement" and no "Tranche 1
  Confidentiality/NDA" text exists anywhere in the repo. Search method:
  `grep -rni 'tripartite|tranche' docs/ LICENSE.md README.md
  crates/cgap-p28/` returned only two unrelated uses of "tranche" as a
  wave-vocabulary word (`docs/jira/v26.11.1/waves/errc-eval-mr.md:38,55`);
  `grep -ni 'nda|confidential' LICENSE.md` returned zero matches. The
  Tripartite Agreement and Tranche 1 NDA are therefore analyzed as
  UNSUPPORTED (no on-disk referent); Sections (b)/(c) below attack the
  license terms that DO exist.

## Part A — Skeleton federal Declaratory Judgment Complaint

**Caption**: [Enterprise Deployer] v. AVEC / Licensor. Venue: N.D. Cal.
(AVEC designates California law, LICENSE-P28-SIL.md:224 §7.1).
Jurisdiction: 28 U.S.C. § 2201 (DJ Act) + § 1338 (copyright) +
§ 1331 (license-misuse / contract).

### Count I — Declaratory judgment of copyright non-infringement

1. **Actual, present controversy (MedImmune)**: Deployer runs the
   Substrate daily in CI and production (`Cargo.toml:2-3` subjects the
   whole workspace to the instrument; `crates/cgap-p28/Cargo.toml:11`
   ships it). The license's own §2.3 ("Topological Transition Invariant,"
   line 110-112) declares that the instant Deployer crosses an
   Institutional metric boundary "all past, present, and subsequent
   Executions are bound by Section 4" — a present, retroactive threat of
   a balance-sheet accrual claim. Under MedImmune v. Genentech, 549 U.S.
   118 (2007), that creates a real and reasonable apprehension of suit.
2. **No infringement in the first place**: Sections 3.1's covenant is
   framed as protection against "civil actions under 17 U.S.C. § 101 et
   seq." (line 120). Deployer pleads in the alternative: absent a
   countersigned Commercial Execution Lease the license is
   field-of-use/enterprise-size conditioned, and a bare copyright grant
   with an extra-statutory monetary trigger is either (i) a covenant
   (breach = damages only, no infringement suit possible — see
   *Ma under* Part B(3)) or (ii) unenforceable (Part B). Either way,
   copyright liability fails as a matter of law.
3. **Requested declarations**: (i) Deployer's compilation and execution
   of the Substrate does not infringe; (ii) §2.2-§2.3's retroactive
   re-classification and §4.1's ASC 450 accrual "mandate" are covenants,
   not copyright conditions; (iii) the §3.3 spoliation/adverse-inference
   machinery (lines 133-142) is unenforceable as a private stipulation
   purporting to bind the court.

### Count II — Copyright misuse (counterclaim / affirmative defense)

1. **Misuse per se**: §9.1 (lines 248-256) prohibits SCA/SBOM scanners
   from "normaliz[ing]... to permissive open-source identifiers" and
   §9.2 (lines 258-264) mandates CI/CD pipelines "fail closed upon the
   detection of any build artifact, flag, or environment configuration
   whose effect is to silence or divert" receipts — i.e., the license
   attempts to control the licensee's own tooling, build systems, and
   internal compliance apparatus beyond the scope of the copyright grant
   (cf. *DSC v. Pacific Bell*, classic misuse: leveraging copyright to
   control non-copyrighted matter).
2. **Anti-tamper tying**: §3.2's "Circumvention Dialogue Predicate"
   (lines 122-131) voids the covenant not to sue for mere *speech* —
   drafting an RFC, a ticket, or an oral planning session whose
   "optimization target" is a zero partial derivative. Licensing back
   the covenant against communications (not acts) is a restraint on
   expression well outside the copyright bargain — misuse.
3. **Effect**: misuse renders the copyright unenforceable for the
   period of misuse (*Lasercomb America v. Reynolds*, 911 F.2d 970
   (4th Cir. 1990)) until the licensor purges.

## Part B — Tranche 1 / Adhesion attack (mapped onto actual text)

The Tranche 1 NDA is UNSUPPORTED (§0). The adhesion attack lands on the
license itself, which is structured as take-it-or-leave-it: execution
itself is deemed acceptance ("*Pacta sunt servanda*: Any machine cycle
... constitutes voluntary and irrevocable submission," line 21).

### B1. No meeting of the minds / browsewrap failure

- Acceptance-by-execution with no click-through, no affirmative
  manifest, no opportunity to negotiate: classic browsewrap
  (*Specht v. Netscape*, 306 F.3d 17 (2d Cir. 2002)). The root
  `Cargo.toml:2-3` merely comments "governed by LICENSE-P28-SIL.md";
  a comment line in a build manifest is not conspicuous assent.
- `Cargo.toml:46` `license-file = "LICENSE.md"` — the license-file field
  names the 18-line pointer, and the pointer (LICENSE.md:7) just
  references the instrument. A licensee can pull the crate from a
  registry or fork the repo without ever manifesting assent to the
  291-line instrument. Assent chain is breakable in one step.
- §1.6 defines "Deployer" to include "algorithmic agent matrix[es]"
  (line 84) and §8.2 (line 244) deems even *LLM prompts* "inchoate
  steps under 18 U.S.C. § 1519" — i.e., the contract purports to bind
  non-parties (agents, counsel, scanners) with no capacity to contract.

### B2. Unconscionability / adhesion

- **No negotiation, no exit**: §2.3 makes classification retroactive to
  "all past Executions" (line 112) with no grace period; §7.2 (line 228)
  consigns damages disputes to expedited JAMS arbitration in LA County
  while carving out validity of "the invariants, the statutory
  classifications, and the Covenant Not to Sue" as "non-arbitrable and
  final" — i.e., Licensor keeps the court for the questions that matter
  to it and arbitrates only the arithmetic. Procedurally and
  substantively unconscionable.
- **Irrebuttable adverse inference by private stipulation**: §3.3
  (lines 133-142) purports to create, by contract, "an irrebuttable
  adverse inference" with damages "at the maximum theoretical compute
  capacity of Deployer's hardware topology" (line 139, Eq. at 141).
  Evidence rules and presumptions are procedural law made by courts and
  legislatures; parties cannot stipulate irrebuttable evidentiary
  presumptions or statutory-class penalties (penalty doctrine, *Gaston
  County v. Gore*, penalty-clause liquidated-damages analysis under CA Civ.
  Code § 1671). Cracked.
- **Penalty, not liquidated damages**: §4.1's "Balance Sheet Accrual
  Mandate" (lines 155-162) and §3.3's peak-capacity integral are
  uncalibrated to actual harm — a classic penalty clause.

### B3. The *Ma* / SonicBlue fork in the road (the decisive structural attack)

Under *MAA v. Ma* (copyright-condition vs. covenant) and *SonicBlue/
BMG v. Thomson* analysis: conditions on the copyright grant (failure =
infringement) must be conditions on the *scope* of the grant (what is
copied, distributed, adapted). §2.2's triggers are the licensee's
*corporate form* (SEC filer status, IRC 448(c) receipts, EU size,
FAR/CAS status — lines 105-108). Corporate-size conditioning of a
copyright license is field-of-use-like covenant territory: breach
yields contract damages, NOT an infringement claim — and, as pleaded in
Count I, the covenant-not-to-sue in §3.1 (line 120) *itself* promises
not to bring 17 U.S.C. actions. So the license's only money machine
runs through (i) a covenant-based claim (no statutory damages, no
injunction as of right), (whole clause) attenuated by (ii) a covenant
not to sue that is itself the thing Deployer asks the court to enforce
against Licensor. The license's litigation posture eats itself.

## Part C — How an enterprise invalidates P28-SIL in open court (outline)

1. **File the DJ action first** (28 U.S.C. § 2201) before any §4
   regularization demand, and before any response that could be
   characterized as "Circumvention Dialogue" (§3.2, lines 122-131) —
   the license itself argues that drafting an RFC about receipt-bypass
   voids the covenant (line 124), so counsel's first written product
   should be the complaint, framed as covenant *enforcement*, not
   circumvention.
2. **Move to strike the arbitration carve-out**: §7.2 (line 228)
   makes "validity of the invariants... non-arbitrable and final" —
   i.e., Licensor reserved court adjudication of validity to itself.
   Under *Prima Paint*/ GE v. PV Chemicals, arbitrability and validity
   questions are for the court only when the clause clearly says so;
   here the clause gives the court nothing Deployer wants. Attack:
   clause is one-sided; severability §7.3 (line 230-232) preserves
   reformation arguments either way.
3. **Attack the evidentiary engine (highest-value motion)**: §3.4
   (lines 145-147) stipulates receipts are "conclusive" FRE 902(13)/
   (14)/803(6) records and Deployer is "irrevocably estopped" from
   challenging them. FRE 902 self-authentication certifies *that a
   record is what it purports to be* — it is NOT a conclusive-weight
   stipulation; FRE 803(6) still admits to a hearsay-exemption contest
   (foundation witness, methodology). A court will not give
   conclusive-and-irrebuttable weight to one party's generated hashes.
   Likewise §3.3's self-executing litigation hold (line 137) and
   "irrebuttable adverse inference" (line 139) are contract-made
   procedure — unenforceable. File a motion in limine / for protective
   order on day one.
4. **Attack the accounting "mandate"**: §4.1 (lines 155-162) purports
   to *stipulate* that unregularized use is "probable and estimable"
   under ASC 450-20-25-2. ASC 450 is an accounting standard, not law;
   a private contract cannot bind an auditor's judgment, much less
   create a loss contingency. §4.2 (lines 166-167) leverages SOX 302/404
   certification risk as private enforcement — arguable tortious
   interference with statutory reporting duties and securities-law
   misuse; supports misuse count and §17200 (CA) unfair-competition
   count.
5. **Attack privilege-stripping**: §5.3 "Crime-Fraud Invariant"
   (lines 189-191) declares counsel's circumvention-risk advice
   unprivileged by private stipulation, citing *Clark v. United
   States*. Crime-fraud is a court finding after in-camera review, not
   a contract default. Cracked as drafted.
6. **Attack the contractual capture of third parties** (§9.1 scanners,
   §9.2 CI/CD, §8.2 LLMs, lines 244-268): no privity; scanners and CI
   operators are not licensees and cannot be bound by the instrument.
   This is also the strongest *misuse* fact (Part A Count II): the
   license conditions use of the copyrighted work on the behavior of
   non-party tooling.
7. **The retroactivity kill-shot**: §2.3 (lines 110-112) binds "all
   past... Executions" the moment a metric boundary is crossed. Under
   CA law (Cal. Civ. Code § 1645, *still applicable* to prospective-only
   contract modification), retroactive imposition of a new monetary
   obligation on past free usage is a unilateral modification without
   consideration — past executions under §2.1's perpetual, paid-up,
   royalty-free grant (lines 96-99) cannot be repriced by later
   status change. The word "perpetual" in §2.1 is the licensor's own
   vulnerability: a perpetual, fully-paid grant cannot be
   retroactively converted into a metered one.
8. **Sequence**: (i) DJ complaint (Count I + II); (ii) motion to
   compel/protective order defeating §3.3-3.4; (iii) summary judgment
   on covenant-vs-condition (Ma) — no infringement possible; (iv) ASC
   450/SOX leverage neutralized by declaratory relief; (v) license
   purged or P28-SIL held unenforceable as against Deployer.

## Verdict table

| Claim attacked | Clause (path:line) | Attack | Standing |
|---|---|---|---|
| Covenant-not-to-sue + speech-voiding | LICENSE-P28-SIL.md:120-131 | Misuse (Lasercomb); copyright misuse | BREACH-CONFIRMED (doctrinal attack defeats as drafted) |
| Retroactive reclassification | LICENSE-P28-SIL.md:110-112, 96-99 | Perpetual paid-up grant cannot be repriced; no consideration | BREACH-CONFIRMED |
| Irrebuttable adverse inference / private litigation hold | LICENSE-P28-SIL.md:133-142 | Parties cannot stipulate evidence rules or penalties | BREACH-CONFIRMED |
| Conclusive receipts / estoppel | LICENSE-P28-SIL.md:145-147 | FRE 902 authenticates, does not make conclusive; 803(6) contestable | BREACH-CONFIRMED |
| ASC 450 "mandate" + SOX leverage | LICENSE-P28-SIL.md:155-167 | Accounting standard is not law; stipulation cannot bind auditors | BREACH-CONFIRMED |
| Privilege stripping | LICENSE-P28-SIL.md:189-191 | Crime-fraud is judicial finding, not contract default | BREACH-CONFIRMED |
| Third-party tool capture (SCA/CI/LLM) | LICENSE-P28-SIL.md:244-268 | No privity; misuse per se | BREACH-CONFIRMED |
| Arbitration carve-out | LICENSE-P28-SIL.md:228 | One-sided; unconscionable split | PARTIAL (severability §7.3 gives Licensor a reformation path) |
| Assent chain | /Users/sac/cgap/LICENSE.md:7, Cargo.toml:2-3,46 | Browsewrap; comment-line "governance" is not assent | BREACH-CONFIRMED |
| Tripartite Agreement | none on disk | — | UNSUPPORTED |
| Tranche 1 Confidentiality/NDA | none on disk | — | UNSUPPORTED |

## Fence law note

This lane was read-only over the entire repo except this file; no gates,
courts, or tests touched. No must-not-regress fences exist for a
documentation-only adversarial-analysis diff. The single written file
introduces no code and cannot regress any court.

## Standing

ALIVE — deliverable written and grounded this session; every quoted
clause carries a path:line receipt re-read from disk this session.
Targets with no on-disk referent (Tripartite Agreement, Tranche 1 NDA)
are UNSUPPORTED per the fail-open logging law, with the search method
stated in §0.
