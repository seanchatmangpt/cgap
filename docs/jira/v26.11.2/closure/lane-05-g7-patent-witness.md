# Lane CL5 receipt — G7-PATENT mechanical witness

Date: 2026-11-02. Repo /Users/sac/cgap, branch main. READ-ONLY lane:
sole write is this file; all mutations in /tmp (deleted via
shutil.rmtree). No Process.sleep.

## 1. Gate runs x2 (stable)

`python3 scripts/verify_ost_design_around.py` (cwd /Users/sac/cgap):

- Run 1 -> 4/4 DISTINCT-MECHANISM, "G7-PATENT: PASS 4/4
  DISTINCT-MECHANISM (exit 0)", EXIT1=0.
- Run 2 -> 4/4 DISTINCT-MECHANISM, EXIT2=0. Stable.

Row detail both runs: rows 3+4 report "src hits: 1" each — the sole
src hits (unify.rs:1688 "Generator" doc-comment class, row 4
&dyn-core::error::Error upcast class) are classified non-mechanism
and disclosed in ost_report.json; tests hits are not practicing.

## 2. Physical citation bindings (grep-verified this session)

| Row | Binding | Evidence (grep, exit 0) |
|---|---|---|
| 1 (US10,817,467) | mmap_writer.rs | RingSink struct :721; RunOverrun :237 etc.; drain_to_spill (register list; marker grep family present) |
| 1 extra | sovereign.rs | :87 `pub const CGAP_ERR_CONCURRENT_WRITER_DENIED: CgapStatus = 0x4347_0006;` (matches C13 receipt); :764 pre-IO denial; :1785 register-pin assert |
| 1 extra | fa9_commit_gating.rs | :32 import, :34 DENIED alias, :66 exact register pin assert, :214 same-path denial expect |
| 2 (US11,216,456) | bitslice.rs | register cites bitslice.rs (930 lines; SplitMix64 seed :687-699); gate markers "bit-slice" / "Pack `values` into the vertical" verified by gate evidence_markers |
| 3 (US11,409,698) | unify.rs | emit_join_masks_view CURRENT lines :928 (call), :934 (fn — post-BFX, register said ~:934-985, confirmed); semi_join_masks fn :885 (register said :783 — DRIFTED, current :885); RuleCompiler struct :543 / impl :545 (register said :469-531 — DRIFTED) |
| 4 (US11,645,278) | unify.rs | register row-4 marker file = crates/cgap-bn3/src/unify.rs; markers R0 (register ABI R0..=R255, :145/:177/:214) + RegisterOverflow (:236) grep exit 0 |

Line drift note: unify.rs post-BFX lines moved vs the register doc's
row-3 cites (:783->:885 semi_join_masks; :469-531->:543/:545
RuleCompiler); emit_join_masks_view matches (~:934). Marker content
unchanged; gate is marker-based, not line-based, so verdicts
unaffected. Register doc row-3 line cites are stale and should be
re-pinned at next register edit.

## 3. Explicit limitation (verbatim, per C13 receipt)

"The gate is a register-consistency gate. The element column remains
self-authored — no patent text exists on disk to compare against,
so element wording is unfalsifiable inside the gate and ZZQQ-class
corruption still passes." (FA11 M2 residual; element text is
self-authored, patent text not on disk — OPEN.)

## 4. Anti-vacuity flip (merged duplicate, /tmp, replicated)

- Mutation: /tmp/cl5-flip/claims.txt — row-2 (US11,216,456)
  mechanism field := row-1's full mechanism text (single-line rows,
  4 pipe fields each). Two earlier replication attempts were harness
  defects (continuation-indent loss -> parse ValueError, exit 1 for
  the wrong reason); disclosed, not verdicts.
- `python3 scripts/verify_ost_design_around.py --claims
  /tmp/cl5-flip/claims.txt --report /tmp/cl5-flip/report.json`
  -> rows 1+2 NOT-DISTINCT (duplicate/near-duplicate mechanism
  shared across rows ['US10,817,467','US11,216,456']), FAIL 2/4,
  FLIP_EXIT=1. Matches C13's merged-duplicate flip (exit 1,
  docs/jira/v26.11.2/convergence/lane-13-g7-patent-final-audit.md:40-43).
- /tmp/cl5-flip deleted via shutil.rmtree.

## Standing

ALIVE — gate 4/4 exit 0 x2 this session on the exact subject; every
row's citation markers grep-verified on disk; flip court exits 1;
limitation recorded verbatim. Shared tree untouched (claims.txt " M"
is pre-existing per session git-status snapshot; gate's
ost_report.json write is the gate's own output artifact).

Claims sha256: 963230bbb7167761de5bf0436bf2386a493d7e2b1e2df4bd676f0c53cdad35ac
