# HyperScaler Replication Timeline Report

Lane AU14, Lead Forensic Synthesizer. 2026-11-02. Repo /Users/sac/cgap @ main.
Synthesis of Phase 1 metrics (docs/audit/metrics/01-07) and Phase 2 models
(docs/audit/models/08-13). Separation law: MEASURED = observed on disk this
cycle; MODEL-ASSUMPTION = stated parameter of a named model; ANALYSIS =
interpretation, not measurement. NOT legal or certification advice.

## 1. Executive Summary

**Effort envelope.** COCOMO II.2000 (embedded, Extra-High CPLX/TIME/STOR)
on the measured 65.8 KSLOC kernel gives 400-740 person-months baseline;
the AU7-class (DO-178C-form) verification uplift x1.5-2.0 yields a
planning envelope of **600-1,480 person-months** (AU8,
docs/audit/models/08_parametric_software_models.md:41, MEASURED inputs,
MODEL-ASSUMPTION multipliers). At the stated conversion of ~1,800
engineering-hours per person-year (= 150 PM per PY), this is
**50-123 person-years** (ANALYSIS: 600-1,480 PM / 12 ≈ 50-123 PY; the
1,800 hr/PY figure is the dispatch conversion, stated not measured).

**Calendar duration.** COCOMO TDEV = 3.67 x PM^0.28 gives **20-23
calendar months** for the conventional-org core build (AU8:37,
MODEL-ASSUMPTION). Duration is sublinear in effort; the uplift band does
not stretch TDEV proportionally (AU8:47).

**Scenario A / B / C (conventional org, per AU9 drag model).**

- **Scenario A — lean org, drag 0.24** (AU9:33, MODEL-ASSUMPTION):
  effective coding fraction 0.76. A 1,480 PM program needs ~65 engineers
  over ~21 months (AU9 capacity table); realistic band **~21-26 months**
  including integration tail (ANALYSIS).
- **Scenario B — standard org, mid drag ~0.40** (AU9:33 mid-band,
  MODEL-ASSUMPTION): coding fraction ~0.60. Same program needs ~74+ staff
  and lands at the AU8 TDEV edge plus Brooks second-order effects not
  priced (AU9:34-35): **~24-30 months** (ANALYSIS).
- **Scenario C — heavy-drag enterprise, drag 0.58** (AU9:33): coding
  fraction 0.42. AU9's own finding: the high scenario is "unreachable by
  any row's heavy-drag capacity except near n=65 lean" (AU9:46-48) —
  i.e. **>30 months or effort-descoped** (ANALYSIS), with the 2,080-channel
  n=65 mesh compounding drag beyond what the table prices (AU9:48-49).

On top of the build: certification wall-clock 12-30 months (FIPS 140-3
class, institution-bound, AU11:22-28) and legal clearance 8-16 weeks per
claim family (AU12:69, MODEL-ASSUMPTION). Total conventional-org calendar
to an equivalent certified, legally cleared posture: **~3-5 years**
(ANALYSIS, summing AU8 build 20-23 mo + AU11 cert 12-30 mo serial-tail +
AU12 legal in parallel).

**The counterfactual.** The actual repository was produced under a
single-checkout AI-fleet architecture at near-zero inter-lane coordination
cost (AU9:51-80, MEASURED structure) — the replication question for a
hyperscaler is therefore not "how many engineers for 50-123 PY of work"
but "can the coordination substrate be reproduced" (Section 6).

## 2. Empirical Codebase Inventory (MEASURED, AU1)

Source: docs/audit/metrics/01_cgap_topology_and_sloc.md, all figures
wc/find/grep at HEAD 2026-11-02.

| Quantity | Value | Source |
|---|---|---|
| Workspace members | 24 (22 crates + 2 ontology packs) | AU1 §1 |
| Kernel src SLOC (15 kernel crates) | 65,776 | AU1 §3 |
| tests/ dir LOC | 22,206 | AU1 §3 |
| In-src cfg(test) LOC | ~38,000 (sum of cfg(test) column, approx) | AU1 §3 |
| Python tooling scripts/ | 12,321 LOC | AU1 §3 |
| Benches | 18,440 LOC | AU1 §3 |
| CLI (non-member, own workspace) | 4,458 src LOC | AU1 §3 |
| Ontology packs (Rust crates) | 4,057 .rs LOC | AU1 §3 |
| External deps in member crates | ZERO (all 24) | AU1 §4 |
| no_std kernel crates | 10 | AU1 §5 |
| Edge leaves | arrow, blake3, bosym, branchless, core, court, git, parquet, integration | AU1 §2 |

