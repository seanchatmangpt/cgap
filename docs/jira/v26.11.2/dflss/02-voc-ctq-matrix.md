# 02 VOC-CTQ Matrix (CTQ-01..07)

Canonical per-cell register for the seven csct/cscb commercial CTQs
(DF-1 lane, DMADV Define/Measure). Parent artifacts:
`../14-dflss-charter.md` (workspace charter, canonical for G1-G6) and
`../15-ctq-tree.md` (canonical for segments S1-S4). VOC: bankable
deterministic-replay substrate that pays for itself (RFC-0042) without
violating the enclave physical envelope. One row per CTQ: target,
measured (re-run this session unless stated), receipt file:line, status.

| CTQ | VOC | Target | Measured (fresh) | Status |
| CTQ | VOC | Target | Measured (fresh) | Status |
|-----|-----|--------|------------------|--------|
| CTQ-01 | no heap on hot path | 0 allocs / 0 B | c4 exit 0; breach flip 1 | VERIFIED |
| CTQ-02 | frame ingress | p99 <= 10 ns | p99 5.00 ns release | VERIFIED-COND |
| CTQ-03 | SRAM fit | <= 131,072 B | 36,048 B; mutant flips | VERIFIED |
| CTQ-04 | seal throughput | median >= 1M/s | median 3,894,347/s | VERIFIED [a] |
| CTQ-05 | closure clock | 60,000 ms DECIDED | min 787.5 ms; 25 ms unmet | GATE VERIFIED [b] |
| CTQ-06 | branch-free kernel | 0 forbidden branches | exit 0; 3 RV32 residuals | VERIFIED |
| CTQ-07 | commercial ROI | >= 350x gross | 351 >= 350 | VERIFIED |

[a] VERIFIED (in-lane; UNVERIFIED-FOR-PAPER pending consolidation),
per the master-table ID-space note and standing rule below.
[b] GATE VERIFIED; re-scope receipt:
`../lanes/lane-mv-05-sla-reconcile.md` (provenance gap CLOSED, see
the CTQ-05 section below).

## CTQ-01 zero-alloc — VERIFIED

- Target: 0 heap allocs / 0 bytes steady-state ingress.
- Measured (fresh, 2026-11-02): `cscb_c4_alloc` clean run exit 0,
  "C4 PASS: steady-state 0 allocs / 0 bytes over 1000000 frames".
- Instrument: `EnclaveTrackingAlloc<System>` global allocator, armed
  window + `last_breach()` (b03 receipt :24-27).
- Anti-vacuity: `CSCB_MUTATE=alloc` injects `Box::new(42u64)` at frame
  0; breach recorded, exit 1 witnessed fresh (allocs=1 bytes=8). The
  null-mutate path exits 1 `C4 VACUOUS` (b03 receipt :28-33).
- Ranking law: instrumented (CountingAlloc / tracking-alloc with
  positive controls) ranks STRONG; static-only rows (nm census) rank
  WEAKER — `../audit/doctrine-sram-and-proofs.md:19-31,73` (TIER 1
  instrumented inventory: soc_full_chip_court lane-git-16, C03 e5 edge
  court positive control, cgap-blake3 tests/zero_alloc.rs).
- Receipt: `../lanes/lane-b03-cscb-c4.md:12-33`.

## CTQ-02 ingress latency <= 10 ns — VERIFIED-CONDITIONAL

- Target: p99 per-frame <= 10.0 ns in release
  (`crates/csct/src/bin/cscb_c1_c2_ingress.rs:49` C2_P99_NS_MAX=10.0).
- Measured (fresh): p99 5.00 ns/frame, max 10.0, C1 205,407,348
  frames/s, exit 0, [META] c2_method=batch64-amortized-instant.
- Method (mandatory citation): the 10 ns SLA cannot be measured with a
  literal per-frame Instant pair on macOS (~40 ns clock overhead; raw
  per-frame witnessed p99 42 ns in the b01 receipt, which measured the
  clock, not the frames). Disclosed BATCH=64 amortization
  (cscb_c1_c2_ingress.rs:50,147-193): each sample = one Instant pair
  spread over 64 frames, p99 over n_batches=FRAMES/64 samples (~15,625
  batch-amortized samples) - a genuine tail quantile, not a max.
