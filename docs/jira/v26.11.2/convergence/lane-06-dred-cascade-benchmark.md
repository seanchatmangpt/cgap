# Lane C6 — DRed dual-mechanism bench legs (receipt)

- **Subject**: `crates/bench/benches/dred_churn.rs` (additive CASCADE +
  FASTPATH legs) + disclosed minimal unblock in
  `crates/cgap-storage/tests/court_bitplane_seen.rs`.
- **Standing**: ALIVE (commands + exits this session, below).

## Legs

| Leg | Mechanism | Result |
|---|---|---|
| FAST-PATH | static bitmask filter (`dred::retract`), 10/50/90% churn, 2^18 facts | 943,530,826 facts/s at 90% churn (277,833 ns), 15M-target MET — receipted certification preserved, output format untouched |
| CASCADE | `dred::cascade_retract` over the N=10,000 linear chain (FA5 corpus), root deleted, alt_support all 0 | `CASCADE_LEG chain=10000 ns_per_lane=3.7333 survivors=0 allocs=0` |
| ANTI-VACUITY | `dred::cascade_retract_mutant_nocascade` (pre-REP4 filter-only, BFS dropped) | `CASCADE_MUTANT_FLIP survivors_mutant=9999 expected=9999 FLIPPED` — in-bench fresh flip, matches repair-04 witness |

## Alloc witness (REP4 zero-heap claim)

Counting `#[global_allocator]` (wraps `System`, counts alloc/realloc) in the
bench binary; counter armed only around the `cascade_retract` walk
(read-and-reset), max across 30 min-of-N rounds = **0 allocations** on the
hot path. Asserted loud: any nonzero count fails the bench.

## Commands (this session)

- `CARGO_TARGET_DIR=/tmp/c6-target cargo bench -p bench --bench dred_churn` → exit 0
  (`BENCH-OK`; FASTPATH_LEG + CASCADE_MUTANT_FLIP + CASCADE_LEG lines present)
- `CARGO_TARGET_DIR=/tmp /c6-target cargo test -p cgap-storage` → exit 0
  (all suites green; includes REP4's 5 dred_cascade courts: oracle
  equivalence, chain root delete no phantoms, cycle/diamond alt rescue,
  typed refusals)

## Disclosed minimal unblock (SLA, disclose-in-both)

`crates/cgap-storage/tests/court_bitplane_seen.rs`
`court_membership_matches_btreeset_oracle_10k` aborted (SIGABRC, stack
overflow) BEFORE this lane's edits — the 128-plane x 16384-slot
`BitPlaneSeenSet` is 16 MB inline, past the default 2 MB test-thread stack
(`Box::new` still overflows in debug: the 16 MB value is staged on the
caller stack pre-move). Fix: run the court body on a 64 MB explicit-stack
thread. No semantics changed. Disclosed here for the owning lane's
receipt; cgap-storage source untouched.

## Anti-vacuity

The cascade leg's survivor==0 assert IS the court; it is provably
non-vacuous because the exported court-only mutant
(`cascade_retract_mutant_nocascade`) leaves exactly 9,999 phantom
survivors on the same corpus — flipped in-bench each run
(`CASCADE_MUTANT_FLIP ... FLIPPED`), consistent with the repair-04
forensic witness.

## Notes

- The dispatch's "~943M facts/s" certification re-measured this session:
  943,530,826 facts/s (90% churn, min-of-30) — matches.
- Perf numbers are measured wall-clock min-of-30 (`Instant`), method
  stated in bench header.
