# MEGA-REPORT-v26.11.2

Master synthesis of the v26.11.2 starDF mega-bench (lane MB12). Inputs:
`mega-bench/lane-01..11` receipts (read in full this session), plus
supporting receipts `waves/w5-l3-remeasure.md`, `waves/w6-budget-elf-profile.md`,
`waves/w6-parquet-seal-zero-alloc.md`, `waves/m1-footprint-rerun.md`,
charter `14-dflss-charter.md`, CTQ tree `15-ctq-tree.md`. No source
edits; no builds; reads only. AS-OF 2026-11-02.

Cite legend: `lane-NN:LINE` = `docs/jira/v26.11.2/mega-bench/
lane-NN-<slug>.md`; `BGP` = `waves/w6-budget-elf-profile.md`;
`P5` = `waves/w6-parquet-seal-zero-alloc.md`; `L3` =
`waves/w5-l3-remeasure.md`; `m1` = `waves/m1-footprint-rerun.md`.

## 1. Executive Dashboard

### Per-lane verdicts

1. **Ingress chain (MB1) — GREEN***: 10.65 ns/f fastest, 11.73
   median-of-fastests < 15 USL (lane-01:50-51). *Caveat*: divan
   per-run medians reached 15.28-30.47 under co-lane load
   (lane-01:52-55).
2. **Lease arithmetic (MB2) — GREEN**: 300.5M dec/s/core = 3.0x the
   100M bar; asm branchless witness (lane-02:34-40, 59).
3. **Datalog closure (MB4) — RED**: >=1M facts/s MET (min 5.77M,
   lane-04:28); zero-alloc REFUTED, 1,117 allocs / 1.20 GB
   (lane-04:64-70); G3 50M TARGET not met (lane-04:29).
4. **DRed churn (MB5) — YELLOW (relabelled, D16)**: 943.7M facts/s
   @90% is real for the bitmask retraction FILTER, entropy 1.0008 <
   4.0, flip exit 101 witnessed (lane-05:27-28, 41-45); the DRed-system
   claim fails the FA5 cascade probe — `dred::retract` (dred.rs:116-127)
   never follows `derived_deps` as lineage edges, N=10,000 root-deletion
   chain leaves 9,999 survivors where 0 is correct
   (adversarial/forensic-lane-05.md; errata
   waves/w6-dred-relabel-errata.md).
5. **Branchless census (MB6) — RED**: 16/19 PASS, 3 BRANCHY; court
   exit 1 is the correct verdict (lane-06:88-98); self-check proves
   non-vacuous (lane-06:15).
6. **Parquet seal (MB7) — GREEN**: seal 16.25 ms <= 30 ms; 0 allocs /
   0 B, P5 2->0 transition witnessed (lane-07:45-53, 84-85).
7. **Stats prune (MB8) — GREEN**: 0.9-2.1 ns/RG vs <=500 ns USL;
   pages_parsed warm delta 0 (lane-08:64-69, 104-107). Full suite:
   1 pre-existing other-lane fail, named (lane-08:113-114).
8. **Ring commit (MB9) — GREEN**: 0.755-1.001 us/tx < 4 us USL, ~5x
   margin (lane-09:30-32, 100); chain_full +17-34% drift disclosed.
9. **Enclave budget gate (MB10) — YELLOW**: envelope 36,810/131,072 B,
   mutant flipped (lane-10:29-30, 68); ONE-image claim [TARGET]
   (lane-10:69-73).
10. **Patent gate G7 (MB11) — GREEN**: 4/4 DISTINCT-MECHANISM, exit 0
    (lane-11:26-29); flip battery cited from w5/w6 (lane-11:63-68).
11. **(MB3) — MISSING**: no lane-03 receipt exists on disk in
    `mega-bench/` — loud, not silently collapsed.

MB1 is GREEN* not YELLOW: the USL is met on both fastest and
median-of-fastests; contention inflation is disclosed and the
quiet-host median band stays with w3-l1/w6-b7 (lane-01:52-55).

## 2. CTQ Flowdown Reconciliation

