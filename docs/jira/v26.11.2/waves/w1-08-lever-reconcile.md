# W1-08 Lever Reconcile Receipt

Lane: W1-08 · 2026-11-01 · repo /Users/sac/cgap · no git writes.

## Subject

Reconciled the V2-B lever table in
`docs/jira/v26.11.2/05-wave-plan.md` to the authoritative
`docs/jira/v26.11.2/02-pareto-levers.md`. The old table mapped levers
to wrong crates (SWAR FNV -> cgap-core, PLNF -> bont, ring sink ->
storage); the reconciled table matches 02:

- L1 ingress digest escalation -> cgap-gate (penstock digest stage)
- L2 SWAR FNV-1a seal -> cgap-parquet (`footer.rs:119-127`)
- L3 PLNF bit-slice semi-join -> cgap-bn3 (`unify.rs:11,35-39,274`)
- L4 pre-leased ring sink -> cgap-ffi (`sovereign.rs:706-730`)
- L5 fused ASC450 metering -> cgap-gate + cgap-p28

Table now carries all six columns (lever | crate | root cause file:line
| target | anti-vacuity mutation | measurement); root-cause detail and
floor multipliers live in a footnote list under the table, sourced
verbatim from 02.

## Files written

- `docs/jira/v26.11.2/05-wave-plan.md` (edited; V2-B section only,
  rest intact)
- `docs/jira/v26.11.2/waves/w1-08-lever-reconcile.md` (this receipt)

## Commands / exits

- `awk 'length > 100 ...' 05-wave-plan.md` -> exit 0, no output
  (all lines <=100 chars, verified after final edit).
- `git diff --stat -- docs/jira/v26.11.2/05-wave-plan.md` -> 1 file
  changed, 31 insertions(+), 9 deletions(-) (read-only git).

## Verification

- Each reconciled cell was cross-checked against 02-pareto-levers.md
  on disk this session: root-cause file:line refs
  (`footer.rs:119-127`, `unify.rs:11,35-39,274`,
  `sovereign.rs:706-730`), targets (<15 ns/frame, <10 ms @ 1M,
  <75 ms @ 1M, <4 us/tx, <1.45 us/row), and mutation classes all
  match 02 verbatim. Targets remain TARGETs until re-measured.

## Divergences

- 02 gives no file:line for L1's root cause (described as the penstock
  digest stage / unconditional BLAKE3 pre-admission) and no file:line
  for L5 (described as the split-pass sequence); the table cells for
  those levers carry the stage description, with full detail in the
  footnote list.
- The 6-column table cannot hold full mutation/floor text within the
  100-char line rule, so floors (>=6.8x / >=5x / >20x / >=11x /
  >=1.6x) and mutation detail moved to the footnote list rather than
  being dropped.
- V2-B agent-arithmetic line cites agent counts only (no crate
  mapping), so it was left untouched.

## Standing

ALIVE for the reconciliation itself (edits + line-length gate run this
session, exits above). The underlying lever specs remain TARGETs /
UNVERIFIED until their V2-B courts and Divan receipts land.
