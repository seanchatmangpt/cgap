# 09 — AX-containerized-court Gage R&R / MSA (DFSS Measure phase)

Campaign v26.11.2 starDF. Companions: `05-cpk-analysis.md` (capability
arithmetic), `03-msa-gauge.md` (gauge law + CONTENTION-SUSPECT
consumption rule), `../15-ctq-tree.md` (CTQ flowdown),
`06-control-plan.md` (SPC row §5.2), `../11-measured-claims.md`
(exclusive numeric source). Re-derivation from landed receipts only;
zero source edits. Claim-rewrite pass RW-08, 2026-11-02: merged into
the landed text (Sec 1/2/4 structure kept), Sec 3 re-grounded to
receipted medians/sigmas + scopes; nothing clobbered silently.

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
  (`lane-ipc-03-latency-court.md:46`); independent-runs table
  min 183 / median 185 / max 187 ns/frame over 7 runs
  (`lane-ipc-03-latency-court.md:47`; max-of-N is not a quantile,
  FIX-04 law).
- CTQ-02 ingress gauge: BATCH=64 amortized `Instant`
  (`05-cpk-analysis.md:63`, citing `../waves/w3-l1-ingress.md:63,69-70`).

**Batch-amortization limit (ADV-09-R1, `03-msa-gauge.md` §1.3):**
batch-amortized timing resolves batch MEANS only. Within a timing
window the per-frame TAIL is structurally invisible: row 28's
"slowest 124.06 ns/frame" (`../11-measured-claims.md:52`) is the
slowest whole 1,048,576-frame SAMPLE amortized — a within-window
frame-level worst case could be far worse and this gauge cannot see
it. Wherever a sub-15 ns capability number appears below (the CTQ-02
USL of 15 ns/frame against a ~10 ns measured gauge), the Cpk /
capability arithmetic computed on batched means is a **BATCH-MEAN
Cpk** — it is a capability of the batched-mean gauge, not of the
per-frame process, and must never be read or quoted as per-frame
capability. The same limit attaches to "slowest"-class maxes exactly
as it does to means.

## 2. Gage R&R verdict for the containerized court (ASPIRATIONAL-UNVERIFIED)

The N=30 Cpk study run INSIDE the AX container court is PENDING. No
containerized capability run exists on disk; IPC-7 had no receipt at
the index's receipt time (`../lanes/lane-ipc-08-index.md:13,27`:
IPC-0..IPC-7 all PENDING as-of that receipt), and the cluster path is
NO-GO this session — `kubectl get svc api -n ate-system` exit 1,
connection refused (`../lanes/lane-ax-00-gate.md:17`), with the
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
gauge DESIGN above (batch-amortized, disclosed, tail-limited) is the
only landed content of this section.

## 3. Cpk story on the darwin host — receipted numbers, scoped, neither rounds up

Row conditions, spelled out:

- (a) G09: idle P-cores, ambient load ~10, n=30 windows, sigma 0.0097,
  mean 1.84 GiB/s (`lane-g09-cpk-stabilization.md:13,25`).
- (b) DF-4: concurrent wave load, sigma 0.276, mean 1.574 GiB/s
  (`lane-dflss-04-cpk.md:33,53`).

Scope law first: a capability number measured on a quiet host is NOT
valid unqualified (`../11-measured-claims.md:5-7`, row 5; receipt
`../redteam/red-10-timing-jitter-exploit.md`). Contention behavior is a
separate row — row 28's REGISTERED DIVERGENCE
(`../11-measured-claims.md:52`: 14.71 / 19.03 / 124.06
fastest/median/slowest @1M under combined cache+CPU load; slowest =
12.2x the quiet 10.20 claim). Every row below carries its own scope.

| run | condition (scope) | CTQ-06 Cpl | CTQ-04 Cpu | receipt |
|---|---|---|---|---|
| G09 rig | quiet window (a) | **28.97** PASS | 4.176 PASS | g09-cpk-stabilization.md:25,56 |
| DF-4 re-run | wave load (b) | **0.694** FAIL | 0.408 FAIL | dflss-04-cpk.md:32-34,53 |

These are CTQ-06 (BLAKE3 throughput, GiB/s, LSL 1.0) and CTQ-04
(variance) — byte/second counters, NOT sub-15 ns frame figures; the
batch-amortization limit of Sec 1 does not attach to these two gates.
It DOES attach to CTQ-02 (frame ingress, USL 15 ns/frame), whose
capability arithmetic below is a BATCH-MEAN Cpk (Sec 1 law).

