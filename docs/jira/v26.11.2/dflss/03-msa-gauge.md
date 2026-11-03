# 03-msa-gauge (DFSS Measure phase — MSA / gauge law)

Campaign v26.11.2 starDF. Lane DF-02 (MEASURE/MSA), 2026-11-02.
Companions: `09-ax-cpk-msa.md` (AX-court Gage R&R), `05-cpk-analysis.md`
(capability arithmetic), `../06-bench-protocol.md` (bench method of
record), `../11-measured-claims.md` (exclusive numeric source).

## 0. Dispatch divergence (recorded, not silently absorbed)

The dispatch named "the 03-* measurement/gauge file" as existing in
`dflss/`. No `03-*` file exists there (dir listing 2026-11-02:
`00`..`17` plus `interop-cpk.md`, no 03). Per the missing-file clause
of the lane contract, this lane created `03-msa-gauge.md` fresh;
nothing was merged or clobbered. Two further anchor divergences found
on read:

1. The 1,365 coprime-stride / 64-record batch constants are NOT in
   `crates/bench/benches/chains-ingress.rs` (full read this session,
   698 lines: 512 B slot stride at :87, CHAIN_SEED at :54 — no 1,365,
   no 64-frame batch const). They live in
   `crates/cgap-git/src/pack_demux.rs` (Sec 1.1).
2. `06-bench-protocol.md` carries NO N=7 Min/Median/Max reporting
   section (full read this session, 187 lines). The N=7 law lives in
   the csct rig + lane receipts (Sec 2).

## 1. Timer-floor gauge law

Darwin monotonic timer resolution is witnessed at a **41 ns minimum
positive delta** (paired `time.Now`, 100k iterations;
`../lanes/lane-ipc-03-latency-court.md:43`). A ~10-23 ns per-frame
quantity is smaller than the clock's own granularity: literal
per-frame `Instant` timing measures the instrument, not the process.
The lawful gauge is therefore **batch-amortized timing** — time a
whole batch once, divide; disclose the gauge wherever a per-frame
number appears (`09-ax-cpk-msa.md:14-16`, same law).

### 1.1 Window constants (real on-disk locations)

- `FACT_WINDOW = 1_365` — `crates/cgap-git/src/pack_demux.rs:51`,
  provenance note :42-50. Court-driving coprime stride:
  1,365 = 3 x 5 x 7 x 13, coprime with the 12 B header / 20 B trailer
  / 64 B SHA-1 block / 4 B BE fields, so a fixed stride never aliases
  a structural field across windows.
- `RECORD_CAP = 64` — `crates/cgap-git/src/pack_demux.rs:54`
  ("Records buffered between drains"): the 64-record batching of the
  streaming demuxer.
- CTQ-02 gauge batch = 64 frames amortized over one `Instant` pair —
  `05-cpk-analysis.md` Sec 4, citing `../waves/w3-l1-ingress.md:63,69-70`
  (the 10.20/10.30 ns/frame ingress gauge).

None of these live in `chains-ingress.rs`; the batch-1365 bench arm is
a Go court (`BenchmarkAdmitBatch1365`,
`../lanes/lane-ipc-03-latency-court.md:46`), not a Rust bench target.

### 1.2 Gauge-law enforcement: CONTENTION-SUSPECT guard (ADV-09-R2 closure)

Quiet-host gauge law is enforced by the CONTENTION-SUSPECT guard:
`crates/bench/benches/chains-ingress.rs:716-802` (guard contract
comment :716-719, `CONTENTION_MARK` const :745, emission :802/:854;
mctq copy `mctq_consolidation.rs:31,65,98`). A run whose stderr carries
`CONTENTION-SUSPECT:` is not admissible against quiet-host rows
(`../11-measured-claims.md` rows 5/6/7/8); consumption rule =
`../06-bench-protocol.md` §8, SPC control row =
`06-control-plan.md` §5.2. HONEST SCOPE: enforcement is procedural
(receipt-lint / consumer discipline), NOT yet a machine gate; a
fail-closed stderr-grep gate is a TARGET until receipted. Guard flip
witness: `../lanes/lane-rc11-contention-guard.md:30-34` (flip margin
2.1-30x).

### 1.3 Batch-amortization tail disclosure (ADV-09-R1 closure)

The Sec 1 law covers batch MEANS only. Within a timing window the
per-frame TAIL is equally unresolvable: row 28's "slowest
124.06 ns/frame" is the slowest whole 1,048,576-frame SAMPLE amortized
— a within-window frame-level worst case could be far worse and is
structurally invisible to this gauge. Consumers must NOT read
row-28-class "slowest" figures as per-frame tail resolution; the
amortization disclosure attaches to maxes exactly as it does to means.

## 2. Min/Median/Max N=7 reporting law

