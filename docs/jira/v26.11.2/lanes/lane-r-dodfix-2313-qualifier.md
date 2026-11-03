# Lane R-DODFIX — 23.13 ms registered-conflict qualifier

Date: 2026-11-02. Repo /Users/sac/cgap, branch main. Owned files ONLY:
`audit/dod-12-g2-decision.md`, `audit/dod-19-shipgate-final.md`.
No git commands. Merge-not-clobber edits; numbers preserved, qualified.

## W1 Subject

ADV-07 SPOILER (narrow): dod-12:26 and dod-19:58 quote the
un-reproduced 23.13 ms chain_ram L2 lane figure unqualified. Fix =
registered-conflict qualifier per `11-measured-claims.md` row 9 and
`dflss/02-voc-ctq-matrix.md:252-265` (R-07 paragraph, read intact
this session).

## W2 Diff (before/after)

dod-12 (5 sites qualified):
- :26 L2 rollup bullet — appended qualifier:
  "chain_ram lane figure; consolidated row 9 = 42.58 ms governs;
  23.13 vs 42.58 same-surface 1.84x divergence registered,
  unreconciled — UNVERIFIED-FOR-PAPER (row 9; matrix:252-265)".
- :85-86 Measured line — same qualifier appended after
  "Sandbox == shared tree within noise" (covers 1.426 ms @65,536,
  same chain_ram receipt family).
- :89 mutant-comparison parenthetical — "23.13 baseline" now reads
  "23.13 baseline — itself UNVERIFIED-FOR-PAPER vs row 9's 42.58 ms
  governing figure".
- :97-98 MV-5 budget — DECIDED 25.0 now carries
  "[lane figure, UNVERIFIED-FOR-PAPER; row 9 = 42.58 governs]".
- :265-266 decision-log re-scope — "fastest 23.13 / median 23.32 —
  lane figure, UNVERIFIED-FOR-PAPER vs row 9's governing 42.58 ms".

dod-19 (1 site qualified):
- :57-61 L2 cell in P2 measured-perf block — qualifier inserted
  after "23.13 ms vs <10 ms": "(chain_ram lane figure; consolidated
  row 9 = 42.58 ms governs; 23.13 vs 42.58 same-surface 1.84x
  divergence registered, unreconciled — UNVERIFIED-FOR-PAPER, row 9
  of ../11-measured-claims.md)"; L3/L4/L5 text preserved.

## W3 Commands / exits

- grep 23.13 both files (pre-edit site census) — exit 0.
- awk length>100 sweep post-edit — zero hits, exit 0.
- sed re-read of all edited regions post-edit (verification) — exit 0.

## W4 Verification ladder

Re-read on disk after every Edit (harness Edit error = fail-closed).
Final grep: every 23.13 site in both files now co-located with a
row-9-governs / UNVERIFIED-FOR-PAPER qualifier; no bare 23.13 remains.

## W5 Anti-vacuity note

Doc-only lane; no court to flip. Non-vacuity = the sweep: a reader
grepping `23\.13` in audit/ now cannot reach a governing-decision
line without also hitting the registered-conflict qualifier
(pre-edit they could — that was the ADV-07 defect).

## W6 Other-numerics sweep (dispatch item)

Grepped both files for ms/ns figures without register cites.
Findings NOT qualified (different receipt families, no registered
conflict; ADV-07 Probes 3/4 found them consistent):
- dod-12:24 L1 11.04 ns/frame (lane-gap-07, VACUOUS already labeled).
- dod-12:114-116 L4 9.35 us/tx and 2.472 ms GAP-07 distribution
  (already carries its own CRITICAL CAVEAT, 15.5x divergence noted).
- dod-12:140-141 L5 3.055/155.2/200.2 ms (own flip witness noted).
- dod-19:59-60 L4/L5 figures (inherit the same GAP-07 rollup cite).
None cite the disputed chain_ram L2 family except the sites fixed
above. 1.426 ms @65,536 IS chain_ram-family and was qualified via
its host line :85-86.

## W7 Standing

ALIVE — all edits + verification commands run this session, exits
above. Divergences: none (dispatch premise matched disk; ADV-07
read before editing). Fence check: this diff touches only the two
owned audit docs; the governing sources (11-measured-claims.md row
9, 02-voc-ctq-matrix.md R-07 paragraph) were read, not modified —
no existing gate or court consumes these audit docs, so the
must-not-regress fence is textual only, verified by the ≤100-char
and bare-23.13 sweeps.
