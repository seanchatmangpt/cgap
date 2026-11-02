# CULL-06 — src #[cfg(test)] vs tests/ court duplication audit

Lane: CULL-06. Scope: cgap-git, cgap-parquet, cgap-storage, cgap-bn3, csct.
Question: do src-inline `#[cfg(test)]` modules duplicate assertions the
tests/ courts also make? Rule honored throughout: no side is deleted when
it witnesses a flip the other lacks.

## 1. Known candidates that turned out NOT duplicated

- object_store.rs zero-alloc source-scan: `src/object_store.rs` has NO
  inline test module at all. The scan lives only at
  `tests/object_store_court.rs:250` (`zero_heap_static_invariant`,
  `include_str!("../src/object_store.rs")`). Same for
  `src/ingress_ring.rs`: no inline tests; the SWAR penstock sieve is
  witnessed only by tests/posh_wire_court.rs / ingress_ring_court.rs.
  Boundary holds; nothing to cull.
- pack_write.rs: src-inline `nostd_hygiene_court`
  (src/pack_write.rs:744-770) source-scans its own module body for
  `std::` paths. No tests/ court re-scans pack_write; the src side is
  the sole witness. Keep.
- csct cold-path boundary VERIFIED: csct_coverage_court.rs:1-13 cites
  receipt.rs:91-93/101-103/123-129/38/69-70 and types.rs:63-117 as
  branches the pipeline court missed. The stage-module inline tests do
  NOT reach those branches (receipt.rs inline tests use a passing
  single-stage sample; types.rs inline uses a one-stage sample), and the
  coverage court's anti-vacuity twins cover them. No overlap on the
  cold paths — the court's reason to exist still holds.

## 2. Confirmed duplication (with file:line both sides)

### 2.1 csct — strongest, actionable

| src inline | tests/ court | Claim |
|---|---|---|
| types.rs:150 digest_is_deterministic_across_clones | coverage_court.rs:289 | determinism |
| types.rs:169 digest_mutates_on_stage_removal | coverage_court.rs:326,330 | stage-removal flip |
| types.rs:191 exit_code_wire_values (0/1/2/3/101) | coverage_court.rs:112-116 | full wire pin |
| types.rs:191 AndonRefusal==101 | pipeline_court.rs:123 (plus andon.rs:109) | 101 pin |
| receipt.rs:184 render_mutates_on_exit_code_change | coverage_court.rs:206 | exit flip |

(coverage_court = tests/csct_coverage_court.rs;
pipeline_court = tests/csct_pipeline_court.rs.)