- Condition: the SLA holds under batch64 amortization only; debug builds
  loud-skip (`LOUDSKIP(release-only)`, exit 0, typed, not silent).
- FIX-04 residual, recorded honestly: lane-fix04 flagged c6 (same max-
  as-p99 defect, owner should apply min/median/max rename); the c1_c2
  file was audited by FIX-04's sibling audit and found HONEST (p99 over
  thousands of batch-amortized samples is a genuine quantile; no fix
  needed) — `../lanes/lane-fix04-cscb-p99.md:31-35`. The in-file doc
  comment at cscb_c1_c2_ingress.rs:23 still says "batch of 8" while the
  const is BATCH=64 (:50) — stale-comment divergence, recorded, not
  owned by this lane.
- Receipt: `../lanes/lane-b01-cscb-c1-c2.md:13-35,53-63`.

## CTQ-03 SRAM fit — VERIFIED

- Target: linked riscv32imc SRAM families (.text+.rodata+.data+.bss)
  <= 131,072 B.
- Measured (fresh): `python3 scripts/verify_cscb_sram.py` exit 0,
  "measured SRAM (riscv32imc-unknown-none-elf): 36048 B", headroom
  95,024 B.
- Anti-vacuity: `--mutate=sram-bloat` mutant measures 136,068 B > bound
  and flips, exit 0 as a flip verdict
  (`../audit/doctrine-sram-and-proofs.md:8-11,66`).
- Non-conflation: 39,184 B is the empirical baseline, REPORT-ONLY
  (lane-b04-cscb-c5.md:36-37,70; doctrine :15-17). "39,184 -> 36,048
  improvement, PASS" is a category error (doctrine :67 NARRATIVE-ONLY).
- Receipt: `../audit/doctrine-sram-and-proofs.md:8-17,64-67`.

## CTQ-04 seal rate — VERIFIED (in-lane; UNVERIFIED-FOR-PAPER
pending consolidation)

Note: tag per the master-table ID-space note (:154-158 region) and the
standing rule — the 3.89M/s fresh-run figure is receipted in-lane
(lane-fix04) and has no `../11-measured-claims.md` row, so it stays
UNVERIFIED-FOR-PAPER until consolidated. The in-lane VERIFIED standing
(court + anti-vacuity flip, this session) is unchanged.

- Target: median >= 1,000,000 receipts/s (RATE_FLOOR).
- Measured (fresh, two runs): median 3,894,347/s (min 3,577,562, max
  4,258,922) and median 3,891,452/s (min 3,116,010, max 4,224,563),
  runs=7; [C7-PASS] rate gate + chain validity (1,000,000 receipts),
  exit 0 both runs. Machine-dependent envelope ~3.1-4.4M/s; the b07
  receipt's 4.2-4.4M/s is same-class, not conflated.
- Labels post-FIX-04: min/median/max over runs=7 with an explicit
  "no p99 computed" note (population too small for a tail quantile) —
  `../lanes/lane-fix04-cscb-p99.md:11-17`.
- Anti-vacuity: CSCB_MUTATE mutated-certificate rejection, exit 0 flip
  verdict (`../lanes/lane-fix04-cscb-p99.md:26-27`).
- Receipt: `../lanes/lane-fix04-cscb-p99.md:20-29`; fresh run this
  session (commands in the DF-1 receipt).

## CTQ-05 closure <= 25 ms — ASPIRATIONAL-UNVERIFIED (gate VERIFIED)

- Target: two-tier, read fresh from
  `crates/csct/src/bin/cscb_c3_closure.rs:33-42`:
  DECIDED_BUDGET_MS = 60,000.0 (lane MV-5 gate re-scope, 2026-11-02,
  min-of-7 gate) and ASPIRATIONAL_MS = 25.0 (never a gate anywhere;
  EXP-03).
- Measured (fresh): exit 0 DECIDED GATE PASS min=787.512 ms
  (batches=1, quiet host) — the gate itself reports loudly: "ASPIRATIONAL
  25ms NOT met: measured min is 32x the aspirational figure — reported
  loudly, not gated".