Measured vs USL/LSL (`14-dflss-charter.md`, `15-ctq-tree.md`).

### Met rows

| CTQ | USL/LSL | Measured | Cite |
|---|---|---|---|
| Frame ingress cost | USL 15 ns/frame | 10.65 fastest / 11.73 med-of-fastests | lane-01:50-51 |
| Commit cost | USL 4 us/tx | 0.755-1.001 us/tx | lane-09:30-32 |
| Deductive throughput | >=1M facts/s/core | 5.77-10.36M across buckets | lane-04:17-19, 28 |
| >=100M dec/s core | >=100M | 300.5M (3.0x) | lane-02:36-40 |
| Seal heap allocs | 0 (hard) | 0 / 0 B, divan column | lane-07:45, 85 |
| Binary footprint | USL 131,072 B | 39,062 B rlibs / 36,810 B linked | BGP:59; lane-10:68 |
| Spoliation pin | 0x5032_38FF exact | staged-const + abort | lane-01:66; lane-02:97-109 |
| Prune cost | <=500 ns/RG | 0.9-2.1 ns/RG | lane-08:64-67 |
| Seal latency @1M | <=30 ms | 16.25 ms | lane-07:60, 84 |

### NOT-met rows (stated loudly)

| CTQ | Spec | Measured | Cite |
|---|---|---|---|
| Closure zero-alloc | 0 allocs (S1 hard) | REFUTED: 1,117 allocs / ~1.20 GB @N=1M | lane-04:64-70 |
| Branchless census | 19/19 | 16/19 (3 BRANCHY) | lane-06:94-98 |
| G3 stretch | >50M facts/s/core | TARGET, not met (max 10.36M) | lane-04:29, 90-91 |
| Cpk | process-capability | UNVERIFIED — no N>=30 distributions | charter T2 OPEN |
| Shape taxonomy | linear/DAG/cycle | MISSING — chains-shape only | lane-04:33-38 |
| G1b parquet object | 55,711 B | UNVERIFIED-BY-RECEIPT | charter:47-48; rlib 55,799 (BGP:57) |
| DRed lineage | cascade to 0 survivors | ABSENT (FA5: 9,999) | forensic-lane-05; D16 |

## 3. The 128 KiB Hardware Boundary

Full ELF ledger: MB10 (lane-10) cross-checked against the BGP receipt.

### Per-crate ledger (riscv32imc)

| crate | tier | audit-rlib B | linked ELF B | budget B | verdict |
|---|---|---|---|---|---|
| cgap-blake3 | tier0 | 114,072 | 35,440 | 49,152 | OK (BGP:54) |
| cgap-gate | tier0 | 1,048 | 484 | 49,152 | OK (BGP:55; lane-10:19) |
| cgap-p28 | tier0 | 9,220 | 886 | 49,152 | OK (BGP:56; lane-10:19) |
| cgap-parquet | tier2 | 55,799 | 2,252 | 24,576 | OK (BGP:57); BLOCKED in MB10 (lane-10:42-48) |
| cgap-arrow | tier1 | - | - | 49,152 | REFUSED exit 1, no entry snippet (lane-10:21) |
| cgap-bn3 | tier0 | - | - | 49,152 | REFUSED exit 1; std strip not landed (lane-10:22) |
| cgap-ffi | tier3 | - | - | 8,192 | REFUSED exit 1, no entry snippet (lane-10:23) |

Envelope sums: audit-rlib envelope 39,062/131,072 (BGP:59); MB10
linked-ELF measured-lane total 36,810/131,072, tier_breaches=0
(lane-10:29-30, 68). Deployment gate exit 0 with `--deploy-mutant`
flip: mutant ELF 66,176 > 49,152 tier0 (lane-10:34-37).

### The three-separate-binaries caveat

