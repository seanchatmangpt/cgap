# AU4 / Phase 1 — Deductive engine and joins (measured 2026-11-02)

## 1. Engine inventory (LOC via `wc -l`, measured this session)

cgap-bn3 (`crates/cgap-bn3/src/`), 10,968 lines total:
- `unify.rs` — 2,946 (prompt figure 2,910 stale by one wave): `RuleCompiler`
  (:553), radix semi-join 4,096 buckets (`radix_bucket` :850,
  `bucket_candidates` :887, `semi_join_masks` :914), `SplitStore` fixpoint
  view (:457), `unify_to_fixpoint` (:1523), typed round budget (:196, :274,
  court :2390).
- `sigil.rs` 857 — codec `encode_term` :265, `BN3Statement` :313-367.
- `arena.rs` 2,284 — slab. `project.rs` 1,472 — `DeltaRows` canonical bytes
  :262-318, `ConsequentProjector` :343. `noalloc.rs` 975 — bounded arm,
  `NA_MAX_MATCHES`=2048 :35, `na_compile` :184, `na_unify` :385,
  `na_unify_radix` :416. `builtins.rs` 1,495 — static dispatch. lib.rs 10.

cgap-storage (`crates/cgap-storage/src/`):
- `closure.rs` 268 — `semi_naive_closure` :114, `naive_closure` oracle :167,
  `mutant_base_only_count` :207, `semi_naive_closure_bitplane_count` :225
  over `BitPlaneSeenSet` (`new_boxed` bitplane_seen.rs:54).
- `dred.rs` 661 — `retract` static filter :122, `build_csr` :261,
  `cascade_retract` :325, `CascadeError::WorklistCapacity` :243,
  `cascade_retract_mutant_nocascade` :422, `cascade_oracle` :446. Plus
  `atomic_commit.rs`, `bitplane_seen.rs`.

cgap-bont (`crates/cgap-bont/src/`, 6,327 total): SlotId primitives
`primitives.rs` 1,519; `oabi.rs` 846; `shacl_compiler.rs` 3,421;
`shacl_mask.rs` 536. cgap-bosym: `slab.rs` 1,521.

## 2. Algorithmic ceilings (measured, receipts on disk)

- Radix semi-join vs nested: **5.6x @65,536, 78.3x @1M** (1.858 s vs 23.72 ms
  fastest; 78.3x median). `docs/jira/v26.11.2/repairs/repair-06-radix-default-flip.md:50-66`.
  Honest caveat (TS3, `repairs/repair-20-xorfilter-gate.md:58-60`): the older
  22.99 ms fence was NOT reproduced in the TS3 session (both arms 24.2-25.3 ms
  median) — "recorded UNVERIFIED against this session's method, not claimed
  met." HANDOFF.md:128 likewise marks G2-PARETO rollup rows UNVERIFIED.
- Closure throughput: 4.85M-11.25M facts/s/core (`waves/w5-g3-mat-closure.md:43`);
  re-measured 10.36M / 7.55M / 5.77M with -8.0% / -8.2% / +19.1% drift
  (`mega-bench/lane-04-datalog-closure.md:16-20`).
- DRed fast path: 943,388,214 / 943,673,481 / 943,530,826 facts/s at
  10/50/90% churn (277.8 us retract) — the **static bitmask filter**,
  explicitly NOT lineage unwinding. The DREDLABEL errata
  (`waves/w6-dred-relabel-errata.md`; cited at
  `adversarial/THE-SMOKING-GUN-REPORT.md:55,102`) relabeled the earlier
  "retraction" framing as filter semantics; pre-errata receipts were flagged
  FRAUD by the adversarial lane. Lineage closes via `cascade_retract` (REP4,
  `repairs/repair-04-dred-cascade.md`): O(V+E) CSR cascade BFS, N=10,000
  chain root delete leaves **0 survivors**; in-court mutant witnesses the
  9,999-survivor FA5 signature (repair-04:51-55,99);
  `hardening/lane-07-dred-cascade-verified.md:76`: "non-transitive filter
  semantics -> `retract`"; cascade covers lineage.

## 3. Soundness machinery

- Alt-support rescue: dred.rs:9 law — purge iff derived AND deleted AND no
  alternative support; :408 `keep = !(static suspect) | alt-rescue, AND-ed
  with !purged`; cascade applies alt rescue only to non-deleted derivations (:319).
- Typed capacity refusals: `NA_MAX_MATCHES` (noalloc.rs:35) oversize sets
  "refuse LOUDLY" (:942, court :949); `WorklistCapacity{capacity, facts}`
  enforced at dred.rs:340,372,398, in-court witness tests/dred_cascade.rs:204-221.
- Budget exit: typed round budget (unify.rs:196,274); "looping forever" and
  "ignoring the budget" mutants MUST flip the court (:2363,2387-2390).
- Oracle differential courts: `naive_closure` oracle (closure.rs:167) with
  in-bench anti-vacuity fence (benches/mat_closure.rs:69-72);
  `differential_court_matches_naive_oracle_over_100_witnessed_corpora`
  (unify.rs:2431) + occurs-check self-join oracle court (:2158);
  `court_retraction_matches_oracle` / `cascade_oracle` (dred.rs:446,529);
  bitplane vs BTreeSet oracle over 10k (repair-04:64). Anti-vacuity flips on
  record: radix-default flip exit 1 (repair-06:72-81); xorfilter probe-gate
  bit-flip 0 passed / 3 failed (repair-20:37-40).

## 4. ANALYSIS (labeled — interpretation, not measurement)

This is deductive-database engineering in the SOUFFLE class: the radix
semi-join with precomputed masks is worst-case-join-avoidance via semi-join
masking; `semi_naive_closure` + the `SplitStore` delta view is classic
semi-naive differential evaluation; `BitPlaneSeenSet` is bit-plane dedup of
the seen set; `dred.rs` is DRed-style view maintenance — static filter pass
plus CSR lineage cascade, i.e. DRed's filter/cascade structure. The measured
ceilings are the kind of algorithmic claims deductive-DB research groups
publish. Two honest notes temper the framing: (a) cgap's DRed lineage
cascade is newly landed (REP4, this cycle) — the long-standing mechanism was
only the non-transitive filter, and the pre-DREDLABEL "retraction" receipts
were relabeled post-hoc by the adversarial lane; (b) the 78.3x headline
carries an unreproduced older fence number (TS3 UNVERIFIED), so it is a
same-session differential, not a cross-session reproduced figure. Capacity
is bounded by typed refusals rather than silent truncation, matching the
refuse-loudly convention used elsewhere in the codebase.