- Provenance chain: the in-file comment (:36-38) cites a concurrent-
  lane-loaded witness min 36,330.134 ms / median 40,258.075 / max
  52,868.876; repair-20 measured 24-25 ms on a DIFFERENT corpus
  (`../repairs/repair-20-xorfilter-gate.md:57` states the 22.99 ms
  fence is from a different session/configuration and was NOT a gate);
  the 22.99 ms witness is EXP-03, not a gate.
- Provenance gap: CLOSED. `../lanes/lane-mv-05-sla-reconcile.md`
  exists on disk (checked this session) and carries the MV-5 re-scope
  receipt: commands+exits table (:22-30, all exit 0), decided budgets
  (:34-43 — C3 DECIDED_BUDGET_MS = 60,000 ms min-of-7, session N=7
  min 36,918.401 ms / median 40,324.179 ms, ASPIRATIONAL_MS = 25.0
  EXP-03 reported loudly, never gates), corroborated by the B02
  receipt, standing ALIVE (:81), courts flip (:85). The earlier
  "no lane-mv-5 receipt" gap note was written before this receipt
  landed; the re-scope provenance is no longer absent; charter
  tollgate T4 evidence restored.
- Receipt: fresh run (DF-1 receipt commands); consts cited at
  cscb_c3_closure.rs:33-42; repair-20 cite above.

## CTQ-06 zero forbidden branches — VERIFIED

- Target: 0 forbidden (data-dependent) branches in the 28-function
  branchless census.
- Measured (fresh): `python3 scripts/verify_disassembly_invariants.py`
  exit 0; census 28 allowlisted fns; [PASS] 0 forbidden branches for the
  SWAR/prefix/radix/poison families; three ALLOWED-RESIDUALs documented
  ISA-ceiling (RV32 has no cmov): cmov_u64 (1x bnez), eytzinger_restore
  (1x beqz), select_slice (beqz len-guard + bnez cond byte; REP1 owns
  re-census).
- Anti-vacuity: the gate's mutation surface is the allowlist itself —
  a forbidden branch added in-census fails the gate; the residual
  allowlist rows are named, not silently absorbed.
- Receipt: fresh run this session (DF-1 receipt commands); residual
  rows cite lane-03 rows in the script output.

## CTQ-07 commercial ROI >= 350x — VERIFIED

- Target: REQUIRED_ROI_X = 350.0 gross formula
  (`crates/csct/src/lib.rs:30-35`; ruling text :30-34, verdict :79-81).
- Measured: gross 5,068,632 / 14,400 = 351.99, integer floor 351 >= 350
  asserted in the c8 court
  (`../lanes/lane-b07-cscb-c8.md:19-23`); full-suite release witness
  `final_commercial_verdict | PASS`, 6 stages Pass, ROI 352x, net delta
  $5,054,232 (`../lanes/lane-g11-csct-court-fix.md:18`).
- Anti-vacuity: zeroed-spark mutant (spark 318,873 -> 0) flips the
  court; test `court_flips_on_zeroed_spark` green
  (`../lanes/lane-b07-cscb-c8.md:24-26,50-51`). Post-ROI-fix mutate mode
  exits 0 with controls-run-first proof (lane-g15:23-25).
- Receipt: `../lanes/lane-b07-cscb-c8.md:19-26`; not re-run end-to-end
  this session (6-stage suite); cited receipts are fresh-landed
  2026-11-02. Status: VERIFIED by receipt; end-to-end re-run marked
  UNVERIFIED-this-session, honest.

## Master CTQ table (DF-01 re-anchor, 2026-11-02)

Landed by lane DF-01. ID-space note: the csct/cscb CTQ-01..07 rows
above are the csct-family register and stay canonical for that
family; this master table is the campaign-level register and uses
M-CTQ ids to avoid conflation. The VOC-to-CTQ translation follows
the red-20 synthesis constraint set (breaches + survivors).

### VOC -> CTQ translation

| VOC (customer voice) | translated CTQ | id |
|---|---|---|
| "ingress latency must not surprise me under load" | scoped
  quiet-host ns/frame figure + registered contention divergence |
  M-CTQ-01 |
