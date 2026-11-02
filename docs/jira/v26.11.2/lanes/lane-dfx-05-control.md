# Lane DFX-5 — AX Control Plan Receipt

Date: 2026-11-02. Repo: /Users/sac/cgap (main). Lane DFX-5. No git commands.
Owned files: docs/jira/v26.11.2/dflss/10-ax-control-plan.md +
docs/jira/v26.11.2/lanes/lane-dfx-05-control.md (this receipt). Only these.

## W1 Commands + exits (this session)

| # | Command | Exit |
|---|---|---|
| 1 | `python3 scripts/doctor_agent_lanes.py` (AX-GATE-01 local analog) | 1 — WARN-class: orphan-leases 12 dirs / 9376.5 MB; dirty-tree 266 paths; stale-server PASS; workspace-count PASS members=23. Zero RED |
| 2 | `ls docs/jira/v26.11.2/dflss/` (census) | 0 — 02/07/08/09 present |
| 3 | `ls docs/jira/v26.11.2/lanes/` (receipt census) | 0 — lane-ipc-02.md ABSENT (divergence below) |
| 4 | `cat` lane-ipc-03/04/06 + lane-ax-00-gate + lane-ax-05-latency | 0 — all read this session, all citations re-read from disk |
| 5 | grep lanes/lane-ipc-08-index.md (PENDING cells) | 0 |

## W2 AX-GATE rows -> witnesses

- 01: doctor analog exit 1 (WARN informational, never rounded to pass);
  cluster path UNVERIFIED (AX-0 exits 127/1).
- 02: alloc purity — lane-ipc-02.md MISSING on disk; grounded on
  alloc_court_test.go presence + IPC-4's 0 B/op (0 allocs, AllocsPerRun)
  and CGO_ENABLED=0 exit 0. Container run PENDING per AX-4 (no receipt).
- 03: 85 ns/call floor + 179-192 ns/frame darwin witnessed (IPC-3 W4, AX-5
  delta table); 10 ns USL NOT met (>18x). OCAP = re-scope the SLA to the
  measured number or mark 10 ns kernel-layer-only (MV-5 decided-gate
  precedent). Batch tuning REFUSED as a control action.
- 04: darwin 4/4 flips witnessed (IPC-3 W6, IPC-4 W3 both-direction
  magic flip); container re-witness PENDING per AX-7.
- 05: SL-001..003 flips witnessed darwin (IPC-6 exits 1/0); container
  parity PENDING per AX-8; SL-004 where-it-ran field NET-NEW, unimplemented.

## W3 Verification ladder / anti-vacuity of THIS doc

The plan is doc-only; its own anti-vacuity = every PENDING/UNVERIFIED cell
names its upgrade condition (a landed receipt postdating this plan). A
vacuous plan would mark container cells anything-but-PENDING. Control rows
cite receipts re-read from disk this session; no session-memory counts.

## W4 Fences (must-not-regress surfaces)

Doc-only diff: no code, no gates, no lib.rs/Cargo.toml. Doctor script is
NOT edited (only executed). Fences that could regress: none — but the
doctor analog run (exit 1) is the current recorded state; a future run
returning exit 2 (RED) invalidates AX-GATE-01's witness and requires a
fresh row update.

## W5 Divergences

1. lane-ipc-02.md absent though dispatch named it. Not fabricated; logged
   in the OUT-OF-CONTROL log + matrix. IPC-3's W5.1 disclosure (the blake3.go
   cast unblock, disclosed in BOTH receipts per the law) is the only
   on-disk trace of IPC-2's work.
2. Dispatch said "run it, record exit" for the doctor — ran, exit 1, WARN
   informational (per DX-1/DXD-02 exit ABI: 0 GREEN/1 WARN/2 RED). Not a
   failure of the gate; recorded as-is, never rounded up.
3. ax ssh -- doctor row stays UNVERIFIED (ax absent, cluster NO-GO) —
   no simulated pass.

## W6 Standing

- AX-GATE-01 local analog: ALIVE (command+exit this session)
- AX-GATE-02..05 darwin arms: ALIVE via landed receipts (cited, re-read
  from disk this session)
- Container cells (02/04/05): PENDING per AX-4/AX-7/AX-8
- SLA (03): DECISION-PENDING with operator; tuning batch = REFUSED action
