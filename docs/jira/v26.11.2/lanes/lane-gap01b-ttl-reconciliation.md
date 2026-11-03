# Lane GAP-01b — TTL reconciliation + license landing court

## Owned-files manifest

| # | Path (absolute) | Disposition | Why this lane owns it |
|---|---|---|---|
| 1 | /Users/sac/cgap/docs/jira/v26.10.31/34-p28-sil-compliance.md | edit | TTL/compliance layer reconciliation (owned) |
| 2 | /Users/sac/cgap/docs/jira/v26.11.2/lanes/lane-gap01b-ttl-reconciliation.md | new | this receipt |

Disjoint ownership honored: LICENSE-P28-SIL.md (GAP-01, landed) NOT
touched; 19-ontology-ttl.md NOT owned, NOT touched — the TTL operand
remap remains open for its owner.

## 1. Identity

Lane GAP-01b, repo /Users/sac/cgap (main). Subjects: compliance doc
(153 → 210 lines on disk this session) and this receipt. Amendment
under reconciliation: LICENSE-P28-SIL.md v2.1.1 rider (333 lines,
verified on disk this session; anchors 103/110/122/126/134/316
re-grepped exit 0).

## 2. Authority

Dispatch prompt (GAP-01b contract, two owned files only). Reconciliation
semantics sourced from LICENSE-P28-SIL.md §2.2.1-§2.4 as landed by
GAP-01 (`lane-gap01-license-amendment.md`) and FIX-10 Part 2's TTL
remap instruction
(`docs/jira/v26.11.2/redteam/fix-10-license-aggregation.md:159-167`).
No novel clause semantics authored; every added note quotes or
paraphrases landed instrument text with dates. No git write commands.

## 3. Commands + exits

- Read lane-gap01-license-amendment.md (110 lines, exit 0), fix-10
  (242 lines, exit 0), 34-p28-sil-compliance.md (153 lines, exit 0).
- grep/wc LICENSE-P28-SIL.md → 333 lines, anchors confirmed (exit 0).
- Landing court: cp to /tmp/p28-court-copy.md; sha256
  8190a31eda077d95... ; python delete §2.2.1 (lines 110-121) →
  /tmp/p28-court-mut.md (321 lines, sha256 1c1f836e4fa4a729...);
  python3 /tmp/p28_court.py → 5/5 TRIPPED→ESCAPES, exit 0.
- Sandbox witness-delete: mutation verified (definition string absent,
  grep exit 1), both copies + court script rm'd, /tmp/p28-court*
  glob no-match confirmed (exit 1 = clean).
- 7x Edit 34-p28-sil-compliance.md (each <50 lines, exit 0); awk
  length census + git diff hunk census exit 0.

## 4. Consequence (compliance-doc diff summary)

- :58 — amendment note on the two-must-reconcile preamble: v2.1.1
  rider landed, TTL remap still pending.
- :79 — Threshold Measure Mismatch: proposal is now instrument law
  (§2.4 pins annual gross revenue verbatim); names the inverted
  divergence state: figure was TTL-only, now instrument-only while
  TTL still reads assetUnderManagement.
- :107-124 — Boundary-Value Pinning: amendment note marks FIX-10's
  "figure in TTL only, not instrument" state superseded-by-amendment
  (2026-11-02, history retained); enumerates the new aggregation
  duties the machine layer must implement: group-wide classification
  per §2.2.2, per-member §2.3 trigger, meter/deployment partition
  ban, strictest-reading rule, §2.2.3→§3.3 spoliation import.
- :151 — Open Items "Canonical P28-SIL license text" row: OPEN →
  CLOSED (2026-10-31), superseded by the Instrument of Record.
- :152 — Open Items threshold-measure row: OPEN → AMENDED
  (2026-11-02); TTL remap still OPEN, now mechanical not semantic.
- :176-178 — Standing footer: unreconciled claim superseded-by-
  amendment; remaining work is the mechanical TTL operand remap.
- :183-200 — new "Amendment of Record (v2.1.1 rider, 2026-11-02)"
  section: rider, anchors, URN retained, landing court pointer to
  this receipt.

## 5. Courts

**Landing court (FIX-10 Part 3, executed — GAP-01's receipt left this
as "available to the coordinator's landing court"):** /tmp sandbox
copy, §2.2.1 deleted, five structure-trip checks re-run. Anti-vacuity
mutation = deletion of §2.2.1 (the G_ctrl definition). Result:

