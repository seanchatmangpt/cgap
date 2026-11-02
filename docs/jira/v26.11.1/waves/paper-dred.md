# PP-3 DRED Receipt: SWAR Lineage-Bitmask Retraction

- Lane: PP-3-dred (paper lane), 2026-11-01
- Home (disclosed): `crates/cgap-storage/src/dred.rs`, NOT `cgap-parquet`.
- Crate sha256 (on-disk, post-run): `ac4eab977b30403888efe6c7bcfa0e077ca36d98fb929dffff7db7b4a34f0792`

## Home choice + reason

Lineage/retraction is storage semantics, not codec semantics: derived facts
are journaled alongside the WAL new-root descriptors of
`crates/cgap-storage/src/atomic_commit.rs` (A05). cgap-parquet is codec
kernels (RLE/FFOR/unpack) with no derivation concept. `dred` sits next to
the commit engine it will retract through.

## Kernel (crates/cgap-storage/src/dred.rs)

- `suspect_mask = derived_deps & del_mask` (word-parallel, u64 SWAR words)
- `keep = !suspect | (alt_support != 0)`, padding lanes purged via a valid
  lane mask so the emitted set is exactly the real facts
- `compact_lanes`: branchless cursor compaction — unconditional store at
  the cursor, cursor advances only by the validity bit. Prior art:
  `cgap_parquet::stream_compact::compact_lanes_branchless`
  (crates/cgap-parquet/src/stream_compact.rs), lifted shape, generalized
  from 8-lane u8 mask to 64-lane u64 words + lineage predicate.
- `retract_unchecked` (#[doc(hidden)], court-only mutant): drops the
  alt-support term. Never for production.
- `oracle_surviving`: independent scalar full-recompute referee.

## Courts (all in-module, #[cfg(test)], run this session)

1. `court_retraction_matches_oracle` — fast path vs scalar oracle over
   seeded (splitmix64, CORPUS_SEED/ENTROPY_SEED canonical) corpora at 6
   entropy levels x 3 sizes (64/1000/4096 facts) + 4-fact hand KAT:
   PASS (debug + release).
2. `court_antivacuity_alt_support_flip` — guarded path keeps the
   alt-supported fact; `retract_unchecked` purges it: PASS. On-disk /tmp
   flip: sandbox copy of the crate at /tmp/pp3-flip with the alt term
   mutated (`if false && ...`) makes ALL THREE courts fail, cargo exit 101
   (/tmp/pp3-flip-log.txt); canonical sha256 verified unchanged after.
3. `court_entropy_flat_variance` — measured ns/lane across 0-100%
   deletion-entropy: PASS with fence ratio < 4.

## Measured (release, min-of-200 Instant timings, 65536 facts, seeded)

| entropy | total ns | ns/lane |
|---|---|---|
| 0% | 51416 | 0.7845 |
| 10% | 51375 | 0.7839 |
| 20% | 51375 | 0.7839 |
| 30% | 51333 | 0.7833 |
| 40% | 51333 | 0.7833 |
| 50% | 51375 | 0.7839 |
| 60% | 51375 | 0.7839 |
| 70% | 51375 | 0.7839 |
| 80% | 51375 | 0.7839 |
| 90% | 51416 | 0.7845 |
| 100% | 51375 | 0.7839 |

**Entropy-variance finding: flat.** max/min = 1.002 (release), 1.006
(debug). Deletion entropy has no measurable effect on compaction cost —
the branchless cursor is data-oblivious in time. The flat-variance claim
is CONFIRMED, not just asserted.

## Reconciliation vs prior art / targets

- Paper 2.819 ns: TARGET, not measured claim. PP-1's challenger
  measurement (crates/bench/benches/challenger_parquet.rs,
  stream_compact at 0/50/100% selectivity, 2.738-2.86 ns per 8-lane
  compact) is a DIFFERENT granularity (per-8-lane call, divan) from this
  lane's per-64-lane-word amortized figure. The two agree that
  compaction cost is entropy-independent; absolute ns/lane figures are
  not comparable across granularity and are reported as measured, not
  reconciled to a target.

## Standing

ALIVE for: kernel, oracle equality at 6 entropy levels, alt-support flip
(in-process + /tmp on-disk), entropy-flatness 1.002. Debug figures are
per-lane ~16.3 ns (unoptimized); the 0.78 ns/lane figure is
release-profile only. Divan-level numbers (vs 2.819 TARGET) remain
UNVERIFIED pending a divan harness wired to dred — out of lane lease.

## Commands

- `CARGO_TARGET_DIR=/tmp/pp3-target cargo test -p cgap-storage --lib dred` exit 0
- `CARGO_TARGET_DIR=/tmp/pp3-target cargo test -p cgap-storage --lib` exit 0 (5 passed)
- `CARGO_TARGET_DIR=/tmp/pp3-target cargo test -p cgap-storage --release --lib` exit 0 (5 passed)
- `CARGO_TARGET_DIR=/tmp/pp3-target cargo build --workspace` exit 0
- /tmp flip sandbox: `cargo test --manifest-path /tmp/pp3-flip/cgap-storage/Cargo.toml --lib dred` exit 101 (FLIP WITNESSED)
