# Charter §5 FMEA Reconciliation (lane CE-07)

Date: 2026-11-02 · Repo: /Users/sac/cgap · Standing: **ALIVE**
Subject: `docs/jira/v26.11.2/14-dflss-charter.md` §5 FMEA table
(`14-dflss-charter.md:94-105`) reconciled against the landed
PFMEA matrix `docs/jira/v26.11.2/dflss/04-pfmea-risk-matrix.md`
and its named mitigation receipts.
Method: every receipt and code line re-read from disk this
session; RPN values are assessment values (SEV x OCC x DET,
1-10 each), never measurements. Charter table retained verbatim
per the matrix's own cross-link law
(`04-pfmea-risk-matrix.md:7-9`).

## Divergence: dispatch vs disk (doc yields to reality)

Dispatch described the charter FMEA as "5 rows, RPN 160-288".
Disk truth (re-read this session): the charter table carries
**6 rows spanning RPN 40-120** (`14-dflss-charter.md:96-105`).
The dispatch range 160-288 matches the PFMEA matrix's R8/R10
rows instead (R8 stale-binary 288, R10 429-lane-death 180).
No on-disk charter carries a 160-288 range; the 160-288 figures
the dispatch attributed to the charter are the matrix's
current-state rows, not the charter's. Recorded as divergence,
not silently reconciled.

## Row map: charter row -> FM-NN -> landed receipt

The charter's 6 rows are pre-wave risk statements. The FM-01..08
register is the red-team FMEA that supersedes the charter DET
column with witnessed flips
(`04-pfmea-risk-matrix.md:7-9, 84-92`).

Per-row map (block form to hold the 100-char cap):

1. Stale-mtime vacuous pass (120) -> R7-class write
   corruption + FM-07 demux poison; receipts
   lane-dx-02-write-integrity.md:7-10,
   lane-fix05-pack-demux-poison.md. Post: R7 84, FM-07 20.
2. 1 blr in fixpoint_opt (84) -> no FM row (charter-local);
   no mitigation receipt on disk. UNCHANGED.
3. Pre-link object as ELF (90) -> closest R8 stale-binary,
   DET=8 ungated (no LSPA-2/4 on disk). Matrix R8=288.
4. Unverified parquet figure (40) -> FM-06 timing jitter
   120 via red-10; claims row closed by QUAL-01/X-04FIX
   qualifier + 11-measured-claims.md row 29
   (lane-x04fix-bare-met-stragglers.md; adv-15).
5. Publish vs lever lanes (42) -> no FM row; publish law
   OVERNIGHT-LOOP.md:146-149. UNCHANGED (law-only).
6. Compile freeze (48) -> no FM row; 10-min SLA + dual
   disclosure. UNCHANGED (law-only).

## Row 1 — stale-mtime vacuous pass (charter RPN 120)

Charter cites the mtime-bump law
(a02-finish-receipt.md:170-184). Post-charter machinery:
DX-2 write-integrity hook landed with a court battery
including fire->clean byte-restore flips
(lane-dx-02-write-integrity.md:26-43), and the PFMEA's fresh
session runs re-derived DET=2 scores from real exits
(`04-pfmea-risk-matrix.md:11-19`).

## Row 2 — vacuous-test row: mutation-court law + ADV-14 attack

Charter row "1 blr in fixpoint_opt regresses" (RPN 84) is the
vacuous-pass class the mutation-court law governs: a court that
cannot flip is FAIL, a mutation-free green run is VACUOUS
(cgap/CLAUDE.md gate law; PFMEA anti-vacuity section,
04-pfmea-risk-matrix.md:55-60).
ADV-14 is the non-vacuity attack on the landed regression
court (`redteam/corrosion/adv-14-regression-court-vacuity.md`):
**5/5 legs FLIPPED, witnessed** — legs 1-5 each exited 101 in
the vulnerable-copy state (M-L1-FULL alias+inode rekey,
M-L2 init_within deletion, M-L3 Done-arm poison, M-L4
edge-triggered andon, M-L5 SRAM budget lie), clean pre/post
controls exit 0 (5 passed), sha256 fail-closed restores with
mtime bumps, shared tree byte-identical post-run.
Leg-1 two-layer finding: neither layer decorative — each
single-layer revert still PASSES, full two-layer revert flips
(no single point of failure, no spoiler). One in-court doc
comment drift noted at breach_regression_court.rs:53-57
(stale canonical_key-only "the" mutation text).
FM-01/FM-07/FM-08 rows carry the same law with their own
witnessed flips: FM-01 MUTANT-ADV10/ADV17 exit 101 each
(lane-r-adv10fix receipt W3); FM-07 m1/m3 exit 101 with
sha256-exact + mtime-bumped restores
(lane-fix05-pack-demux-poison.md); FM-08 M1-M4 4/4 flip
(lane-r-adv12fix-quota-repair.md §3).

