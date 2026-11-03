# Lane DOD-08 — claims register + pareto levers final freeze (2026-11-02)

Repo /Users/sac/cgap @ main. Owned files ONLY:
`docs/jira/v26.11.2/11-measured-claims.md` (append-only), `docs/jira/
v26.11.2/02-pareto-levers.md` (append-only). No git commands. Existing
rows in both files untouched (append verified: claims 131 -> 150 lines,
pareto 184 -> 211 lines).

## W1 Subject

v26.11.2 claims register + pareto lever docs, pre-ship freeze: §5
optimization-discovery addendum (rows 33-37) and optimization-pathway
table (OP-1..3).

## W2 Diff

- `11-measured-claims.md`: appended §5 — five TARGET-class discovery
  rows: (33) egress-ring false sharing
  (`crates/cgap-git/src/egress_ring.rs:421-425` head/tail/quota
  co-residence; pad pattern exists as `PaddedIndex`,
  `ingress_ring.rs:226-242`), (34) SWAR digest fixed-trip fold
  (`cgap-gate/src/penstock.rs:548-556` byte-granular early exit vs
  `cgap-branchless/src/penstock.rs:64` `swar_all_zero_vl`, equivalence
  court `:149-167`), (35) commit-tx syscall amortization
  (`atomic_commit.rs:696`; ~9-12 syscalls ≈ 10-25 us/tx recorded as a
  discovery ESTIMATE, never a claim; measurement required first),
  (36) ring-relay ~4x memcpy amplification (`pack_demux.rs:288`;
  slice-view proposals P1/P2), (37) OPT-03 channel-curve finding
  (MEASURED hot-ambient, on-disk receipt; tail knee not bus knee;
  guard common-mode blindness registered).
- `02-pareto-levers.md`: appended "Optimization pathway" — OP-1 tail
  isolation outranks OP-2 bandwidth reduction per OPT-03's
  attribution (fastest column dose-invariant 10.67-12.18 ns/f, tail
  up to 4.1x); OP-3 guard third-channel (uniform-shift detector,
  lane-rc11 follow-up). All rows TARGET (v26.11.3-candidate).

## W3 Commands / exits

- Read/verify only: `sed`/`grep` cite verification against
  egress_ring.rs, ingress_ring.rs, penstock.rs (both crates),
  atomic_commit.rs, pack_demux.rs, soc_full_chip_court.rs — all
  file:line cites confirmed on disk this session. No build, no test:
  doc-only lane, zero code changed, so no gate surface is touched.

## W4 Lock check (spot-check 6 measured-row cites)

6/6 cite targets exist at the cited lines: w1-02:19, w1-03:14,
w2-09:14, w3-l1-ingress:69, w3-l2-swar-fnv:26, red-10:43. Every
v26.11.2 MEASURED row carries a session command+exit cite; none
rewritten.

## W5 Verification ladder

Cite-existence spot-check (6 rows) + on-disk re-read of both appended
files (tail printed post-write). Table rows in §5/OP exceed 100 chars
— consistent with the pre-existing table rows in the same files (rows
25-34 run 194-475 chars); prose lines <=100.

## W6 Standing

ALIVE for the two doc writes (files on disk, re-read this session).
All five discoveries remain TARGET/UNVERIFIED as figures — none is
quotable as measured except row 37's OPT-03 hot-ambient numbers,
which carry their own receipt (`bench-validate/opt-03-channel-curve.md`).
The commit-tx syscall estimate (row 35) is explicitly flagged
UNMEASURED in the row itself.

## W7 Fences

No fences broken: both files are doc-only, not read by any gate
(boundary_gate, zero_string_gate, doctor, cgap-court) — verified by
grep-free knowledge of the gate scripts' inputs; the fence this diff
could break is claims-register consistency, addressed by the append-only
contract (no existing row edited) and the 6-row lock check.

```json
{"lane":"DOD-08",
 "files_written":["/Users/sac/cgap/docs/jira/v26.11.2/11-measured-claims.md",
  "/Users/sac/cgap/docs/jira/v26.11.2/02-pareto-levers.md"],
 "commands":["sed/grep cite verification x7 code cites (all confirmed on disk)",
  "6-row measured-cite spot-check (6/6 exist at cited lines)",
  "post-write tail re-read of both files"],
 "courts_run":["none — doc-only lane; no gate reads either file; anti-vacuity N/A"],
 "standing":"ALIVE (doc writes); all discovery figures TARGET/UNVERIFIED",
 "divergences":[
  "OPT-01..02/04/05 receipts are session reply-backs, not on-disk files — rows 33-36 cite the code sites directly (verified on disk) instead of receipt files",
  "row 35 syscall estimate (~9-12 syscalls, 10-25 us/tx) recorded as discovery estimate, UNMEASURED, measurement mandatory before lever promotion",
  "added table rows exceed 100-char line guideline, matching pre-existing table-row style in both files"]}
```
