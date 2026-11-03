# Lane GAP-01 — P28 license disk amendment (FIX-10 application)

## Owned-files manifest

| # | Path (absolute) | Disposition | Why this lane owns it |
|---|---|---|---|
| 1 | /Users/sac/cgap/LICENSE-P28-SIL.md | edit | coordinator-delegated license edit |
| 2 | /Users/sac/cgap/docs/jira/v26.11.2/lanes/lane-gap01-license-amendment.md | new | this receipt |

Disjoint ownership: FIX-10 owned only the redline doc (explicitly NOT
the license file); no sibling lane claims LICENSE-P28-SIL.md.

## 1. Identity

Lane GAP-01, repo /Users/sac/cgap (main). Subject:
LICENSE-P28-SIL.md, amended in place this session (git status: ` M
LICENSE-P28-SIL.md`, exit 0).

## 2. Authority

Dispatch prompt (GAP-01 contract, owner = LICENSE-P28-SIL.md only);
redline text supplied verbatim by
/Users/sac/cgap/docs/jira/v26.11.2/redteam/fix-10-license-aggregation.md
(Part 1 §2.2 redline + §2.2.1/2.2.2/2.2.3 text, Part 2 §2.4 text).
No novel clauses were authored; every inserted sentence is quoted
from FIX-10.

## 3. Commands + exits

- Read fix-10-license-aggregation.md (242 lines, exit 0) — full
  redline text re-read before any edit.
- Read LICENSE-P28-SIL.md (291 lines pre-edit, exit 0).
- 5x Edit LICENSE-P28-SIL.md (each <50 lines, exit 0).
- python3 verification block (exit 0): anchor resolution, section
  ordering, $-balance, line-length census.
- wc -l → 333 lines on disk; git status exit 0.

## 4. Consequence (diff summary + inserted line ranges)

- §2.2 lead paragraph amended (line 103): "acting severally or as a
  member of a Controlled Group as defined in §2.2.1" + boundaries
  "measured ... at the Group Aggregation Level per §2.2.2".
- §2.2.1 Controlled Group, five prongs — inserted at line 110
  (bullets 112-116; economic-reality closing 118).
- §2.2.2 Group Aggregation Level — inserted at line 122 (per-member
  §2.3 trigger, meter/deployment partition ban, strictest-reading).
- §2.2.3 Anti-Structural-Avoidance — inserted at line 126 (extends
  §3.2 to reorganizations with classification-avoidance principal
  purpose; imports §3.3 consequences).
- §2.4 Numeric Safe-Harbor Floor — inserted at line 134; states
  $50,000,000.00 in the instrument, ≥ duty boundary, TTL remap note.
- REVISION RIDER (v2.1.1, 2026-11-02, FIX-10) appended at line 316
  (after the TC-08 ERRATA RIDER at 279-290, matching that precedent:
  dated, cites source, no silent rewrite). Canonical URN retained
  verbatim per §9.1 fingerprint pin; URN bump flagged as
  coordinator-owned.

Line convention: the file's existing convention allows long prose
lines (pre-edit max >1000 chars, e.g. old line 15), so inserted
paragraph lines follow the same convention; new-paragraph body
verified parse-clean by python (section ordering 2.1→2.2→2.2.1→2.2.2
→2.2.3→2.3→2.4→3.1... confirmed monotonic).

## 5. Courts — structure-trip spot-checks (textual, per FIX-10 Part 3)

No executable court exists for a markdown license instrument; the
five-trip checks are textual cross-checks with spot quotes (this is a
disclosed limitation, not a silent skip). FIX-10's anti-vacuity
instruction (delete §2.2.1 in a /tmp copy → each check flips to
ESCAPES) remains available to the coordinator's landing court.

| Attack structure (FIX-10 Part 3) | Spot quote from landed instrument |
|---|---|
| 1. sub-$50M SPEs under $500M parent | "No member may partition deployments, meters, or executions among members to remain below any boundary" (§2.2.2, line 122) |
| 2. dispersed LLC trusts / affiliate rings | "Membership in a Controlled Group is a question of economic reality ... not by the entity's state-law form ... or absence of a single majority holder" (§2.2.1, line 118) |
| 3. 50/50 non-consolidated JV | "including a 50/50 or dispersed-interest JV that escapes consolidation under ASC 810" (§2.2.1 bullet 4, line 115) |
| 4. classification-avoidance restructuring | "any corporate reorganization ... undertaken with classification-avoidance as a principal purpose ... constitutes an **Act of Disruption or Circumvention Dialogue**" (§2.2.3, line 126) |
| 5. $50M advertisement-orphan | "strictly less than Fifty Million United States Dollars ($50,000,000.00) is presumed to remain within $\mathcal{U}_{\text{exempt}}$" (§2.4, line 136) |

## 6. Standing

ALIVE — every edit command and the verification block ran this
session against the exact subject with exit 0; anchors resolve on
disk (lines 103/110/122/126/134/316 verified by grep-equivalent
python scan). Compile courts N/A (markdown-only diff; no crates
touched).

## 7. Divergences / disclosures

- D1: FIX-10 Part 1 wraps §2.2.2/§2.2.3/§2.4 as separate
  blockquote-numbered blocks; landed text uses the instrument's
  existing plain `N.N.N.` heading style (matches §2.2.1 numbering
  style elsewhere in the file). Text is otherwise verbatim.
- D2: the `$50,000,000.00` literals carry raw `$` (odd-count lines
  136/324 by $-balance scan) exactly as FIX-10 drafted them; harmless
  in markdown, flagged for any future LaTeX-in-md pipeline.
- D3 (coordination item, carried from FIX-10 Part 2): the TTL
  `odrl:leftOperand` remap in
  docs/jira/v26.10.31/34-p28-sil-compliance.md is NOT landed by this
  lane (file not owned); until it lands, instrument states annual
  gross revenue while the TTL still reads assetUnderManagement.
  Also: the URN stays v2.1.0 per §9.1 pin — coordinator must decide
  whether a URN bump accompanies the TTL remap.
- D4: REVISION RIDER added per the dispatch ("add a v2.1.1 revision
  note"); the fix-10 doc itself names no rider, so its wording is
  lane-authored (flagged, not silent).
- No SLA unblocks applied; no other lane's files touched; no git
  write commands.


---

## ERRATA (2026-11-02, lane I-12 — stale legal-cite refresh)

Line 53 above cites the TC-08 ERRATA RIDER at LICENSE-P28-SIL.md
:279-290. The FIX-10 amendment inserted §2.2.1-2.2.4 + §2.4 mid-file,
growing the instrument 291 -> 333 lines and shifting the TC-08 rider to
:303-314 (the REVISION RIDER itself appends below it at :316). The cite
":279-290" as written now points into §9 SBOM rules; current truth is
:303-314. Original text retained verbatim per ERRATA-never-rewrite;
line 53's cite is hereby corrected to :303-314. Receipt:
docs/jira/v26.11.2/redteam/fix-11-covenant-repair-register.md (I-12
cite-refresh table carries the full old->new mapping).
