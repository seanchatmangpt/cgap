# 09 — AX-containerized-court Gage R&R / MSA (DFSS Measure phase)

Campaign v26.11.2 starDF. Companion: `05-cpk-analysis.md` (capability
arithmetic), `../15-ctq-tree.md` (CTQ flowdown), `06-control-plan.md`.
Re-derivation from landed receipts only; zero source edits by this lane.

## 1. The gauge defines the number (measurement-system law)

Timer resolution on darwin is witnessed at **41 ns minimum positive
monotonic delta** (paired `time.Now`, 100k iterations;
`../lanes/lane-ipc-03-latency-court.md:43`). Literal per-frame timing of
a ~10 ns frame admit cannot resolve against a 41 ns clock: the reported
10 ns figure would be smaller than the clock's own granularity. Therefore
the lawful gauge is **batch-amortized timing** — time a whole batch once,
divide — and every per-frame nanosecond number in this campaign is a
batch-amortized gauge reading, disclosed as such wherever it appears.

Witnessed gauges:
- Membrane floor: 85 ns/call via 200k null-refusal proxy calls
  (`lane-ipc-03-latency-court.md:44`) — itself an amortized read, since no
  empty-path C symbol exists (11-symbol table, sovereign.rs:413).
- Batch-1365 amortized: 179-181 ns/frame, 5 runs
  (`lane-ipc-03-latency-court.md:46`).
- CTQ-02 ingress gauge: BATCH=64 amortized `Instant`
  (`05-cpk-analysis.md:63`, citing `../waves/w3-l1-ingress.md:63,69-70`).

## 2. Gage R&R verdict for the containerized court (ASPIRATIONAL-UNVERIFIED)

The N=30 Cpk study run INSIDE the AX container court is PENDING. No
containerized capability run exists on disk; IPC-7 has no receipt
(`../lanes/lane-ipc-08-index.md:13,27`: IPC-0..IPC-7 all PENDING), and the
cluster path is NO-GO this session — `kubectl get svc api -n ate-system`
exit 1, connection refused (`../lanes/lane-ax-00-gate.md:17`), with the
GO/NO-GO table binding AX-2 to LOCAL-ONLY=DRAFT-UNRUN and AX-9 to
UNVERIFIED-by-design (`lane-ax-00-gate.md:44-48,51-53`).

Exact missing prerequisite (either unblocks):
1. **Cluster GO** — a reachable ate-system cluster flips the AX-2/AX-9
   NO-GO rows, enabling a real in-cluster N=30 run; or
2. **Local docker re-measurement** — docker 28.0.4 CLI + daemon are ALIVE
   (`lane-ax-00-gate.md:19-22,44`), so the IPC-3 linux port of the
   latency court, once AX-1/AX-3 image builds land, can exec the same
   gauges via `docker run` (AX-4..8 rows: GO via LOCAL exec mode).

Until one of those lands, any in-container Cpk figure is hearsay. The
gauge DESIGN above (batch-amortized, disclosed) is the only landed
content of this section.

## 3. Cpk story on the darwin host — both receipts stand, neither rounds up

| run | condition | CTQ-06 Cpl | CTQ-04 Cpu | receipt |
|---|---|---|---|---|
| G09 rig | quiet-window, sigma 0.0097 | **28.97** PASS | 4.176 PASS | lane-g09 (runs 2-3) |
| DF-4 re-run | wave load, sigma 0.276 | **0.694** FAIL | 0.408 FAIL | lane-dflss-04-cpk W4 |

Host-load variance is a MEASUREMENT property of this MSA: the same rig,
same leg code, same seeds produced Cpl 28.97 quiet vs 0.694 loaded. Both
receipts are honest ALIVE records; neither figure rounds up to the other.
The control implication lives in `06-control-plan.md`: Cpk gates on this
host require a quiet-window condition pin, or the gate measures the wave,
not the process.

## 4. REFUTED causal story (errata by re-derivation)

The draft-era claim "Cpk 0.855 -> 1.57 via optimization" is REFUSED as a
causal story:
- 12.4 ns / 6.00 ns are the CSCB C2 p99 ladder numbers
  (`../lanes/lane-b01-cscb-c1-c2.md:19`), a different metric — the draft
  conflated two surfaces.
- The G09 root cause stands: within-sigma Cpk 2.131 vs overall-sigma
  0.855 in repair-13 (`../repairs/repair-13-cpk-study.md:36,44`) is
  load-induced BETWEEN-SAMPLE drift on a contended M3 Max during the
  15-lane wave (`lane-g09-cpk-stabilization.md`, Drift audit). The
  effective repair was the n=30 sliding-window sampling methodology
  (WINDOW=30, exit floor 1.33, CPK_INJECT_FAIL anti-vacuity), not
  codegen.
- The intermediate re-run claim (Cpk 1.572 / CTQ-04 4.947) has NO receipt
  on disk (`lane-g09-cpk-stabilization.md`, standing section) — it stays
  UNVERIFIED and is not used as evidence here either.

## 5. Standing

- ALIVE (this session): all receipt citations re-read fresh from disk;
  cited line numbers verified present this session.
- ASPIRATIONAL-UNVERIFIED: in-container N=30 Cpk study (Sec 2), pending
  cluster GO or the IPC-3 linux port via local docker.
- No commands were run against containers this lane (docs-only lane); no
  container claim is made.
