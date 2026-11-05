# bm6 — Benchmark Claims Table (Closing Lane 6)
Standing: BLOCKED

- Date: 2026-11-04. Lane: BENCHMARK LANE 6 (closing consolidation).
- Inputs: bm1, bm2, bm3, bm4 (cgap + arcus), bm5 — all five on disk at
  consolidation time (poll-wait completed; no cap-fire).
- Priors re-read at use time: zp-11-uds-transit-floor.md (G5),
  v4-corrected-claims-ledger.md rows 2/4/9, rx-3/ZP-17 (1.26394
  µs/link), X9 soak 63,223 fps pre-v3, lsp-max ab-17, ab-10
  (113,132 fps writerv plane), legacy-JSON 4633 ns/op (bm5's own
  prior line).
- No git operations (coordinator owns staging/commit).

## Claims table

| # | Claim | Prior (receipt cite) | New measured (cite, cmd + exit) | Verdict |
|---|---|---|---|---|
| 1 | UDS trip/residual: trip p50 25.9-27.1 µs, residual ~15.9-16.6 µs (256B) | zp-11-uds-transit-floor.md (G5; standing UNVERIFIED — load 10-22, no quiet witness) | NONE — bm1-transit-floor-v2.md is SKIP-AND-REPORT: C4 BUSY-retry precondition absent from uds_transit_probe_test.go (grep exit 1); probe NOT run | NOT-RERUN (bm1 BLOCKED; zp-11 standing unchanged: UNVERIFIED, upper bound only) |
| 2a | cgap-blake3 per-call 64B: p50 125.0 ns, min 83.0 ns | v4-corrected-claims-ledger row 2 (125.0-167.0 band, min 83.0); af-22 | bm2-crypto-kernels.md: p50 125.0 / p99 167.0 / min 83.0 (exact), `cargo test -p cgap-blake3 --release --test percall_latency -- --nocapture`, exit 0; min-of-3x10000; fenced (LOUD-SKIP dev-profile witness fires); load 7.63 disclosed; timer floor ~30-40 ns inside every number (disclosed) | CONFIRMED (exact p50 + min) |
| 2b | blake3 batch speedup at N=64x1KiB: 2.515x | bm2's own cited prior (2.515x, same bench, same shape); PL15 predicted 1.4-1.6x | bm2: 2.480x, `cargo test -p cgap-blake3 --release --test independent_batch_perf -- --nocapture`, exit 0, min-of-5; delta -1.4% | CONFIRMED (inside noise; both above PL15 band) |
| 3 | RingReceiptChain 50k-span µs/link (cgap-ffi plane): 1.26394 µs/link | rx-3 (25.5202 µs/link debug, SUPERSEDED) + zp-17-gate10-settling.md (1.26394, min-of-7, POST-FOLD per errata-2) | bm3-chain-scale-v2.md: 63.330 ms -> 1.26660 µs/link, `cargo test --release -p cgap-ffi --test receipt_chain_scale -- --nocapture`, exit 0, min-of-3 internal; inside same-day band ±0.33% (8-sample set 63.197-63.619 ms) | CONFIRMED (in band; 8th same-day sample) |
| 3b | bm7 supplemental HEAD reconfirm | same prior | bm7-gate10-head-reconfirm.md: 63.929 ms -> 1.27858 µs/link, exit 0 x3, min-of-3; +0.49% above band top, +0.95% vs bm3 — OUTSIDE band; committed HEADs 3e0339e6/b595f276 do not compile (compile-abort, 0 tests) — working-tree measurement only; load 44-48 | NOT CONFIRMED (as-measured, disclosed out-of-band; no fence/target moved) |
| 4a | Soak fps over real aoq_sidecar UDS: prior 63,223 fps (X9, pre-v3) | X9 soak 15.8 s / 63,223 fps (pre-v3 Finish law); ab-10's 113,132 fps is the writerv plane, NOT this one | bm4-soak-churn.md: 1,000,000 verified echo trips in 18.047 s = 55,411 fps, `cargo test --release -p cgap-ffi --test aoq_sidecar_soak_1m_court -- --nocapture --test-threads=1`, exit 0, 2 passed; load 37.28 disclosed; run 1 exit 101 (v2-shaped client vs v3 Finish law) repaired by premise-rot re-pin (v3 body 1104 B, AEAD seal/open + byte-identity assert per trip) | SUPERSEDED (lawful-work delta: v3 adds real per-trip AEAD seal+open where v2 client did none; -12.3% is not a transport regression; prior band not comparable across the law change) |
| 4b | Churn fd/RSS: no leak across kill cycles | lsp-max ab-17-churn-stress.md: fd table drops 34->10, plateaus 13088 KiB RSS from cycle 100; bounds fds <= baseline+16, RSS <= 2x baseline+16 MiB | bm4-churn.md (arcus): 8 generations vs REAL aoq_sidecar child, /dev/fd census delta == 0 per cycle, dir-orphan asserts hold; `go test ./internal/arcus/flight -run TestSupervisorChurnFDCourt -count=1 -v`, exit 0; supervisor family 5/5 exit 0; tighter per-cycle law (delta==0) passed with zero tolerance slack consumed (not the looser ab-17 fence shape — disclosed) | CONFIRMED (0-leak re-witnessed, tighter law than prior) |
| 5 | Legacy JSON ns/op (arcus Go plane): 4633 ns/op | bm5's prior line: 4633 ns/op, 42.52 MB/s, 440 B/op, 6 allocs/op | bm5-legacy-baseline.md: 4531/4564 ns/op (mean ~4548), 43.48/43.17 MB/s, 440 B/op, 6 allocs/op exact; `go test ./internal/arcus/adapter -run '^$' -bench BenchmarkLegacyJSONUnmarshal -benchmem -benchtime 1s -count=2`, exit 0; spread 0.73% STABLE; sink/marshal-leg DCE audit PASS | CONFIRMED (~-2.2%, inside machine-noise class; alloc profile exact match) |

