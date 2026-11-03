# 06-bench-protocol

Measurement method of record for the v26.11.2 starDF campaign. Every perf number in
`docs/jira/v26.11.2/` is a [TARGET-v26.11.2] until measured by THIS method with a receipt in
`waves/`. Prior-art inputs: `docs/jira/v26.11.2/research/g07-bench-conventions.md` (receipt
conventions, ratio-bound-as-committed-test pattern) and
`docs/jira/v26.11.2/research/g20-perf-claims-inventory.md` (anti-laundering: ggen's headline
numbers are CLAIMED-unreceipted and workload-mismatched; none migrate into this campaign).

## §1 Harness law

- Harness: **Divan 0.1.21, exact pin** (`crates/bench/Cargo.toml:25` —
  `divan = "=0.1.21"`; comment at :22 records the pin provenance, 2026-11-01).
- Every bench target is `harness = false` (`crates/bench/Cargo.toml:30-97`, one line per
  `[[bench]]` block).
- The five chains-* bench files are
  `crates/bench/benches/chains-{ingress,bn3,parquet,seal,sovereign}.rs`.
- Consts pattern: bench fns are const-generic over corpus size, instantiated via
  `#[divan::bench(consts = [...])]` — `chains-ingress.rs:513` and `chains-bn3.rs:351`
  (65_536 / 1_048_576), `chains-parquet.rs:225`, `chains-seal.rs:196` (1024 / 65536),
  `chains-sovereign.rs:413` (256 / 4096). Sizes are records, not bytes.
- ItemsCount semantics: `b.counter(ItemsCount::new(N))` counts chain frames / records
  processed — the unit every ns/frame and facts/s figure divides by
  (`chains-ingress.rs:22` doc law; `:49` import; counters at :523, :546, :569, :596).
  `BytesCount` appears only where the claim is byte-width (`chains-bn3.rs:30`).
- Each file installs `divan::AllocProfiler` as `#[global_allocator]` so every Divan row
  carries the alloc-bytes column; the alloc column is part of the verdict
  (`chains-parquet.rs:32-35` documents the pattern).

## §2 Per-lever method (L1-L5)

Mapping bench file -> lever -> corpus seed law. Seed discipline: the canonical SplitMix64
seed/multiplier is `0xBF58_476D_1CE4_E5B9` (`crates/cgap-parquet/src/bitslice.rs:694`), and
`0xA20C_0FFE_E5A7_E000` (`bitslice.rs:699`) is a corpus-spread constant XORed onto every
value (`bitslice.rs:698-699`) — it is **never a seed**; a spread constant in a seed position
is corpus corruption (CLAUDE.md seed-conflation law; PRD §3 unit note,
`00-ard-prd.md:88`).

| Lever | Bench file | Consts |
|---|---|---|
| L1 ingress digest (cgap-gate) | chains-ingress | 65_536, 1_048_576 |
| L2 SWAR FNV seal (cgap-parquet) | chains-seal | 1024, 65536 |
| L3 PLNF semi-join (cgap-bn3) | chains-bn3 | 65_536, 1_048_576 |
| L4 ring sink (cgap-ffi) | chains-sovereign | 256, 4096 |
| L5 fused ASC450 (gate+p28) | chains-parquet | 65536, 1048576 |

Seed law per lever (const name, file:line):

- L1 `chains-ingress.rs:54`: `CHAIN_SEED = 0xC001_0000_5EED_0001` (its own chain
  lineage); spread `(i & 65_535)` across FRAMES (:40).
- L2 `chains-seal.rs:55`: `CORPUS_SEED = 0xBF58_476D_1CE4_E5B9` verbatim; per-record
  spread via the penstock form `seed ^ shape*0x2544_5491_4F6C_DD1D` (:21).
- L3 `chains-bn3.rs:33`: `CANONICAL_SEED = 0xBF58_476D_1CE4_E5B9` verbatim; SplitMix
  form at :47; fixed `CHAIN_EPOCH = 7` (:37, determinism contract).
- L4 `chains-sovereign.rs:60`: `CORPUS_SEED` canonical verbatim; record spread
  `0x2544_5491_4F6C_DD1D` (:64); frame-stride const-assert 512 (:80).
