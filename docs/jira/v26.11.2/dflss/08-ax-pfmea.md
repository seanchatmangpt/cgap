# v26.11.2 DFSS — AX-Fleet PFMEA (08)

Phase: Analyze (DFSS lane DFX-3). Supersedes the AX-relevant rows of
`04-pfmea-risk-matrix.md` and extends them with the interop/AX failure
classes earned this session (2026-11-02). RPN = SEV x OCC x DET,
1-10 scales. DET columns are scored from ACTUAL witnessed evidence;
a control with no witnessed flip is DET scored HIGH, never low
aspirationally. Charter-level FMEA retained at
`docs/jira/v26.11.2/14-dflss-charter.md:94-105` (cross-link, no
duplication).

## Detection-evidence citations (witnessed, on disk)

| evidence | path:lines | what it witnesses |
|---|---|---|
| AUD-01 stale-binary catch | audit/dod-lsp-max-wire.md:139-142 | release binary G05 cited was stale vs post-12:53 source edits; caught by post-hoc audit, rebuilt fresh |
| G05 wire rebuild | lanes/lane-g05-lsp-wire-rebuild.md:11-16 | fresh rebuild + literal-count witness (ANDON_REFUSED count=3 both profiles) |
| AX-0 gate probes | lanes/lane-ax-00-gate.md:15-26 | docker daemon down exit 1 -> `open -a Docker` exit 0 -> bounded poll exit 0; `ax` absent exit 127; cluster exit 1 conn-refused |
| AX-0 vendor marking | lanes/lane-ax-00-gate.md:71-76 | ax CLI BLOCKED (absent, install unapproved); AX API surface UNVERIFIED (vendor-README material) |
| GAP-05 DOC-RECEIPT class | lanes/lane-gap-05-receipt-lint.md:48-50,62-76 | DOC-RECEIPT-002/003 STRICT-FAIL exit 1 witnessed; counts 002=81 / 003=145 findings |
| DXD-03 cgo drift | audit/dxd-03-go-ergonomics.md:12-16,61-64 | two divergent /tmp cgo -L/-rpath target dirs (ipc2 vs ipc3); absolute paths bake /tmp + /Users/sac |

## Matrix (ranked by RPN, desc)

### AX-fleet rows (new this session)

| # | Risk | SEV | OCC | DET | RPN | Detection control (witness status) | Rank |
|---|------|-----|-----|-----|-----|-----------------------------------|------|
| A1 | Stale-binary false-pass (darwin binary predates source; container analog = frozen .so in image) | 9 | 4 | 7 | 252 | darwin WITNESSED post-hoc: AUD-01 caught G05's stale release binary (audit/dod-lsp-max-wire.md:139-142), rebuilt per G05 (:11-16). Detection is manual audit, no mechanical gate. Container analog: AX-3 symbol-count court PENDING (no lane-ax-03 receipt on disk, checked this session) — that channel ungated. DET=7 scored for CURRENT state. | 1 |
| A2 | Vendor-claim drift (AX README mechanics asserted as fact) | 7 | 4 | 8 | 224 | Detection = AX-10's VENDOR-CLAIM-UNVERIFIED marking. Currently MANUAL: grep of docs/jira/v26.11.2 found ZERO occurrences of the token; the only landed marking is AX-0's prose UNVERIFIED verdict (lane-ax-00-gate.md:73-76). No mechanical rule. DET=8 honest. | 2= |
| A3 | Dual-target-dir drift (two divergent /tmp cgo -L/-rpath dirs; wrong dylib linked, court passes against stale artifact) | 8 | 4 | 7 | 224 | WITNESSED occurrence: dxd-03 found /tmp/ipc2-target vs /tmp/ipc3-target with divergent -L/-rpath and absolute paths baking /tmp + /Users/sac (audit/dxd-03-go-ergonomics.md:12-16,61-64). Prevention only: AX-3's fixed /opt/cgap/lib path (design, not yet built). Container court PENDING. DET=7 honest. | 2= |
| A4 | Cluster-apply hearsay (doc cites `ax apply`/k8s apply with no exit code) | 8 | 5 | 5 | 200 | Partial-mechanical: GAP-05 DOC-RECEIPT-002 (cited lanes/waves path does not resolve -> STRICT-FAIL exit 1, witnessed lane-gap-05-receipt-lint.md:49,66) + DOC-RECEIPT-003 (status-token/as-of header, :50,68) catch unbacked docs AFTER landing; they do not verify that an exit code exists behind a resolvable path. AX-0's falsifier (lane-ax-00-gate.md:60-62) defines the class; OCC witnessed across the IPC fleet. Residual detection manual. DET=5. | 4 |
| A5 | docker-daemon-down at gate time | 5 | 4 | 3 | 60 | LANDED, WITNESSED: AX-0 probe sequence daemon-down exit 1 -> `open -a Docker` exit 0 -> bounded poll on `docker info` exit 0 (lane-ax-00-gate.md:20-22). No Process.sleep; observable poll with stated deadline. Detection works; DET=3 for a loud, fast, typed failure. | 5 |