## Row 3 — ABI row: c_header_abi_court 4/4 + ADV-02 IMMUNE

Charter row "Pre-link object asserted as ELF" (RPN 90) is the
subject-class/lies-about-artifact class. Post-charter state:
`dod/dod-03-abi-seal.md` — `c_header_abi_court` 4/4 exit 0
(dod-03-abi-seal.md:75-76,104); ADV-02 foreign C-client court
exit 0, "ADV02-C-CLIENT: IMMUNE (0 mismatches)"
(dod-03-abi-seal.md:64,103). ADV-02 receipt
(`redteam/corrosion/adv-02-ffi-abi-corruption.md`) is the
corrosion attack this closure answers: symbol/dylib ABI
desynchronization attempted, court-immune result witnessed.
Charter RPN 90 is superseded in assessment only by the
PFMEA's R8=288 row, which honestly scores the still-ungated
stale-binary class (LSPA-2/4 receipts absent on disk,
04-pfmea-risk-matrix.md:32) — a DIFFERENT risk from the ABI
seal, which is closed with witnessed exits.

## Row 4 — claims row: unverified figures, QUAL-01/X-04FIX + row 29

Charter row "Unverified parquet figure in prose" (RPN 40).
Landed machinery: `11-measured-claims.md` row 29 pins
"constant-time" as UNSUPPORTED (BR3 typed-refusal class);
X-04FIX qualifier landed inline across five files, grep
1/1/1/1/1 exit 0 (lane-x04fix-bare-met-stragglers.md:15-26);
12-ard-prd-cgap.md:56-58 carries the row-29 QUALIFIER;
ADV-15 hostile-reading court confirms row 29 forbids the
hostile/constant-time reading
(redteam/corrosion/adv-15-claims-boundary-injection.md:73-80),
post-QUAL-01 CTQ wiring per adv-15:14. Charter RPN 40 vs
FM-06 timing jitter 120: FM-06 is the red-team-measured
version of the same class (unqualified latency claim dead,
red-10 12.2x under contention) — the charter under-scored
it because the attack had not yet run when the charter
landed.

## Rows 5-6 — publish law + freeze SLA (unchanged, law-only)

Both rows are governed by standing laws, not mechanical
courts: publish sequencing (OVERNIGHT-LOOP.md:146-149, one
attempt per crate, back-off 1h) and the 10-min compile-freeze
SLA with dual disclosure (same-checkout-fanout additions 2/6).
No FM-NN counterpart; no mutation court exists for either;
RPN carried forward unchanged, honestly marked law-only.

## Contradiction flags (charter claim vs landed matrix)

1. Dispatch's "RPN 160-288" attribution to the charter is
   contradicted by disk: charter spans 40-120 (6 rows);
   160-288 belongs to matrix rows R8/R10. Flagged above.
2. FM-01 192->72: the charter predates the FM register
   entirely; the 192 pre-repair assessment appears in the
   red-13/fix01 lineage, the 72 post-repair assessment in
   `04-pfmea-risk-matrix.md` §3.1 (FM-01 row). Both are
   assessment values; the charter never carried 192.
3. FM-07 30->20: the §3.1 row body carries SEV 5 / OCC 2 /
   DET 3 => RPN 30 in the row header while the STATUS text
   records the re-score "DET re-score 3->2, RPN 30->20" —
   the header column and the body text disagree by 10 points
   on disk (04-pfmea-risk-matrix.md:224 vs :240). Flagged as
   an internal matrix inconsistency for the DF-6 owner; not
   fixed here (file not owned by CE-07).
4. Charter DET column is explicitly superseded by the matrix
   per the matrix's own cross-link law
   (04-pfmea-risk-matrix.md:7-9); the charter rows remain
   verbatim as historical record, per law.