Largest kernels: cgap-parquet 15,452; cgap-arrow 10,124; cgap-bn3 10,039;
cgap-ffi 6,520; cgap-bont 6,327 (AU1 §3). cgap-court is an empty scaffold
(AU1 §1). Only the standalone CLI carries external deps (blake3,
clap-noun-verb, serde et al., AU1 §6).

## 3. Systems Complexity Matrix

### 3.1 Branchless census and the ISel ceiling (MEASURED, AU2)

28 public fns in cgap-branchless (2,240 LOC), families: SWAR, cmov,
eytzinger, radix, bit-matrix transpose, Kogge-Stone scans, fixed-point
mix; `#![no_std]` + `#![forbid(unsafe_code)]`; zero floating point
(AU2 §1). Verification investment: 1,753 test LOC (~0.78:1) + 34 in-module
courts + per-symbol llvm-objdump disassembly census on aarch64 AND
riscv32imc asserting EXACT per-target branch counts, plus differential
oracles and five named anti-vacuity mutants asserted to flip (AU2 §2).

The load-bearing finding: on riscv32imc (no Zicond), instruction
selection lowers EVERY 2-way select to a branch regardless of source
form; source inspection cannot distinguish authored branchiness from
ISel-manufactured branches — the census asserts exact residuals (1/1,
2/2) as an ISA-ceiling contract, and the 0-branch unsafe mask form is
proven as a sandbox twin (AU2 §3, MEASURED with receipts).

### 3.2 Memory and allocator discipline (MEASURED, AU3)

128 KiB envelope with linker-ASSERTed sub-budgets (kernel 16 KiB,
storage/compute 48 KiB each, attest 12 KiB; measured whole-envelope
39,184/131,072 B, exit 0), fail-closed budget gate with anti-vacuity
mutant/self-check arms; allocator-reentrancy playbook derived from two
measured SIGABRT witnesses; pool-leased provisioning-phase allocation;
typed capacity refusals (NA_MAX_MATCHES=2048, WorklistCapacity);
stack-frame budgeting with a structural allocate-at-the-allocation fix;
honest H8 ceiling 278 allocs/14.2 MB pinned at 2x loud (AU3 §1-3).
Scope caveat: the gate measures entry surface only — "the engine fits"
is probe-derived, not gate-enforced (AU3 §1, FA10/D21).

### 3.3 Gate and court fleet (MEASURED, AU7)

