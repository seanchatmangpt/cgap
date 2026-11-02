# Lane DF-6: DMADV rollup

## Identity

- **Lane**: v26.11.2 DF-6 (five-phase rollup).
- **Subject**: /Users/sac/cgap @ main, 2026-11-02.
- **Files written** (owned, exclusive):
  docs/jira/v26.11.2/dflss/00-dmadv-rollup.md (new, 241 lines)
  docs/jira/v26.11.2/lanes/lane-dflss-06-rollup.md (this receipt)
- **No git commands run** (coordinator owns transitions).

## Authority

Dispatch prompt + owned-files manifest above. DF-1..DF-5 had NOT
landed at lane start (dflss/ dir absent, ls exit 1); rollup rebuilt
from 14-dflss-charter.md, 15-ctq-tree.md, audit/ledger + blockers, and
the lane/wave receipts, per the dispatch's else-branch.

## Commands + exits

| Command | Exit | Purpose |
|---|---|---|
| ls docs/jira/v26.11.2/dflss/ ... | 1 | DF-1..5 absent census |
| Read 14-dflss-charter.md, 15-ctq-tree.md | 0 | Define/VOC sources |
| Read audit/release-blockers.md, audit-ledger.md | 0 | register of truth |
| grep census: lane-g09, fix07, g10, g05, git-07 receipts | 0 | Analyze root causes |
| ls crates/*/tests/*court*.rs | wc -l -> 62 | court inventory (Design) |
| ls docs/jira/v26.11.2/lanes | wc -l -> 68 | receipt inventory |
| grep -rn 429 waves/*.md HANDOFF.md | 2 hits (w7-errata) | 429 witnesses |
| awk length>100 sweep | 0 over-length | doc gate |

## Courts

No new courts authored (docs lane). Consumed 62 court files + the
flips recorded in audit-ledger.md; unflipped courts named loudly:
CSCB zero-tax (R10, FAIL per law), boundary poison arm (OPEN,
lane-g16), AE4 arm E loud [SKIP].

## Consequence

Rollup lands the DMADV ledger with the falsifier applied: 8 VERIFIED /
5 DEFECT-OPEN / 2 ASPIRATIONAL-UNVERIFIED; Verify = NOT PASSED;
T2/T3/T5 OPEN. Fences that could regress: none — this lane wrote no
code and ran no mutations on the shared tree; the doc-only diff cannot
break cargo/pytest/just gates (boundary_gate.py, zero_string_gate,
make gate0 unaffected).

**Falsifier verbatim (recorded in the rollup, governing):** DMADV
Verify phase passes iff every CTQ row cites a this-session command+
exit and zero rows are aspirational without a decided-gate pointer.

## Standing

ALIVE for the rollup artifact itself (written, byte-verified on disk
this session; sha256 recorded at write time; line-length gate 0
over-length). All cited numbers are as-of-receipt and were re-read
from audit/ on disk this session; nothing re-executed, so no code
standing is claimed. DF-1..DF-5: BLOCKED-ABSENT (dflss/ dir did not
exist at lane start).

## Divergences / disclosures

1. Dispatch asserted 429 deaths for GIT-03 AND GIT-07; on disk only
   GIT-07 (lane receipt line 3) and repair-05 (w7-errata.md:18) carry
   the 429 witness. GIT-03's 429 is UNVERIFIED — recorded as such in
   the rollup rather than asserted.
2. DF-5 not landed: the Verify gate matrix cites fresh exits from
   audit-ledger.md instead, per the dispatch else-branch.
3. Falsifier honestly NOT satisfied: 5 DEFECT-OPEN + 2
   ASPIRATIONAL-UNVERIFIED rows; recorded, not rounded up.
