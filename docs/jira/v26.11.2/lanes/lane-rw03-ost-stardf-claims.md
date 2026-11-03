# Lane RW-03 — OST register + stardf closure claim rewrite

Date: 2026-11-02. Repo: /Users/sac/cgap, branch main. No git writes.
Owned files (only these written):
- docs/jira/v26.11.2/03-ost-design-around-register.md
- docs/jira/v26.11.2/04-stardf-closure.md

Registry of record read IN FULL first:
docs/jira/v26.11.2/11-measured-claims.md (rows 1-32 + §2-§4).

## W1 Subject
Quantitative claim rewrite over two spec files against the exclusive
numeric registry; refresh stale file:line cites by re-reading source.

## W2 Diff (per-claim before/after)

03-ost-design-around-register.md:
1. Row 1 + G7F block, "64 KiB ring sink" — no scope qualifier.
   AFTER: "64 KiB (CAP = 65,536 B, mmap_writer.rs:775; capacity
   counts BYTES here, never lanes)" in both sites. Registry law:
   bytes and lanes never conflate (CLAUDE.md status-code note).
2. G7F block cite: "mmap_writer.rs:721-842 (drain_to_spill :781-802;
   RunOverrun :829-842)" — STALE. AFTER: ":721-860 (RingSink :721;
   drain_to_spill :828; typed RunOverrun refusals at :843, :851)";
   receipt_chain cite ":34-76" -> ":51-57, :117-120" (prev_hash
   field + link fn, both re-read on disk).
3. "fa9_commit_gating.rs (226 lines)" — STALE. AFTER: 606 lines
   (wc -l this session).
4. Row 2: AHDCLT doctrine note added — compute-kernel half
   RETIRED-REFUTED per docs/jira/v26.10.31/16-ahdclt.md:3-4,13
   (REVISED 2026-11-02, RED-01 clean-room rebuild); no register row
   rests on claimed unconstructability.
5. Row 3 cites: unify.rs:469-531 -> :553-563 (RuleCompiler);
   :783 -> :914 (semi_join_masks); :665 -> :744 (survival mask);
   :650 -> :740 (JoinProbeScratch); closure.rs:94 -> :114
   (semi_naive_closure); Generator doc-comment :1688 -> :1887.
6. Row 4 cites: R0..=R255 :100,169 -> :155, :486;
   RegisterOverflow :191 -> :246; sigil.rs:757 -> :773;
   builtins.rs:497 -> :497-498.
7. Receipt section: RW-03 sweep note added (registry-scoped perf
   figures absent from both files — nothing to rewrite there).

04-stardf-closure.md:
8. Standing section: quantitative sweep note appended — no campaign
   perf/footprint figures present; 11/13/5264/10-vs-10/128 counts
   stay receipt-bound; 392 riscv count remains DISPATCH-CLAIMED /
   UNVERIFIED (never quoted as fact). No numeric edits needed.

## W3 Commands / exits (this session)
- grep verify sovereign.rs:87 (0x4347_0006), mmap_writer RingSink/
  drain_to_spill/RunOverrun lines, receipt_chain prev_hash, unify.rs
  all six cites, sigil:773, builtins:497-498, closure:114,
  bitslice 930 lines + seeds at :687-699: exit 0 each.
- wc -l: bitslice 930, fa9_commit_gating 606, unify 2946,
  sigil 857, builtins 1495, noalloc 1086.
- awk >100-char sweep: only pre-existing register TABLE rows exceed
  100 chars; all prose lines <= 100.

## W4 Verification ladder
- Every rewritten cite re-read on disk this session (grep + awk at
  exact lines) — not recalled from the prior receipt.
- Line-length sweep: prose <= 100 chars; >100 lines are the
  pre-existing register table rows (unchanged format).
- diff stat: 03 = 60 lines changed, 04 = 12 inserted. Minimal diffs,
  merge-not-clobber (all prior content retained).

## W5 Replay
Files: docs/jira/v26.11.2/03-ost-design-around-register.md,
docs/jira/v26.11.2/04-stardf-closure.md, this receipt. Replay =
re-run the greps in W3 against the cited lines; every AFTER cite
resolves to the named symbol.

## W6 Standing
ALIVE (edits written and verified on disk this session; every cite
command+exit run this session). No test/bench runs were in scope —
this is a claim-rewrite lane; no perf claim is introduced.

## W7 Divergences
- D1: dispatch's mission items "36,048 B subject-scoped + bn3
  6,520,984 B REFUSED", "10.20/row-28", and "constant-time row 29"
  do NOT appear in either owned file — no rewrite target exists;
  recorded in both files' sweep notes instead of inventing edits.
- D2: "AHDCLT compute-kernel RETIRED-REFUTED" also absent as a claim
  in the owned files; landed as a preventive doctrine note (row 2
  block of 03) rather than a correction.
- D3: this lane's first receipt write contained a typo'd filename
  (04-stardf-sweep-claims.md); corrected in place on the same
  session's second write before any cite could land. Self-caught,
  no downstream citation existed.
- Fences: no existing gates/courts touch these two spec files
  (verify_ost_design_around.py reads patent/claims.txt + source
  greps, not this doc); the G7-PATENT gate's row statuses cited
  here were re-read from disk unchanged. No regression surface.

## JSON terminus
{"lane":"RW-03","files_written":["docs/jira/v26.11.2/03-ost-design-around-register.md","docs/jira/v26.11.2/04-stardf-closure.md","docs/jira/v26.11.2/lanes/lane-rw03-ost-stardf-claims.md"],"commands":["grep/awk cite verification x6 (exit 0)","wc -l census (exit 0)","awk line-length sweep (exit 0)","git diff --stat (read-only)"],"courts_run":[],"standing":"ALIVE","divergences":["D1 registry perf figures absent from owned files","D2 AHDCLT landed as preventive note","D3 receipt filename typo self-corrected"]}
