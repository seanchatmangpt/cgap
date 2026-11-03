# Lane RW-08 — Cpk/capability claim rewrite over dflss/09-ax-cpk-msa.md

Repo /Users/sac/cgap @ main, 2026-11-02. Owned file (only file written
as subject): `docs/jira/v26.11.2/dflss/09-ax-cpk-msa.md`. Plus this
receipt per receipt law. No git commands. No source edits. No benches.

## W1 Method

Read in full first: `11-measured-claims.md` (131 lines),
`dflss/03-msa-gauge.md` (234), `redteam/fix-18-measured-claims-sweep.md`
(128), target file (88). Every citation in the target was re-read from
disk this session before it was kept: ipc-03 :43-47, ipc-08 :13,27,
ax-00 :17,19-22,44-53, g09 :7-29,56-57, dflss-04 :19,32-34,53, b01
:18-19, repair-13 :34-44, w3-l1 :62-72, 05-cpk :60-68, sovereign.rs
:408-418, 11-measured :29,:52,:5-7, fix-18 W2/W3. All verified present.

## W2 Diff shape (merge, not clobber)

- Sec 1 structure kept; added 185/7-run row (ipc-03:47, max-of-N is
  not a quantile) and the **batch-amortization limit block** (ADV-09-R1
  / 03-msa §1.3): batched-mean arithmetic on sub-15 ns CTQ-02 numbers
  is a BATCH-MEAN Cpk, never per-frame capability; the limit attaches
  to "slowest"-class maxes too.
- Sec 2 kept, as-of wording ("IPC-7 had no receipt at the index's
  receipt time") so the pending index cannot read stale.
- Sec 3 re-grounded: scope law (quiet-host per row 5 / red-10),
  row-28 contention divergence cited, G09/DF-4 rows now carry receipt
  file:line + spelled-out conditions (sigma 0.0097 vs 0.276, means
  1.84 vs 1.574 GiB/s, n=30), and R-15's fresh quiet legs
  10.26/10.42/10.65 named as the as-of CTQ-02 numbers with the sweep
  as cite; R-15's typed contention divergence (25.60 ns/frame, guard
  fired) disclosed without amending row 28.
- Sec 4 kept + added: -9.6525 inject Cpk cite (dflss-04:19), "-5.23"
  marked UNVERIFIED-WITHOUT-METHOD per 03-msa §2.2.
- Sec 5 standing rewritten to RW-08 session evidence + fences.

## W3 Divergences

1. Dispatch said "quiet legs (10.26-10.65)"; the sweep's three legs
   are 10.26 / 10.42 / 10.65 fastest — same band, exact per-leg values
   cited.
2. No batch-amortized Cpk computation exists on disk for CTQ-02 (no
   receipt computes one), so Sec 3 carries the LIMIT LAW rather than a
   capability number — inventing a Cpk was out of contract.
3. Table cells use bare receipt filenames (<=100 char line law); full
   `../lanes/...` paths for the same receipts sit in the bullet list
   directly above the table.

## W4 Standing

ALIVE (citation-grounding pass): every cite re-read from disk this
session; no number in the file without a receipt read this session or
an explicit UNVERIFIED-WITHOUT-METHOD / ASPIRATIONAL-UNVERIFIED mark.
No bench/container command run this lane, so no execution claim.

## W5 Fences named (fence-law reporting)

- Exclusivity law (`11-measured-claims.md:7-8`): re-verified — every
  number traces to a receipt read this session or a typed U-W-M mark.
- Do-not-promote fence on 23.35 ns/fact (`03-msa-gauge.md` §2.2):
  intact — the figure does not appear in 09.
- 11-measured rows 5/28 scoping: strengthened, not weakened; row 28
  magnitudes untouched, R-15 divergence disclosed as typed.

```json
{"lane":"RW-08",
 "files_written":["/Users/sac/cgap/docs/jira/v26.11.2/dflss/09-ax-cpk-msa.md",
  "/Users/sac/cgap/docs/jira/v26.11.2/lanes/lane-rw-08-ax-cpk-claims.md"],
 "commands":["wc -l 4 input files (exit 0)",
  "Read 11-measured-claims.md / 03-msa-gauge.md / fix-18 sweep / 09 target in full (exit 0)",
  "sed+grep receipt re-reads: ipc-03, ax-00, g09, dflss-04, posh-02, ipc-08, repair-13, b01, w3-l1, 05-cpk, sovereign.rs (all exit 0)",
  "python3 chunk appends x3 + table-line fixes (exit 0)",
  "awk length>100 sweep (exit 0, 0 overlong lines)",
  "full Read of written file post-write (exit 0)"],
 "courts_run":["docs-only lane; falsifier = citation grounding",
  "anti-vacuity: '-5.23' explicitly marked U-W-M instead of silently dropped"],
 "standing":"ALIVE",
 "divergences":[
  "quiet legs cited per-leg 10.26/10.42/10.65 (dispatch band 10.26-10.65)",
  "no receipted CTQ-02 Cpk exists on disk; limit law landed, number not invented",
  "table cells carry bare filenames for the 100-char line law"]}
```