| "the hot path must not allocate" | scoped zero-alloc sentry
  (ring/demux/pkt-line), scope caveats stated | M-CTQ-02 |
| "the kernel must fit the enclave envelope" | SRAM gate fit per
  gated subject, loud refusal when over | M-CTQ-03 |
| "sealing must keep up with the frame rate" | receipts/s court
  floor, uncontended release build | M-CTQ-04 |
| "the FFI boundary must not dominate the budget" | membrane floor
  ns/call, layered budget stated | M-CTQ-05 |
| "the shared-memory read must be effectively free" | mmap zero-copy
  ns/op, pread fallback priced | M-CTQ-06 |
| "closure must be bounded by a real budget" | chain/fixpoint ms vs
  the re-scoped USL row | M-CTQ-07 |

### Master CTQ register

- **M-CTQ-01 ingress ns/frame — VERIFIED (SCOPED QUIET-HOST).**
  Anchor: `../11-measured-claims.md` row 5 (10.20 fastest / 10.30
  median ns/frame; divan, 1M const, 100 samples). The quiet-host
  scope is part of the claim; contention row 28 (14.71 / 19.03 /
  124.06 ns/frame, slowest 12.2x row 5) is a REGISTERED DIVERGENCE,
  never quoted unqualified (red-10-timing-jitter-exploit.md:30-41,
  77-88). "Constant-time" is UNSUPPORTED (row 29) and NOT claimed
  by this row.
- **M-CTQ-02 zero-alloc sentry — VERIFIED (SCOPED).** Anchor:
  `crates/cgap-git/tests/posh_alloc_sentry_court.rs` (CountingAlloc
  window; positive control flips). Red-09 battery: 15 adversarial
  cases, exit 0, allocs=0 bytes=0 under open window
  (red-09-alloc-sentry-breach.md W3-W5). Scope caveats are part of
  the claim: the bn3 `unify_to_fixpoint` leg (2,662 allocs /
  650,294,994 B measured) and the cross-FFI leg are NOT zero-alloc
  claims (red-09 :74-82).
- **M-CTQ-03 SRAM fit — VERIFIED (SCOPED to gated subjects); bn3
  gated-and-REFUSED.** Anchors: `scripts/verify_cscb_sram.py`
  subject list; 36,048 B baseline referent at
  `02-ax-charter-ctq.md:135` (gate exit 0, headroom 95,024 B;
  sram-bloat mutant 136,068 B flips, lane-b04:35,40). RED-12 scoped
  the original claim to blake3+branchless only; FIX-02 landed
  (`../lanes/lane-fix02-bn3-sram-gate.md`, re-read on disk at write
  time): bn3 now a gated subject; static charge 6,520,984 B >
  131,072 B = loud `BN3_ENCLAVE_REFUSED` exit 1; bn3-pin mutant
  flips. Envelope verdict: baseline subject fits (VERIFIED); bn3
  over ceiling and REFUSED by gate (loud, not silent).