The 36,810 B envelope sums three independently-linked ELFs
(blake3 35,440 + gate 484 + p28 886, lane-10:68), NOT one enclave
image. The combined single-link claim (total .text+.rodata <= 131,072 B
in ONE link) is [TARGET]: blake3 alone is 35,440 B, paper headroom
exists for three tier0 slots plus a unified link, but no unified
artifact exists (lane-10:69-73). Measurement-basis drift is recorded,
not reconciled: blake3 35,440 gate-sum vs 40,208 coordinator witness
vs 66,176 raw section sum (lane-10:56-58); gate linked 484 < 940 rlib
class (lane-10:60-62).

## 4. Throughput & Latency Master Table

Every number ran this session per its lane receipt.

| Metric | Value | Witness |
|---|---|---|
| Ingress chain, ns/frame | 10.65 fastest / 11.73 med-of-fastests | lane-01:24-27 |
| Penstock admission, ns/f | 7.89 fastest / 8.55 med-of-fastests | lane-01:28-31 |
| Digest fold | 0.97-1.19 ns/fold | lane-01:33 |
| Prefilter fold | 0.66-0.73 ns/f | lane-01:34 |
| saturated_subs_p28 | 3.33 ns/op = 300.5M ops/s/core | lane-02:34-36 |
| poison_mask | 0.50 ns/op = 1987.6M ops/s | lane-02:37-38 |
| mat_closure N=65,536 | 10.36M facts/s/core | lane-04:17 |
| mat_closure N=262,144 | 7.55M facts/s/core | lane-04:18 |
| mat_closure N=1,048,576 | 5.77M facts/s/core | lane-04:19 |
| DRed retract (bitmask filter) | 277,792 ns/batch = 1.06 ns/lane | lane-05:18-19 |
| DRed thruput (filter) | 943.39-943.67M facts/s | lane-05:20-24; filter only (D16) |
| Entropy variance | 1.0008 max/min < 4.0 fence | lane-05:26 |
| cmov_u64 | 0.70 ns/op | lane-06:72 |
| eytzinger_search | 1.54 min / 12.85 max ns/op | lane-06:80 |
| swar primitives | 0.69-1.66 ns/op | lane-06:75-77 |
| chain_ram @1M | 23.05 ms mean | lane-07:24, 57 |
| seal_arm @1M | 16.25 ms, 0 allocs / 0 B | lane-07:26, 45 |
| verify_arm @1M | 6.805 ms | lane-07:27 |
| decode_arm @1M | 760 us, 0 allocs | lane-07:29 |
| Stats prune, per skipped RG | 0.9 ns quiet / 2.1 ns loaded | lane-08:64-67 |
| Prune saved vs full scan | +54.1 / +48.8 / +27.2 us @1/10/50% | lane-08:108-109 |
| stage_commit_parquet | 0.755 us/tx @256 / 0.832 @4096 | lane-09:30-31 |
| ring_sink_commit | 0.803 us/tx @256 | lane-09:32 |
| ring spill leg | 17.7 us/tx (22x warm) | lane-09:63-65 |
| chain_full | 7.40 us/tx @256 / 7.43 @4096 | lane-09:38-39 |
| eval_bn3 stage leg | 4.39 us/tx @256 (blocking) | lane-09:41-42 |
| Envelope (linked) | 36,810 / 131,072 B | lane-10:68 |
| Envelope (audit rlibs) | 39,062 / 131,072 B | BGP:59 |
| G7 patent gate | 4/4 DISTINCT-MECHANISM, exit 0 | lane-11:26-29 |

## 5. Defect Ledger & Active Seams

### D1-D15

- **D1 BN3NS+RV1 in flight at compilation time** — AS-OF: unlanded
  during the mega-bench window; bn3 riscv observability today
  manifests as the gate's no-entry-snippet refusal, not a measured
  BLOCKED (lane-10:50-52).
- **D2 footer.rs:1008/:1073 mid-edit incidents** — SLA-held x3+
  witnesses: MB1 poll resolved ~2 min (lane-01:76-79); MB4/MB5/MB7/
  MB9/MB10 hit the same corruption, owner repaired in-SLA each time
  (lane-05:10-12; lane-07:16-18; lane-09:92-96; lane-10:42-48).