As-of quiet-host gauge status (R-15 sweep): the CTQ-02 ingress number
is re-witnessed at fastest **10.26 / 10.42 / 10.65 ns/frame** across 3
quiet legs (guards green, CONTENTION-SUSPECT markers 0/0/0; median
11.99-12.17 ms @1M; `../redteam/fix-18-measured-claims-sweep.md` W2,
exits 0 x3). These quiet legs are the as-of numbers for any capability
statement against the 15 ns/frame USL; row 5's landing figures (10.20
fastest / 10.30 median, `../11-measured-claims.md:29`) are confirmed
within red-10's 4.5% reproduction tolerance (leg 3 at 10.26 sits 1.3%
below the 10.4 band floor, typed in the sweep, not silent). The
contention arm is NOT re-quietable into this table: R-15's re-witness
under 4x512MiB + 8 cpu fired the guard (2 markers) but measured
slowest 25.60 ns/frame — a typed divergence from row 28's magnitudes
(bw8 load), which stays governed by red-10's receipt
(`fix-18-measured-claims-sweep.md` W3).

Host-load variance is a MEASUREMENT property of this MSA: the same rig,
same leg code, same seeds produced Cpl 28.97 quiet vs 0.694 loaded
(sigma 0.0097 vs 0.276). Both receipts are honest ALIVE records;
neither figure rounds up to the other. The control implication lives in
`06-control-plan.md` §5.2: Cpk gates on this host require a
quiet-window condition pin (CONTENTION-SUSPECT consumption rule,
`03-msa-gauge.md` §1.2), or the gate measures the wave, not the
process.

## 4. REFUTED causal story (errata by re-derivation)

The draft-era claim "Cpk 0.855 -> 1.57 via optimization" is REFUSED as a
causal story:
- 12.4 ns / 6.00 ns are the CSCB C2 p99 ladder numbers
  (`../lanes/lane-b01-cscb-c1-c2.md:19`), a different metric — the draft
  conflated two surfaces.
- The G09 root cause stands: within-sigma Cpk 2.131 vs overall-sigma
  0.855 in repair-13 (`../repairs/repair-13-cpk-study.md:36,44`) is
  load-induced BETWEEN-SAMPLE drift on a contended M3 Max during the
  15-lane wave (`lane-g09-cpk-stabilization.md:7-19`, Drift audit). The
  effective repair was the n=30 sliding-window sampling methodology
  (WINDOW=30, exit floor 1.33, CPK_INJECT_FAIL anti-vacuity; flip
  witnessed at `lane-g09-cpk-stabilization.md:57`, Cpl -84.69 at
  injected LSL 10.0), not codegen.
- The intermediate re-run claim (Cpk 1.572 / CTQ-04 4.947) has NO receipt
  on disk (`lane-g09-cpk-stabilization.md:26-29`, standing section) — it
  stays UNVERIFIED and is not used as evidence here either.
- Nearest measured negative Cpk under the inject: -9.6525
  (`lane-dflss-04-cpk.md:19`; also `05-cpk-analysis.md:33-34`). The
  "-5.23" figure circulating in dossier prose is
  UNVERIFIED-WITHOUT-METHOD (`03-msa-gauge.md` §2.2) — never quote it
  as a capability figure.

## 5. Standing

- ALIVE (RW-08 pass, this session): every receipt citation in Secs
  1-4 re-read fresh from disk this session (ipc-03 :43-47, ipc-08
  :13,27, ax-00 :17,19-22,44-53, g09 :7-29,56-57, dflss-04 :19,32-34,53,
  b01 :18-19, repair-13 :34-44, w3-l1 :62-72, 05-cpk :60-68, sovereign.rs
  :408-418, 11-measured :29,:52, fix-18 sweep W2/W3).
- ASPIRATIONAL-UNVERIFIED: in-container N=30 Cpk study (Sec 2), pending
  cluster GO or the IPC-3 linux port via local docker.
- No commands were run against containers or benches this lane
  (docs-only lane); no bench or container claim is made.
- Fences this diff could break: none mechanical (docs-only). Nearest
  fence is the exclusivity law (`../11-measured-claims.md:7-8`) — no
  number appears here except receipt-cited figures re-read this
  session, R-15's sweep receipts, or explicit UNVERIFIED-WITHOUT-METHOD
  marks. Second fence: the do-not-promote rule on 23.35 ns/fact
  (`03-msa-gauge.md` §2.2) is respected — that figure does not appear
  here.