Verdict: the coverage court strictly subsumes these (multi-stage,
exhaustive variants, anti-vacuity twins). Delete src-side:
- types.rs:150 `digest_is_deterministic_across_clones`
- types.rs:169 `digest_mutates_on_stage_removal`
- types.rs:191 `exit_code_wire_values`
- receipt.rs:184 `render_mutates_on_exit_code_change`
(andon.rs:109 may stay as the andon unit semantic; it is 3 lines and
locally scoped. Optional second cut: csct_pipeline_court.rs:123 pin,
subsumed by the coverage court's exhaustive pin.)

KEEP — witnessed flips the courts lack:
- types.rs:157 `digest_mutates_on_field_flip` mutates the observed
  value; the coverage court flips stage_index/name/exit, appends
  assertions, flips wall_clock/alloc_count/receipt bytes, but never
  mutates an existing assertion's observed value. Unique witness.
- types.rs:180 `digest_mutates_on_verdict_flip` — no court flips
  final_commercial_verdict against report_digest. Unique witness.
- receipt.rs:169 `render_is_byte_stable_across_calls` also pins the
  economic block ("$5,068,632", "$14,400", "352x") and "PASS"; the
  courts assert byte-stability only (coverage_court.rs:344). Unique.
- receipt.rs:195 `render_mutates_on_assertion_flip` — no court flips
  assertion.pass against the rendered bytes. Unique witness.

### 2.2 cgap-storage — dred oracle equivalence, 3-way

All three assert guarded retract/keep_mask == naive oracle:
- src/dred.rs:529 `court_retraction_matches_oracle` (entropy x facts
  sweep, plus a KAT)
- tests/dred_swar_wire.rs:72 `court_rescue_matches_oracle` (facts sweep,
  public-API keep_mask witness)
- tests/dred_cascade.rs:151 `court_oracle_equivalence` (chain/diamond
  cycle topology — a shape neither other side builds)

Mutant/flip overlap: src/dred.rs:554 `court_antivacuity_alt_support_flip`
vs tests/dred_swar_wire.rs:90 `court_rescue_term_is_load_bearing` (both
witness the alt/rescue term as load-bearing) and src/dred.rs:585
`court_antivacuity_inverted_bitmask` vs tests/dred_swar_wire.rs:110
`court_mutants_diverge`.

Verdict: the core oracle-equivalence claim is triply asserted. The
entropy-axis sweep exists only src-side; the topology shapes only
court-side; the mutant functions are exercised from both. Cut proposal:
drop the src entropy sweep loop (dred.rs:536-548) and fold its entropy
axis into dred_swar_wire's corpus generator — keep src KAT + flips
(dred.rs:529 KAT part, 554, 585) as fast unit semantics. Do NOT delete
either test fn wholesale: each carries a flip the others lack.

### 2.3 cgap-parquet — round-trip triple assertion (soft)

RLE round-trip is asserted three ways:
- src/rle.rs:279-310 round_trip / repeated_runs / literal_groups /
  mixed_and_widths
- tests/rle_court.rs:36 every_kernel_width_round_trips_random_and_
  adversarial + :74 exhaustive_lane_values_round_trip (strictly stronger)
- tests/codec_properties.rs:102 prop1 (widths 1-8 x 500 corpora)

src/rle.rs:402 `decode_into_zero_allocations` vs
tests/page_zero_alloc.rs:84 targets a different function
(decode_page_into, page.rs public API) — layered, NOT a duplicate.
src/page.rs:197/217/239 round-trip family vs rle_court — layered
(page framing vs kernel); page.rs:256 truncation-at-every-prefix is a
unique witness nowhere in tests/. Verdict: keep all; if a cut is
forced, drop src/rle.rs:287 `repeated_runs` and :304 `mixed_and_widths`
(fully subsumed by rle_court.rs:36/:74) — but they are cheap fast units,
so this is optional, low value.

### 2.4 cgap-bn3 — unify.rs inline courts vs edge_closure_courts

Overlapping claims, different layers:
- no-match empties: src/unify.rs:2298 `kernel_court_no_match_is_empty`
  (MatchSet level) vs tests/edge_closure_courts.rs:165 e1 (fixpoint
  batch level, plus chain twin). Layered — both survive.
- zero-alloc steady state: src/unify.rs:2655 (scan/project window,
  4096-row batch) vs tests/edge_closure_courts.rs:422 e5 (driver
  window). e5 additionally witnesses the AE7 honesty arm (fixpoint IS
  allowed to allocate, edge_closure_courts.rs:501-509 region) which the
  inline court does not. Both survive.
- differential oracle: src/unify.rs:2431 (100 witnessed corpora) vs
  e3's saturated-bucket probe (:268). Distinct inputs. Both survive.

Verdict: no safe deletions in bn3. unify.rs's inline module IS a court
suite (named *_court) with its own mutants; edge_closure adds the
driver/fixpoint layer and retraction-cascade shapes the inline module
never builds. The duplication is by layer, not by assertion.

### 2.5 cgap-git — ladder/idx probe constancy, 3 sites

Stage-count constancy (probes == LADDER_STAGES == 16) is asserted in:
- src/ladder.rs:263-291 `iteration_count_constant_hit_miss_edge`
  (pure predicate level)
- src/idx_read.rs:466 `ladder_iteration_count_constant_through_idx_lookup`
  (through IdxV2::lookup — witnesses the adapter path)
- tests/ladder_mutations.rs:519-527 (constancy must survive a
  sorted-order violation — a mutation flip neither src side has)
- (recorded, not gated) tests/perf_baseline.rs:115-133

Verdict: each side witnesses a different flip (pure predicate, adapter,
mutated table). Keep all three; this is the "witnessed flip" rule in
action, not a cull target. Similarly inflate.rs inline typed-error tests
(inflate.rs:527-614) vs tests/fuzz_codec.rs differential buckets: inline
pins exact error variants; the court pins dual-reader agreement and
bucket distributions. Overlap is only the generic round-trip; both
survive. dag.rs inline d1-d9 have NO tests/ counterpart (lineage_
workflow and soc_full_chip are different claims) — keep.

## 3. Summary of proposed deletions

Definite (subsumed, flip witnessed on the surviving side):
1. csct src/types.rs:150, :169, :191 — delete 3 inline fns
2. csct src/receipt.rs:184 — delete 1 inline fn
Optional:
3. csct tests/csct_pipeline_court.rs:123-126 — single-variant pin,
   subsumed by coverage court's exhaustive pin
4. cgap-parquet src/rle.rs:287, :304 — subsumed by rle_court sweeps

Everything else: keep. The dominant pattern across the five crates is
layered redundancy (kernel inline vs public-API court), not duplication;
in bn3, cgap-git ladder/inflate/dag, and the object_store/ingress_ring
candidates the src-inline vs court boundary holds as designed.