## Plane / methodology columns (per row)

| Row | Plane | Profile | Method | Fenced | Quiet witness |
|---|---|---|---|---|---|
| 1 | Go<->sidecar UDS trip (arcus flight) | release sidecar + go test binary | would-be p50/p99 probe | n/a | NOT ACHIEVED (bm1 load 7.31; moot, skipped) |
| 2a | cgap-blake3 userland per-call | release (assert_release gate; dev run LOUD-SKIPs) | min-of-N: 3 rounds x 10000, best-round percentiles | Y (release fence + anti-DCE) | N (LA 7.63 disclosed) |
| 2b | cgap-blake3 batch (64x1KiB) | release | min-of-5 frames | Y (same gate family) | N (LA 7.63) |
| 3/3b | cgap-ffi RingReceiptChain ONLY (never AoqChain, never arcus) | release (SKIP-grep=0 witnessed) | min-of-3 internal per cell; bm3 1 run, bm7 3 runs | Y (assert_release + fence on digest I/O) | N (bm3 LA 19-27; bm7 LA 44-48) |
| 4a | Rust-reachable echo fps, real aoq_sidecar UDS (NOT Go-reachable; NOT ab-10 writerv) | release, --test-threads=1 | 1M verified trips, wall-clocked; full count, no shrinkage | Partial (wall budget 120s; no per-trip fence) | N (LA 37.28) |
| 4b | Go supervisor plane ONLY (arcus flight; no Rust code) | go test (default) | 8-generation fd census, per-cycle delta==0 + dir-orphan assert | Y (hard asserts, delta 0) | N (LA 37.28) |
| 5 | arcus Go legacy-JSON membrane (never the AOQ1 codec; never a fleet claim) | go bench 1s benchtime | -count=2, spread 0.73%; b.SetBytes both legs; sink audit | Y (benchSink DCE audit PASS) | N (LA 37.28) |

Quiet-machine witness: NOT ACHIEVED on any row (fleet load 7.3-48 across
the window). Every number in this table is an upper bound under disclosed
concurrent load; min-of-N rows are contention-robust, wall-clock rows
(4a) less so. No row may be cited as a quiet-machine floor.

## Contradiction ranking (new vs prior beyond disclosed noise band)

1. **4a soak fps 55,411 vs 63,223 (-12.3%)** — beyond any noise band, but
   the measured surface changed underneath: the v3 Finish law adds real
   per-trip AEAD seal+open work the v2 client never did. Verdict
   SUPERSEDED (lawful-work delta), not CORRECTED. Residual risk named by
   the lane: the re-pin diff is uncommitted in X9/M2a's ownership — the
   coordinator must land it before the 55,411 number is citable.
2. **3b bm7 50k-span 63.929 ms — outside the ±0.33% same-day band**
   (+0.95% vs bm3). NOT CONFIRMED; as-measured; load 44-48 and uncommitted
   C1 in-flight edits named as candidate causes, not adjudicated. bm3's
   in-band 1.26660 remains the citable value; zp-17's 1.26394 stands
   (min-of-7 is the tighter set).
3. **5 legacy JSON -2.2% (4633 -> ~4548 mean)** — slightly beyond the
   0.73% intra-run spread but inside the run-to-run machine-noise class
   the prior itself exhibited (4602-6399 across v4-ledger count=5);
   alloc profile exact. CONFIRMED, no regression.
4. **2b batch speedup 2.480x vs 2.515x (-1.4%)** — inside noise; both
   runs well above the PL15 predicted 1.4-1.6x band. CONFIRMED.
