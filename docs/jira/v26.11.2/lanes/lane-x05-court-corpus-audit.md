# Lane X-05 — court-corpus audit (TRANSCRIBED)

**TRANSCRIBED from coordinator session record (the lane's reply-back);
original audit executed 2026-11-02.** Transcription lane FS-07, also
2026-11-02, re-grounded the load-bearing facts on disk this session
(§4). This file resolves the "receipt pending on disk" annotation at
`07-court-corpus.md:194-195` (lane X-05FIX's citation qualifier).

## 1. Verdict table — REPAIRED (12/12 confirmed)

Each row: court exists on disk + receipt witness resolves. Rows are the
DEFECT-REGISTER RW-09-1..12 entries; the coordinator audit re-read each
finder/repair cite from disk and confirmed REPAIRED-RECEIPTED status.

| Row | Defect | Court exists | Receipt witness |
|---|---|---|---|
| RW-09-1 | FA9 alias spill-gate bypass | fa9_commit_gating.rs | lane-fix01-fa9-alias-rekey.md |
| RW-09-1 | FA9 alias spill-gate bypass | fa9_commit_gating.rs | lane-fix01-fa9-alias-rekey.md |
| RW-09-2 | ref_store ISR reentrancy | reentrancy_court.rs | lane-rc01-ref-store-reentrancy.md |
| RW-09-4 | SRAM gate gap | verify_cscb_sram.py bn3-pin | lane-fix02-bn3-sram-gate.md |
| RW-09-4 | SRAM gate gap | verify_cscb_sram bn3-pin | lane-fix02-bn3-sram-gate.md |
| RW-09-5 | rename TOCTOU + symlink | fa9 court 7/7 | lane-r-adv10fix-spill-gate-rename-toctou.md |
| RW-09-7 | gate-battery UTF-8 crash | probe 2b flip | lane-r-adv05fix-gate-battery-utf8.md |
| RW-09-8 | 23.13 ms unqualified | doc layer, 6 sites | lane-r-dodfix-2313-qualifier.md |
| RW-09-8 | 23.13 ms unqualified | doc layer, 6 sites | lane-r-dodfix-2313-qualifier.md |
| RW-09-9 | CONTENTION-SUSPECT | bench-protocol §8 | lane-r-adv09fix-contention-doc-wiring.md |
| RW-09-10 | zero-string debris refusal | gate skip law | lane-rc08-zero-string-debris-skip.md |
| RW-09-12 | shm_open SPOILER | ERRATA append only | lane-f04-redteam-errata.md |

## 2. Verdict table — OPEN (7/7 confirmed)

Finder/owner cites verified resolving on disk at the original audit;
each row names the missing step (no repair receipt on disk).

| Row | Residual | Finder cite | Owner / close condition |
|---|---|---|---|
| O1 | case-variant portability | adv-10-fmea:73-78 | case-fold lane; court on case-mount |
| O2 | O_NOFOLLOW swap window | adv10fix:39-40 + f02:25 | O_NOFOLLOW\|O_EXCL creation lane |
| O3 | implcell behavioral legs | red-11:93 | behavioral RMW court on implcell leg |
| O4 | commit-leg receipt wiring | ocel01:99 + fix08:31 | wire commit leg into receipt chain |
| O5 | R1-R10 execution legs | tc-10:38,93-96 | G2-PARETO + NEW-5 gate receipts |
| O6 | M-CTQ wire legs | 11-measured-claims:126-127 | re-measure on wire instruments |
| O7 | machine contention gate | adv09fix W2 item 1 | machine-consumed SUSPECT gate |

## 3. Manifest determinism + live count

The original audit re-ran the XC-05 exporter twice
(`python3 scripts/conformance_kit_export.py --out <tmp>` x2) and
compared bytes: **byte-identical across both runs**. Live court count:
**95 = 85 cargo-test + 5 corpus-doc + 5 gate-mode**. The +4 delta vs
XC-05's historical 91 is the wave's post-XC-05 courts, as recorded in
`07-court-corpus.md` §6.

FS-07 re-grounding this session (2026-11-02): two fresh exporter runs,
both sha256
`c3ccb0bf1f91f5d6df6286bf7d61f019e1c31`
`3ccb24c3d85c993271ec383ca64` (split for line length; cmp exit 0),
exporter exit 0 each run; parsed count total 95 (85/5/5) — the
transcribed figures are reproduced on this session's tree, not
hearsay.

## 4. Drift finding (single)

`07-court-corpus.md` §6 carried XC-05's stale bare "91 courts" figure.
Fixed by lane X-05FIX (2026-11-02,
`lanes/lane-x-05fix-court-count.md`) — §6 now marks 91 superseded
(historical) and states the live 95 (85/5/5) figure. No other drift
found. This receipt's landing clears X-05FIX's "receipt pending on
disk" annotation at `07-court-corpus.md:194-195` — the annotation
itself stays until the owning lane edits it (out of FS-07's file
ownership).

## 5. FS-07 session grounding (commands + exits)

1. `python3 scripts/conformance_kit_export.py --out /tmp/fs07-m1.json`
   → exit 0; run 2 (`fs07-m3.json`) → exit 0; `cmp -s` → exit 0;
   sha256 both runs identical (hash in §3).
2. Parsed manifest: `total 95`, kinds 85/5/5 (cargo-test /
   court-corpus-doc / gate-mode) — exit 0.
3. Existence sweep of all 16 cited receipt/finder docs in §1/§2
   tables → 16/16 OK (exit 0; list includes lanes/lane-fix01,
   rc01, r-adv11fix, fix02, r-adv10fix, r-adv12fix, r-adv05fix,
   r-dodfix, r-adv09fix, rc08, rc09, f04-redteam-errata,
   adv-10-fmea, tc-10-open-recensus, 11-measured-claims,
   redteam/fix-19-corpus-audit).
4. Prior-state witness: `ls lanes/lane-x05-court-corpus-audit.md` was
   ABSENT per X-05FIX's receipt at edit time (lane-x-05fix:17); this
   file's creation is that receipt landing.

## 6. Fences (must-not-regress surfaces this diff could break)

- `07-court-corpus.md` §6/§7 prose (RW-05/X-05FIX edits): this file is
  NEW-only; no edit to that doc, so no fence can regress. The pending-
  receipt annotation becomes stale-but-accurate history after this
  lands (owner may clear it later).
- DEFECT-REGISTER RW-09 rows: transcription only; register untouched.
- No gates exercise this lane file; the manifest determinism re-run is
  the court here, and its mutation-arm (X-05FIX's flip evidence)
  remains in the cited repair receipts.

Report JSON:
{"lane":"FS-07","files_written":["docs/jira/v26.11.2/lanes/lane-x05-court-corpus-audit.md"],
"commands":["conformance_kit_export.py x2 (exit 0, exit 0)","cmp (exit 0)",
"manifest parse count 95 (exit 0)","cite-existence sweep 16/16 (exit 0)"],
"courts_run":["manifest byte-identical x2 re-grounding (§5.1)"],
"standing":"ALIVE","divergences":["original audit ran in coordinator session;
FS-07 transcribed and re-grounded §3/§5 on disk this session"]}
