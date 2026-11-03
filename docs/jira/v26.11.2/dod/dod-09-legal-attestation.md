# DOD-09 — Legal/P28-SIL v2.1.1 compliance corpus alignment attestation

Lane: DOD-09 (2026-11-02). Contract: read-only verification + this file.
Grade as-of-disk. No git commands run.

## 1. Enforced clause → enforcement court/symbol map

| P28-SIL clause | Enforcement referent | Status |
|---|---|---|
| §2.4 safe-harbor ceiling ($50,000,000.00, ≥ duty boundary) | `DEFAULT_SAFE_HARBOR_CEILING_CENTS = 5_000_000_000` (`crates/cgap-p28/src/asc450.rs:72-74`), pinned equal to A17 `SAFE_HARBOR_CEILING_MAX_CENTS` (`attestation.rs:58`) | ALIGNED |
| §3.3 spoliation invariants | `ERR_P28_SPOLIATION = 0x5032_38FF` (`crates/cgap-p28/src/trap.rs:36`); "all deaths carry the staged code" asserted at :445, :708 | ALIGNED |
| §2.2 lineage / attestation record | 80-B `P28LicenseAttestation` (`attestation.rs:59-80`), `ATTESTATION_LEN=80` :57, lineage bitfield :40-49 | ALIGNED |
| §5 attestation integrity (keyed MAC, RC-04) | 48-B keyed-MAC layer over the 80-B body — `crates/cgap-p28/tests/attestation_mac_court.rs` (4 tests: deterministic admit, 384 MAC-wire bit mutations refuse :112-129, key-never-on-wire :132-145, tag-binds-whole-body :163) | ALIGNED |
| Receipt law (chain integrity) | `SIGNED_SPAN=104`/`PROOF_LEN=168` (`asc450.rs:61-66`); 640 bit-flip sweep + layout-freeze integration (`tests/p28_invariant_courts.rs`: bit_flip_sweep_640_integration, layout_frozen_integration) | ALIGNED |

**Honest partial-by-design note:** the remaining instrument machinery is
contract text, not code. §2.2.1-§2.2.3 (Controlled Group prongs, group
aggregation, anti-structural-avoidance), §2.3 topological transition, §4
regularization duties, §7.2 arbitration, §9.1 fingerprinting: **no code
referent**. These are legal/audit obligations enforced by the instrument
itself; the code meters the numeric ceiling and the evidence chain only.
The mapping above is the complete set of code-enforceable clauses.

## 2. Controlled-group evasion structures → tripping clauses

All 5 structures from FIX-10 Part 3
(`docs/jira/v26.11.2/redteam/fix-10-license-aggregation.md:171-225`),
with textual trip-checks witness-verified by GAP-01b's landing court
(lane-gap01b-ttl-reconciliation.md:73-94 — sandbox mutation deleting
§2.2.1, 5/5 TRIPPED→ESCAPES flip, exit 0, sha256 fail-closed restore):

| # | Structure | Tripping clause |
|---|---|---|
| 1 | Sub-$50M SPEs sheltering a $500M enterprise | §2.2.1 Ownership+SPE prongs; §2.2.2 group attribution; per-entity meter partition barred verbatim |
| 2 | Distributed LLC trusts / non-controlled affiliate rings | §2.2.1 Trust prong + economic-reality closing + §318(a)(3) Attribution prong |
| 3 | SPE joint ventures (50/50, non-consolidated) | §2.2.1 JV prong (expressly covers ASC 810 escapes); §2.2.2 |
| 4 | Restructuring that leaves receipts intact | §2.2.3 anti-structural-avoidance, importing §3.3 spoliation consequences |
| 5 | $50M figure mismatch (LICENSE.md advertisement-orphan) | §2.4 figure + ≥ boundary now stated in instrument; LICENSE.md:13-14 advertises §2.4 with group aggregation |

## 3. Alignment status across the corpus (as-of-disk)

- **TTL (`docs/jira/v26.10.31/19-ontology-ttl.md`) — I-11 remap LANDED.**
  Amendment comments at :206-214; both `odrl:leftOperand` values remapped
  to `avec:annualGrossRevenue` (:222-224, :238-241); E-08 CLOSED note
  :258-274. **Divergence noted:** `34-p28-sil-compliance.md:59-64` still
  reads "the TTL operand remap itself ... is still pending" — stale prose
  against the landed TTL; that doc's own precedence rule (:55-57, TTL
  wins) governs, so the TTL is the operative layer. Reconcile entry owed
  in the compliance doc (owned elsewhere; not written by this lane).

- **LICENSE.md — ALIGNED.** :11-14 identifies P28-SIL v2.1.0 by URN and
  advertises §2.4 ($50M annual gross revenue, group-aggregated per
  §2.2.1-2.2.3, v2.1.1); `LicenseRef-P28-SIL-2.1.0` identification form
  retained. Structure-5 advertisement-orphan closed.
- **LICENSE-P28-SIL.md — LANDED.** v2.1.1 rider :316-333; §2.2.1-§2.2.3
  :110-128, §2.4 :134-136 (cross-refs updated 2026-11-02 post-GAP-01b).
- **NOTICE-blake3: no NOTICE file exists** under
  `crates/cgap-blake3/` (find returned zero NOTICE/LICENSE* files beyond
  `Cargo.toml:11 license-file = "../../LICENSE.md"`). License conveyance
  for cgap-blake3 is via `license-file` → LICENSE.md, which is aligned.
  If a NOTICE file was expected to exist, it does not — recorded
  honestly as absent, not invented.
- **Compliance doc (`34-p28-sil-compliance.md`) — PARTIAL**: measure +
  boundary notes amended (:57-105) but carries the stale "remap pending"
  line cited above.

## 4. URN decision status

**Option (b) on file, coordinator-open (not executed).** GAP-01b
receipt :109-116 lays out (a) bump URN vs (b) keep
`urn:avec:license:p28-sil:v2.1.0-sovereign` + rider-noted versioning,
recommends (b) (URN is a frozen byte-pin per CLAUDE.md; a bump forces a
LicenseRef sweep). No coordinator decision recorded on disk as of this
read; the URN remains v2.1.0 everywhere (instrument header, LICENSE.md,
CLAUDE.md pin).

## 5. Enforcement-court witness: cgap-p28 test suite

Command: `CARGO_TARGET_DIR=/tmp/dod09-target cargo test -p cgap-p28`
→ **exit 0, 50 passed, 0 failed** — lib unittests 38,
attestation_mac_court 4, lever_courts 2, p28_invariant_courts 6,
doc-tests 0. Matches the 50-test suite count.

## Standing

ALIVE for what ran this session: cargo test (exit 0, 50/50), disk reads
of LICENSE-P28-SIL.md, 19-ontology-ttl.md, 34-p28-sil-compliance.md,
LICENSE.md, asc450.rs, attestation.rs, trap.rs, fix-10, GAP-01b
receipt. PARTIAL for the clause→code map (partial by design, §1 note).
No mutations executed by this lane (the §2.2.1-deletion flip court
already ran in GAP-01b; not re-run here — cited, not duplicated).

Divergences: (1) 34-p28-sil-compliance.md stale "remap pending" prose;
(2) no NOTICE file under cgap-blake3; (3) URN decision coordinator-open.