- L5 `chains-parquet.rs:55`: `CORPUS_SEED` canonical verbatim; `LANE_SPREAD =
  0x2544_5491_4F6C_DD1D` (:58).

Lever definitions are verbatim `00-ard-prd.md` §3 (:51-55); each row's court names its
mutation class and the /tmp-sandbox flip method (e.g. `chains-parquet.rs:25-45`,
`chains-bn3.rs:14-19`).

## §3 Machine + memory disclosure

Every bench receipt carries, verbatim, an environment block:

- Machine: host model, CPU model + core count, RAM, OS version, rustc version
  (`rustc -vV`), Divan version, exact command line, and repo commit SHA. The receipt is
  replayable only with all seven fields present (g07 receipt-convention A; g07
  bench_bound.rs pattern).
- Memory: allocator column (alloc-bytes per iteration from AllocProfiler), peak RSS when
  the claim is capacity-shaped (1M-slot corpus), and whether the corpus is mmap-backed or
  heap-resident. Memory disclosure is part of the receipt, not an appendix: a GB/s number
  without the allocator column is incomplete and the receipt refuses.
- Load disclosure: load average at run time, and the no-overclaiming rule that
  ratios - not absolutes - are the regression signal under ambient load (g07
  bench_bound.rs load_average_1m note).

## §4 PARETO admission rule

A lever closes only with a before/after delta receipted in the claimed direction:

- Admission form: `before_us -> after_us, delta in direction D (claimed), method = this
  document, receipt in waves/`. If the measured delta is NOT in the claimed direction, the
  receipt refuses - the lever stays open even if the absolute number is attractive.
- Anti-vacuity: each receipt names the mutation that must move its number, flipped in a
  /tmp sandbox with sha256-exact restore + mtime bump (CLAUDE.md mtime-bump restore law;
  exemplar `docs/jira/v26.11.1/waves/a01-finish-receipt.md`).
- v26.11.1 precedent coordinates: P3-16 chains-ingress
  (`docs/jira/v26.11.1/waves/bench-chains-ingress.md`), P3-17 chains-parquet
  (`bench-chains-parquet.md`), P3-18 chains-bn3 (`bench-chains-bn3.md`), P3-19
  chains-seal + chains-sovereign (`bench-chains-seal.md`, `bench-chains-sovereign.md`).
  These are the before-arm baselines for v26.11.2 lever deltas.

## §5 Ratio-fence law

- No absolute ms/ms-ns fences in acceptance criteria. A fence is a relative ratio against
  an independent, sleep-free-by-construction baseline (paired rounds, min-of-N), per the
  no-sleep testing rule and g07's ratio-bound-as-committed-test pattern.
- Paired discipline: both sides of a ratio run interleaved in the same process under the
  same ambient load (g07 paired_medians/paired_mins pattern), so load cancels instead of
  accumulating.
- Process.sleep is banned in benches and acceptance criteria. Time enters only through
  Divan's own sampling; coordination uses bounded polls on observables, never waits.
- Fences ride in receipts as committed tests where practical (g07 bench_bound.rs shape);
  a fence nobody can run is prose, not a gate.

## §6 MW4 like-for-like method

- Identical workload shape is the load-bearing constraint: the same fact set, same rule
  set, same materialization closure definition, sized so the incumbent's published method
  describes the same work. A workload the published envelope does not describe is not
  like-for-like and the receipt refuses.
- RDFox is cited from its PUBLISHED envelope only - 2-6.1M facts/s
  (`00-ard-prd.md:15,29,60`) with the published paper/method line named in the receipt.
  **RDFox is NOT executed in this campaign** - no RDFox install, no RDFox subprocess, no
  vendor-supplied run. The comparison arm is a published number, the cgap arm is a measured
  number, and the receipt must say which is which.
- Until measured, every MW4 figure is [TARGET-v26.11.2] (`00-ard-prd.md:20`);
  G3-MAT-BENCH is >50M closure facts/s/core single-core with the RDFox baseline named with
  source + machine/method lines (`00-ard-prd.md:59-60`).
- Prior-art guard: g20 shows the incumbent-comparison like-for-like method boilerplate - workload-mismatched
  headline numbers quoted without method. MW4 receipts must show workload shape, machine,
  and the exact published-source citation or they are void.

### §6.1 Like-for-like boilerplate (method of record, W4-K3 2026-11-01)