- **M-CTQ-04 seal throughput — CONSOLIDATION-ADDENDUM (claims §4
  row 30); cross-instrument wire legs UNVERIFIED.** Consolidated
  figures: min 3,529,029 / med 3,824,869 / max 3,846,425 and run-2
  min 4,069,561 / med 4,130,106 / max 4,135,837 receipts/s
  (`../11-measured-claims.md` row 30; N=7 csct Min/Med/Max,
  FACT_WINDOW=1,365, batch-amortized, quiet-contention guard).
  Caveat D2 (inline): 1,365-receipt window vs b06's 1M chain —
  NOT b06-comparable. In-lane receipt kept:
  `../lanes/lane-b06-cscb-c7.md:24,51` (median 4,356,280 receipts/s
  vs 1M floor, exit 0, ALIVE, uncontended release; contended/debug
  runs honest exit-1 typed failures). The 1M-window seal leg
  (b06's instrument) remains UNVERIFIED until re-measured at that
  window; the addendum row supersedes the earlier "NO row in
  11-measured-claims" note.
- **M-CTQ-05 membrane floor — CONSOLIDATION-ADDENDUM (claims §4
  rows 31/31a); cross-instrument wire legs UNVERIFIED.** Row 31
  (in-process extern "C" into real `cgap_admit_frame`, REAL 152 B
  A04 frames): min 123.81 / med 124.02 / max 125.12 and min
  112.48 / med 113.37 / max 114.44 ns/frame; row 31a null-refusal
  in-process floor 0.38-0.41 ns/call. Caveat D1 (inline): NOT cgo —
  the in-process route omits the cgo trampoline ipc-03 prices;
  `../lanes/lane-ipc-03-latency-court.md:44,50`'s 85 ns/call
  membrane floor remains the WIRE-LEVEL AUTHORITY (ipc-03 saw
  183-187 ns/frame through cgo; the ~70 ns delta is the
  in-process-vs-cgo membrane delta). Positive control flipped
  (0x43470002 OABI_MISMATCH -> CGAP_OK), exit-flip witnessed.
  In-lane receipt kept: ipc-03 :44,50. Cited downstream
  (09-ax-cpk-msa.md:19, 10-ax-control-plan.md:27). The cgo wire leg
  remains UNVERIFIED until re-measured through cgo.
- **M-CTQ-06 SHM zero-copy read — CONSOLIDATION-ADDENDUM (claims
  §4 row 32); cross-instrument wire legs UNVERIFIED.** Row 32
  (Rust mmap, producer RW MAP_SHARED / consumer PROT_READ,
  "CGAPSHM1", 1,365 facts @24 B): min 0.15 / med 0.18 / max 1.56
  and min 0.15 / med 0.15 / max 1.47 ns/op; anti-vacuity flip
  witnessed both runs (cross-mapping corruption visible,
  byte-exact restore; run-1 max is page-table warmup). Caveat D3
  (inline): Rust L1-hot loop (32,760 B working set) vs
  `../lanes/lane-ipc-04-shm-transport.md:57`'s Go-harness read call
  included in its 4.52 ns/op — different instrument, same
  mechanism; ipc-04 receipt kept. The Go-harness wire leg remains
  UNVERIFIED until re-measured on that instrument.
- Standing note (post-addendum, ADV-07): §4 rows 30/31/31a/32 are
  receipted but NOT full MEASURED promotion — the claims file says
  so itself (:97-99); the cross-instrument wire legs stay
  UNVERIFIED per its :126 verdict.
- **M-CTQ-07 chain/fixpoint budget — RE-SCOPED;
  UNVERIFIED-FOR-PAPER (chain_ram 23.13 ms receipted in-lane at
  `../lanes/lane-gap-07-g2-pareto.md:52`, not consolidated;
  consolidated row 9 = 42.58 ms governs).** The dossier's
  "$23.13 ms fixpoint" referent is
  `../lanes/lane-gap-07-g2-pareto.md:52,105,160`: 23.13 ms @1M
  chain_ram, L2 <10 ms target NOT MET (2.31x over). It is a
  chain-latency number, not a fixpoint number — the dossier label
  is a misnomer, recorded. REGISTERED CONFLICT: 23.13 ms
  (lane-gap-07-g2-pareto.md:52, chain_ram @1M, divan) vs 42.58 ms
  (`../11-measured-claims.md` row 9, chain_ram @1M, divan, 100
  samples) is a SAME-SURFACE divergence of 1.84x, with no
  re-measure reconciling them since; per the standing rule the
  consolidated row 9 figure governs for the paper, and the lane
  figure stays UNVERIFIED-FOR-PAPER until consolidated. Governing USL row: control-plan
  AX-GATE-03 (`10-ax-control-plan.md:27` — 10 ns USL NOT met at any
  layer; lawful move is re-scope, never tune-to-target), echoed by
  red-20 §4 P4 item 3. The dispatch said "red-20 USL row": red-20
  contains no row labeled USL — the on-disk USL row is
  10-ax-control-plan.md:27; divergence recorded.
- Standing rule for this register: a number quoted from a lane
  receipt but absent from `../11-measured-claims.md` carries
  UNVERIFIED-FOR-PAPER, never MEASURED, until consolidated. This
  table names no number lacking an on-disk referent checked at
  write time.