- **D3 P5 seal zero-alloc** — P5 receipt landed (P5:31-38, 2->0
  witnessed); effect re-witnessed by MB7 and MB9 (lane-07:45-53;
  lane-09:79-82).
- **D4 p28 886 vs 7,140 B** — measurement-basis drift: linked ELF
  886 B vs 7,140 B rlib witness (m1); basis differs, not reconciled
  (lane-10:63-64; m1:32-36).
- **D5 chain_full +17-34% drift** — 7.40 vs 6.19-6.33 us/tx witness;
  eval_bn3 (4.39 us/tx) is the blocking leg; environment-attributed
  (lane-09:50-52, 41-42).
- **D6 L3 lever LANDED-NOT-MET** — radix:nested = 0.97-0.98x on the
  chained corpus; <75 ms @1M missed 22x (L3:30-42).
- **D7 MB4 shape taxonomy missing** — DAG/cycle-rich corpora do not
  exist; chains-shape numbers only, loudly scoped (lane-04:33-38).
- **D8 MB8 truncation nuance** — stats-length truncation is a typed
  MalformedFooter refusal at OPEN (whole file), distinct from the
  fail-open path A (lane-08:80-90).
- **D9 GS latent pack type-6/7 gap** — dispatch-carried; no
  mega-bench receipt touches it — UNVERIFIED in this report's scope
  (dispatch MB12, AS-OF).
- **D10 gate0 FM-PACK-008 ggen.lock drift** — pending re-lock; no
  mega-bench lane receipt carries it — AS-OF from dispatch.
- **D11 publishes FROZEN** — operator-gated; P5 header confirms
  "publishes frozen" (P5:4; publish law, CLAUDE.md).
- **D12 MB3 receipt absent** — `mega-bench/` holds lane-01,02,
  04..11 only — 10 of 11 receipts on disk (ls, receipt tail).
- **D13 w5 writer-encode lever open** — writer encode ~10.2 ms @1M;
  falsifier < 2x encode_arm unfired (lane-07:76-78).
- **D14 audit sprawl** — rlib vs linked: blake3 3.22x, p28 10.4x,
  parquet 24.8x — linking strips (BGP:52-57).
- **D15 footer.rs:1151 court fail** —
  `max_row_groups_over_capacity_refused_...` panics under the
  in-flight +327/-30 diff — other-lane subject, named not folded
  (lane-08:21-26, 113-114).
- **D16 DRed lineage propagation ABSENT (HIGH)** —
  `dred::retract` (dred.rs:116-127) is a bit-parallel retraction
  filter over caller-supplied static masks; it never follows
  `derived_deps` as lineage edges. FA5 probe: N=10,000 root-deletion
  chain leaves 9,999 survivors where 0 is correct (probe exit 0,
  replayable at /tmp/fa5-probe; adversarial/forensic-lane-05.md,
  Interrogation 3). The 947M facts/s certifies bitmask
  tuple-invalidation speed, not end-to-end DRed lineage unwinding.
  Close condition: lineage-edge walk in dred + cascade court with a
  root-deletion corpus flipping 9,999 -> 0 survivors. Prior receipts
  carrying the misleading framing: p1-dred-retraction-antijoin.md,
  w5-g4-dred-churn.md, lane-05 — see
  waves/w6-dred-relabel-errata.md for the quoted sentences.

## 6. Immediate Milestone Closure Checklist (to tag v26.11.2)

Ordered punch list; publishes stay operator-gated throughout.

1. Drain in-flight lanes: land or revert BN3NS+RV1 (D1); re-run
   `cargo bench -p bench --bench chains-sovereign` post-landing to
   retire the D5 chain_full drift attribution.
2. Census 19/19: rewrite `select_slice`/`pred_inc` branchless and/or
   restructure the RV32 saturated-sub (lane-06:96-98); court must stay
   self-check-green (lane-06:15).
3. bn3 riscv check: land the std strip and add the budget-gate entry
   snippet so cgap-bn3 moves from KeyError refusal to a measured ELF
   (lane-10:50-52).
4. boundary --deps: re-run `python3 scripts/boundary_gate.py` after
   the drain so the boundary gate sees the post-P5 tree.
