# Lane T-X02 — Legal-Residue Audit (archival transcription, 2026-11-02)

**TRANSCRIBED** — this receipt is an archival transcription of X-02's
definitive pre-publish legal-residue list. The original audit predates
the I-11 / I-12 / X-02FIX closures; items below are transcribed in the
tense of the original audit and marked with their closing receipts. No
source file was edited by this lane; every cite was re-read from disk
this session before transcription (grounding reads, exit 0 each).

## Transcribed residue list

### (1) TTL operand remap — OPEN + unannotated in the original audit

19-ontology-ttl.md:212 / :223 carried `avec:assetUnderManagement` as
the `odrl:leftOperand` in the SovereignInvariantPolicy constraint,
with zero pending-note matches (no annotation flagged the mismatch
against instrument §2.4's annual-gross-revenue measure).
**Since CLOSED by lane I-11**: on disk this session, the operand at
:212-223 reads `avec:annualGrossRevenue` under an inline dated comment
`# AMENDMENT 2026-11-02 (lane I-11): was avec:assetUnderManagement;`
(verified via `sed -n '210,226p'`, exit 0).

### (2) URN decision — recorded with option-(b) recommendation

lane-gap01b:109-119 (D1 in the receipt's §7) and
34-p28-sil-compliance.md:197-210 record the URN-bump question with the
option-(b) recommendation: keep URN
`urn:avec:license:p28-sil:v2.1.0-sovereign` (frozen byte-pin) + carry
the version delta in the v2.1.1 rider; a bump is a coordinated release
act, not a lane edit. **Still coordinator-open** — no coordinator
decision receipt located this session.

### (3) Stale-cite enumeration (post-GAP-01b line drift)

The +24 drift map for old LICENSE lines >=110 — the ERRATA RIDER
remapped :279-290 -> :303-314. FIX-11's 15 stale cites, as enumerated
in the original audit:

| Item | Old -> New |
|---|---|
| table cites | :6 / :284 |
| :53 | :130-132 |
| :74 | :163 / :165 |
| :77 | :171 |
| :103 | :179 / :190-191 |
| :130 | :213-215 |
| :146 | :268 / :272-280 |
| :146-147 | :282-288 |
| :175 | :142-144 |
| :220 | :252 |
| :240 | :181-183 |

Carriers of the stale cites (the docs quoting them): lane-gap01:53,
lane-gap01b:115, fix-11 register :6 and :284.

LICENSE instrument §2.4 cross-cites into 34-p28-sil-compliance.md:
:60-71 -> :66-88 and :75-88 -> :90-129; 34-p28's own self-inflicted
stale cites :209 / :220 -> :212 / :223; LICENSE.md:13 carries the
safe-harbor semantic drift ("$50M universal safe harbor").

## Closing status note

Every residue item above has since landed its closure receipt; this
file is archival and supersedes none of them:

- Item (1) TTL operand remap: CLOSED by I-11 (inline dated amendment
  comment at 19-ontology-ttl.md:218-219; operand now
  `avec:annualGrossRevenue`).
- Item (3) stale-cite drift: CLOSED by I-12
  (`lanes/lane-i12-stale-legal-cite-refresh.md` — refresh table,
  ERRATA appended to the fix-11 register and both carrier receipts)
  and by X-02FIX (`lanes/lane-x-02fix-legal-residue.md` — R2 §2.4
  cross-cites :66-88 / :90-105, R7 LICENSE.md:13 measure corrected,
  R6 fix-10 ERRATA).
- Item (2) URN decision: still coordinator-open, option-(b)
  recommendation standing (gap01b §7 D1, 34-p28 amendment block).

## W7 receipt fields

1. **Identity** — lane T-X02; subject:
   docs/jira/v26.11.2/lanes/lane-x02-legal-residue-audit.md (NEW);
   repo /Users/sac/cgap, branch main.
2. **Authority** — dispatch prompt (archival transcription lane);
   owned-files manifest = this file only; zero path overlap with any
   sibling lane (documentation-only write).
3. **Commands+exits** — grounding reads this session, exit 0 each:
   `sed -n '210,225p'` 19-ontology-ttl.md; `sed -n '195,212p'`
   34-p28-sil-compliance.md; `sed -n '100,121p'` lane-gap01b receipt;
   `sed -n '1,10p'` and `sed -n '275,295p'` fix-11 register;
   `sed -n '205,226p'` 34-p28-sil-compliance.md; `sed -n '1,15p'`
   LICENSE.md; receipt count via `wc -l` on lane-x-02fix receipt.
4. **Consequence** — one new archival markdown file; no code, gates,
   courts, or shared-tree surfaces touched. Fences: none exist over
   docs-only surfaces; nearest fence is license cross-cite integrity,
   re-verified by the grounding reads above.
5. **Courts** — none dispatched; transcription lane with no mutable
   subject, so no anti-vacuity mutation applies. [SKIP] no court
   surface in scope — loud, per skip law.
6. **Standing** — ALIVE for the transcription itself (file on disk,
   cites re-read this session, exits above). Item (2) closure remains
   coordinator-open: PARTIAL, not rounded up.
7. **Divergences** — (a) the original audit predates I-11/I-12/X-02FIX;
   closure statuses are annotated, not rewritten into the transcribed
   list (ERRATA-never-rewrite). (b) dispatch cited "fix-11's 15 stale
   cites"; the table above transcribes 11 mapping rows + the 2 table
   cites + 2 LICENSE §2.4 cross-cite rows, matching the enumeration
   given verbatim. (c) gap01b receipt's live filename is
   `lane-gap01b-ttl-reconciliation.md`; the audit's "lane-gap01b:109-119"
   resolves to its §7 D1 block at :105-121 (verified on disk).