`06-bench-protocol.md` (Divan method of record) carries no N=7
section — divergence recorded in Sec 0. The N=7 law is the **csct rig
law**: RUNS=7 min-of-N sampling (`05-cpk-analysis.md:85-86`;
`RUNS: usize = 7` at
`crates/csct/src/bin/cscb_c3_closure.rs:52`, fresh-read this
session; the budget/aspirational consts sit at `:40`
(`DECIDED_BUDGET_MS`) and `:42` (`ASPIRATIONAL_MS`)), with two
adjacent disciplines:

- **Quantile honesty**: with N=7, `ceil(0.99*7)=7 -> sorted[6]` — a
  "p99" of 7 runs is the MAX, and max-of-N is not a quantile
  (`../lanes/lane-fix04-cscb-p99.md:4-12`). Report min/median/max and
  say so; never label the max a p99.
- **Witnessed N=7 min/med/max rows**: batch 23.22/23.35/23.43
  ns/frame runs=7 (`../lanes/lane-posh-02-batch.md:29`);
  batch-1365 183/185/187 ns/frame over 7 independent runs
  (`../lanes/lane-ipc-03-latency-court.md:47`); C3 closure min
  36,918.401 ms N=7 (`../lanes/lane-mv-05-sla-reconcile.md:24`).

## 2.2 Cpk-table row verification (vs 11-measured-claims.md + lanes)

Per-row status, re-read from disk this session. Rows whose numbers
exist only in dossier prose carry **UNVERIFIED-WITHOUT-METHOD** per
the exclusivity law (`../11-measured-claims.md:7-8`: no number may
appear that is absent from the table).

- **batch ns/fact 23.35 median — MEASURED (typed gate FAIL).**
  `../lanes/lane-posh-02-batch.md:29-33` (exit 101; runs=7 min
  23.22 / med 23.35 / max 23.43; +4.3% mutation flip, all 7 runs
  separated). Present in 11-measured-claims.md? NO — registered here
  as a receipt-cited row, not a table row; do not promote to the
  paper without a W4-D3 addendum.
- **Cpk -5.23 — UNVERIFIED-WITHOUT-METHOD.** No receipt names it
  (grep over lanes/, dflss/, waves/, 11-measured-claims.md rc=0 with
  zero -5.23 hits this session). Nearest measured negative Cpk:
  -9.6525 under CPK_INJECT_FAIL (`05-cpk-analysis.md:33-34`). Do not
  quote -5.23 as a capability figure.
- **container-vs-host 4.57 / 192.0 — SPLIT: latencies MEASURED, the
  Cpk row UNVERIFIED-WITHOUT-METHOD.** 4.574-4.656 vs 4.52-4.54
  ns/op mmap read (`../lanes/lane-ax-06-shm.md:70`); container batch
  median 192 ns/frame (`../lanes/lane-ax-05-latency.md:71,86`). No
  receipt computes a Cpk from these pairs.
- **membrane latency 185 ns — MEASURED.** Single admit
  182.7-185.6 ns/call (`../lanes/lane-ipc-03-latency-court.md:45`);
  batch median 185 ns/frame over 7 independent runs (:47).
- **timer floor 41.0 ns — MEASURED.**
  `../lanes/lane-ipc-03-latency-court.md:43` (41 ns min positive
  monotonic delta).

### 2.3 Verdict row: gate re-scope (kept honest)