### Carried rows from 04 (kept ONLY where detection witness exists)

| # | Risk | SEV | OCC | DET | RPN | Detection control (witnessed flip) | Rank |
|---|------|-----|-----|-----|-----|-----------------------------------|------|
| C1 (=04 R7) | Agent long-file write corruption | 7 | 6 | 2 | 84 | DX-2 PostToolUse hook landed, court battery incl. fire->clean byte-restore flips (lane-dx-02-write-integrity.md:7-10,26-43) | 6 |
| C2 (=04 R9) | Receipt drift in landed docs | 5 | 7 | 2 | 70 | receipt_lint --strict exit 1 (444 findings); summary-path KeyError crash noted (04-pfmea-risk-matrix.md:18,33) — findings list prints fully before crash | 7 |
| C3 (=04 R1) | Forbidden branch in branchless kernel | 8 | 3 | 2 | 48 | disassembly-invariants court, fresh exit 0 incl. arm E binary mutant (04-pfmea-risk-matrix.md:15,25) | 8 |
| C4 (=04 R3) | SRAM/.text envelope overflow | 7 | 3 | 2 | 42 | sram-bloat mutant measured 136,068 B > bound — output flips, process exit 0 report-only (04:17,27) | 9 |
| C5 (=04 R2) | Heap alloc in no_std kernel path | 8 | 2 | 2 | 32 | CountingAlloc breach court, CSCB_MUTATE=alloc -> exit 1 (lane-b03-cscb-c4.md:18-20) | 10 |

### Dropped from 04 (detection witness ABSENT — stay in 04, not carried)

| 04 row | why not carried |
|---|---|
| R6 shared-fixture race (RPN 336) | 04 scored DET=7 for the then-missing DX-6 court; lane-dx-06-fixture-races.md NOW EXISTS on disk (landed after 04) — its witness must be re-read and re-scored by its owner before this matrix can carry it. Dropped rather than carried UNVERIFIED. |
| R8 stale-binary (RPN 288) | superseded by A1 (same failure class, AX-fleet scope, darwin witness now on file). |
| R10 429 lane death (RPN 180) | detection is manual receipt-absence inventory; no mechanical witness — honest DET=6, stays in 04. |

## Top-3 flagged for DFX-6 control plan

1. **A1 stale-binary false-pass — RPN 252.** Control: land the AX-3
   symbol-count container court (frozen-.so analog of the darwin catch);
   re-score DET 7->2 once a lane-ax-03 receipt with a witnessed flip
   exists. Darwin side: promote AUD-01's post-hoc audit into a mechanical
   mtime-freshness gate.
2. **A2/A3 tie at RPN 224.** A2 vendor-claim drift: make VENDOR-CLAIM-UNVERIFIED
   a receipt_lint rule (mechanical), re-score DET 8->4. A3 dual-target-dir:
   land AX-3's fixed /opt/cgap/lib as the sole cgo -L/-rpath root + a court
   asserting one resolved dylib path; re-score DET 7->3.
3. **A4 cluster-apply hearsay — RPN 200.** Control: extend receipt_lint with
   an exit-code-existence rule for apply/apply-adjacent citations
   (complements DOC-RECEIPT-002/003); re-score DET 5->3.

A5 (60) needs no DFX-6 control — its detection is landed and witnessed.

## Anti-vacuity of this matrix

Mutation named: delete any DET-evidence citation and the corresponding
RPN rises and the ranking changes — e.g. A1 without the AUD-01 cite
falls back to 04-R8's ungated DET=8 -> RPN 288 and A5 without the AX-0
poll witness rises to DET=8 -> RPN 160, reshuffling ranks 4-5. The
ranking is falsifiable by receipt absence. Each honest-high DET (A1
container channel, A2, A3 residual) is scored for the CURRENT on-disk
state, checked this session; PENDING courts are named, never assumed run.

## Divergences (doc-yields-to-reality)

- VENDOR-CLAIM-UNVERIFIED as a mechanical rule does not exist on disk;
  only AX-0's prose marking lands. Dispatch phrasing "AX-10's
  VENDOR-CLAIM-UNVERIFIED marking" is treated as planned, not landed.
- No lane-ax-03 (symbol-count court) receipt exists; both A1 and A3
  container channels are PENDING and scored DET-high honestly.
- 04-R6's dispatch-named missing receipt has since landed
  (lane-dx-06-fixture-races.md); not re-scored here (not this lane's
  evidence to re-derive) — flagged for the 04 owner.
- receipt_lint strict summary crash (04:18) remains open; GAP-05's
  landed rule class (lane-gap-05-receipt-lint.md) is the later state
  and is what this matrix cites.
