# PP-2 BUILD-SIP Receipt (paper-sip)

Lane: PP-2 BUILD-SIP · 2026-11-01 · repo /Users/sac/cgap @ main

## Subject

- `crates/cgap-parquet/src/bitslice.rs`
  sha256 (post-write, this session)
  `14ca6a83f8c27d2a74de8e58dc9d1a5425ca5b09f1b3893d5ed08b92d0a67044`
- Prior art (per coordinator re-grounding): `crates/bench/benches/challenger_parquet.rs`
  already benched the lifted kernel (`pack_vertical` / `scan_range_vertical`
  imported from `cgap_parquet::bitslice`); this lane productionized the
  surface, not a blind re-derivation.
- `docs/jira/v26.11.1/waves/paper-sip.md` (this file).
- `lib.rs` `pub mod bitslice;` already present (line 21); NO lib.rs edit
  was made, so the concurrent PP-3 dred.rs edit has no collision with this
  lane.

## What landed

1. `semi_join_masks(a, b, out) -> u64` — mask-AND semi-join composition,
   returns the qualifying-row popcount (final-state assertion surface).
2. `semi_join_two_hop(col_a, col_b, w, ...) -> u64` — two range scans +
   mask-AND, the multi-hop composition; differential court asserts the
   composed mask is bitwise the row-wise conjunction.
3. Courts (in-file, canonical seeds): `court_differential_bitplane_equals_rowwise`
   (bit-plane == row-wise, bitwise, w ∈ {4,8,12,16}, n ∈ {1,64,4096,100003},
   4 range classes, seed 0xBF58_476D_1CE4_E5B9 / corpus XOR 0xA20C_0FFE_E5A7_E000);
   `court_differential_two_hop_semi_join`; `court_mutation_control_disagrees`
   (in-tree anti-vacuity: mutant cascade dropping `lo_lt` must over-admit);
   `pruning_fires_on_random_data` (pruning-dead mutation control).
4. Ignored perf harness `bench_bitslice_perf_and_pareto` (min-of-5 over
   20k iters, no Process.sleep, black_boxed inputs).

## Commands (this session, exits quoted)

- `CARGO_TARGET_DIR=/tmp/pp2-target cargo test -p cgap-parquet --lib bitslice`
  → exit 0: 13 passed, 0 failed, 1 ignored (all courts green).
- `CARGO_TARGET_DIR=/tmp/pp2-target cargo test -p cgap-parquet --release
  --lib bitslice -- --ignored --nocapture` → exit 0, measured:
  - single_hop_scan 990 ns/4096 (w=8, canonical seed)
  - two_hop_scan+AND 2073 ns/4096
  - PARETO: scan_a=1010, scan_b=1010, and=0 (sub-ns; 64 AND words is
    below timer precision 41 ns) ns/4096
- `CARGO_TARGET_DIR=/tmp/pp2-target cargo bench -p bench --bench
  challenger_parquet -- range_scan --sample-count 10 --max-time 8` → exit 0:
  vertical_bitslice 989.2 ns fastest / 994.5 slowest; horizontal incumbent
  4.374 µs fastest → 4.42x this run.
- `cargo rustc -p cgap-parquet --release --lib -- --emit=asm` → exit 0;
  per-symbol census (correct symbol bounds):
  semi_join_masks 76 lines 0 blr / 0 br xN / 0 brk;
  semi_join_two_hop 210 lines 0 / 0 / 0;
  scan_range_vertical_128 339 lines 0 / 0 / 0;
  scan_range_vertical_counted 208 lines 0 / 0 / 0; `bl` sites are
  panic/bzero/alloc/collaborator paths only, zero inside loop spans.

## Court (non-vacuity, /tmp sandbox flip)

- Sandbox `/tmp/pp2-court` (copytree of the crate; manifest inlined;
  driver `pp2court` with the canonical-seed differential vs row-wise).
- Baseline (sha256 14ca6a83...) build 0, run `DIFFERENTIAL PASS` exit 0.
- Mutant M1: `out_valid[g] = !(hi_gt | lo_lt);` → `!(hi_gt);` (drops the
  lo bound). Build 0, run `DIFFERENTIAL FAIL extra=827 missing=0` exit 101.
  COURT FLIPS. (A second in-tree control,
  `court_mutation_control_disagrees`, also flips: asserts over-admission.)
- Restore: byte-exact (sha256 match). Sandbox + `/tmp/pp2-target` removed
  via `python3 shutil.rmtree`.

## Measured vs paper (divergence policy applied verbatim)

- Paper/compendium claim: 973.5 ns/4096 (4.06x).
- Measured this session: 989.2 ns (divan) / 990 ns (in-file harness);
  PP-1 first-hand 978.7 ns. All within ~1% of the claim — MATCH, paper
  number NOT corrected (no contradiction). Speedup this run 4.42x vs
  incumbent 4.374 µs (paper 4.06x vs 3.957 µs; incumbent host variance).
- Multi-hop cost: two-hop ≈ 2 x single-hop (2073 vs 990 ns); mask-AND
  composition alone measures below the 41 ns timer precision — the
  composition is scan-dominated (~97%+ of two-hop cost is the two scans).

## Standing

- ALIVE: kernel + courts ran this session (commands + exits above).
- Leaves: divan alloc-profile of the two-hop path not run (UNVERIFIED;
  single-hop 0 B alloc claim in the compendium was not re-measured here).