5. G7: done this session — 4/4 DISTINCT exit 0 (lane-11:26-29); keep
   the row-2 PLNF cardinality remainder named in
   `03-ost-design-around-register.md` (lane-11:54-56).
6. Deployment gate: re-run MB10's exact command post-parquet-drain so
   cgap-parquet moves BLOCKED -> measured tier2 ELF (lane-10:42-48).
7. gate0 post-re-lock: resolve FM-PACK-008 ggen.lock drift, re-lock,
   re-run gate0.
8. ggen.lock re-lock BEFORE the receipts metagate: `cd crates/cli &&
   rm Cargo.toml && ggen sync run && python3 manifest_patch.py`,
   verify byte-identical replay.
9. Receipts metagate: one pass over all wave/mega-bench receipts for
   standing vocabulary (no UNVERIFIED rounding up); close charter
   action A3 by attaching G1/G4/G6 witness receipts (charter:113).
10. Tag readiness: coordinator ladder (host gate, riscv build, symbol
    audit, mutation battery) then tag. Publishes remain
    FROZEN/operator-gated (D11) — one attempt per crate post-window,
    never concurrent with lever lanes.


## 7. Post-hardening refresh (2026-11-02)

Post-report hardening/repair lanes closed or superseded rows in
sections 1, 2 and 5. The original D1-D16 rows above are preserved
verbatim as history; this section records the corrections, each
receipt-cited. Divergence: the dispatch shorthand "census 16/19 ->
18/19" does not appear verbatim in REP1; the landed receipt records a
16/16 pub-fn / 26-cell grid (20 HOT cells 0-branch, 6 documented
residuals) enforced by `cargo test` exit 0 — reported as receipted,
not as the shorthand.

### 7.1 Stale-row corrections (W15 list)

- **D10 gate0 FM-PACK-008** — pending re-lock (AS-OF) -> CLOSED:
  re-lock landed, lock pins 22ba9f10, second `ggen sync run` cmp
  byte-identical PASS, post-fix `make gate0` exit 0
  (closure/lane-01-lockfile-sync:13-21).
- **D16 DRed cascade** — YELLOW relabel / FA5 9,999 survivors ->
  CLOSED on the cascade leg: `build_csr` + `cascade_retract` BFS;
  C1 chain N=10,000 guarded 0 survivors, no-cascade mutant yields
  9,999 (flip pinned in-court), 5-court dred_cascade.rs exit 0;
  static filter leg unchanged (repairs/repair-04:51-63,105-110).
- **D9 pack type-6/7** — UNVERIFIED in report scope -> CLOSED: typed
  delta refusal landed; type-6/7 header tests FAILED pre-fix (typed
  panic witness), then `cargo test -p cgap-git` exit 0,
  delta_refusal_court 4/4 (waves/w6-git-typed-refusals:46-62).
- **D15 footer court** — other-lane in-flight fail -> CLOSED: H5
  final suite green, `cargo test -p cgap-parquet` 182 lib passed,
  0 panics, exit 0; seal_rows_bound_court 3/3 with mutant flip
  (hardening/lane-05-parquet-seal-bounds:43-57).
- **D1 BN3NS+RV1** — unlanded at compilation time -> CLOSED: BN3NS
  landed; riscv32imc `cargo check -p cgap-bn3` exit 0;
  `cargo test -p cgap-bn3` exit 0 (H8)
  (hardening/lane-08-bn3-zero-alloc-fixpoint:81-89).
- **D6 radix 0.97-0.98x** — LANDED-NOT-MET (22x missed) ->
  SUPERSEDED by RADIXFIX/repair-06: `JoinProbeScratch::default()`
  flipped to radix=true; honest re-measure radix 23.72 ms vs nested
  1.858 s @1M = 78.3x fastest (5.6x @65,536); LANDED-MET
  (repairs/repair-06-radix-default-flip, re-measure table).