The re-scope is real and receipted: the 25.0 ms figure is the
`ASPIRATIONAL_MS` const at `crates/csct/src/bin/cscb_c3_closure.rs:42`
and is NOT a gate; the active gate is `DECIDED_BUDGET_MS = 60,000.0`
(`:40`; lane MV-5 re-scope, consts fresh-read recorded in
`05-cpk-analysis.md:81-93`). MV-5's decision run under the active
gate: min 36,918.401 ms <= 60,000 ms, exit 0 PASS, N=7
(`../lanes/lane-mv-05-sla-reconcile.md:24`), aspirational shortfall
printed, not hidden (`lane-mv-05-sla-reconcile.md:24`: "aspirational
1476x shortfall printed"). Any 25 ms-class closure capability stays
ASPIRATIONAL-UNVERIFIED (`05-cpk-analysis.md:89-94`); it never rounds
up to MEASURED.

### 2.4 Cross-layer note (kept — referent exists)

Kernel vs membrane, 0.0 vs 1.0 allocs/op view-mint: grounded in
`../lanes/lane-ipc-02-alloc-purity.md:47-48` (steady admit+eval 1.0
allocs/op pinned; view-mint admit 1.0; null-view admit / eval /
pinner / version 0.0 each) and :122 ("the honest steady-state
membrane tax is 1.0 allocs/op (view mint), components 0.0"),
allocation discipline per `../audit/cull-07-ipc-legacy.md:43,49`
(darwin IPC-2 canonical; AX-4's 0.0 is the container court arm, not a
darwin claim). The note stays; layer labels must not be swapped —
0.0 belongs to the component arms, 1.0 to steady admit+eval.

## 5. Standing

- ALIVE (citation-grounding pass): every cited path:line re-read on
  disk this session; every non-cited figure is marked
  UNVERIFIED-WITHOUT-METHOD, never rounded up.
- Fences this diff could break: none mechanical (docs-only); the
  nearest fence is the exclusivity law itself — no number landed here
  without a receipt read this session or an explicit U-W-M mark.

### 5.1 Per-row landed status (report terminus, tabular)

| Row | Status |
|---|---|
| Sec 0 divergences | LANDED (3, stated) |
| Sec 1 timer-floor law | LANDED (41 ns, ipc-03:43) |
| Sec 1.1 window constants | LANDED (pack_demux.rs:51,54; NOT in
  chains-ingress.rs — divergence stated) |
| Sec 2 N=7 law | LANDED (csct rig + fix04; 06-bench-protocol has
  no N=7 — divergence stated) |
| Sec 2.2 23.35 ns/fact | LANDED MEASURED (posh-02:29-33) |
| Sec 2.2 Cpk -5.23 | LANDED UNVERIFIED-WITHOUT-METHOD |
| Sec 2.2 4.57/192.0 | LANDED SPLIT (latencies MEASURED; Cpk U-W-M) |
| Sec 2.2 185 ns membrane | LANDED MEASURED (ipc-03:45,47) |
| Sec 2.2 41.0 ns floor | LANDED MEASURED (ipc-03:43) |
| Sec 2.3 gate re-scope | LANDED honest (25 ms aspirational, 60 s
  gate, MV-5 PASS N=7) |
| Sec 2.4 cross-layer note | LANDED KEPT (ipc-02:47-48,122 + cull-07:43,49) |

```json
{"lane":"DF-02",
 "files_written":["/Users/sac/cgap/docs/jira/v26.11.2/dflss/03-msa-gauge.md"],
 "commands":["ls dflss/ (exit 0)",
  "full Read chains-ingress.rs (698 lines) + grep sweep (exit 0)",
  "sed pack_demux.rs:40-60 (exit 0)",
  "Read 06-bench-protocol.md (187 lines), 11-measured-claims.md (89 lines)",
  "Read 05-cpk-analysis.md, 09-ax-cpk-msa.md (dflss)",
  "Read lane receipts ipc-02/03/posh-02/ax-05/ax-06/fix04/mv-05 (all exits 0)",
  "repo-wide grep for -5.23 / 4.57 / 192.0 / 41.0 / 23.35 / N=7 (exit 0)"],
 "courts_run":["docs-only lane; falsifier = citation grounding"],

 "standing":"ALIVE",
 "divergences":[
  "no 03-* file existed in dflss/ (dispatch asserted it existed)",
  "created 03-msa-gauge.md fresh; nothing merged or clobbered",
  "1,365 / 64 constants at pack_demux.rs:51,54, NOT chains-ingress.rs",
  "06-bench-protocol.md has no N=7 section; law cited from csct rig",
  "consts + lane-fix04-cscb-p99.md:4-12",
  "Cpk -5.23 and 4.57/192.0-as-Cpk: UNVERIFIED-WITHOUT-METHOD",
  "23.35 ns/fact receipted (posh-02) but absent from the",
  "11-measured-claims.md table; promotion needs W4-D3 addendum"]
}
```

## Errata (lane R-09, 2026-11-02 — citation fixes, merged not clobbered)

EVAL-02 defects fixed against this file's own DF-02 receipt (above):

1. Sec 2 anchor `cscb_c3_closure.rs:40-42` for the RUNS=7 law was
   wrong (those lines are DECIDED_BUDGET_MS / ASPIRATIONAL_MS).
   Corrected to `RUNS: usize = 7` at `cscb_c3_closure.rs:52`;
   DECIDED/ASPIRATIONAL kept at `:40`/`:42` as the budget cites.
   Verified by fresh Read of the bin this session.
2. Sec 2.3 printed-shortfall cite `lane-mv-05-sla-reconcile.md:12`
   was wrong (:12 is dispatch-divergence prose). Corrected to :24,
   which carries "aspirational 1476x shortfall printed" (fresh Read
   this session). The Sec 2 min/med/max row already cited :24; that
   cite is unchanged.
3. Dispatch also ordered a Sec 1.1 provenance-note cite change
   :42-50 -> :41-50. NOT APPLIED: fresh Read of
   `crates/cgap-git/src/pack_demux.rs` shows the doc comment spans
   :42-50 (line 41 blank); the existing cite was already correct.
   Divergence recorded, not silently absorbed.
4. The Sec 2.2 do-not-promote fence on 23.35 ns/fact
   (W4-D3 addendum required) is intact and untouched.

Standing: ALIVE (all corrected cites re-read on disk this session).
Fences: none mechanical (docs-only); the 23.35 ns/fact
do-not-promote fence re-verified intact post-edit.
