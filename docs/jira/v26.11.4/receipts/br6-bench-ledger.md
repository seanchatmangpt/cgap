# BR-6 — Bench-Results Ledger (v26.11.4)

Date: 2026-11-04. Scope: every numbered receipt in
`/Users/sac/cgap/docs/jira/v26.11.4/receipts/` and
`/Users/sac/arcus/docs/jira/v26.11.4/receipts/`. Method: re-read each receipt
on disk, verify the cited raw output exists (committed raw log or in-receipt
verbatim block), re-derive every derived quantity from the raw numbers
(arithmetic re-run by this lane, shown where non-trivial). Read-only against
both repos except this receipt.

Machine (all metrics, unless the receipt's own witness differs): Apple M3 Max,
16 cores, 48 GB, macOS Darwin 25.2.0 — a shared fan-out host; every number is
load-bound as disclosed per receipt.

## 1. Per-receipt verdict table

Verdicts: VERBATIM-OK = cited raw output present in-receipt or as a committed
raw log AND all derived quantities re-derive correctly; MISMATCH = derived
quantity does not reproduce from the cited raw numbers; NO-RAW-LOG = a cited
number's generating output exists only in volatile `/tmp` or an unreceipted
session lane.

### cgap

| receipt | carries | raw evidence | derivation re-check | verdict |
|---|---|---|---|---|
| bm2-crypto-kernels.md | BLAKE3 64B/1KiB/64KiB p50/p99/min; batch 2.480x; ML-KEM 25.50/28.44/31.33 µs | in-receipt verbatim blocks (3 blocks + ML-KEM) | 125.0/2125.0/32750.0 as printed; 2.480x printed by test; no unstated division | VERBATIM-OK |
| bm3-chain-scale-v2.md | 50k-span 63.330 ms; 1.26660 µs/link; band ±0.33% | verbatim block + claims `/tmp/bm3_runs_v2.log` as source of record (volatile) | 63,330/50,000 = 1.26660 ✓; band: half-range 0.211 ms on mid 63.408 ms = 0.333% ✓; curve cells 1.336/12.649 → 1.3360/1.2649 ✓ | VERBATIM-OK (committed-state caveat: `/tmp` log only) |
| bm4-soak-churn.md | 1M trips 18.047 s = 55,411 fps | in-receipt verbatim line | 1,000,000/18.047 = 55,411 fps ✓ (18.047 s → per-trip 18.0 µs ✓); −12.3% vs 63,223 fps: (63223−55411)/63223 = 12.35% ✓ | VERBATIM-OK (working-tree subject; court re-pin uncommitted) |
| bm7-gate10-head-reconfirm.md | 50k-span 63.929 ms = 1.27858 µs/link; committed-HEAD cell BLOCKED | verbatim blocks (run 1 + 3-run SPAN-CURVE) + `/tmp/bm7_runs_wt.log` (volatile) | 63,929/50,000 = 1.27858 ✓; delta vs band top 63.619: +0.310 ms = +0.49% ✓; vs 1.26660: +0.95% ✓ | VERBATIM-OK |
| bm9-crypto-primitives.md | divan table (aead 6.0 µs, blake3 153.1 ns/3.52 µs/38.29 µs, mldsa 734 ns/1.03 µs/194 ns, mlkem 29.41/32.7/36.24 µs); census 76/67/12 | in-receipt verbatim divan table | derived throughput: 64 KiB/38.29 µs = 1.75 GB/s ✓; 2 KB/6.0 µs = 341 MB/s ✓; marked UNVERIFIED-beyond-arithmetic in-source | VERBATIM-OK |
| zp-17-gate10-settling.md | 7-run table 63.197–63.619 ms; min 1.26394 µs/link; ERRATA + ERRATA-2 | in-receipt table, `/tmp/zp17_runs2.log` (volatile) | all 7 µs/link cells = ms/50 exactly ✓; sorted set: min 63.197, median 63.501, max 63.619 ✓ | VERBATIM-OK (attribution label corrected twice, see §3) |
| zp-18-profile-guard-census.md | census 73 TIMING / 64 guarded / 11 allowlist | in-receipt verbatim gate output | counts as printed; self-check line verbatim | VERBATIM-OK |
| zp-18b-purity-gate-registration.md | 10 passed / 0 failed purity suite | in-receipt verbatim | counts as printed; compile-abort first attempt disclosed | VERBATIM-OK |
| zp-14-cbmqs-cli.md | 4 tests passed; input_hash f67c044c…76b | in-receipt verbatim stdout | 0+1+1+2 = 4 ✓ | VERBATIM-OK |
| wire-law-v3-landing.md | 130/0 cgap-aoq; 9/0 aead_courts | in-receipt command list, no verbatim stdout | counts cited without pasted test output | VERBATIM-OK (weak: counts only, no raw block) |
| pl-x2-failure-domain.md | fd delta 0 over 8 cycles; 11.150s suite; flake ledger | in-receipt verbatim court lines | fd delta 0 asserted by court | VERBATIM-OK |
| pl-x4-pinner-lease.go receipt | suite 8.681s exit 0, GC-court LOUD-SKIP | in-receipt verbatim | skip loud, honest | VERBATIM-OK |
| pl-x5-batch-admission.md | speedup 2.466x (re-run) vs 2.515x (landing) | in-receipt verbatim perf line | both print directly from test; 2.466/2.515 as-of-run stated | VERBATIM-OK |
| aad-ruling-executed.md | 9 passed / 0 failed | in-receipt command + exact result line | as printed | VERBATIM-OK |
| decision-aad-ruling.md, decision-d1/d2/d3 | no perf numbers | n/a | n/a | NO-NUMBERS (out of ledger scope) |
| l7-push-state-sweep.md | dirty/ahead counts, 6-repo pushed table | in-receipt tables | state snapshot as-of T2 2026-11-04T21:04:33Z — stale by construction, cite as-of | VERBATIM-OK (as-of only) |
| br2-linearity-fence-form.md (UNTRACKED) | 5-invocation distribution; b=1.260 µs/link, a=−11.5 µs; max dev 0.127% | in-receipt tables, no committed raw log | per-link 63.090/50 = 1.2618 ✓; 12.598/10 = 1.2598 ✓; 1.259/1 = 1.2590 ✓; mean 1.2602 ✓; max dev (1.2618−1.2602)/1.2602 = 0.127% ✓; residuals +0.68%/−0.08%/+0.003% consistent with a=−11.5 µs intercept | VERBATIM-OK but UNTRACKED — see §4 |

### arcus

| receipt | carries | raw evidence | derivation re-check | verdict |
|---|---|---|---|---|
| bm1-transit-floor-v2.md | none (SKIP-AND-REPORT) | grep outputs quoted | n/a | NO-NUMBERS (BLOCKED receipt) |
| bm4-churn.md (UNTRACKED) | fd delta 0 / 8 generations; 5/5 supervisor courts | in-receipt verbatim | as printed | VERBATIM-OK but UNTRACKED — see §4 |
| bm5-legacy-baseline.md | 4531/4564 ns/op, 43.48/43.17 MB/s, 440 B/op, 6 allocs | in-receipt verbatim, -count=2 | MB/s inverse-consistent with ns/op ✓; spread 33 ns = 0.73% ✓; prior-band delta ~2.2% ✓ | VERBATIM-OK |
| bm8-pre-alloc-baseline.md | 9 allocs/warmed Seal; trip p50 min-of-3 52,750 ns; dial 6,084; residual 45,314 ns; first-pass 25,458 ns (unpreserved) | in-receipt verbatim transcript (mechanically inserted from /tmp, now in-receipt) | residuals: 78167−14125−1352.1 = 62,689.9 → 62,690 ✓; 52750−6084−1352.1 = 45,313.9 → 45,314 ✓; 57792−8541−1352.1 = 47,898.9 → 47,899 ✓; min-of-3 picks correct cells ✓ | VERBATIM-OK (first-pass 25.5 µs cell: NO-RAW-LOG — transcript not preserved, disclosed in-source) |
| lane3-linearity-certificate.md | curve 1.257/12.622/64.034 ms; per-link + devs; flatness FAIL 1.884% | committed raw log `raw-lane3/lane3_chainscale.log` — byte-matched this session | 1.257→1.25700 ✓; 12.622→1.26220 ✓; 64.034→1.28068 ✓; devs −1.849%/−1.443% ✓ (vs 1.28068); spread (1.28068−1.25700)/1.25700 = 1.884% ✓; allocs 2→2, bytes 266→266 match MIN-OF-3 line ✓ | VERBATIM-OK |
| lane3-queue-divergence-certificate.md | 5547/5140 ns/op 440 B/op; AOQ1 47.8/44.5 amortized, 42.0/42.0 per-call; transit 24,333 ns; 55.7x; μ/μ'/ρ/time-to-1GiB | committed raw logs `raw-lane3/*.log` — byte-matched this session | 5140/92.3 = 55.69 → 55.7x ✓; Ws = 47.8+44.5 = 92.3 ✓; μ = 10,834 kframes/s ✓; Ws' = 92.3+16,606 = 16,698.3 ✓; μ' = 59.9 kframes/s ✓; ρ'@100k = 1.6698 ✓; 1 GiB/64 B / (100,000−59,900) = 418 s ✓; residual 24333−6375−1352.1 = 16,605.9 → 16,606 ✓ | VERBATIM-OK |
| m2a-confirm-tag.md | 23/0 lib, 5/6 session_courts, 15 red / 5 targets | in-receipt counts | counts internally consistent (6+2+1+2+6 = 17 vs "15 failing" — minor narrative rounding, reds enumerated explicitly) | VERBATIM-OK (count-math note: 15 vs enumerated 17 — disclosed-class, not a measured quantity) |
| pl-x1-crypto-parity.md | 16 courts PASS, 2.083s | in-receipt re-run block | as printed | VERBATIM-OK |
| zp-11-real-sidecar-witness.md | ladder PASS timings | in-receipt verbatim | as printed | VERBATIM-OK |
| zp-11-uds-transit-floor.md | trip p50 25,875–27,083 ns band; residual 15,899 ns; floor 9,064 ns | in-receipt verbatim | 26,959−9,708−1,352.1 = 15,898.9 → 15,899 ✓; floor 15,750−5,334−1,352.1 = 9,063.9 → 9,064 ✓ | VERBATIM-OK (standing UNVERIFIED per its own load disclosure) |
| zp-12-supervisor.md | 16 courts PASS 16.413s | in-receipt PASS list | as printed | VERBATIM-OK |
| zp-13-blake3-abb.md | 4/4 + 11/11 PASS timings | in-receipt | as printed | VERBATIM-OK |
| r-1-le-ruling.md | court 2.083s PASS | in-receipt | as printed | VERBATIM-OK |

Summary: **0 MISMATCH** across all numbered receipts; 2 VERBATIM-OK-but-untracked;
1 in-receipt NO-RAW-LOG sub-cell (BM-8 first-pass 25.5 µs); volatile-`/tmp`
source-of-record flags on bm3/bm7/zp-17 (all carry in-receipt verbatim blocks,
so not NO-RAW-LOG, but the committed source of record for the /tmp logs is the
verbatim block itself).

## 2. Cross-check appearance table (receipt × value × as-of)

### 1.26 µs-class (cgap-ffi RingReceiptChain 50k-span, µs/link = ms/50)

| receipt | value | as-of conditions |
|---|---|---|
| zp-17-gate10-settling | 1.26394 min-of-7 (63.197 ms) | release, POST-FOLD (per ERRATA-2), load 3 other lanes, working tree, `/tmp/zp17_runs2.log` |
| bm3-chain-scale-v2 | 1.26660 (63.330 ms), band ±0.33% | release, min-of-3 internal, load 19–27, same working tree as zp-17 |
| bm7-gate10-head-reconfirm | 1.27858 (63.929 ms), +0.49% above band, NOT CONFIRMED | release, working tree (committed HEAD compile-aborts), load 44–48 |
| lane3-linearity-certificate | 1.28068 (64.034 ms), +0.65% above band | release, --test-threads=1, HEAD 23436067 + tree, load 26 |
| br2-linearity-fence-form | 1.2602 mean-of-5 (63.090–63.853), max dev 0.127% | release, 5 invocations, load 37.6–47.8, base ede3fe4d + tree |
| ggen v4 row 5 | cites 1.26394 (zp-17) | consistent with zp-17 as corrected |

Consistency: all five samples are one law (1.26–1.28 µs/link, release,
min-of-N, same plane). The only cross-cite law violated would be citing any
of these onto cgap-aoq AoqChain (0.291 µs/link, a different plane) or arcus
(Go, unmeasured) — no receipt in scope does so; every receipt carries the
plane-law block.

**Committed-state caveat (ledger-wide)**: BM-7 proved the committed HEADs
(3e0339e6, b595f276) compile-abort on cgap-aoq; at this receipt's write time
HEAD is e251f258 and `aead.rs`/`session.rs`/`generated_aoq_header_layout.rs`
remain uncommitted working-tree modifications. Every 1.26-class number above
therefore measures uncommitted ground.

### 25.0 µs-class (UDS transit trip p50, membrane plane)

| receipt | value | as-of conditions |
|---|---|---|
| zp-11-uds-transit-floor | 25.875–27.083 µs band (3 runs), residual 15.9–16.6 µs | dev+release sidecar, load 10–22, standing UNVERIFIED by its own disclosure |
| bm8 first-pass (in-receipt, unpreserved) | 25.458 µs | load 36, same SHA as committed pass; transcript not on disk |
| bm8 committed transcript | 52.750 µs min-of-3 p50 | load 45.7, HEAD 9661bbf, sidecar d6fc77eb |
| lane3-queue-divergence | 24.333 µs (24,333 ns) | arcus 4dd6e39e, load 26, BUSY-retry present |
| L4 (lane, NO receipt on disk) | "25.0 µs disclosure" | cited by bm8; no receipt path exists in cgap/arcus/ggen v26.11.3–4 receipts — session-only claim |

Consistency: the sub-30 µs cells (zp-11, BM-8 first-pass, lane3, L4) agree
within a ~2× load envelope; BM-8's committed 52.8 µs is the same code at
higher load — receipt states this correctly. **The L4 25.0 µs figure is the
only cross-cite with NO generating receipt on disk.**

### 55.7× vs 35× (legacy-JSON ÷ AOQ1 codec, ratio)

| receipt | value | as-of conditions |
|---|---|---|
| ggen v4-corrected-claims-ledger row 9 | "~35x" | 2026-11-04 earlier session; median 6033 ns/op vs codec 42–45 ns/header — ratio derived loosely, pairing not stated |
| lane3-queue-divergence-certificate | **55.7×** (5140/92.3, paired same-host same-session) | today's lawful pairing; explicitly states the historical 35× does not reproduce |

Verdict: **35× is REFUTED-as-stated** by lane3-B; the lawful ratio is 55.7×
under the declared pairing (legacy 5140 ns/op ÷ AOQ1 encode+decode 92.3 ns).
Any later citation of 35× must be re-pointed to lane3-B.

### 440 B/op, 6 allocs (Go legacy-JSON membrane)

| receipt | value | as-of conditions |
|---|---|---|
| bm5-legacy-baseline | 440 B/op, 6 allocs (both runs) | darwin/arm64, -count=2 |
| lane3-queue-divergence | 440 B/op, 6 allocs (both runs) | later run, load 26 |
| ggen v4 row 9 | 440 B/op, 6 allocs (4 of 4 runs) | count=5 session |

Consistency: 440 B/op is invariant across all three sessions (10 printed
observations, zero variance). ALIVE as the membrane alloc profile.

### 9 allocs/warmed Seal (Go boundary round trip)

| receipt | value | as-of conditions |
|---|---|---|
| bm8 leg 1 | 9 allocs, budget 9, test SKIPs after recording | HEAD 9661bbf, pre-C4 |
| ggen v4 row 8 | 0 B/op — scope law: refusal paths + Rust windows only, NOT the Go boundary | cross-check consistent (different plane) |

### BLAKE3 64 B p50

| receipt | value | as-of |
|---|---|---|
| bm2 | 125.0 ns p50, min 83.0 | release, n=10000×3, min-of-N |
| bm9 (divan) | 153.1 ns fastest / 158.3 median | bench profile, loaded host, 100 samples — different method, not a contradiction |
| ggen v4 row 1/2 | honest band 83 min / 125–167 p50 | consistent |

### Other corrected chains checked

| chain | state |
|---|---|
| zp-17 PRE-FOLD → ERRATA → ERRATA-2 (POST-FOLD, commit 8ccb43d0) | both receipts carrying the number (zp-17, ggen v4 rows 5/5b/5b-precision) carry the corrected label; no in-scope receipt cites the inverted label |
| OE-3 flatness: lane3-A FAIL(honest) 1.884% → br2 diagnosis: measurement noise, min-of-5 max dev 0.127% PASSES ±0.5% fence | lane3-A's FAIL verdict is superseded-as-diagnosis by br2 (untracked); the 1.884% number itself is not corrected — the interpretation is. Fence form: keep flat per-link fence, not residual-vs-model |
| Teachability chain 45/45 → 44/45 → 45/45 | **no v26.11.4 receipt in cgap or arcus cites this chain** — grep-verified (hits exist only in v26.11.1/v26.11.2 dflss/lane docs); nothing in ledger scope cites a stale teachability number |
| BM-2 "prior 2.515x" vs PL-X5 landing 2.515x | consistent, same generating run |
| zp-18 census 73/64/11 vs BM-9 census 76/67/12 | both as-of-session; gate grew between runs (BM-9 later); not a correction |

## 3. Consolidated metric ledger

| metric | current lawful value | generating command | receipt path | standing | as-of conditions |
|---|---|---|---|---|---|
| cgap-ffi chain-scale 50k-span per-link | 1.26394 µs/link min-of-7 (band 1.26394–1.27238; later samples up to 1.28068) | `cargo test --release -p cgap-ffi --test receipt_chain_scale -- --nocapture` | /Users/sac/cgap/docs/jira/v26.11.4/receipts/zp-17-gate10-settling.md (ERRATA-2 applied) | **UNVERIFIED-COMMITTED-STATE-PENDING** — committed HEAD compile-aborts (BM-7); all samples measure the uncommitted working tree | release, POST-FOLD, loaded shared host |
| chain-scale flatness | per-link cost flat, max dev 0.127% at min-of-5 | same command ×5 | br2-linearity-fence-form.md (untracked) | PARTIAL (untracked receipt; replay command is the evidence) | load 37–48 |
| cgap-ffi 100k census | append 184.1 ms / verify_all 169.3 ms min-of-3 | same command | bm3-chain-scale-v2.md | PARTIAL (working-tree subject) | release, load 19–27 |
| BLAKE3 64 B p50 | 125.0 ns (min 83.0) | `cargo test -p cgap-blake3 --release --test percall_latency` | bm2-crypto-kernels.md | ALIVE | release, load 7.63 |
| BLAKE3 64 KiB p50 | 32,750 ns | same | bm2 | ALIVE | release |
| BLAKE3 batch speedup N=64×1KiB | 2.480× (band 2.466–2.515 across sessions) | `cargo test -p cgap-blake3 --release --test independent_batch_perf` | bm2 + pl-x5 | ALIVE | release |
| ML-KEM-768 keygen/encaps/decaps | 25.50/28.44/31.33 µs (first baseline); divan 29.41/32.7/36.24 µs loaded | `cargo test -p cgap-crypto --release --test mlkem768_bench`; `cargo bench -p cgap-crypto --bench primitives` | bm2 + bm9 | ALIVE (first-sample, band widening under load) | release/bench |
| ML-DSA-65 P1 kernels | ntt 733.9 ns, intt_tomont 1.025 µs, pointwise 193.5 ns | `cargo bench -p cgap-crypto --bench primitives` | bm9 | ALIVE (P1 kernel only, no sign/verify) | bench, loaded |
| AOQ1 codec per header | 47.8 enc / 44.5 dec amortized (lane3); 44.5/41.5 (v4, earlier session); per-call 42.0/42.0 | `cargo test --release -p cgap-aoq --test aoq1_bench_court` | lane3-queue-divergence (raw log committed) | ALIVE | release, load 26 |
| soak fps (real UDS echo, v3 law) | 55,411 fps / 18.047 s / 1M trips | `cargo test --release -p cgap-ffi --test aoq_sidecar_soak_1m_court -- --test-threads=1` | bm4-soak-churn.md | PARTIAL — working-tree subject; soak-court re-pin uncommitted | release, load 37 |
| Go legacy-JSON membrane | 4531–4564 ns/op, 440 B/op, 6 allocs (band across sessions 4531–6399 ns/op) | `go test ./internal/arcus/adapter -run '^$' -bench BenchmarkLegacyJSONUnmarshal -benchmem` | bm5-legacy-baseline.md | ALIVE | darwin/arm64, 1s benchtime |
| legacy:AOQ1 ratio | **55.7×** (35× refuted-as-stated) | paired: legacy bench + aoq1_bench_court same host/session | lane3-queue-divergence-certificate.md | ALIVE (supersedes ggen v4 row 9's ~35×) | paired same-host, load 26 |
| UDS transit trip-256B p50 | 24.3–27.1 µs under disclosed load; loaded envelope bounded at 52.8 µs; residual-socket 15.9–16.6 µs | `go test ./internal/arcus/flight -run TestUdsTransitProbeRealSidecar -v` | zp-11-uds-transit-floor.md + bm8 + lane3 transit2 log | UNVERIFIED for production citation (zp-11's own standing; no quiet-machine run in scope) | load 10–48 across samples |
| Go boundary alloc baseline | 9 allocs/warmed Seal (budget 9 until C4/ZP-19) | `go test ./internal/arcus/flight -run TestAoqRoundTripAllocBaseline` | bm8-pre-alloc-baseline.md | ALIVE (baseline recorder, pre-C4 at 9661bbf) | load 36–45.7 |
| supervisor fd churn | delta 0 over 8 generations | `go test ./internal/arcus/flight -run TestSupervisorChurnFDCourt` | bm4-churn.md (untracked) + pl-x2 | ALIVE (receipt untracked) | load 37 |
| profile-guard census | 76 TIMING / 67 guarded / 12 allowlist (latest); was 73/64/11 earlier | `python3 scripts/profile_guard_census.py` | bm9 (latest) + zp-18 | ALIVE (counts are as-of-run, monotone growth) | release tree |
| purity gate | 10 passed / 0 failed | `cargo test -p cgap-aoq --test purity_gate` | zp-18b | ALIVE | dev profile |
| OE-3 flatness fence | fence form: flat per-link ±0.5% at min-of-N≥5 (NOT residual-vs-model) | 5× receipt_chain_scale | br2 | PARTIAL (untracked receipt) | load 37–48 |
| push-state (6 repos) | snapshot as-of T2 21:04:33Z; ggen + smalt HEAD unpushed at T2 | `git ls-remote` gate | l7-push-state-sweep.md | as-of only (stale by construction) | — |

## 4. Flags

1. **UNVERIFIED-COMMITTED-STATE-PENDING (ledger-wide)**: every chain-scale
   (1.26-class) number and the v3-law soak number measure uncommitted
   working-tree ground. BM-7's compile-abort evidence at 3e0339e6/b595f276
   plus this session's observation that `aead.rs`, `session.rs`,
   `generated_aoq_header_layout.rs` remain modified at HEAD e251f258 means no
   committed-HEAD reconfirm exists for any of them. Close condition: C1's
   aead/session commits land, then one archive-of-HEAD `receipt_chain_scale`
   run.
2. **Untracked receipts** (violating untracked-evidence law):
   `/Users/sac/cgap/docs/jira/v26.11.4/receipts/br2-linearity-fence-form.md`
   and `/Users/sac/arcus/docs/jira/v26.11.4/receipts/bm4-churn.md`. Both are
   VERBATIM-OK on content; they must be committed in their owning repos.
   This ledger commits the cgap one only (arcus is read-only for this lane) —
   the arcus bm4-churn.md commit remains outstanding.
3. **NO-RAW-LOG cross-cite**: BM-8's citation of "L4's 25.0 µs disclosure"
   has no generating receipt on disk in any repo (grep over cgap, arcus, ggen
   v26.11.3/v26.11.4 receipts). Session-only claim — do not cite forward
   without a receipt. BM-8's own first-pass 25.458 µs cell is likewise
   unpreserved (disclosed in-source).
4. **35× → 55.7×**: ggen v4 row 9's "~35x" is refuted-as-stated by
   lane3-queue-divergence-certificate's paired 55.7×. Consumers must re-point
   to lane3-B. (The 35× never appeared in a cgap/arcus v26.11.4 receipt as a
   standalone claim — the refutation and the refuted claim are both
   correctly paired across repos.)
5. **Teachability chain**: the 45/45 → 44/45 → 45/45 curriculum numbers are
   cited by NO v26.11.4 receipt in scope; no stale-cite flag fires.
6. **Volatile source-of-record**: bm3, bm7, zp-17 name `/tmp` logs as source
   of record; the in-receipt verbatim blocks are the only committed evidence.
   lane3's certificates did this correctly (committed raw-lane3/ logs,
   byte-matched this session).
7. Zero arithmetic MISMATCH found: every derived quantity in every numbered
   receipt re-derives from its cited raw numbers.

## See Also

zp-17-gate10-settling.md · bm3-chain-scale-v2.md · bm7-gate10-head-reconfirm.md ·
lane3-linearity-certificate.md · lane3-queue-divergence-certificate.md ·
br2-linearity-fence-form.md · bm8-pre-alloc-baseline.md ·
ggen v4-corrected-claims-ledger.md
