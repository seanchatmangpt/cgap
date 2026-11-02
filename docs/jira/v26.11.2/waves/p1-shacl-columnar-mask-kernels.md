# p1-shacl-columnar-mask-kernels

Lane p1-shacl, v26.11.2 campaign. Subject:
`crates/cgap-bont/src/shacl_mask.rs` (rewritten from the 33-line hole-ridden
skeleton), `crates/cgap-bont/src/shacl_compiler.rs` (one added bridge fn),
`crates/cgap-bont/src/lib.rs` (module export).

## What landed

- `shacl_mask.rs`: columnar SHACL kernels over aligned `(slot_s, slot_p,
  slot_o)` columns, no SPARQL strings. `Constraint` = Cardinality (min/max
  count via group-by slot_s + count filter), Membership (vectorized
  membership on slot_o), HasValue (word equality on (p,o) pairs).
  `MaskPlan::evaluate` = scope kernel (rdf:type/target_class) + per-shape
  single-pass kernels; violations sorted by
  (shape_idx, constraint_idx, subject) for determinism. Fixed prior
  corruption holes (`None_hole`, `sh{hole2}in`, "at vhole3").
- `shacl_compiler.rs`: `mask_plan_from_ttl` — parses the existing TTL
  subset, maps `sh:targetClass` -> scope, `sh:minCount`/`sh:maxCount` ->
  Cardinality, `sh:datatype` -> Membership over the datatype slot. Shapes
  that are not `sh:closed true`, duplicate-closed, malformed, or missing
  targetClass refuse as typed `ShaclParseError` (never silent drops).
  The existing eligibility court of `compile_shapes` was NOT loosened:
  minCount>1/maxCount still refuse there (frozen W1 behavior); the mask
  plan carries arbitrary min/max via the direct plan API.
- `lib.rs`: `pub mod shacl_mask;`.
- Frozen pins held: corpus seed `0xA20C_0FFE_E5A7_E000` for the court/bench
  corpora (CorpusRng), algorithm multiplier `0xBF58_476D_1CE4_E5B9` in
  SplitMix64 — named separately, never conflated (bitslice.rs:687-699
  precedent). No new crates; DataFusion-free; naming law respected.

## Courts run (command + exit witnessed 2026-11-02)

- T1 oracle parity, court corpus: kernel evaluate == brute-force
  per-subject counting, exact violation set equality.
- T2 oracle parity, 5 SplitMix64 fuzz corpora (512 rows x 8 shapes).
- T3 mutation flip (in-test): minCount 1->2 flips the verdict set.
- T4 mutation flip (in-test): membership mask corruption flips the set.
- T5 TTL bridge end-to-end: shapes TTL -> mask plan -> evaluate == oracle.
- T6 bench: release build, `cargo test -p cgap-bont --release
  shacl_mask::tests::t6 -- --nocapture` ->
  `p1-shacl bench: 1676 shape-evals/s over 65536 statements x 8 shapes
  (total 47.704875ms, sink 217980)`. [TARGET-v26.11.2] until
  Divan-measured; units never cross-multiplied.

## Anti-vacuity shim courts (real /tmp sandbox, fail-closed restore)

Baseline (sandbox byte-identical to tree): `cargo test -p cgap-bont
shacl_mask` in /tmp/p1-shacl-sandbox -> ok, 6 passed.
- M1 floor shift (`n < min` -> `n + 1 < min` in the kernel) -> FAILED,
  3 courts flipped (t1/t2/t5). Exit nonzero.
- M2 mask corruption (`!allowed.contains(&o[i])` -> `false`) -> FAILED,
  2 courts flipped. Exit nonzero.
The sandbox + both target dirs were removed after the runs (lane leases
deleted at integration). Canonical tree was never mutated.

## Verification ladder

- `cargo test -p cgap-bont` -> ok, 41 passed; 0 failed (35 pre-existing
  + 6 new shacl_mask courts incl. t6).
- `cargo build --workspace` -> Finished (dev), exit 0 (pre-existing
  cgap-integration manifest warning unchanged).

## Standing

- Kernel + bridge + courts: ALIVE (commands + exits above, this session).
- Bench throughput: TARGET-v26.11.2 (std Instant, release build;
  Divan measurement pending).
- Divergences: `shacl_mask::evaluate` Cardinality/Membership/HasValue
  semantics defined here (shape scope = rdf:type/targetClass; Membership
  vacuously true when the subject carries no pred statements). The
  `compile_shapes` eligibility court remains intentionally stricter
  (refuses minCount>1/maxCount) than the mask-plan API — both surfaces
  documented, no behavior of the frozen compiler changed.

```json
{"lane":"p1-shacl","files_written":["crates/cgap-bont/src/shacl_mask.rs","crates/cgap-bont/src/shacl_compiler.rs","crates/cgap-bont/src/lib.rs","docs/jira/v26.11.2/waves/p1-shacl-columnar-mask-kernels.md"],"commands":["cargo test -p cgap-bont (exit 0, 41 passed)","cargo test -p cgap-bont --release shacl_mask::tests::t6 -- --nocapture (exit 0, 1676 shape-evals/s over 65536 statements x 8 shapes)","cargo build --workspace (exit 0)","/tmp shim court baseline: cargo test -p cgap-bont shacl_mask in sandbox (ok, 6 passed)","/tmp shim court M1 floor-shift mutation (FAILED, 3 courts flipped, exit 1)","/tmp shim court M2 mask-corruption mutation (FAILED, 2 courts flipped, exit 1)"],"courts_run":["t1_oracle_parity_court_corpus","t2_oracle_parity_fuzz","t3_min_count_mutation_flips_verdict","t4_mask_corruption_flips_verdict","t5_ttl_bridge_end_to_end","t6_bench_shape_evals_per_s","M1 sandbox mutation flip","M2 sandbox mutation flip"],"standing":"ALIVE","divergences":["compile_shapes eligibility court intentionally stricter (refuses minCount>1/maxCount) than mask-plan API; frozen behavior unchanged","bench is std-Instant release number, TARGET-v26.11.2 until Divan-measured"]}
```
