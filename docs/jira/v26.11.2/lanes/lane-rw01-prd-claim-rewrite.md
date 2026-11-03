# lane-rw01-prd-claim-rewrite

Lane RW-01 receipt — claim hygiene sweep over `docs/jira/v26.11.2/00-ard-prd.md`
(only file owned/written). Read `11-measured-claims.md` in full first (rows 1-29 +
§4 addendum 30/31/31a/32) — registry of truth.

## W1 Subject

`docs/jira/v26.11.2/00-ard-prd.md` (SUPERSEDED charter record, kept as
campaign-operational record). No other file touched; no git commands run.

## W2 Diff (30 insertions, 2 modified lines)

Minimal tag/qualifier insertions; charter transcription blocks left verbatim:

1. §1 success sentence — added standing block: cgap MW4 arm is [TARGET] (row 26,
   RDFox NOT executed; row 30 not RDFox-comparable per addendum D2); lever
   post-measurement standing (L1 rows 5/8 scoped, row 28 divergence; L2 row 10;
   L5 row 12; L3/L4 row 27 loud skip).
2. §3 after verbatim lever block — post-measurement status: 5.70 NOT reproduced
   (row 7 records 7.71 ns), 10.20 quiet-host-only, 42.7-43.1 ms vs <10 ms,
   3.04 vs 2.37 µs/row, L3/L4 [SKIP] (row 27).
3. §4 G1 — count scope note: "19/22" is the publishable-SET definition, not a
   published-count claim (guarded table 01-release-gates.md:28-32).
4. §4 G2 — post-measurement note: G2 cannot pass today;
   UNVERIFIED-FOR-PAPER until L3/L4 receipts land.
5. §4 G3 — ">50M facts/s/core" tagged [TARGET-v26.11.2] (row 26) + registry §2
   category-error guard for rows 5-10.
6. §4 G4 — all G4 figures tagged UNVERIFIED-FOR-PAPER: no register row covers
   any DRED figure (missing referent; paper-dred.md is prose, not a bench).
7. §4 G5 — budget tagged [TARGET]; nearest receipted figure is row 15's 28,679 B
   pre-link OBJECT (not linked ELF; repo footprint law).

## W3 Commands + exits

- `grep -nE '5\.70|10\.20|124\.06|36,048|6,520,984|…'` over the PRD — exit 0,
  confirmed the superseded RESULT figures appear NOWHERE in the PRD (it carries
  only charter baselines + gate targets).
- `awk 'length>100{...}'` over the PRD — exit 0, zero lines >100 chars (194 lines).
- `git diff --stat` + `git diff` (read-only) — exit 0, 1 file, +30/-2, on-disk
  diff eyeballed; G3 tag typo "rows 5-#10" fixed to "rows 5-10".

## W4 Courts

None run: doc-only lane, no executable gate exists for claim-hygiene edits.
Anti-vacuity equivalent: every inserted number was re-read from
`11-measured-claims.md` rows 5/6/7/8/10/12/15/26/27/28/30 this session (not
recalled), and the grep above proves no untagged superseded figure remains.

## W5 Standing

ALIVE — all edits verified on disk this session (grep exit 0, awk exit 0,
git diff exit 0 against the exact subject file).

## W6 Divergences

- Dispatch named rewrites for 36,048 B (subject-scoped) and bn3 6,520,984 B
  (REFUSED at admission): neither figure appears in 00-ard-prd.md, so no
  rewrite was possible — recorded here as dispatch-vs-reality divergence;
  those figures live in other docs outside this lane's ownership.
- 10.20/124.06/5.70/7.71 also absent from the PRD; recorded via the §3
  status block instead of in-place rewrites.

## W7 Fences

Must-not-regress: PRD is cited by OVERNIGHT-LOOP.md as loop entry; edits are
additive blocks that change no gate names, thresholds, or exit-code contract.
Verbatim §3 charter transcription preserved unchanged.
