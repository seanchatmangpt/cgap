# Lane RW-06 — claim rewrite: 09-overnight-mutation-set + 10-ggen-max-projection

Date: 2026-11-02. Repo: /Users/sac/cgap (main). No git write commands.
Owned files (only files written besides this receipt):

- `docs/jira/v26.11.2/09-overnight-mutation-set.md`
- `docs/jira/v26.11.2/10-ggen-max-projection.md`

## W1 Identity

Read `11-measured-claims.md` in full first (32 claims + addendum + 4
instrument divergences). Swept both owned files for quantitative and
mutation/court inventory claims; rewrote to receipted values.

## W2 Diff — 09-overnight-mutation-set.md

- L1-M1, L2-M1, L5-M1b rows: [UNFLIPPED-t26.11.2] -> FLIPPED-t26.11.2
  with this-campaign cites (w3-l1-ingress.md:75-83 escalate variant;
  w3-l2-swar-fnv.md:38-42; w3-l5-fused.md:44-52) — the prior blanket
  "no mutation executed against v26.11.2" claim was stale hearsay.
- Legend paragraph rewritten to the conditional default.
- NEW "Wave witnesses (2026-11-02)" section: 22 WV-* bullet rows, all
  FLIPPED, zero skips, each citing the executing lane receipt (fix02,
  rc03, fix04, adv12fix, r14, rc04, rc13, ocel01, rc08, rc09). Every
  cite verified to exist on disk with matching content at the named
  lines.
- Unwitnessed mutants loudly listed as [UNFLIPPED-t26.11.2],
  TARGET-fenced (G1-M1, G2-M1, G3-M1, G4-M1, G5-M1, G6-M1/M2, G7-M1,
  G8-M1, L3-M1, L4-M1a/M1b/M2, L5-M1a/M1c).

## W3 Diff — 10-ggen-max-projection.md

- Freshness addendum (2026-11-02, this session's census): 1,716 files /
  14,261,546 B tracked (was 903 / 9,156,129); docs/jira 6,110,372 B
  (was 3,162,488); scripts/ 877,494 B (was 590,104); crate src rs
  3,903,678 B; benches 19 (was 15); waves 339 (doc named 414);
  custom_handlers seam 5,279 + 38,743 siblings = 44,022 B.
- 1.20% machinery share marked UNVERIFIED for today's tree (the G1K-5
  classification was not re-run — only the absolute census was).
- Ladder item 4's "15 files" and item 3's "~34%" flagged stale;
  ladder shares are TARGET until re-derived.

## W4 Commands + exits (this session)

- `git ls-files | xargs wc -c | tail -1` -> 14261546 total; count 1716;
  bench glob 19; waves glob 339; wc -c custom_handlers.rs -> 5279;
  siblings total 38743. Exit 0.
- python census (getsize): docs/jira 6110372; scripts 877494;
  crates/**/src/**.rs 3903678. Exit 0.
- cite-existence checker over 22 WV cites: all OK. Exit 0.
- awk length>100 sweep: new sections clean; pre-existing G/L table
  rows (09 lines 26-34, 40-48) exceed 100 chars from HEAD — markdown
  single-line table rows, left intact (merge, never clobber).
- `git diff --check` on both files -> exit 0.

## W5 Fences

- 11-measured-claims.md untouched (not owned) — all numbers here
  cite into it or into lane receipts, never past it.
- No gate/court runs required: doc-only lane; no code touched; shared
  tree untouched outside the two owned files + this receipt.

## W6 Standing

ALIVE for the lane surface: every rewritten claim cites a receipt run
this session or a receipt verified on disk at cite time. Census
figures are command-backed this session.

## W7 Divergences (disclosed)

1. Mid-lane self-corruption: a python splice keyed on startswith
   '| ID | Subject' matched the G-gates table header too and clobbered
   the G/L registers; detected by full-file Read; repaired by
   reassembling from `git show HEAD` + my recorded edits. Final
   on-disk state verified section-by-section (headers at 9/22/36/50/76,
   22 WV rows, closing TARGET fence intact).
2. Pre-existing >100-char table rows in 09 (lines 26-34, 40-48) kept
   verbatim from HEAD — wrapping breaks markdown tables; recorded as
   pre-existing format divergence, not introduced here.
3. Doc 10's baseline "414 waves files" vs today's 339 under
   'docs/jira/**/waves/**': both are real measurements at different
   times; both kept with dates rather than silently overwritten.
