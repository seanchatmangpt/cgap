# 11-measured-claims

Exclusive numeric source for position-paper-v2. Every MEASURED number this
campaign (v26.11.2 starDF) is consolidated here with claim, number, unit,
method, receipt cite (file:line), and class. Scope annotations are part of
a claim: a number measured on a quiet host is NOT valid unqualified
(receipt `redteam/red-10-timing-jitter-exploit.md`, 2026-11-02). **No number may appear in the
paper that is absent from this table.** Method of record:
`docs/jira/v26.11.2/06-bench-protocol.md` §6 (MW4 like-for-like) and §7
(unit discipline: GB/s never a multiple of facts/s).

Consolidated by lane W4-D3, 2026-11-01. Receipt:
`waves/w4-d3-claims-table.md`.

## §1 Measured claims table

Class key: MEASURED (command + exit witnessed in the cited receipt this
campaign), REFUTED-ATTRIBUTION (measured, but the plan's causal attribution
failed contact with measurement), TARGET (stated target, not yet met or
unmeasured by the method of record), UNSUPPORTED (claimed on disk, no
measured referent exists; never quote as a number).

| # | Claim | Number | Unit | Method | Receipt cite | Class |
|---|---|---|---|---|---|---|
| 1 | CLI six-pack regen byte-identical replay: 5-surface digest sets identical across baseline/run1/run2 | 5 surfaces, 3 states identical | surfaces x states | 2x `ggen sync run` + `manifest_patch.py`, sha256 + `cmp` digest sets | waves/w1-02-cli-regen-replay.md:19-29 (addendum table) | MEASURED |
| 2 | Root projection replay determinism: 6 generated outputs byte-identical across 4 sync runs; graph hash stable | 6 outputs, 4 runs, `cmp` byte-identical | outputs x runs | `ggen sync run` x4 + sha256 A/B + `make gate0` (`-std=c99 -ffreestanding`) | waves/w1-03-root-projection-replay.md:14-22,39 | MEASURED |
| 3 | Root workspace build green post-metadata-repair | exit 0, dev profile 1.84 s | s | `cargo build --workspace` (lane target /tmp/w2-09-target) | waves/w2-09-build-baseline.md:14 | MEASURED |
| 4 | cgap-court verdict harness green | 14 passed / 0 failed / 0 ignored in 26.05 s | tests / s | `cargo test -p cgap-court` (court_verdict.rs) | waves/w2-09-build-baseline.md:15 | MEASURED |
| 5 | L1 two-stage ingress escalation: full chain ns/frame — QUIET HOST ONLY (divan fastest/median columns; hostile-environment behavior is row 28) | 10.20 fastest / 10.30 median | ns/frame | divan 0.1.21, 1,048,576-frame const, 100 samples, fastest+median columns (06-bench-protocol §1); quiet-host scope re-confirmed: red-10 baseline reproduced within 4.5% (10.66/11.76) | waves/w3-l1-ingress.md:63,69-70; redteam/red-10-timing-jitter-exploit.md:30-41 | MEASURED (scoped) |
| 6 | L1 speedup vs P3-16 baseline (102.7 ns/frame) | 10.1x (10.20 vs 102.7) | ratio | divan fastest column, same method as P3-16 baseline; per-frame = ms*1000/1,048,576 | waves/w3-l1-ingress.md:70 | MEASURED |
| 7 | L1 penstock admission stage | 7.71 | ns/frame | divan fastest, same run; dispatch's 5.70 ns row recorded as NOT reproduced on this host | waves/w3-l1-ingress.md:64,71-72 | MEASURED |
| 8 | L1 <15 ns/frame target — MET ON QUIET HOST; NOT robust under contention (median 19.03 under combined load, row 28) | MET (10.20 < 15) | ns/frame | same run as #5 | waves/w3-l1-ingress.md:69; redteam/red-10-timing-jitter-exploit.md:87-88 | MEASURED (target met, scoped) |
| 9 | L2 SWAR FNV: parquet chain did NOT move vs 41.54 ms baseline | 42.58-43.1 ms (file 42.82f/43.1m; ram 42.58f/42.74m) | ms @1,048,576 | divan, chains-parquet, 100 samples | waves/w3-l2-swar-fnv.md:26-28 | MEASURED |
| 10 | L2 <10 ms @1M chain target | NOT MET (42.7-43.1 ms) | ms | same run as #9 | waves/w3-l2-swar-fnv.md:61 | MEASURED (target failed) |
| 11 | L2 bottleneck attribution (FNV dominant chain share) | refuted: chain dominated by encode/decode; FNV share small | — | chain decomposition vs baseline 41.54 ms | waves/w3-l2-swar-fnv.md:61-63 | REFUTED-ATTRIBUTION |
| 12 | L5 fused ASC450-into-P28 row fold regresses vs unfused control | 3.04 vs 2.37 µs/row @65536 (fused 199.1 ms, unfused 155.5 ms) | µs/row | divan, chains-seal, 100 samples, same host/session/corpus; unfused control reproduces P3-19 baseline 2.37 | waves/w3-l5-fused.md:38-42,63 | MEASURED (target <1.45 µs/row FAILED-BY-MEASUREMENT) |
| 13 | L5 bottleneck attribution ("redundant span walk") | refuted: meter-stage cost is the emission crypto (sealed-state mint), not traversal | — | PARETO decomposition + kernel latch-law analysis | waves/w3-l5-fused.md:63-66 | REFUTED-ATTRIBUTION |
| 14 | Zero-indirection asm census (gate ingress, bn3 noalloc, p28 asc450 regions) | 0 blr / 0 br xN / 0 brk; direct `bl` only (na_unify bl=19, ingress 158-line body) | counts | `cargo rustc --release -- --emit asm` + python label-bounded regex census | waves/w3-t2-asm-census.md:23-29 | MEASURED (census, mutation-free by nature — not a court) |
| 15 | cgap-arrow audit-profile riscv32imc object within 32,768 B budget | 28,679 B total (text 28,354; 87.5% of budget); blake3::hash cgu 26,046 B | bytes | llvm-size -A over rcgu.o (audit profile, lto=false); NOT linked-ELF | waves/w3-t3-silicon.md:36,50 | MEASURED (pre-link OBJECT; ELF subject not claimed) |
| 16 | alloc/panic purity census, arrow | 0 defined alloc, 0 defined panic, 0 refs | counts | rust-toolchain llvm-nm over rcgu.o | waves/w3-t3-silicon.md:56-62 | MEASURED |
| 17 | blake3/parquet riscv lib objects (first baselines) | blake3 114,072 B; parquet 55,711 B (both OVER 32,768 as pre-link lib totals) | bytes | llvm-size -A, audit profile | waves/w3-t3-silicon.md:37-38 | MEASURED (non-authoritative subject class) |
| 18 | blake3/parquet census: 0 DEFINED alloc/panic symbols | 0/0 defined; refs only (blake3 1 U; parquet alloc=22 U, panic=15 U) | counts | llvm-nm | waves/w3-t3-silicon.md:56-62 | MEASURED |
| 19 | cgap-pack re-qualification post-G1K | 14/14 gates green, 14/14 flip battery, 0 vacuous | gates | witness courts + rdflib flip battery + blake3 lock-hash recompute (54 files) | waves/w3-t5-pack-requal.md:20,74-75 | MEASURED |
| 20 | ggen.lock content_hash reproduction | blake3:cc8c83d3…671986 over 54 files | digest | independent recompute (blake3 unframed chain, byte-wise sort) | waves/w3-t5-pack-requal.md:50 | MEASURED |
| 21 | Hygiene window two: credential sweep | 0 hits over 2459 insertions + 70 untracked paths | hits | grep over `git diff HEAD` + untracked | waves/w3-t7-hygiene.md:11-12 | MEASURED |
| 22 | Seed-conflation spot-check | PASS (bitslice.rs:687-699 pair intact) | — | grep both frozen constants over *.rs | waves/w3-t7-hygiene.md:13-14,59 | MEASURED |
| 23 | L2 SWAR FNV flip court (M-B shift corruption) | court FAILED exit 101 on mutation; sha256-exact restore re-green | 1 flip / 1 court | /tmp sandbox mutation `(w>>24)`→`(w>>25)`, equity court | waves/w3-l2-swar-fnv.md:38-42 | MEASURED (court flip) |
| 24 | L5 flip court (W3L5-DEMOTE) | exit 101, proven==N/2 fires (511 vs 512); root+checksum diverge | 1 flip | sandbox mutation, trace-patch baseline | waves/w3-l5-fused.md:44-52 | MEASURED (court flip) |
| 25 | L1 flip court (escalate-always-true) | court FAILED (2 passed/1 failed) on mutant; sha256-exact restore | 1 flip | /tmp sandbox injection `view.escalate = true` | waves/w3-l1-ingress.md:75-83 | MEASURED (court flip) |
| 26 | MW4 (RDFox-comparative) closure throughput | any figure | facts/s/core | 06-bench-protocol §6: RDFox published envelope only (2-6.1M facts/s), RDFox NOT executed; G3-MAT-BENCH >50M facts/s/core is [TARGET-v26.11.2] | 06-bench-protocol.md:109-120; 00-ard-prd.md:59-60 | TARGET |
| 27 | W4 perf/sovereign lanes (w4-p1/p2/p3/mw2/k1) | not consolidated | — | bounded 60 s poll for waves/w4-* receipts: none present at consolidation time | this receipt §3 | TARGET (loud skip) |
| 28 | L1 ingress under combined cache(bw6)+CPU(cpu9) contention — REGISTERED DIVERGENCE of row 5's quiet-host scope | 14.71 / 19.03 / 124.06 fastest/median/slowest @1M (slowest = 12.2x row 5 claim 10.20; >50 ns bar CROSSED) | ns/frame | divan same method as row 5, only environment differs: /tmp load generators (bw: LLC/DRAM saturation, 4x512MiB streams; cpu: 15 issue-port/branch threads); Mac15,9 16-core, no SMT (PortSmash channel structurally absent); bw-only degraded fastest +14.8%, cpu-only left fastest intact (-3.4%) | redteam/red-10-timing-jitter-exploit.md:43-66,77-88 | MEASURED (registered divergence) |
| 29 | "Constant-time" ingress/header validation (PRD wording) | no number — UNSUPPORTED on disk | — | term appears in NO measured claim; PRD prose only (12-ard-prd-cgap.md:49, audit/cgapprd-prd-v2.md:82); the validation-without-branches dispatch item already REJECTED per BR3 (branches are typed refusals) | hardening/lane-11-arrow-swar-validate.md:85-93; redteam/red-10-timing-jitter-exploit.md:15-26 | UNSUPPORTED |

