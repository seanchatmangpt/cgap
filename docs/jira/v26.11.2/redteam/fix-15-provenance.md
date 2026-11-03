# FIX-15: CLA / AI-Provenance Policy Skeleton (red-07 human-layer remediation)

**Lane:** FIX-15 · **Date:** 2026-11-02 · **Standing:** ALIVE
(deliverable written against on-disk referents this session) · **Mode:**
policy skeleton — placeholders are marked `[ORG RECORD REQUIRED]` and are
deliberately NOT filled with invented employers, names, or dates.

**Subject under fix:** red-07 §4 verdict — 25/45 subpoena items are
BREACH-CONFIRMED with zero on-disk referent, centering on the human layer
(contributor history, COI screening, clean-room protocol, LLM provenance,
litigation hold). This file addresses remediation items 2-5 of
`docs/jira/v26.11.2/redteam/red-07-retaliatory-discovery-plan.md:201-210`
plus the NOTICE half of item 1 (sibling file:
`/Users/sac/cgap/docs/NOTICE-blake3.md`).

**Scope law:** this file is a POLICY SKELETON, not executed org fact.
Where an org fact exists on disk it is cited; where it does not, the
placeholder stands loudly. No placeholder in this file may be filled by
an agent without citing a real org record.

---

## Section A — Contributor license assurance (CLA / IP assignment)

Closes red-07 items 2, 4, 5, 8, 31-34 (partial).

A1. Current state on disk: `LICENSE.md:3-9` identifies the governing
instrument (P28-SIL v2.1.0) but names no assignors; no CLA, no
contributor register, no IP-assignment records exist in the repo.
BREACH-CONFIRMED posture stands until the register below is filled from
real records.

A2. Register (to be maintained from signed records, not from agent
memory):

| # | Contributor | Legal name | CLA ver | IP-assign instr | COI attest date | Record locus |
|---|---|---|---|---|---|---|
| 1 | [ORG RECORD REQUIRED] | | | | | |
| 2 | [ORG RECORD REQUIRED] | | | | | |

A3. Agent contributors: the repo's fan-out lanes run under named agent
identities with receipts per lane (`docs/jira/v26.11.2/lanes/`); those
receipts evidence authorship trail (defeats red-07 item 14) but do NOT
constitute legal contributor identity. Each agent lane's human principal
of record: [ORG RECORD REQUIRED].

A4. Rule going forward: no merge to main by a contributor absent a CLA
row in A2. Enforcement mechanism: [ORG RECORD REQUIRED — reviewer/gate
owner and the gate that refuses unsigned heads].

---

## Section B — Conflict-of-interest / employment screening

Closes red-07 items 4, 6, 7, 8, 31-33, 42 (partial).

B1. On disk: zero referent. The repo cannot and must not assert any
contributor's employment history.

B2. Policy to adopt: every contributor files a one-time COI attestation
covering (a) current/former employers 2019-present, (b) consulting
clients whose code norms could constrain contributions, (c) equity or
fiduciary ties to competitors of any P28-SIL counterparty. Attestation
form locus: [ORG RECORD REQUIRED].

B3. Register: [ORG RECORD REQUIRED — maintained outside the repo or in a
private access-controlled locus; a public repo is the wrong store for
this data; note the tension and resolve it at org level].

B4. Access-boundary rule: any contributor with access to a counterparty's
confidential materials is barred from contributing to modules in the
design-around register's blast radius
(`docs/jira/v26.11.2/03-ost-design-around-register.md:10-17`) until
cleared. Clearing authority: [ORG RECORD REQUIRED].

---

## Section C — AI-assistance disclosure policy

Closes red-07 items 10, 11, 12, 40, 43 (partial).

C1. Current true posture, stated plainly: cgap development is heavily
agent-assisted (fan-out lanes, ggen generation, court harnesses); the
repo evidences the TOOLS (ggen replay chain, lane receipts, journals) but
records no per-commit MODEL identity, version, or date. That gap is
red-07 P10 / items 10 and 12, and this skeleton does not retroactively
fabricate it.

C2. Disclosure standard (prospective): every lane receipt in
`docs/jira/v26.11.2/lanes/` and `docs/jira/v26.11.1/lanes/` gains a
`model:` field naming the model identity and version used for generated
code in that lane. Retroactive backfill: [ORG RECORD REQUIRED — feasible
only where run transcripts survive; mark UNVERIFIED where not].