23 gate scripts in scripts/ (12,321 LOC); workspace-wide 1,259 `#[test]`
fns (92 integration files, 159 cfg(test) modules); ~118 flip-witness
lines grep-derived across v26.11.2 receipts (APPROXIMATE, AU7 §1-2);
test:src ratio ~0.31 (AU7 §3). Form is DO-178C-requirements-coverage
plus mutation coverage cgap makes a gate law ("a court that cannot flip
is FAIL"; flip-before-PASS) — exceeding DO-178C, which has no mutation
objective (AU7 §4, ANALYSIS). Honest gaps: quotas cover 4/24 crates;
density uneven (bn3 0.09, gate 0.08); flip counts not machine-aggregated;
no trybuild/compile-fail machinery (AU7 §4).

### 3.4 Remaining metric lanes (MEASURED, AU4-AU6)

- Deductive engine: SOUFFLE-class radix semi-join (5.6x @65k, 78.3x @1M,
  same-session differential with an UNVERIFIED older fence), semi-naive
  closure 4.85M-11.25M facts/s/core, DRed static filter relabeled by
  errata with lineage closed by the newly landed CSR cascade (0/10,000
  survivors) (AU4 §2-4).
- Columnar conformance: Arrow C ABI, compact thrift, pack V2,
  DEFLATE/zlib re-derived in-crate with zero reference-library linkage;
  conformance via struct-exact ggen-frozen headers (31 compile-time
  offset pins) and differential courts against real external
  collaborators (system python3 zlib, real git pack-objects) (AU5 §1-2).
  Documented subset, not spec-complete; delta resolution NOT implemented
  (AU5 §3).
- Ontology compiler: one graph, ggen.lock blake3 pins, byte-stable
  gate0 replay (cmp run1 vs run2), freestanding C99 header compile;
  determinism was BROKEN in practice (FM-PACK-008 stale lock) and the
  generated-surface guard was vacuous on absolute paths until fixed —
  determinism is a property of the gate, not the generator (AU6 §2, §4).

## 4. Phase-by-Phase Timeline (conventional-org model)

Effort from AU8; phases sequenced per AU13 misstep classes (an enterprise
pays each detected-late class before reaching cgap's design), AU11/AU12
tails. All durations MODEL-ASSUMPTION unless noted.

1. **Architecture & constraint discovery** (~3-5 mo of the 20-23 mo TDEV,
   ANALYSIS): zero-dep/no_std/128 KiB constraints set first. Skipping this
   is AU13 class 5 (port-late: riscv32imc E0463 x410 errors, months-years
   detection lag) — the class cgap avoided; a standard org typically does
   not (AU13 rows 2,5).
2. **Prototype & algorithm selection** (~4-6 mo, ANALYSIS): closure
   evaluation shape. AU13 class 2: the interpreter-first fixpoint is the
   default; the semi-naive delta shape landed only late in cgap too.
   Expected enterprise rework: one release cycle to a load-test cliff
   (AU13 row 2, class L).
3. **Implementation** (bulk of 20-23 mo TDEV, AU8): 65.8 KSLOC kernel at
   COCOMO embedded rates; n ~18-37 staff (AU9 §1).
4. **Hardening & verification** (the x1.5-2.0 uplift, AU8 §2c): courts,
   disassembly census, alloc budgets, gate fleet — the dominant cost.
   AU13 classes 3/4/7 are the measurement-lie classes an enterprise
   absorbs here (wrong-arm benches, vacuum-ELF budgets).
5. **Compliance & legal tail** (serial after build, institution-bound):
   FIPS 140-3 class 12-30 mo; CC 12-24 mo; SOC 2 Type II 6-12 mo
   observation (AU11 §2, ASSUMPTION); counsel opinions 8-16 wk/family
   across 4 families, parallel wall-clock ~one committee season
   (AU12 §2, ASSUMPTION). Receipt-grade evidence compresses the
   developer-side phase only; the queues are indifferent (AU11 §3).

## 5. Brooks's Law & Cross-Silo Friction Ledger

**Drag fractions** (AU9:24-33, all MODEL-ASSUMPTION industry ranges):
sum 0.24 (lean) to 0.58 (heavy) across RFC review, gate meetings, syncs,
status reporting, hiring, onboarding. Channels c = n(n-1)/2: 10 (n=5) to
2,080 (n=65); AU8 band endpoints 153 (n=18), 666 (n=37), 2,701 (n=74)
(AU9 §2). Capacity table: the 1,480 PM scenario is unreachable under
heavy drag below n~65 (AU9 §4).

**Cross-silo seams** (AU10, MEASURED intra-repo incidents): four
boundaries B1-B4 carry shared artifacts with NO version contract on any:
B1 FNV-1a constants re-implemented in 4+ places (cross-silo identity
drift; 10+ torn-write incidents); B2 closure/dred dual implementation
(production-vs-oracle semantic drift, MB3 arena panic found in a bench);
B3 ggen projections + cgap_c_data.h (generated-artifact staleness);
B4 gates own the contracts with no declared version (enforcer-coupled
opacity) (AU10 §Boundaries, Scaling summary).

**Friction prediction** (labeled ANALYSIS): the dispatch figure of a
30-100x coordination-friction ratio between an unpinned-seam org and the
pinned-substrate model does not appear in AU10 or any input file; it is
recorded here as an unbacked ANALYSIS-level claim, UNVERIFIED. What AU10
does support: incident rate tracks independent-copy count, and the
conventional org pays the AU9 drag fractions (0.24-0.58) that this repo's
architecture structurally avoids (AU9 §5, AU10 Scaling).

**The repo's structural alternative** (AU9 §5, MEASURED structure,
ANALYSIS interpretation): single canonical checkout with coordinator-owned
git concentrates integration in one hub; ~151 wave receipts over ~2 days
at peak concurrency 7 coordinate by disjoint file ownership, not
person-to-person channels; the hook/census fleet mechanizes what orgs
price as review committees and status reporting at ~zero marginal
eng-time. Brooks is inverted, not repealed: O(n^2) pairwise becomes O(n)
hub-and-spoke, with coordinator saturation as the priced risk (AU9:73-78).
Scope caveat: fleet lanes are AI agents — no onboarding ramp, no hiring
load — so the human drag fractions do not transfer (AU9:79-80).

## 6. Concluding Forensic Synthesis

### What this repo's model did that a standard org cannot easily replicate

1. **Near-zero inter-lane coordination drag.** Disjoint per-lane file
   ownership plus coordinator-owned git eliminates the pairwise channel
   for the common case; the conventional equivalent prices 0.24-0.58 of
   every engineer's time (AU9 §3-5). Replicating requires adopting the
   substrate (one checkout, one owner, ownership partition), not hiring
   more — a structural change hyperscalers find hard because it removes
   the team boundary their org chart is made of (ANALYSIS).
2. **Mechanical enforcement replacing process.** Review-committee and
   policy functions run as fail-closed hooks and gates with exit
   contracts and anti-vacuity legs: a green gate without a witnessed
   flip is FAIL; an unrun gate is indistinguishable from no gate (AU7
   §2, AU6 §4). Enterprises enforce by narrative; here enforcement is
   parse-level (AU7 §4 ANALYSIS).
3. **Receipt immutability and evidence hygiene.** sha256-exact restores,
   mtime-bump law against stale-fingerprint vacuous passes, append-only
   errata (W7 metagate), receipts as flip witnesses (~198-receipt fleet,
   AU11 §3). The second-order property — courts on courts, mutants
   asserted to flip — does not exist in conventional library or even
   safety-critical practice, where mutation coverage is not an objective
   (AU2 §4, AU7 §4).
4. **Compiler-backend-grade completion criteria.** "Done" defined at
   objdump level with exact residual counts and typed ISA-ceiling
   documentation (AU2 §3-4); a discipline whose marginal machinery is
   the dominant LOC cost and is rarely staffed anywhere.

### What it cannot claim

1. **Single-run Cpk unproven at N>=30.** Gates are single-run
   deterministic; soak/staging cycles (weeks-months) are assumed tables,
   never run (AU11 §4, stated limit). Process-capability statistics on
   any gate are absent.
2. **DRed lineage newly landed.** The cascade (REP4) landed this cycle;
   the long-standing mechanism was the non-transitive filter, pre-errata
   "retraction" receipts were relabeled post-hoc, and the 78.3x headline
   carries an unreproduced older fence (AU4 §2, §4).
3. **The legal half is absent.** The OST register is self-authored
   element text with no patent text on disk; ZZQQ-class corruption still
   passes the gate (AU12 §1, FA11 M2 verbatim). No counsel opinions
   exist; engineering evidence is input to an opinion, not a substitute.
4. **Publish unfrozen pending.** Publish remains guarded/frozen under
   coordination receipts (registry_sweep "Never publishes", AU11 §1);
   release posture is not shipped-standing.
5. **Bounded evidence coverage.** Budget gate proves entry surface only;
   "the engine fits" is probe-derived (AU3 §1). Gate quotas cover 4/24
   crates; flip counts are receipt-grep, not machine-aggregated (AU7 §4).

**Verdict** (ANALYSIS): a hyperscaler replicating the artifact should
budget 50-123 PY of effort at AU7 rigor, 20-23 months build, 12-30
months certification tail — Scenario C realistically >30 months or
descoped. But the faster replication path is not headcount; it is
adopting the coordination substrate (single checkout, ownership
partition, receipt law, anti-vacuity gates), which is a fixed
one-time authoring cost plus one saturated coordinator, and which no
measured input prices in person-months. The envelope estimates the
software; the substrate, not the staffing, is the part a standard org
cannot easily buy.

---

## Receipt tail — lane AU14 reads (this session, 2026-11-02)

- `ls -la docs/audit/metrics/ docs/audit/models/; wc -l ...` — exit 0;
  7 metrics files (01-07) + 6 model files (08-13), 912 lines total.
- Read tool on all 13 input files, full contents, exit 0 each:
  metrics/01_cgap_topology_and_sloc.md (89 ln),
  metrics/02_branchless_algorithmic_complexity.md (90),
  metrics/03_sram_and_allocator_discipline.md (35),
  metrics/04_deductive_engine_and_joins.md (89),
  metrics/05_columnar_storage_conformance.md (53),
  metrics/06_ontology_compiler_pipeline.md (73),
  metrics/07_anti_vacuity_and_gates.md (89),
  models/08_parametric_software_models.md (47),
  models/09_brooks_law_and_coordination_drag.md (80),
  models/10_enterprise_silo_fragmentation.md (91),
  models/11_security_compliance_and_certification.md (77),
  models/12_legal_and_patent_clearance.md (74),
  models/13_architectural_missteps_and_rework.md (25).
- Write of this file, then on-disk byte verification via wc/grep below.
