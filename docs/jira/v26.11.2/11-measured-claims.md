# 11-measured-claims

Exclusive numeric source for position-paper-v2. Every MEASURED number this
campaign (v26.11.2 starDF) is consolidated here with claim, number, unit,
method, receipt cite (file:line), and class. **No number may appear in the
paper that is absent from this table.** Method of record:
`docs/jira/v26.11.2/06-bench-protocol.md` §6 (MW4 like-for-like) and §7
(unit discipline: GB/s never a multiple of facts/s).

Consolidated by lane W4-D3, 2026-11-01. Receipt:
`waves/w4-d3-claims-table.md`.

## §1 Measured claims table

Class key: MEASURED (command + exit witnessed in the cited receipt this
campaign), REFUTED-ATTRIBUTION (measured, but the plan's causal attribution
failed contact with measurement), TARGET (stated target, not yet met or
unmeasured by the method of record).

| # | Claim | Number | Unit | Method | Receipt cite | Class |
|---|---|---|---|---|---|---|
| 1 | CLI six-pack regen byte-identical replay: 5-surface digest sets identical across baseline/run1/run2 | 5 surfaces, 3 states identical | surfaces x states | 2x `ggen sync run` + `manifest_patch.py`, sha256 + `cmp` digest sets | waves/w1-02-cli-regen-replay.md:19-29 (addendum table) | MEASURED |
| 2 | Root projection replay determinism: 6 generated outputs byte-identical across 4 sync runs; graph hash stable | 6 outputs, 4 runs, `cmp` byte-identical | outputs x runs | `ggen sync run` x4 + sha256 A/B + `make gate0` (`-std=c99 -ffreestanding`) | waves/w1-03-root-projection-replay.md:14-22,39 | MEASURED |
| 3 | Root workspace build green post-metadata-repair | exit 0, dev profile 1.84 s | s | `cargo build --workspace` (lane target /tmp/w2-09-target) | waves/w2-09-build-baseline.md:14 | MEASURED |
| 4 | cgap-court verdict harness green | 14 passed / 0 failed / 0 ignored in 26.05 s | tests / s | `cargo test -p cgap-court` (court_verdict.rs) | waves/w2-09-build-baseline.md:15 | MEASURED |
| 5 | L1 two-stage ingress escalation: full chain ns/frame | 10.20 fastest / 10.30 median | ns/frame | divan 0.1.21, 1,048,576-frame const, 100 samples, fastest+median columns (06-bench-protocol §1) | waves/w3-l1-ingress.md:63,69-70 | MEASURED |
| 6 | L1 speedup vs P3-16 baseline (102.7 ns/frame) | 10.1x (10.20 vs 102.7) | ratio | divan fastest column, same method as P3-16 baseline; per-frame = ms*1000/1,048,576 | waves/w3-l1-ingress.md:70 | MEASURED |
| 7 | L1 penstock admission stage | 7.71 | ns/frame | divan fastest, same run; dispatch's 5.70 ns row recorded as NOT reproduced on this host | waves/w3-l1-ingress.md:64,71-72 | MEASURED |
| 8 | L1 <15 ns/frame target | MET (10.20 < 15) | ns/frame | same run as #5 | waves/w3-l1-ingress.md:69 | MEASURED (target met) |
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