C3. Training-corpus representations: the repo cannot produce vendor
training-data provenance (red-07 items 13, 40). Policy: contributor
attests which agent tooling was used; vendor-side corpus representations
are out of repo control and are recorded as an accepted, disclosed
limitation rather than a promise the licensor cannot keep.

C4. Prompt/journal retention: `journal.jsonl` is post-run truth per
CLAUDE.md but retention windows and durable storage are unpinned
(red-07 item 11). Retention policy (window, store, custodian):
[ORG RECORD REQUIRED].

---

## Section D — Clean-room protocol record (BLAKE3 and successors)

Closes red-07 items 16-21 (structural half; the five org placeholders
live in `/Users/sac/cgap/docs/NOTICE-blake3.md` §3 and are cross-owned
there).

D1. Protocol class adopted: SPEC-ONLY DERIVATION, evidenced on disk for
`crates/cgap-blake3` (see NOTICE §1-3: zero-dep crate, no vendored
reference source, spec constants only, KAT conformance via published
vector recipe).

D2. Required record set for any future spec-derived module: designated
implementer, spec version pinned, spec-access log locus, isolation
agreement, derivation date range. Template identical to NOTICE §3.

D3. Independence study (red-07 item 21): conformance vectors prove
correctness, not independence. A diff-study against the reference
implementation — architectural divergence inventory (stack Merkle
reduction, SIMD multi-parent, no unsafe transcription surface) — is
owed: [ORG RECORD REQUIRED — owner and due date].

---

## Section E — Litigation-hold posture

Closes red-07 items 15, 45.

E1. On disk: no hold policy, no custodian list. Repository-side
preservation posture that CAN be stated today: git history is immutable
(fix-forward only, no rebase/force-push per repo CLAUDE.md), lane
receipts are append-only, and build roots are lease-deleted at
integration — the last of which is a DESTRUCTION-vector fact that a hold
must supersede.

E2. Hold trigger and custodian assignment: [ORG RECORD REQUIRED].

E3. Hold scope, once triggered: suspend `scripts/lease_janitor.py --clean`
and any journal/`journal.jsonl` rotation; snapshot per-lane build roots
before deletion; freeze retention windows in C4. Custodian and mechanism:
[ORG RECORD REQUIRED].

E4. Disclosure duty: this section creates no legal hold by itself. A
policy file is not a hold notice; issuing notices is an org act.

---

## Cross-references and fences

- NOTICE half of remediation item 1/2:
  `/Users/sac/cgap/docs/NOTICE-blake3.md` (written this session by
  FIX-15; same lane owns both files).
- red-07 §5 remediation items addressed: 1 (partial — BLAKE3 NOTICE
  only; consolidated VENDOR register is FIX-10 territory), 2 (Section D),
  3 (Section A), 4 (Section C), 5 (Section E).
- Fences: this diff is documentary only — no code, no gate touched. No
  executable court exists over provenance docs (itself red-07 finding
  P10's mechanism gap); anti-vacuity is satisfied by path:line
  re-readability of every citation above. Mutations would be doc edits,
  which are the remediation itself, so no mutation court was run and no
  skip is silent: the absent gate is recorded here and in red-07 §6.

## Report terminus

```json
{
 "lane": "FIX-15",
 "files_written": [
  "docs/NOTICE-blake3.md",
  "docs/jira/v26.11.2/redteam/fix-15-provenance.md"
 ],
 "commands": [
  "ls/head inventory crates/cgap-blake3 (exit 0)",
  "grep blake3 Cargo.lock (exit 0)",
  "sed reads LICENSE/Cargo.toml/official_vectors/red-07/fix-10 (exit 0)",
  "awk line-length gate, both files (exit 0)"
 ],
 "courts_run": [
  "documentary: path:line claims re-readable from disk; no provenance gate exists"
 ],
 "standing": "ALIVE",
 "divergences": [
  "upstream BLAKE3 license (CC0/PD + Apache) from upstream docs, not bytes; UNVERIFIED (NOTICE §2)",
  "org facts left as loud [ORG RECORD REQUIRED] placeholders per dispatch law"
 ]
}
```
