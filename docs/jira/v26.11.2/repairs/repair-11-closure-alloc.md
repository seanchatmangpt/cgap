# Repair-11: closure.rs allocation elimination (lane F9)

Date: 2026-11-02. Owner: lane F9. Spec: H8 hardening/lane-08 §5.
Measurement basis: MB4 (1,117 allocs / ~1.20 GB @ N=1,048,576).

## Design

Two defects named by H8, both fixed in
`crates/cgap-storage/src/closure.rs` (sole file ownership):

1. **Fresh delta Vec per round** (closure.rs:118-133): replaced with a
   persistent double buffer — four pre-sized `Vec<u32>` (`cur_x/cur_z`,
   `nxt_x/nxt_z`), swapped per round via `std::mem::swap` + `clear()`;
   capacity retained, zero per-round realloc. Same for all four delta
   buffers pinned to the base edge count.
2. **Unpinned-growth seen-set** (`SeenSet`, closure.rs:33): usage is
   insert-only membership, so the defect was never the hash-set
   structure but UNPINNED growth — `HashSet::new()` then ~11 organic
   re-tables over a 25.7M-pair closure. Fix: `SeenSet::with_capacity(2
   * |B|)` pre-sizing.

The H8-suggested BTreeSet-with-FixpointHasher / flat seen pattern was
tried, measured, and REJECTED on facts/s: an insert-only open-addressing
flat table measured 343k facts/s vs std HashSet 4.70M facts/s at the
N=1M bench bucket (linear-probe clustering on a 264 MB table). std
HashSet (hashbrown) retained, pre-sized. This is the D5-verified
membership-only reading: keep semantics, pin the growth.

## Measurements (all this session, command+exit in commands list)

Counting-allocator probe at N=1,048,576, chain_len=50, seeded corpus
(0xA20C_0FFE_E5A7_E000, splitmix64 shuffle), gated on the closure call
only:

| metric | BEFORE | AFTER | delta |
|---|---|---|---|
| allocs | 1,771 | 23 | 76.6x drop; ceiling 1000 — PASS |
| churn (bytes) | 1,675,546,700 | 1,117,013,416 | 1.50x drop |
| pairs / rounds | 25,689,475 / 49 | identical | byte-identical semantics |

mat_closure bench leg (`cargo bench --bench mat_closure`, harness=false,
Instant method, best-of, facts/s/core):

| bucket | BEFORE facts/s/core | AFTER facts/s/core |
|---|---| measured this session |
| N=65,536 cl=8 | 11,258,178 | 12,111,065 (+7.6%) |
| N=262,144 cl=16 | 7,852,339 | 8,500,586 (+8.3%) |
| N=1,048,576 cl=32 | 4,699,605 | 4,971,396 (+5.8%) |

No bucket regressed. Anti-vacuity in-bench fence (closure facts must
strictly dominate mutant_base_only_count) held on every run.

## Courts

- `tests/closure_court.rs` 3/3 (oracle set-equality, bitplane parity,
  drop-second-atom mutation) — PASSED, derived-set semantics
  byte-identical (25,689,475 pairs / 49 rounds identical before/after).
- New `tests/closure_alloc_court.rs` 3/3 (release mode; debug N=1M
  runs are impractically slow, ~196 s; release chosen for both modes'
  determinism of counted bytes):
  - court_f9a alloc falsifier: <1000 allocs on the calling thread via
    thread-local counting global allocator; anti-vacuity mutation =
    revert to unpinned growth + fresh per-round deltas (measured 1,771
    allocs — fails the assert).
  - court_f9b SeenSet semantics: boundary keys (0, u64::MAX), 50k
    inserts through growth, oracle set-equality at N=10k.
  - court_f9c churn envelope: < 1,675,546,700 bytes (the measured
    pre-repair churn; counted bytes are deterministic — the counter
    sums requested Layout sizes, not allocator behavior).
- Full sweep: `cargo test -p cgap-storage --release --tests` exit 0,
  11/11 suites ok (includes dred suites owned by other lanes — the
  dred lib courts that failed mid-session at dred.rs:551/576/652 were
  fixed by their owning lane concurrently and pass in the final sweep).

## Divergences / corrections vs H8's spec

1. The flat/BTreeSet-FixpointHasher suggestion was measured and
   rejected on facts/s (14x regression) — pre-sized std HashSet is the
   landed mechanism; H8's falsifier (<1000 allocs) is satisfied by
   pre-sizing, not by the flat table.
2. Court C's envelope is the measured pre-repair churn (1.676 GB), not
   the initially drafted 1 GB (unsatisfiable: growth-doubling of the
   25.7M-pair output vectors alone exceeds 1 GB).
3. MB4's 1,117 allocs vs this session's 1,771: same defect class,
   different corpus shape (MB4 basis unknown to this lane); both far
   above the 1000 ceiling.
4. A mid-session incident is disclosed: the probe's process-global ON
   gate cross-counted parallel tests (7.9M allocs witness) — fixed
   with thread-local counters; and a BEFORE-bench measurement
   temporarily swapped pristine closure.rs into the shared tree
   (sha256 fail-closed restore was verified by MISMATCH-then-reapply,
   final sha256
   a0d4d2799d0096711f427e5028ac2d554dbe1debc778ac0ff73add089f7eff50).
