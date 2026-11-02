# Lane DFX-6 — AX DMADV Rollup (consume-only)

Date: 2026-11-02. Repo: /Users/sac/cgap (main, canonical checkout).
No git commands. Owned files ONLY:

- `docs/jira/v26.11.2/dflss/11-ax-dmadv-rollup.md` (new)
- this receipt

## W1 Commands + exits (this session)

| # | Command | Exit | Result |
|---|---|---|---|
| 1 | `ls docs/jira/v26.11.2/{dflss,audit,lanes}/` | 0 |
| 2 | `ls lanes/lane-ipc-02-alloc-purity.md` | 0 |
| 3 | grep release-blockers.md for ax-/interop/ipc/suspend | 1 |
| 3b | grep audit-ledger.md for ax-/ipc | 1 |
| 4 | `awk 'length > 100'` over both owned files | 0 |
| 5 | `shasum -a 256` both owned files | 0 |
  ^ all DFX docs + 118 receipts on disk (result of row 1)
  ^ DFX-1 divergence 1 RESOLVED: receipt landed post-DFX-1/5 (row 2)
  ^ ZERO rows: AX items not registered; gap disclosed, not re-registered (row 3)
  ^ one substring false positive, 'max-consumer'; no real AX/ipc rows (row 3b)
  ^ zero over-length lines (row 4)
  ^ on-disk bytes verified post-write (row 5)

## W2 Content (point-at-rows law honored)

11-ax-dmadv-rollup.md (~170 lines): Define (02/07 cites), Measure
(gate probes AX-0 W2, two published delta tables AX-5/AX-6, alloc
columns IPC-2 + AX-4, resolved ipc-02 divergence), Analyze (PFMEA top
RPNs + 5 witnessed root causes incl. the resolved ipc-02 incident),
Design (13-row court inventory: darwin vs container, each with flip
witness), Verify (DFX-5 gate matrix deltas + honest OPEN list +
per-CTQ verdict table + falsifier applied clause-by-clause -> NOT
PASSED on clause 1: flips witnessed in stock images, not the
cgap-ax-testbed image; suspend/resume loudly UNVERIFIED; zero
cluster-apply claims). Per-CTQ tally: 2 VERIFIED (CTQ-05, CTQ-04 kernel layer),
3 DEFECT-OPEN (CTQ-03 hot-path 1.0 allocs/op honest FAIL, CTQ-04
membrane >18x off 10 ns, CTQ-06 container matrix exit 3).

## W3 Standing

- ALIVE: both owned files written this session, bytes verified
  (sha256), line-length law holds; every cited receipt re-read from
  disk this session before citing.
- One errata-by-re-derivation recorded: DFX-1's divergence 1
  (lane-ipc-02 receipt absent, `02-ax-charter-ctq.md:168-177`) is
  RESOLVED — `lanes/lane-ipc-02-alloc-purity.md` exists on disk now;
  the original receipt text is left untouched per errata law.
- Fences: none touched (two new markdown docs; no code, no gates, no
  shared files). Compile-freeze SLA not engaged.
- Shared tree untouched outside the two owned files; no git commands.

## W4 Divergences

1. Dispatch named "the PENDING linux/arm64 columns AX-4 left" — in
   fact AX-4's receipt LANDED with measured linux/arm64 alloc columns;
   the genuinely PENDING columns are the AX-7 container matrix (exit 3)
   and AX-8 (no receipt). Recorded as such in the rollup.
2. DFX-5's gate-matrix "PENDING per AX-4/AX-7/AX-8" cells are
   partially superseded by AX-4/5/6's landed receipts; the rollup
   points at the deltas instead of restating DFX-5.
