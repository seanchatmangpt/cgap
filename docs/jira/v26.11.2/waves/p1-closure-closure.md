# p1-closure receipt (lane p1-closure)

Semi-naive transitive closure on arrow columns. Module
`crates/cgap-storage/src/closure.rs` (cgap-storage; name clean of the
forbidden word, grep-witnessed below). Courts:
`crates/cgap-storage/tests/closure_court.rs`. Bench:
`crates/bench/benches/closure.rs` (harness=false, registered in
`crates/bench/Cargo.toml`).

## Disclosure: bitplane lane landed mid-flight

At authoring time no `p1-bitplane-*.md` receipt existed in
`docs/jira/v26.11.2/waves/` (witnessed this session); then
`bitplane_seen.rs` landed in cgap-storage from a sibling lane while
this lane ran. Per contract, consumed as the dedup collaborator in
court 2 (`semi_naive_closure_bitplane_count`). Divergence (typed):
the engine's primary seen-set is a packed `(x<<32)|z` hash set, NOT
the bitplane, because `BitPlaneSeenSet::contains` is O(slots) per
probe and the bitplane is fixed-capacity (an N^2 plane is 4 TiB at
N=1048576). Bitplane parity is proven at a feasible 40-node /
180-pair corpus; the bench path uses the packed-hash seen-set.

## Courts (real collaborators, final-state asserts)

1. Oracle: naive BTreeSet fixpoint, SET equality at N in {1k, 10k},
   splitmix64-seeded from 0xA20C_0FFE_E5A7_E000 (corpus-seed pin; the
   multiplier 0xBF58_476D_1CE4_E5B9 lives only inside splitmix64).
2. Bitplane parity: packed-hash count == bitplane count (1024-slot
   capacity) on the 40-node corpus, plus a loud `Err(SeenFull)`
   capacity-refusal assert (a 4-slot set must refuse, not drop).
3. Mutation (anti-vacuity): drop the second atom (B) ->
   `mutant_base_only_count`; `assert_ne!(full, mutant)` at N=1k/10k —
   the court flips (full=24500 vs mutant=980 at 1k scale), not
   VACUOUS. The mutant is a court-only fn in closure.rs (in-repo
   court pattern per dred.rs / bitplane_seen.rs precedent; the shared
   tree was never mutated in place).

## Commands + exits (witnessed this session)

- `CARGO_TARGET_DIR=/tmp/p1-closure-target cargo test -p cgap-storage
  --test closure_court` -> exit 0, `3 passed; 0 failed`. First run
  FAILED: the engine emitted deltas without appending them to the
  output columns; defect repaired, court re-run green — the court
  caught a real defect, so it is non-vacuous by construction too.
- `CARGO_TARGET_DIR=/tmp/p1-closure-target cargo bench -p bench
  --bench closure` -> exit 0:
  - N=65536 (chain_len=8): pairs=229376, best=0.020333s,
    pairs_per_sec=11280787
  - N=1048576 (chain_len=32): pairs=16252928, best=3.258673s,
    pairs_per_sec=4987591
  Units: pairs/s only. Method: harness=false Instant, min of 4 runs.
  Status: [TARGET-v26.11.2] (measured this session with the stated
  method, but not Divan-instrumented; Divan flip is a close condition).
- `CARGO_TARGET_DIR=/tmp/p1-closure-target cargo build --workspace`
  -> exit 0.

## Naming / pin hygiene

- `grep -rn "<forbidden-word>" closure.rs closure_court.rs
  benches/closure.rs` -> no matches (rc 1), witnessed; the literal
  token itself is never reproduced in any cgap artifact (naming law).
- Seeds: corpus 0xA20C_0FFE_E5A7_E000; algorithm multiplier
  0xBF58_476D_1CE4_E5B9 lives only inside splitmix64. Never conflated.

## JSON

```json
{"lane":"p1-closure","files_written":["crates/cgap-storage/src/closure.rs","crates/cgap-storage/src/lib.rs (module registration)","crates/cgap-storage/tests/closure_court.rs","crates/bench/benches/closure.rs","crates/bench/Cargo.toml ([[bench]] closure entry)","docs/jira/v26.11.2/waves/p1-closure-closure.md"],"commands":["cargo test -p cgap-storage --test closure_court (exit 0, 3/3)","cargo bench -p bench --bench closure (exit 0; 11280787 pairs/s @65536, 4987591 pairs/s @1048576)","cargo build --workspace (exit 0)"],"courts_run":["court1_oracle_set_equality PASS","court2_bitplane_parity PASS","court3_mutation_drop_second_atom PASS (flips)"],"standing":"ALIVE","divergences":["seen-set is packed-hash, not bitplane, at bench scale (O(slots) contains + 4 TiB plane infeasible at N=1048576); bitplane parity proven at 40-node corpus","bench is Instant harness=false, not Divan; numbers stay [TARGET-v26.11.2]"]}
```