| Structure-trip check | Baseline | Mutated (§2.2.1 deleted) |
|---|---|---|
| S1 SPE-shelter (§2.2.2 partition ban) | TRIPPED | ESCAPES |
| S2 dispersed trusts (§2.2.1 closing) | TRIPPED | ESCAPES |
| S3 50/50 JV (§2.2.1 bullet 4) | TRIPPED | ESCAPES |
| S4 avoidance restructuring (§2.2.3) | TRIPPED | ESCAPES |
| S5 $50M orphan (§2.4) | TRIPPED | ESCAPES |
| **flips** | | **5/5, exit 0** |

Court design note (disclosed): S1/S4/S5's spot quotes live in
§2.2.2/§2.2.3/§2.4, which survive the §2.1 deletion, so the court
gates each check on the §2.2.1 definition heading
(`"Controlled Group" ($\mathbf{G}_{\text{ctrl}}$):`) being present —
i.e. each structure's legal force is the quote AND its defining
chain. This is the faithful reading: without the G_ctrl definition,
group attribution at §2.2.2 is undefined and S1/S4/S5 collapse. A
court that cannot flip is FAIL; this one flipped 5/5.
Non-vacuity of the mutation itself: grep for the definition string in
the mutated copy → 0 matches (exit 1), witness sha256 recorded before
witness-delete; restore not applicable (sandbox copy, deleted after
with witness per dispatch).

## 6. Standing

ALIVE — every command ran this session against the exact subjects
with exits recorded in §3; the court flip is measured, not asserted;
sandbox deleted with witness; shared tree touched only in the two
owned files (git status exits 0).

## 7. Divergences / disclosures

- D1 (URN question → coordinator): instrument fingerprint pin §9.1
  and SPDX line (:3, :278, :298) still say v2.1.0 while the rider
  (line 316) says v2.1.1. Options: (a) bump URN to
  `...v2.1.1-sovereign` + update §9.1/SPDX pins — breaks the frozen
  byte-pin law (CLAUDE.md pins the URN verbatim) and forces a
  LicenseRef sweep across consumers; (b) keep URN v2.1.0 + rider-noted
  amendment (TC-08 ERRATA RIDER precedent at :279-290 did exactly
  this). **Recommendation: (b)** — URN is a frozen byte-pin; the
  rider carries the version delta; bump is a coordinated release act,
  not a lane edit. Decision explicitly coordinator-owned; nothing
  edited.
- D2: two new Open-Items table rows are 147/172 chars — this file's
  pre-existing table rows already run to 216 chars (lines 154-155,
  165-168), so single-line table rows follow the file's own table
  convention; prose lines kept ≤100.
- D3: the court's S1/S4/S5 gating (definition-present gate) is this
  lane's design choice, disclosed in §5; FIX-10 specified the
  mutation (delete §2.2.1) and the flip expectation, not the check
  mechanics.
- D4: GAP-01's receipt said the landing court was "available to the
  coordinator's landing court" (lane-gap01:68-70); this lane executed
  it as dispatched — supersession of that line is recorded here, no
  errata needed to GAP-01's receipt (its §5 disclosed the gap).
- No SLA unblocks applied; no other lane's files touched; no git
  write commands; /tmp sandbox deleted with witness.

## 8. Fences (must-not-regress surfaces named and run)

- LICENSE-P28-SIL.md untouched: sha256 not taken pre-session (file
  was already ` M` from GAP-01), but git diff of the shared tree was
  confined to the two owned files this session (edits tool-gated);
  GAP-01's receipt stands as the license-diff record.
- Compliance doc: pre-existing >100-char lines (44/45, 154-155,
  165-168) left as-is (not lane-introduced, verified by git diff
  hunk census — only the two table rows and prose notes added).

---

## ERRATA (2026-11-02, lane I-12 — stale legal-cite refresh)

Line 115 above cites the TC-08 ERRATA RIDER precedent at
LICENSE-P28-SIL.md :279-290. Post-GAP-01b the rider sits at :303-314
(the v2.1.1 insertion of §2.2.1-2.2.4 + §2.4 grew the instrument
291 -> 333 lines); ":279-290" as written now points into §9 SBOM
rules. Original retained verbatim per ERRATA-never-rewrite; corrected
cite: :303-314. Full mapping:
docs/jira/v26.11.2/redteam/fix-11-covenant-repair-register.md (I-12
ERRATA section).