- **Branchless census 16/19** — RED, 3 BRANCHY -> SUPERSEDED by
  REP1: final grid 26 cells over aarch64+riscv32imc, 20 HOT cells
  0-branch, 6 documented residuals; `cargo test -p cgap-branchless`
  exit 0 REQUIRES the grid — NO fake 19/19
  (repairs/repair-01-branchless-census:56-104,126-127).
- **Branchless dead library** — unconsumed -> CLOSED via AE5:
  2 symbols consumed (swar_all_zero_vl + penstock_prefilter8 into
  cgap-gate via REP5), 17 justified exceptions, stale-exception
  guard exits 1, stub mutant flip witnessed, exit 0
  (countermeasures/lane-05-dead-library-gate:31,43-51).

### 7.2 Post-report findings added (D17-D21)

- **D17 DRed semantics split (DREDLABEL)** — the static bitmask
  `retract` and the CSR `cascade_retract` are distinct semantics: the
  cascade purges deleted+derived+alt>0 roots the static path keeps;
  courts pin both sides (repair-04:31-35; relabel errata).
- **D18 staging-latch (FA9)** — commit-path concurrent-writer gate:
  new register value `CGAP_ERR_CONCURRENT_WRITER_DENIED = 0x4347_0006`,
  per-(path-hash, thread-id) `spill_gate` registry, fail-closed
  acquisition before the lease-grant `File::create`; the MB9 "22x
  spill" premise is corrected by the FA9 errata (w6-fa9-commit-gating:12,35,49).
- **D19 ISel residuals (REP1)** — riscv32imc ISel lowers every 2-way
  select to a branch; residual branchy cells are recorded as
  documented residuals, not fakes (repair-01:69,104-116).
- **D20 seen-set O(store/11) (H8 refutation)** — the strong claim
  "allocs scale with delta not store" is REFUTED: the BTreeSet
  seen-set seed is O(store/11) B-tree node allocations per fixpoint
  run (~9.1k nodes @100k facts) (hardening/lane-08:52-78).
- **D21 entry-surface coverage (FA10)** — the budget gate measures the
  ENTRY_SNIPPETS entry surface only (blake3 31.1% / gate 46.2% /
  p28 9.6% of audit rlib); maximal one-link no_std consumer 102,187 B
  is CITED (FA10 probe), not re-derived. Verdict semantics: "minimum
  entry surface fits", never "the engine fits"
  (hardening/lane-13-enclave-budget-witness:52-63,107).

### 7.3 Dashboard echoes (history untouched)

- MB5 (D16): cascade leg now CLOSED via repair-04; the 943.7M facts/s
  figure stands for the bitmask filter leg only (see 7.1 row D16).
- NOT-met row "Branchless census 16/19": superseded by the REP1
  16/16-fn / 26-cell grid (see 7.1).
- NOT-met row "DRed lineage cascade to 0 survivors": closed on the
  cascade leg by REP4 (0 survivors, flip pinned in-court).
- G3 stretch / shape taxonomy / Cpk / G1b rows: still OPEN, no
  superseding receipt found this session.

## Receipt: reads this session (commands + exits)

| Command | Exit |
|---|---|
| `ls docs/jira/v26.11.2/mega-bench/` + `wc -l` on inputs | 0 |
| `cat -n` lane-01,02,04 / lane-05,06,07 / lane-08,09 / lane-10,11 | 0 x4 |
| `cat -n` 14-dflss-charter.md 15-ctq-tree.md | 0 |
| grep sweeps: lane-03 / BN3NS / RV1 / FM-PACK-008 / BGP | 0 (misses named above) |
| `sed -n` BGP, P5, L3, m1 receipts | 0 |
| `awk length>100` gate on this file | 0 |

| `ls` waves/ repairs/ hardening/ closure/ countermeasures/ adversarial/ explore/ | 0 |
| grep sweeps: RADIXFIX/GITFIX/DREDLABEL/CL1; repair-01/04/06 heads | 0 |
| grep: C6-cascade, H5-suite, H8-seen-set, AE5, FA9, FA10, ISel lines | 0 |
| `sed -n`/`head` repair-06, repair-04, repair-01, CL1, GITFIX heads | 0 |

