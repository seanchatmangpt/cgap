# Lane RW-09 -- defect-register reconciliation (corrosion/fix-wave merge)

Date 2026-11-02. Repo /Users/sac/cgap @ main. Sole owned file:
`docs/jira/v26.11.2/DEFECT-REGISTER.md` (append-only; read the full
11-measured-claims.md first, per contract). Receipt file is the lane
receipt required by receipt law; no other file written. No git.

## W1 Diff

Append-only reconciliation section "RW-09 append (2026-11-02)" in
DEFECT-REGISTER.md (lines 211-360 on disk): 12 REPAIRED-RECEIPTED rows
(RW-09-1..12) and 7 OPEN rows (RW-09-O1..O7), each citing finder
receipt, repair receipt, and witnessed flip, all file:line re-read on
disk this session. No existing row rewritten; D-G08-1..5 + DOD-20
refresh cites re-read and still resolve (stale-cite refresh N/A).

## W2 Commands + exits (this session)

- Read-only: full reads (11-measured-claims.md, DEFECT-REGISTER.md),
  targeted greps/seds over redteam/, redteam/corrosion/, lanes/, audit/
  -- all exit 0.
- One benign sed BSD-arg error mid-exploration (exit 1, no state
  change; corrected with sed -n ranges).
- On-disk verification: wc -l 360 (exit 0); awk length>100 sweep
  over the register: zero over-length lines (exit 0).

## W3 Fences (must-not-regress)

- Register integrity: no D-G08-1..5 or DOD-20-refresh row was edited
  or deleted (append-only diff; grep for the row headers confirms 12
  prior headings intact).
- Duplicate guard: each new row de-duplicated against D-G08 rows and
  the DOD-20 refresh before writing.

## W4 Court / non-vacuity

This is a reconciliation lane, not a court lane; the named
anti-vacuity instrument is the duplicate sweep + on-disk wc/awk
verification above. No mutation battery applies (zero shared-tree
mutation; /tmp probes were none).

## W5 Standing

ALIVE -- reconciliation executed this session against receipts
re-read from disk; every row cites file:line verified above. No
number invented; UNVERIFIED items (O2/O3/O6 legs) stay OPEN, never
rounded to repaired.
