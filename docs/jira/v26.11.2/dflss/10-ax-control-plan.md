# 10 — AX Control Plan (DFX-5, DMADV Control Phase)

Date: 2026-11-02. Repo: /Users/sac/cgap (main, canonical checkout).
Lane DFX-5. Owned files: this file + docs/jira/v26.11.2/lanes/lane-dfx-05-control.md.
Authority: lane-ax-00-gate.md (the verdict), dxd-02-doctor-design.md,
lane-ipc-02/03/04/06 receipts, lane-ax-05-latency.md delta table.
Prior DFX docs: 02 (charter), 07 (SIPOC/C4), 08 (PFMEA), 09 (c pk/MSA) — all on disk this session.

## Scope law (per AX-0, restated so no row can silently upgrade)

- CLUSTER PATH: NO-GO (lane-ax-00-gate.md W4: `kubectl get svc api -n
  ate-system` exit 1, connection refused). No row may claim a cluster
  witness.
- LOCAL PATH: GO (docker 28.0.4, daemon ALIVE via `open -a Docker` + bounded
  poll, exit 0). Container rows are docker-exec only.
- `ax` CLI: ABSENT (exit 127), install NOT APPROVED (docs grep zero hits).
  Every `ax ssh` / `ax apply` claim is hearsay-by-absence.

## AX-GATE matrix

Frequency legend: P = per-run, B = per-batch, W = weekly, C = per-change.

| Gate | What it controls | Frequency | Current real state (cited) | Exit witness (THIS session) | OCAP |
|---|---|---|---|---|---|
| AX-GATE-01 | toolchain parity (ax ssh -- doctor) | C | UNVERIFIED — no cluster, no ax CLI (AX-0 W2 #1,#3: exit 127 / exit 1). Local analog substituted: DX-1's doctor_agent_lanes.py | RUN: `python3 scripts/doctor_agent_lanes.py` -> exit **1** (WARN-class: orphan-leases 12 dirs / 9376.5 MB; dirty-tree 266 paths, expected mid-wave). Zero RED. | re-run analog until cluster path opens; doctor gate on the merged surface per dxd-02 |
| AX-GATE-02 | alloc purity (darwin ipc-02 numbers) | P | **REWRITTEN RW-11: lane-ipc-02-alloc-purity.md has LANDED** (ALIVE, commands+exits ipc-02:7-17). Darwin cgo membrane steady-state admit+eval = **1.0 allocs/op, 24 B/op** (View mint escapes; ipc-02:53-64) — the 0 B/op target is NOT met at the membrane; components (null-view admit, eval/Pinner, version) = **0.0** each (ipc-02:65-67). IPC-4's mmap zero-copy READ (no cgo) stays **0 B/op / 0 allocs** (ipc-04 W4). Container run: **PENDING** per AX-4 (no AX-4 receipt on disk) | darwin cgo membrane witness ALIVE (IPC-2 W1 exits 0; 10 PASS/1 loud SKIP/0 FAIL); container UNVERIFIED | container alloc court must re-witness through the linux dylib; the darwin budget of record is now 24 B/op steady, NOT 0 — any membrane alloc claim must match it or disclose |
| AX-GATE-03 | amortized SLA (ns/frame) | P | Witnessed darwin: **85 ns/call floor** (IPC-3 W4) / **179-187 ns/frame** amortized (IPC-3 5-run 179-181, 7-run 183/185/187; AX-5 delta table darwin column this-session 176/178/182). Container: 154 ns/call floor, 239-448 ns/frame (AX-5 delta table). The **10 ns USL is NOT met** at the membrane — off by >18x at the floor alone; RE-SCOPED per the landed records (MV-5), kept on the books, never silently dropped. Kernel-layer context: the campaign's only receipted ~10 ns/frame quiet-host figures are kernel-layer (L1 ingress, 11-measured-claims row 5: 10.20 fastest / 10.30 median, reproduced 10.26-10.65 in-tolerance by R-15, `../redteam/fix-18-measured-claims-sweep.md:22-37`) — a DIFFERENT counter from the membrane ns/frame; unit law 06-bench-protocol §7 | Honest OCAP per the MV-5 decided-gate precedent: **re-scope the SLA to the measured number (85/179-187 ns darwin, 154/239-448 ns container) or mark 10 ns kernel-layer-only** — NEVER tune the batch to chase the USL | batch tuning to hit 10 ns is a REFUSED control action; re-scope is the only lawful move |
| AX-GATE-04 | anti-vacuity (courts must flip) | P | darwin 4/4 POSH-5 flips witnessed (IPC-3 W6 magic/digest mutation; IPC-4 W3 TestMagicCorruptionFlip both directions). Container re-witness: **PENDING** per AX-7 (no AX-7 receipt on disk) | darwin witnesses cited from landed receipts; container layer UNVERIFIED | a green court run without a witnessed flip is VACUOUS, exit nonzero |
| AX-GATE-05 | receipt sealing (SL-001..003) | P | IPC-6 flips witnessed darwin (bad fixture -> exit 1, REFUSED SL-001/002/003, no file; good fixture -> exit 0, 939 B). Container parity: **PENDING** per AX-8. SL-004 (`where-it-ran` field) is NET-NEW — no implementation on disk | darwin sealer ALIVE (IPC-6 courts table, exits 1/0) | SL-004 lands only with its own flip: sealing a receipt whose environment is misstated must REFUSE, else VACUOUS |

## OUT-OF-CONTROL log (the AX-0 divergence record)

| Event | Witness | Resolution |
|---|---|---|
| docker daemon down at first probe | AX-0 W2 #6, exit 1 | `open -a Docker` exit 0 + bounded poll `docker info` 60x2s -> up after 1 poll (AX-0 W2 #8) — the sanctioned no-sleep pattern |
| cluster NO-GO | AX-0 W2 #3, exit 1 conn refused (12 contexts, none reachable) | fleet re-scope to LOCAL-ONLY docker-exec path; cluster rows never simulated |
| podman absent | AX-0 W2 #4, exit 127 | docker 28.0.4 substituted; podman install not requested |
| ax CLI absent, install unapproved | AX-0 W2 #1 exit 127 + W3 (docs grep zero hits) | lanes must not self-approve; `ax` surface stays UNVERIFIED |
| AX-9 UNVERIFIED-by-design | AX-0 W4 row AX-9 | no simulated pass permitted; suspend/resume stays open until cluster + ax exist |
| lane-ipc-02 receipt was absent at plan time; LANDED since — SUPERSEDED (RW-11 re-read: lane-ipc-02-alloc-purity.md on disk this session) | original witness: `ls` showed only ipc-00/01/03/04/06/08 | AX-GATE-02 now cites the LANDED receipt (24 B/op steady darwin membrane tax, components 0.0); the absence window's record survives in IPC-3's W5.1 disclosure |

## Reaction plan (out-of-control -> action)

1. Any row flipping to RED (doctor exit 2, court flip FAIL, sealer exit 1
   on a good fixture) -> halt the affected gate family, repair, re-witness
   the flip before re-admission.
2. Container waves AX-4/AX-7/AX-8 landing -> each PENDING cell in the
   matrix upgrades ONLY on a receipt on disk with command+exit; receipt
   mtime must postdate this plan (stale claims are hearsay).
3. SLA re-scope decision (AX-GATE-03) is an operator decision, not a lane
   decision — recorded here as the only two lawful options.
