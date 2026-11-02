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
| CTQ-04 | seal throughput | median >= 1M/s | median 3,894,347/s | VERIFIED |
| CTQ-05 | closure clock | 60,000 ms DECIDED | min 787.5 ms; 25 ms unmet | GATE VERIFIED |
| CTQ-06 | branch-free kernel | 0 forbidden branches | exit 0; 3 RV32 residuals | VERIFIED |
| CTQ-07 | commercial ROI | >= 350x gross | 351 >= 350 | VERIFIED |

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

## CTQ-04 seal rate — VERIFIED

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
- Provenance gap: no `lane-mv-5` receipt exists in
  `../lanes/` (only lane-mv-04-warnings-census.md and
  lane-mv-08-doc-refresh.md match "mv") — the MV-5 re-scope's receipt
  provenance is absent and recorded here as a gap; charter tollgate T4.
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