5. Non-contradictions: 2a exact; 3 in-band; 4b tighter-than-prior pass;
   1 unmeasured (bm1 BLOCKED), so no contradiction is possible.

## PLANE-LAW audit (cross-plane citation scan of bm1-bm5)

Scanned for the three defect classes: (i) refusal-trips cited as echo
throughput; (ii) RingReceiptChain µs cited onto AoqChain/arcus; (iii)
debug-profile numbers unlabeled.

- bm1: emits no numbers; plane disclosure explicit; no defect.
- bm2: all rows carry plane + profile + method; the dev-profile run emits
  a LOUD-SKIP line, zero debug numbers; ML-KEM rows labeled first-sample
  UNVERIFIED baseline; timer-floor disclosure present; no defect.
- bm3: explicit plane-law block ("ONLY RingReceiptChain... never
  cross-cite onto AoqChain or arcus"); V3's 64.479 ms reported as cited-
  not-re-measured; no defect.
- bm4 (cgap): scope line names the plane "Rust-reachable echo fps... NOT
  Go-reachable"; echo trips are REAL verified AppData round trips
  (post-re-pin), not refusal trips — refusal-shaped trips were the zp-11
  probe class, kept separate; the X9 prior is same-court (comparable);
  ab-10's 113,132 fps correctly left uncited as a different plane; no
  defect.
- bm4 (arcus): supervisor-plane-only disclosure; cross-references cgap
  bm4-soak-churn by path rather than importing its numbers; ab-17's
  looser fence shape named as not-what-ran (no tolerance-slack laundering);
  no defect.
- bm5: plane-law block ("before for serialization ratios only", never the
  membrane, one-contender lane); sink/DCE audit; no defect.

Defect hits: **0**.

## Exit audit

- Numbers lacking a producing command: **0** (every measured number in
  bm2/bm3/bm4/bm5/bm7 carries its command + exit; bm1 emits none).
- Planes cited onto surfaces they never measured: **0** (audit above).
- Missing lane receipts at close: **none** (bm1, bm2, bm3, bm4 x2,
  bm5 all on disk; poll completed before the 75-min cap).

## Receipt

- Claims table: /Users/sac/cgap/docs/jira/v26.11.4/receipts/bm6-benchmark-claims-table.md
- Poll-wait: bm receipts completed during the wait (lane 4 landed last,
  ~13:1x PST); no cap-fire, no fail-closed gap.
- Exit code: 0.
- Standing: consolidation receipt over ALIVE lane receipts (bm1 BLOCKED
  for its own cell); quiet-machine witness absent fleet-wide — all values
  upper bounds under disclosed load. Untracked evidence: staging/commit
  coordinator-owned; this file is untracked at write time.

## Addendum — Load-profile re-publication (L27, 2026-11-04, KD-2 close)

Supersedes the unbounded ns/frame reading of the rows above (which carry
no ns/frame ingress figures themselves; the contested surface is
`docs/jira/v26.11.2/11-measured-claims.md` rows 5-8/28, quiet-host
scoped). Paired solo/contended re-derivation, min-of-3, same method
both profiles; raw TSV + per-leg logs at `/tmp/l27-kd2-paired/`
(driver: `scripts/l27-kd2-paired-bench.sh`, exit 0 x6 legs;
`scripts/l27-loadgen.c` reproduces the RED-10 W4 load classes;
receipt: `docs/jira/v26.11.5/receipts/l27-kd2-claims.md`).

| Row | Claim | Load profile (machine-stated) | fastest | median | slowest | Ratio fence |
|---|---|---|---|---|---|---|
| 5/6 | Ingress chain 1M const (ns/frame) | SOLO (min-of-3) | 10.35 | 10.70 (min) | 37.83 (worst) | baseline |
| 5/6 | Ingress chain 1M const (ns/frame) | CONTENDED bw6+cpu9 (min-of-3, paired) | 11.21 (+8.3%) | 22.27 (+108%) | 98.23 (worst) | **9.49x worst/best vs solo** |
| 7 | Penstock admission stage | SOLO (min-of-3, L57 paired) | 8.02 | 8.31 (min) | 86.48 (worst) | baseline |
| 7 | Penstock admission stage | CONTENDED bw6+cpu9 (min-of-3, paired, L57) | 9.65 (+20.3%) | 16.64 (+100%) | 318.15 (worst) | **39.7x worst/best vs solo** |

Load-profile law for consumers: solo numbers are upper bounds under
disclosed background, never worst-case bounds; under RED-10-class
combined load the worst sample is 9.49x the solo fastest for the
ingress chain (L27) and 39.7x for the penstock stage alone (L57;
prior bw-only tail 12.73 is not a contended bound). RED-10 measured
12.2x on 2026-11-02 — same class.
