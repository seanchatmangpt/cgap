# Bench Receipt: chains-bn3 (P3-18 BENCH-BN3)

- **Lane**: P3-18-bench-bn3, repo `/Users/sac/cgap`, branch main @ cfd085a9 (dirty tree)
- **Date**: 2026-11-01
- **Files written**: `crates/bench/benches/chains-bn3.rs` (found complete on disk,
  token-swept line-by-line, verified against `cgap-bn3` API surface —
  `BN3Statement::from_words` sigil.rs:322, `CompiledRule{rule_id,atoms,var_count}`
  unify.rs:298-308, `ConsequentProjector::emit_into` project.rs:361 — and RUN,
  not rewritten), `docs/jira/v26.11.1/waves/bench-chains-bn3.md` (this receipt).
- **Commands**: `cargo bench --manifest-path crates/bench/Cargo.toml --bench chains-bn3`
  exit 0; `cargo bench --manifest-path crates/bench/Cargo.toml --bench bn3` exit 0.
  `CARGO_TARGET_DIR=/tmp/p318-target` (deleted after run via python3 shutil.rmtree).
- **Method**: Divan `bench_local`, 100 samples, bench profile, host-local, same
  session/host for both targets. Seed `0xBF58_476D_1CE4_E5B9` canonical SplitMix64.
- **Divergences**: (1) `r2-xip-noalloc.md` did not land within the bounded 25-min
  poll — benched the alloc-based arm as-is (disclosed, per dispatch). (2) Full
  `--benches` build aborts on a sibling lane's `chains-sovereign.rs` (E0277,
  crates/bench/benches/chains-sovereign.rs — pre-existing, not lane-owned); built
  and ran only `--bench chains-bn3` / `--bench bn3`. (3) `[[bench]] chains-bn3`
  declaration was already present in crates/bench/Cargo.toml (poll satisfied
  immediately; no manifest edit needed).

## Measured (median, bench profile)

| bench | N | median | throughput |
|---|---|---|---|
| deduction_chain (full chain) | 65,536 | 8.385 ms | 187.5 MB/s |
| deduction_chain | 1,048,576 | 1.470 s | 17.11 MB/s |
| stage_scan_24b | 65,536 | 125.8 us | 12.49 GB/s |
| stage_scan_24b | 1,048,576 | 2.048 ms | 12.27 GB/s |
| stage_compile_rules | 65,536 | 562 ns | — |
| stage_compile_rules | 1,048,576 | 567 ns | 0 |
| stage_unify_join | 65,536 | 6.808 ms | — |
| stage_unify_join | 1,048,576 | 1.472 s | — |
| stage_project_consequents | 65,536 | 170.7 us | — |
| stage_project_consequents | 1,048,576 | 2.086 ms | — |
| bn3::statement_scan_24b (isolated target) | 65,536 | 19.58 us | 80.32 GB/s |
| bn3::unify_join_2atom_pairs | 65,536 | 6.916 ms | — |

## Chicago / anti-vacuity

Asserts riding the benched closure: scan closure-count == N; unify selector/join
match counts == raw-word O(N) hash-join oracle (no unify code in the oracle path —
differential, not tautological); projected row counts == match counts; M12-7
source_mask_popcount == match counts; digest checksums stable across iterations.
Named mutation classes: skipped-row scan (count assert), binding-alias comparator
(doe-4 SHIM-DOE4 class, oracle assert), one-row-per-two-matches projector
(popcount assert), dropped join body atom (atoms.len assert). All asserts passed
in both runs — a mutation of any named class flips at least one assert.

## PARETO (mandatory)

Stage attribution of the chain (median, share of chain total):

| stage | N=65,536 | share | N=1,048,576 | share |
|---|---|---|---|~---|
| scan | 125.8 us | 1.50% | 2.048 ms | 0.14% |
| compile | 562 ns | 0.007% | 567 ns | ~0% |
| unify | 6.808 ms | 81.2% | 1.472 s | 99.9% |
| project | 170.7 us | 2.0% | 2.086 ms | 0.14% |

**~80% segment = UNIFY** (81% at 65k, degenerating to 99.9% at 1M — it scales
O(N) probe pairs while every other stage scales with bytes at line rate).
**20% lever**: the nested-loop join probe — probe work is
`sum(cur_len x cand_len)` pairs (65,536 pairs at N=65k; 1,048,576 at N=1M
pinned by the corpus shape). Replacing the per-chunk nested-loop with a hash/
trie join keyed on the bound (s,o) word pair collapses probe work from
O(cur x cand) to O(cur + cand) — the raw-word oracle inside this bench IS that
hash join and computes the same match count in setup time, demonstrating the
shape on the same corpus. Secondary observation: the isolated-target scan
(bn3.rs, XOR-fold, no column write) hits 80 GB/s while the chain's scan+SoA
column fill hits 12.3-12.5 GB/s — the SoA materialization write bandwidth, not
the read scan, is the scan-stage cost; a fused scan-into-unifier (no column
materialization) is the second-order lever once the join dominates.

Standing: ALIVE (both benches executed this session, exit 0, asserts passed).
