# lane-02-semantic-oracle — AE2 backfill receipt (SATISFIED-BY mapping)

Date: 2026-11-02. Lane: F3 (AE-lane receipt backfill, per W7 metagate).
Branch: main, canonical checkout /Users/sac/cgap. Doc-only lane:
read-only vs source, no git transitions, no source edits.

## W7 metagate finding (what this receipt closes)

W7 found AE2's `dred_adversarial_oracle` court NEVER landed
(NOT-STARTED): no test file named for the adversarial-oracle contract
exists, and AE2's lane receipt was never written. This receipt records
the backfill decision: the AE2 spec is ALREADY SATISFIED by landed
courts — REP4's `dred_cascade` 5/5, SP5's self-play cascade witness,
and C6's churn bench legs — so building a duplicate `dred_adversarial_
oracle` test file would be redundant surface with two owners. DEDUP
DECISION: no new test file; the mapping below is the receipt of record.

## Spec-item → landed-court mapping (invariants 1+2, depth >= 500)

| Spec item (adversarial-oracle contract) | Landed court (file:line) | Witness exit |
|---|---|---|
| Invariant 1 — cascade purges the full lineage; root delete of a linear chain leaves 0 phantoms | `crates/cgap-storage/tests/dred_cascade.rs:69` `court_chain_root_delete_no_phantoms` (chain N=10,000 -> 0 survivors, guarded); in-court flip mutant `cascade_retract_mutant_nocascade` at `dred_cascade.rs:88-105`, proc fn `crates/cgap-storage/src/dred.rs:416` | F3 session run `cargo test -p cgap-storage --test dred_cascade` exit 0, 5 passed / 0 failed; repair-04 receipt cites exit 0 |
| In-court mutant flip (anti-vacuity, in-court form) | `dred_cascade.rs:88-105` asserts the mutant yields exactly 9,999 phantoms ("mutant no longer reproduces the FA5 chain" = court flip); PLUS the /tmp sandbox mutation cycle in `repairs/repair-04-dred-cascade.md` (BFS dropped, court FAILED with the FA5 signature, sha256-exact restore 98eee5e6...) | repair-04 receipt (this campaign) |
| Invariant 2 — alt rescue: alt-supported derived facts survive the purge | Court 2 `dred_cascade.rs:113` `court_diamond_alt_rescue` (diamond, survivors {2,3}); Court 3 `dred_cascade.rs:131` `court_cycle_alt_rescue` (3-cycle purges whole without alt, {2} survives with alt) | same run, exit 0 |
| Invariant 2 — cycles terminate with typed semantics | Court 3 (cycle corpus, `dred_cascade.rs:127-145`) | same run, exit 0 |
| Depth >= 500 without recursion (CSR BFS, non-recursive) | Chain N=10,000 (depth 9,999 >= 500) in Court 1; BFS is iterative over a caller-leased worklist — `dred.rs:319 cascade_retract`, `dred.rs:255 build_csr`; overflow is typed (`CascadeError`), never stack recursion | Court 1 of the same run, exit 0 |
| Oracle set-equality (least fixpoint, cycle-safe) | Court 4 `dred_cascade.rs:151` `court_oracle_equivalence`: cascade == naive `cascade_oracle` (`dred.rs:440`) over 8 deterministic pseudo-random possibly-cyclic corpora, 32..704 facts, corpus seed 0xa20c0ffee5a7e000 | Court 4, same run, exit 0 |
| Typed refusals (lease under-provisioning, out-of-range edge) | Court 5 `dred_cascade.rs:191` `court_typed_refusals` (WorklistCapacity / EdgeOutOfRange / AdjacencyLease / CountsLease) | Court 5, same run, exit 0 |
| Bench leg (retraction fast path, churn protocol) | `crates/bench/benches/dred_churn.rs` (C6/w5-g4); after-repair rows 943.3-943.7M facts/s (1.54x/3.79x/5.87x oracle at 10/50/90% churn), BENCH-OK exit 0 | receipt `waves/w5-g4-dred-churn.md` + `repairs/repair-04-dred-cascade.md` (their session) |
| SP5 self-play integration witness | `crates/cgap/tests/self_play_radix_dred_cascade.rs` (246 lines, on disk) | CL2 census `closure/lane-02-self-play-witness.md:23`: 2/2 ok |

## F3 session receipts (reads + exits, this session)

| Command (cwd /Users/sac/cgap) | Exit | Note |
|---|---|---|
| CARGO_TARGET_DIR=/tmp/f3-target cargo test -p cgap-storage --test dred_cascade | 0 | 5 passed / 0 failed — fresh witness for the mapping rows above |
| Read repairs/repair-04-dred-cascade.md; reads of dred_cascade.rs, dred.rs, w5-g4-dred-churn.md, closure/lane-02-self-play-witness.md | 0 | cites verified on disk |

## Standing

ALIVE for the contract as mapped: every AE2 spec item resolves to a
landed court file:line whose owning run exited 0 this session (or, for
the bench leg, cites repair-04/w5-g4 receipts). The court count is 5
in dred_cascade plus SP5's 2 self-play courts; the adversarial-oracle
contract is covered without a duplicate file. Perf numbers above are
cited from receipts, not re-measured by F3.

## Divergences

1. The dispatch's name `dred_adversarial_oracle` matches no landed
   symbol; the contract lands under `dred_cascade` naming. Recorded
   here, not renamed.
2. Bench-leg numbers are cited from repair-04/w5-g4 receipts (their
   session), not re-measured by F3 — F3 is doc-only by contract.