## §2 RDFox-comparative classification

Per 06-bench-protocol §6 + §7 (enforced rule), every RDFox-comparative row:

| Row family | Class | Basis |
|---|---|---|
| Scan/ingress GB/s numbers (e.g. #5-#9) vs RDFox facts/s | CATEGORY-ERROR if compared | §7 unit law: GB/s is never a multiple of facts/s; ns/frame vs facts/s are different counters (frames vs facts). No campaign row converts them. |
| MW4 closure facts/s vs RDFox 2-6.1M facts/s | LIKE-FOR-LIKE-ELIGIBLE but UNVERIFIABLE-BY-ABSENCE today | §6: identical workload shape required; RDFox never executed in this campaign — comparison arm is a published envelope citation, cgap arm unmeasured (MW4 receipt absent). Both arms must be named measured-vs-published; today the cgap arm is [TARGET]. |
| #6 10.1x vs RDFox | CATEGORY-ERROR if compared | 10.1x is ns/frame ratio vs our own P3-16 baseline, not vs RDFox facts/s. |
| g20 incumbent headline numbers (ggen, others) | UNVERIFIABLE-BY-ABSENCE | 06-bench-protocol header: claimed-unreceipted, workload-mismatched; none migrate into this campaign. |

No MEASURED row in §1 is RDFox-comparative today: the campaign has zero
measured facts/s figures, so no RDFox comparison can be LIKE-FOR-LIKE yet.
Rows #5-#10 are frame/byte-counter rows and are CATEGORY-ERROR if ever
quoted against RDFox facts/s.

## §3 Loud skips and divergences

- [SKIP] w4-p1/w4-p2/w4-p3/w4-mw2/w4-k1: bounded 60 s poll for
  `waves/w4-*` receipts returned none (deadline exhausted). These lanes
  had not filed at consolidation time; their numbers are absent from this
  table BY ABSENCE, not skipped silently. If they land later, a W4-D3
  addendum re-runs the sweep.
- Dispatch named `docs/jira/v26.11.2/06-bench-protocol.md` §6 as
  method-of-record cite; that file exists and §6 is the MW4
  like-for-like method (lines 109-120), §7 the unit law (GB/s never a
  multiple of facts/s). Cited throughout §1/§2.
- w1-02's digest evidence was originally /tmp-parked (W1-17 audit D5);
  the embedded addendum table (lines 19-29 + addendum) is the cite
  used here.
- w3-t2's asm census is mutation-free by nature — a census, not a court;
  its non-vacuity story is the bounded label-regex counting method,
  stated in the receipt. No vacuity claim is made beyond the named
  regions.

## §4 Consolidation addendum — M-CTQ stations (lane R-12, 2026-11-02)

Source receipt: `lanes/lane-xc04-mctq-consolidation.md` (bench
`cargo bench -p bench --bench mctq_consolidation`, exit 0 twice this
session). Method of record: csct N=7 Min/Median/Max
(`dflss/03-msa-gauge.md` §2; no p99 — max-of-7 is not a quantile,
FIX-04 law), batch-amortized windows (41 ns timer floor). Class:
**consolidation-addendum** — numbers are receipted but the
cross-instrument legs remain UNVERIFIED until re-measured on the wire
instruments (cgo/Go harness); not full MEASURED promotion. §2's
"zero measured facts/s" statement predates this addendum: station 1
below is the campaign's first receipted facts/s figure, and it is
NOT RDFox-comparative (different window scale, see divergence D2).

| # | Claim | Number (run1 / run2) | Unit | Method | Receipt cite | Class |
|---|---|---|---|---|---|---|
| 30 | M-CTQ-04 seal throughput (80 B lease frames, real blake3 hash, batch-1365) | min 3,529,029 med 3,824,869 max 3,846,425 / min 4,069,561 med 4,130,106 max 4,135,837 | receipts/s | csct N=7 Min/Med/Max (03-msa-gauge §2), FACT_WINDOW=1,365, batch-amortized, contention guard quiet (1.02/0.93 ns/64B-line) | lane-xc04-mctq-consolidation.md:36-37 | consolidation-addendum (D2: window 1,365 vs b06's 1M — not b06-comparable) |
| 31 | M-CTQ-05 admit transition cost, IN-PROCESS extern "C" into real `cgap_admit_frame`, REAL 152 B A04 frames, batch-1365 | min 123.81 med 124.02 max 125.12 / min 112.48 med 113.37 max 114.44 | ns/frame | same N=7 method; positive control flipped (0x43470002 OABI_MISMATCH before fixture fix, CGAP_OK after) | lane-xc04-mctq-consolidation.md:51-52 | consolidation-addendum (D1: NOT cgo — ipc-03's 85 ns membrane floor remains the wire-level authority; ipc-03 saw 183-187 ns/frame through cgo) |
| 31a | M-CTQ-05 null-refusal floor, in-process route | 0.38-0.41 | ns/call | same N=7 method; prices kernel-side short-circuit only (folds hard in-process) | lane-xc04-mctq-consolidation.md:47-49 | consolidation-addendum (D1: cgo floor 85 ns stands) |
| 32 | M-CTQ-06 mmap zero-copy read (producer RW MAP_SHARED, independent consumer PROT_READ, "CGAPSHM1", 1,365 facts @24 B) | min 0.15 med 0.18 max 1.56 / min 0.15 med 0.15 max 1.47 | ns/op | same N=7 method; anti-vacuity flip witnessed both runs (cross-mapping corruption visible, byte-exact restore) | lane-xc04-mctq-consolidation.md:66-67 | consolidation-addendum (D3: Rust L1-hot loop vs ipc-04 Go harness 4.52 ns/op — different instrument, same mechanism; run-1 max is page-table warmup) |

### 4.1 Disclosed instrument divergences (verbatim carriers)

- **D1 (station 2):** measured route is IN-PROCESS extern "C" via the
  cgap-ffi rlib — NOT the cdylib/cgo route; ipc-03's 85 ns membrane
  floor prices the cgo trampoline this harness does not, and remains
  the wire-level authority. The ~70 ns delta vs ipc-03's 183-187
  ns/frame is the in-process-vs-cgo membrane delta.
- **D2 (station 1):** window 1,365 receipts/run (FACT_WINDOW,
  pack_demux.rs:51) vs b06's 1M-receipt chain; run-1 median sat near
  the b06 1M floor, run-2 at b06's 4.3M — the figures are not the
  same instrument scale; consolidate with the disclosure, never
  silently.
- **D3 (station 3):** Rust L1-hot zero-copy loop (32,760 B working
  set) vs ipc-04's Go-harness read call included in its 4.52 ns/op —
  different instrument, same mechanism.
- Cross-instrument legs (cgo admit, Go-harness mmap read, 1M-window
  seal) remain UNVERIFIED until re-measured on the wire instruments.
  Quiet-host scope of any bench-cited figure is enforced by the
  CONTENTION-SUSPECT control (control plan §5.2,
  `dflss/06-control-plan.md:143-157`; landed by
  `lanes/lane-r-adv09fix-contention-doc-wiring.md:25-26`).


## §5 Optimization-discovery addendum (lane DOD-08, 2026-11-02)

Discoveries from the OPT-01..05 optimization fan-out (session reply-back
termini; OPT-03 has an on-disk receipt,
`bench-validate/opt-03-channel-curve.md`). Every row is a TARGET class
(v26.11.3-candidate): no number here has been measured by the method of
record; nothing in §5 may be quoted as a measured figure. These rows are
non-blocking for v26.11.2 ship — each is recorded to route the next
campaign, not to gate this one.

| # | Discovery | Cite (verified on disk this session) | Candidate class | Why non-blocking for 26.11.2 |
|---|---|---|---|---|
| 33 | Egress-ring false-sharing hazard: `head`/`tail`/`quota` co-reside on one cache line; producer tail-loads contend with serve-side counter writes | `crates/cgap-git/src/egress_ring.rs:421-425` (struct fields); fix pattern exists: `PaddedIndex` `#[repr(align(64))]`, `crates/cgap-git/src/ingress_ring.rs:226-242` | TARGET (perf, v26.11.3-candidate): pad egress indices to 64 B; expected effect: tail-latency variance under concurrent serve, not throughput | Single-accessor `&mut self` law means no live contention in shipped SoC court paths; hazard only materializes under a multi-handle egress design not yet built |
| 34 | SWAR digest fixed-trip fold candidates: `digest_all_zero` byte loop early-exits per nonzero BYTE, not per chunk; `swar_all_zero_vl` already ships a chunk-exit form | `crates/cgap-gate/src/penstock.rs:548-556` (byte loop); `crates/cgap-branchless/src/penstock.rs:64` (`swar_all_zero_vl`); equivalence court `:149-167` (vs `is_zero_at` oracle) | TARGET (perf): fold the gate-side loop to chunk granularity reusing the already-oracle-proven SWAR kernel; zero new equivalence risk | L1 chain is at 10.20 ns/frame quiet-host (row 5) — this path is not the dominant segment; fold pays only if row-28 tail isolation lands first |
| 35 | Commit-tx syscall amortization: identity-key memoization of the per-tx open/spill sequence (rename-safe by construction — key includes the digest, so a re-open after rename cannot alias) | `crates/cgap-storage/src/atomic_commit.rs:696` (durability fence models fsync); discovery estimate ~9-12 syscalls ≈ 10-25 us/tx (UNMEASURED — session estimate, not a bench receipt) | TARGET (perf): count syscalls with dtrace/ktrace first (measurement REQUIRED before any lever row) — estimate never rounds up to a claim | L4 already met <4 us/tx via RingSink (row re-measure A3: 0.589-0.603 us/tx, `waves/a3-ring-sink.md`); amortization is a residual-path candidate only |
| 36 | Ring-relay memcpy amplification: SoC relay path copies window bytes ~4x (ingress relay -> demux staging -> object closure -> egress projection); slice-view proposals P1 (demux borrows instead of copies) / P2 (relay passes span views) target 4 copies -> 1-2 | `crates/cgap-git/src/pack_demux.rs:288` (`copy_from_slice` staging copy); pipeline in `crates/cgap-git/tests/soc_full_chip_court.rs:12,286-306` | TARGET (perf): P1/P2 borrow/span refactor; allocation-neutral, correctness gated by existing demux typed-refusal courts (magic corrupt -> SubjectMissing, already flipped in SoC court) | All copies are in-court hot loops on a 16-core host; zero-copy already proven at the mmap station (row 32, 0.15-0.18 ns/op) — the chain-level win is unmeasured and unneeded for 26.11.2 gates |
| 37 | OPT-03 channel curve (MEASURED, on-disk receipt): dose-response attribution — TAIL knee, not bus knee; fastest column dose-invariant (10.67-12.18 ns/f, max +13.8% at bw6), tail degrades up to 4.1x (cpu16 slowest 52.65 ns/f); guard blindness: max/min-ratio ceiling missed the common-mode bw6 slowdown (leg 4, 0 markers at +13.8%) | `bench-validate/opt-03-channel-curve.md` (7 legs, exit 0 x7, guard flipped 4x/4 mechanisms; hot-ambient scoped, loadavg1 6.65-15.43) | MEASURED (hot-ambient, scoped) + TARGET (guard third-channel: common-mode/uniform-shift detector, v26.11.3-candidate) | Attribution only — selects the optimization family (tail isolation); no ship gate reads this row; row 28 remains the registered divergence |


## §6 SRAM static-charge addendum (lane A-02, 2026-11-02)

Registers the 36,048 B SRAM number DOD-04 found UNREGISTERED in this
table (receipt `dod/dod-04-unsafe-sram-census.md:122-128`, the
divergence note). Scope law from that receipt §4: the 36,048 B figure
is the gated blake3+branchless kernel SLICE only — it is NOT a
whole-workspace SRAM figure and may never be quoted without its scope;
the bn3 typed refusal is the registered complement. Gate contract:
exit 0 pass / 1 typed refusal / 2 subject missing / 3 loud skip;
`--subject bn3` exercises the exit-1 leg.

| # | Claim | Number | Unit | Method | Receipt cite | Class |
|---|---|---|---|---|---|---|
| 38 | SRAM static charge, gated kernel slice (blake3+branchless freestanding caller, riscv32imc): .text+.rodata+.data+.bss families vs 131,072 B hard bound, headroom 95,024 B. Complement: `--subject bn3` typed refusal BN3_ENCLAVE_REFUSED at 6,520,984 B = 49.75x ceiling (red-12) | 36,048 | B | `python3 scripts/verify_cscb_sram.py` exit 0 + `--subject bn3` exit 1, both run this session (dod-04 receipt legs 1-2); gate = `scripts/verify_cscb_sram.py:45` (SRAM_BUDGET), gated subjects `:52-56`, bn3 red-12 comment `:57-60`; anti-vacuity: sram-bloat flip + bn3-pin flip witnessed (`lanes/lane-fix02-bn3-sram-gate.md:45-62`) | `dod/dod-04-unsafe-sram-census.md:72-89,112-120` | MEASURED (gated slice, subject-scoped; refusal leg exit 1 witnessed) |

### §6.1 Microbenchmark consolidation addendum (lane PROMO-01, 2026-11-02)

Promotes the two register-absent v26.11.1 microbenchmark figures flagged
by FS-02 (`bench-validate/bench-03-drift-check.md:129-141`: NEITHER
figure had an 11-measured-claims row). Both are now doubly witnessed
in-campaign. Exact-subject law per FS-02 (:198): these are NOT
chains-bn3 legs — the 82.24 GB/s subject is `--bench bn3`
`statement_scan_24b@65536` and the ~460 ns subject is `--bench seal`
`p28_attestation_verify@1024`. Ambient: neither run held a quiet-host
marker — F-06 hot ambient (loadavg snapshots per leg), FS-02 drift
check loadavg1 5.78-8.91 — figures are hot-ambient-scoped, verdicts
rest on cross-lane agreement per the chains-seal/bn3 no-guard
disclosure. Compendium §6.3 quote sites (`REVENUE_GENERATION_COMPENDIUM.md:115,263,319,345`)
carry their quiet-host / UNVERIFIED-WITHOUT-METHOD qualifiers
(FS-13FIX closed); this register row is the v26.11.2 method-of-record
carrier.

| # | Claim | Number | Unit | Method | Receipt cite | Class |
|---|---|---|---|---|---|---|
| 39 | BN3 statement_scan_24b @65536 (v26.11.1 dossier 82.24 GB/s): CONFIRMED-IN-BAND twice independently — F-06 80.49 GB/s (-2.1%); FS-02 drift check 19.12 µs fastest = exact 82.24 reproduction; @1M companion 78.3 GB/s vs dossier 78.94 (-0.8%) | 82.24 (dossier) / 80.49 (F-06) / 82.24 (FS-02) | GB/s | `cargo bench -p bench --bench bn3 -- statement_scan_24b`, divan fastest column @65536, exit 0 both runs; NOT a chains-bn3 stage leg (FS-02 correction :198); ambient per §6.1 above | `bench-validate/bench-02-seal-bn3.md:83-85,103,118-119,147` (F-06); `bench-validate/bench-03-drift-check.md:107,129-135,159` (FS-02) | consolidation-addendum (MEASURED, hot-ambient scoped, doubly witnessed; exact subject stated) |
| 40 | P28 attestation verify @1024 (v26.11.1 dossier ~460 ns = 459.96): CONFIRMED-IN-BAND twice independently — F-06 462.3 ns (+0.51%); FS-02 458.0 ns (468.8 µs/1024, -0.4%) | ~460 (dossier 459.96) / 462.3 (F-06) / 458.0 (FS-02) | ns/attestation | `cargo bench -p bench --bench seal -- p28_attestation_verify`, divan fastest @1024, exit 0 both runs; NOT a chains-bn3 leg; ambient per §6.1 above | `bench-validate/bench-02-seal-bn3.md:90-92,105,121-123,148` (F-06); `bench-validate/bench-03-drift-check.md:109,138-141,160` (FS-02) | consolidation-addendum (MEASURED, hot-ambient scoped, doubly witnessed; exact subject stated) |

