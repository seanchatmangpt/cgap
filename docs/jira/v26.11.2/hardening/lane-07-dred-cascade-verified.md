# Lane H7 — DRed cascade verification + fast-path bench

Date: 2026-11-02. Lane contract: READ-ONLY on source; only write = this receipt.
Build root lease `/tmp/h7-target` (deleted at lane end).

## (1) REP4 verification — cgap-storage suite

Command: `RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=/tmp/h7-target cargo test -p cgap-storage` — **exit 0**.
Full suite: 9 test binaries, 36 tests, 0 failed. `dred_cascade` **5/5**:
`court_chain_root_delete_no_phantoms`, `court_diamond_alt_rescue`,
`court_cycle_alt_rescue`, `court_oracle_equivalence`, `court_typed_refusals`.

**Divergence (environmental, pre-existing):** default test-thread stack
overflows in `court_membership_matches_btreeset_oracle_10k`
(`crates/cgap-storage/tests/court_bitplane_seen.rs:27`) — the 10k-pair
BTreeSet oracle in debug mode exceeds the default test-thread stack. Passes
deterministically with `RUST_MIN_STACK=67108864` (269s / 226s debug-mode).
Not a source defect; classified loud, not silent. `dred_cascade` itself runs
in well under 1s.

Mutant flip: in-court mutant `cascade_retract_mutant_nocascade` reproduced
exactly **9,999** phantom survivors against the guarded path's **0**
(`dred_cascade.rs:102-105`, the FA5 9,999→0 witness), plus the bench's
`CASCADE_MUTANT_FLIP survivors_mutant=9999 expected=9999 FLIPPED` line.

## (2) FA5 adversarial probe class (cascade API)

`/tmp/fa5-probe` exists but drives the OLD single-pass `retract`; lane
reconstructed the probe class against the cascade API in `/tmp/h7-probe`
(sandbox, not shared tree). Command:
`CARGO_TARGET_DIR=/tmp/h7-target cargo run --release --manifest-path /tmp/h7-probe/Cargo.toml` — **exit 0**.
Output: `CHAIN_N=10000 CASCADE_NS=55625 survivors=0`; purged-mask sanity
assert (all N facts purged) passed; honesty anchor on the same corpus:
`SINGLE_PASS retract survivors=9999` (asserted equal — the defect remains a
witnessable property of `retract`).

## (3) Fast-path bench (dred_churn, 2 runs, both exit 0)

`RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=/tmp/h7-target cargo bench -p bench --bench dred_churn`

| frac | facts/s (run 2) | vs REP4 (943.39M / 943.67M / 943.53M) | ≥900M |
|---|---|---|---|
| 10% | 943,527,430 | parity | MET |
| 50% | 943,388,214 | parity | MET |
| 90% | 943,670,084 | parity | MET |

Run 1 agreed within noise (943,388,214 / 943,388,214 / 943,530,826).
`NS_PER_BATCH retract=277792` (run 2; run 1: 277,833).
`ENTROPY_VARIANCE max/min=1.0012` (spread 0.0013 ns/lane, informational);
11 entropy points 0–100% all ≥ 942.5M facts/s.
All three fractions printed `15M-target MET`; probe-level 15M target remains
a TARGET (bench mechanism is the static path at ~943M facts/s).

Static single-pass path preserved, two ways:
- behavioral: `retract` still witnesses FA5 (9,999 single-pass survivors) —
  unchanged semantics, not cascade-delegating.
- structural: `dred.rs:116-133` — `retract` = `keep_mask` + `compact_lanes`
  only; `cascade_retract` is a separate lineage-walking entrypoint; the bench
  prints `FASTPATH_LEG mechanism=static_bitmask_filter` and
  `CASCADE_LEG chain=10000 ns_per_lane=3.74 survivors=0 allocs=0`.

## (4) Honesty ledger — API-semantics note

- `dred::retract` remains a **static single-pass bitmask filter**: it reads
  `derived_deps` as predicate bits and does NOT follow them as lineage edges.
  Root deletion on a chain leaves N-1 survivors (witnessed 9,999 this
  session). Correct only for callers whose delete semantics are
  non-transitive ("retract exactly the deleted+derived facts whose predicate
  bits say so").
- `dred::cascade_retract` is the **lineage-walking path**: CSR built via
  `build_csr` from real `(parent, child)` edges, worklist BFS bounded by
  alt-support rescue; 0 survivors on the same chain, typed
  `CascadeError` refusals on under-provisioned leases
  (WorklistCapacity / EdgeOutOfRange / AdjacencyLease / CountsLease).
- Caller rule: **transitive retraction semantics → `cascade_retract` with a
  real CSR; non-transitive filter semantics → `retract`.** The DREDLABEL
  relabel-stands note in the REP4 receipt stands unchanged for `retract`;
  the fast-path numbers (~943M facts/s) are a property of the static path
  and must not be attributed to `cascade_retract` (measured 55,625 ns for
  the 10k chain via cascade, ~3.74 ns/lane).

## Standing

ALIVE — every claim above cites a command + exit run this session.
