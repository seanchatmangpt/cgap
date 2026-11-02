# SM-6 INDEX-RECONCILE receipt (v26.11.2)

Lane: SM-6-index. Lease: CLAUDE.md pointer-index sections (Layout doc
pointers only) + this receipt. No git writes. Branch: main.

## Trigger

Bounded poll for SM-1's archive marker
`docs/jira/v26.11.2/readme-archive/CLAUDE-pre.md`: present on first poll
(SM-1 CLAUDE-ROOT landed).

## Pointers removed

None. The only pre-campaign pointer in CLAUDE.md's pointer sections was
`docs/jira/v26.11.1/{lanes,waves}/`, which still resolves (152 wave
receipts) — kept, annotated with the bench-chains PARETO receipts.

## Pointers added (CLAUDE.md, Layout section)

- `docs/jira/v26.11.1/waves/bench-chains-{bn3,ingress,parquet,seal,sovereign}.md`
  (annotation on the existing v26.11.1 pointer)
- `docs/jira/v26.11.2/` — 00-ard-prd.md, 01-release-gates.md,
  02-pareto-levers.md, 03-ost-design-around-register.md,
  04-stardf-closure.md, 05-wave-plan.md, OVERNIGHT-LOOP.md,
  RESTART-NOW.md, waves/ (sm-*, v2-* receipts),
  readme-archive/ (CLAUDE-pre.md)
- `docs/diataxis/` — tutorials/, how-to-guides/, reference/, explanation/
  (plus how-to/, marketplace-generated/), INDEX.md / SUMMARY.md / README.md

## Resolution checks

21/21 pointers resolve on disk (`ls` receipts in session log, all OK;
bench-chains-bn3.md and bench-chains-sovereign.md spot-checked
explicitly, all five bench-chains-* enumerated via ls).

## Divergences vs dispatch

1. `docs/WORK-BOARD.md` (stardf): DOES NOT EXIST. Not added.
2. `docs/jira/v26.11.2/HANDOFF`: DOES NOT EXIST. Not added. RESTART-NOW.md
   exists (not in dispatch list) — added instead.
3. Dispatch said "7 files" under v26.11.2; actual top-level docs: 8 files
   + RESTART-NOW.md + waves/ + readme-archive/.
4. readme-archive/ contains only `agents/` and `CLAUDE-pre.md`; no
   README archive yet (SM-1 may be mid-archive; marker criterion met).

Standing: ALIVE (all pointers verified with ls this session).
