# Repair-04: DRed cascade lineage retraction (defect FA5)

Lane: REP4 · 2026-11-02 · crate `cgap-storage` (disclosed: cgap-storage,
not cgap-bn3 — the dispatch's crate name was wrong; dred.rs lives at
`crates/cgap-storage/src/dred.rs`).

## Defect (FA5)

`dred::retract` (dred.rs:116-127 pre-repair) is a single-pass bitmask
filter: it reads `derived_deps` as predicate bits but never follows them
as lineage edges. Root deletion of an N=10,000 linear chain left 9,999
phantom survivors. Reproduced this session BEFORE repair via
`/tmp/fa5-probe` (release, exit 0):

    CHAIN N=10000 retract_ns=16834 survivors=9999 (expected 0 if cascade)
    HEADLINE_REPRO facts=262144 best_ns=276500 facts/s=948.08M ns/lane=1.0548

## Design choice

The flat `derived_deps` bitmask is one bit per fact ("is derived") —
per-fact dependent LISTS are NOT expressible in the masks alone. Per the
dispatch's data-model-honesty clause, the strongest expressible cascade
was implemented: an explicit caller-leased CSR lineage adjacency built
from `(parent, dependent)` edges (`dred::build_csr`), with a zero-heap
BFS (`dred::cascade_retract`) over a caller-leased worklist (capacity ==
facts suffices: mark-before-push, each fact enqueues at most once ->
overflow is a typed `CascadeError`, never a silent drop). The static
single-pass `retract` is untouched (its courts and the churn bench are
byte-identical code paths, so the fast path cannot regress).

Cascade law: purged(root) for every deleted root (unconditional —
deletion is the actuation; the static rule's alt rescue applies only to
non-deleted derived suspects); purged(d) if purged(p), edge p->d, and
`alt_support[d] == 0`. Deleted+derived+alt>0 roots: the static path
keeps them, the cascade purges them (documented semantics difference;
courts pin both).

## Convergence argument

Each fact is marked purged BEFORE enqueue and never re-enqueued, so the
worklist holds <= facts entries and BFS terminates in <= facts pops.
The cascade computes the LEAST fixpoint of the rule above: BFS from the
deleted-root seed set saturating purged-along-edges-skipping-alt is the
standard least-fixpoint saturation; `cascade_oracle` computes the same
least fixpoint by naive rounds-based propagation (no CSR, no BFS,
O(V*E)-worst). Court 4 asserts set-equality cascade == oracle across 8
deterministic pseudo-random (possibly cyclic) corpora at 32..704 facts.

## Court receipts (cargo test -p cgap-storage, this session)

tests/dred_cascade.rs — 5 courts, exit 0:

- C1 chain N=10,000 root delete: guarded 0 survivors; in-court mutant
  `cascade_retract_mutant_nocascade` (doc-hidden, BFS dropped) yields
  9,999 — flip witness (a), pinned in-court.
- C2 diamond (0->1,0->2,1->3,2->3, alt on 2 and 3): survivors {2,3}.
- C3 cycle 0->1->2->0: no-alt purges all; alt on 2 -> {2} survives.
- C4 oracle set-equality, 8 pseudo-random corpora (may contain cycles).
- C5 typed refusals: WorklistCapacity / EdgeOutOfRange / AdjacencyLease /
  CountsLease all typed, never silent.

Existing suite: lib 6, closure_court 3, crc_identity 3, court_faults 5,
court_flash 3, court_mutations 6, court_sweep 2 — all ok, exit 0.
court_bitplane_seen `court_membership_matches_btreeset_oracle_10k`
stack-overflows (SIGABRT) PRE-EXISTING at HEAD: verified by extracting
git HEAD crates to /tmp/rep4-headcheck (patched manifests only to make
the standalone sandbox build; code byte-identical) and rerunning — same
SIGABRT. Not session-introduced; left for its owner.

## /tmp mutation flip witness (anti-vacuity)

Baseline sha256 (shared-tree dred.rs):
98eee5e6b901797c2a8f82e305af4d6a0706973dd0b81d876cab178859db66aa

Sandbox /tmp/rep4-sandbox/cgap-storage: BFS loop dropped (marked
/* SHIM-COURT mutation: BFS dropped */, `while false`), marked mutation,
court run → FAILED with exactly the FA5 signature:

    assertion failed: cascade left 9999 phantom survivors on the FA5 chain

Restore: cp from shared tree → sha256-exact (98eee5e6...), RESTORE-EXACT,
sandbox re-run 5/5 green. Shared tree sha256 identical before/after —
shared tree untouched by the mutation cycle.

## Bench leg (dred_churn, exit 0, BENCH-OK)

The bench times `dred::retract` — byte-identical code pre/post repair,
so the honest latency delta on the fast path is ZERO by construction;
measured delta within noise. After-repair rows (facts=2^18, rounds=30,
corpus seed 0xa20c0ffee5a7e000):

    10% | 943,388,214 facts/s | 1.54x oracle | retract 277,875 ns
    50% | 943,673,481 facts/s | 3.79x oracle | retract 277,791 ns
    90% | 943,530,826 facts | 5.87x oracle | retract 277,833 ns
    15M-target MET on all fractions; BENCH-OK exit 0

FA5 probe headline before/after (static path): 948.08M -> 947.94M
facts/s (+/-0.02%, noise). Cascade cost (separate fn, lease-only): the
FA5 chain cascade is O(V+E) BFS, chain N=10,000 measured in-court at
<1 ms debug-profile. Perf numbers are measured this session with the
stated methods; they are not targets.

## Standing

ALIVE: implementation + 5-court dred_cascade.rs green this session
(cargo test exit 0), mutation flip witnessed with sha256-exact restore,
bench leg exit 0. Divergences recorded: (1) dispatch said cgap-bn3,
actual crate cgap-storage; (2) flat derived_deps mask cannot express
dependent lists -> explicit CSR edge lease, strongest expressible
cascade; (3) pre-existing court_bitplane_seen SIGABRT at HEAD,
unrelated, disclosed.
