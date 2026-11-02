# p1-dred-retraction-antijoin

Lane p1-dred (v26.11.2): retraction as columnar anti-join in
`crates/cgap-storage/src/dred.rs` — retracted-delta x rule-premises
anti-join -> invalidated rows -> negative bitmask filter, zero-heap hot
path (caller-leased scratch, no allocation inside `retract`).

## State found (PARTIAL -> advanced)

`dred.rs` already carried the anti-join core: `suspect_mask`,
`keep_mask` (negative bitmask `!suspect | alt != 0`), branchless
`compact_lanes`, `retract` (caller-leased scratch), oracle
`oracle_surviving`, mutant (a) `retract_unchecked` (alt-support
bypassed), courts 1-3. Missing: mutation (b) witness.

## Added this lane

- `retract_inverted` (doc-hidden mutant (b)): negative-bitmap polarity
  inverted (`k = !suspect`, no alt rescue) — on an all-suspect corpus
  the surviving set goes EMPTY (all purged), oracle set-equality fires.
- Court 2b `court_antivacuity_inverted_bitmask`: KAT (guarded keeps the
  two alt-supported facts; inverted purges ALL 3) + corpus-level
  `assert_ne` vs oracle at facts=64/1000/4096 (100% churn) — a mutant
  matching the oracle would be VACUOUS and the court refuses.

## Commands (this session, /tmp/p1-dred-target)

- `cargo test -p cgap-storage --lib dred` — 4/4 ok (courts 1, 2, 2b, 3)
- `cargo test -p cgap-storage` — FAILED at `court_bitplane_seen`
  (stack overflow). PRE-EXISTING: file has zero diff vs HEAD, no dred
  usage (`grep -rn dred tests/` matches only `closure_court.rs:12`,
  which passes). Not session-introduced.
- `cargo bench -p bench --bench dred_churn` — BENCH-OK. Ratio fence
  (oracle/retract, ratio only): 1.60x @ 10%, 3.66x @ 50%, 4.85x @ 90%
  churn; facts/s [TARGET-v26.11.2]-class, min-of-30, wall-clock.

## Courts run

- Court 1 oracle set-equality (KAT + 6 entropies x 3 sizes): PASS
- Court 2 mutation (a) alt-support bypass flip: PASS (fires)
- Court 2b mutation (b) inverted negative bitmask flip: PASS (fires)
- Court 3 entropy flatness (max/min < 4.0): PASS
- dred_churn bench ratio fence + per-batch set-equality: PASS

## Standing

ALIVE for the lane surface (anti-join + both anti-vacuity mutations
witnessed + ratio fence). Standing vocabulary honored; perf claims are
ratios, never cross-multiplied units.

```json
{"lane":"p1-dred","files_written":["crates/cgap-storage/src/dred.rs","docs/jira/v26.11.2/waves/p1-dred-retraction-antijoin.md"],"commands":["cargo test -p cgap-storage --lib dred: 4 passed","cargo test -p cgap-storage: lib+closure ok, court_bitplane_seen SIGABRT stack overflow (pre-existing, zero diff vs HEAD)","cargo bench -p bench --bench dred_churn: BENCH-OK"],"courts_run":["court_retraction_matches_oracle PASS","court_antivacuity_alt_support_flip PASS","court_antivacuity_inverted_bitmask PASS","court_entropy_flat_variance PASS","dred_churn ratio fence PASS"],"standing":"ALIVE","divergences":["court_bitplane_seen stack overflow pre-existing on committed code, unrelated to dred, logged not repaired"]}
```
