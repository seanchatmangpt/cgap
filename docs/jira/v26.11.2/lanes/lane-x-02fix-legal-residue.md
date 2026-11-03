# Lane X-02FIX — legal-residue R2/R6/R7 (2026-11-02)

Standing: **ALIVE** — every item below executed and verified on disk this
session against the exact subjects.

## Files written (owned only)

- `LICENSE-P28-SIL.md` (:136 region only)
- `LICENSE.md` (:13 region)
- `docs/jira/v26.11.2/redteam/fix-10-license-aggregation.md` (append-only)

## R2 — instrument §2.4 cross-cites (LICENSE-P28-SIL.md:136)

Before: `34-p28-sil-compliance.md:60-71` and
`34-p28-sil-compliance.md:75-88`.
After: `:66-88` and `:90-105`, with inline dated note
"(Cross-reference line numbers updated 2026-11-02 post-GAP-01b
reconciliation; previously :60-71 and :75-88.)"

Both targets re-read this session: measure-mismatch heading at :66
(range :66-88), boundary-value heading at :90, ≥-pin section ends
"resolved." at :105. Range :90-105 verified exact via awk range read
(exit 0).

## R7 — LICENSE.md:13 safe-harbor measure

Before: "the $50M universal safe harbor".
After: "the $50,000,000.00 annual gross revenue safe-harbor floor per
LICENSE-P28-SIL.md §2.4 (group-aggregated per §2.2.1-2.2.3, v2.1.1)".

## R6 — fix-10 ERRATA (append-only, body preserved verbatim)

Appended `## ERRATA (2026-11-02, lane X-02FIX)` with 4 items:
(1) :123 "NO dollar figure" row SUPERSEDED — figure now at
LICENSE-P28-SIL.md:134-136; (2) pre-GAP-01b line refs at :124, :142,
:158, :162, :237 remapped (:66-88 / :90-105); (3) LICENSE.md:13 R7
correction noted; (4) R2 noted; TTL operand remap flagged as lane I-11
in-flight, not landed here.

## Commands / exits

- `sed`/`awk`/`grep` range reads of all three targets + compliance doc
  — exit 0 each; on-disk re-read post-Edit confirms all diffs.
- No git commands. No lib.rs / Cargo.toml / shared-tree edits.

## Divergences

- None. Dispatch cited "34-p28-sil-compliance.md" without the
  v26.10.31 dir prefix; actual path is
  `docs/jira/v26.10.31/34-p28-sil-compliance.md` — resolved by reading,
  matches the pre-GAP-01b instrument cites.

## Fences

No gates or courts exist over these three markdown surfaces; the
must-not-regress surface is the license cross-cite integrity itself —
every new cite was verified against a live range read this session.
Residual: TTL `odrl:leftOperand` remap (I-11) still pending; until it
lands, instrument and TTL disagree (instrument-only figure), per the
compliance doc's own amendment note at :59-65.