Every MW4 receipt carries this block VERBATIM, with placeholders filled from the
measurement environment; a receipt missing any line refuses. Perf figures not
accompanied by this block stay [TARGET-v26.11.2].

**Workload definition (verbatim, `crates/stardf-datalog/src/lib.rs:24-30`):**

- Transitive closure: `anc(x,z) :- anc(x,y), anc(y,z). anc(x,y) :- par(x,y).`
- Same-generation: `sg(x,y) :- par(x,x0), sg(x0,y0), par(y,y0). sg(x,x) :- par(x,_z).`

**Seeds and consts (verbatim):** corpus generator = SplitMix64 with canonical seed
`0xBF58_476D_1CE4_E5B9` (`crates/stardf-datalog/src/seed.rs:10`); corpus constant
`0xA20C_0FFE_E5A7_E000` is spread seasoning ONLY and is never the generator seed
(`seed.rs:11-12`); corpus = `n` edges over `n` nodes modulo the node id
(`seed.rs:47-52`). Bench buckets are records, not bytes (§1).

**Baseline disclosure (fixed wording, no substitution):** "RDFox was NOT executed.
Baseline = published 2-6.1M facts/s envelope, source: [TARGET-v26.11.2 — operator to
supply citation URL]". A receipt that replaces this with an executed RDFox run, or
omits the [TARGET] marker while the citation is unsupplied, refuses. The cgap arm is
always a measured number; the incumbent arm is always a cited number.

**Machine + method template (fill per receipt):**

```text
machine: <host model> / <CPU model, N cores> / <RAM> / <OS version>
method:  rustc <version from rustc -vV> / divan =0.1.21 harness=false /
         cargo bench -p bench --bench mat_closure /
         semi-naive delta fixpoint, facts/s = derived facts at fixpoint / wall time,
         ItemsCount("facts") counter, single core, warm cache
commit:  <repo commit SHA>
```

**Unit law:** facts/s is never converted from GB/s and never quoted as a multiple of
GB/s (§7). A facts/s figure must come from the ItemsCount("facts") counter of derived
facts at fixpoint — wall time alone never produces the number.

## §7 Unit discipline

- GB/s and facts/s are different units and never convert silently: GB/s is never quoted as
  a multiple of facts/s (`00-ard-prd.md:88` - verbatim law).
- Throughput claims state the counter that produced them (§1 ItemsCount semantics); a
  facts/s number must come from an ItemsCount of facts, a GB/s number from BytesCount or a
  byte-exact wire-size disclosure.
- Latency units (ns/frame, us/tx, us/row in PRD §3) divide by the frame/tx/row counter,
  never by wall time alone.
- Any unit mismatch voids the receipt; the number stays [TARGET-v26.11.2] until re-measured
  with correct units.

## §8 Contention-marker consumption law (ADV-09-R2 closure, R-ADV09FIX)

- A run whose stderr carries the `CONTENTION-SUSPECT:` marker is
  REJECTED for quiet-host-scoped claims: it must not be quoted against
  `11-measured-claims.md` rows 5/6/7/8, and any claims-file row citing
  a marked run is VOID.
- Mechanism: the host-noise detector in
  `crates/bench/benches/chains-ingress.rs:716-802` (guard contract
  comment :716-719, `CONTENTION_MARK` const :745, emission at :802 and
  :854). The same pattern is copied into
  `crates/bench/benches/mctq_consolidation.rs:31,65,98`.
- Enforcement is PROCEDURAL today (receipt-lint / consumer discipline
  per this section): no machine gate yet greps bench stderr. A machine
  gate (post-receipt `grep CONTENTION-SUSPECT` over the receipted
  stderr, fail-closed) is a TARGET row until it exists with a receipt.
- Consumers of any claims-file row must state the marker check was
  clean; a receipt that does not disclose the marker check on a
  quiet-host-scoped row is incomplete.

## JSON footer

```json
{
"protocol":"06-bench-protocol","harness":"divan 0.1.21 harness=false",
"levers":["L1","L2","L3","L4","L5"],
"rdfox_mode":"published-envelope-citation-only",
"units_law":"GB/s never a multiple of facts/s",
"standing":"UNVERIFIED until measured by this method with a waves/ receipt"
}
```
