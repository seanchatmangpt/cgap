# Lane BN3D receipt: unify.rs doc cleanup + blr re-census

Date 2026-11-02. Repo /Users/sac/cgap @ main 9a730320. Branch main.
OWNERS: `crates/cgap-bn3/src/unify.rs` (doc-only), this receipt,
p2-asm-0blr-census.md addendum. No git writes; no mocks; no Process.sleep.

## Doc edits (unify.rs, doc-level only)

1. **Module header (:7-18)** — rewritten from the stale Wave-2 framing
("nested-loop join ... chosen for v1") to the landed state: RADIXFIX
radix-default join probe with court-guarded nested leg, W6-BFX [`SplitStore`]
borrowed-prefix + owned-delta-tail fixpoint view, W6-BN3NS no_std posture,
H8 honesty (fixpoint is NOT budget-0; counting allocator in noalloc.rs), AE6
behavioral patent-gate as campaign G7 admission surface.
2. **FixpointHasher doc (:107-118)** — resolved the doc-vs-annotation
conflict by rewording to frozen-artifact framing: current membership is the
W6-BN3NS BTreeSet; the struct is NOT in the hot loop; the no-TLS-blr
rationale is marked historical; retained solely for the
0xBF58_476D_1CE4_E5B9 seed lineage.
3. **Redundant std BTreeSet import (:1587)** — DOES NOT EXIST. The only
BTreeSet/BTreeMap import is `alloc::collections::{BTreeMap, BTreeSet}` at
:103, used throughout. Nothing removed. W10's line number drifted (a
2026-11-01 W2-9 unblock comment sits at ~:1583-1588). Recorded divergence,
not a silent skip.

## Census (owed re-run; W10 blr claim was receipt-level UNVERIFIED)

Fresh asm this session, exit 0 (command in census addendum). Regions:

| Region | blr | br xN | brk |
|---|---|---|---|
| unify_to_fixpoint_opt (1565 ln) | 0 | 0 | 0 |
| RuleCompiler::compile (432 ln) | 0 | 0 | 0 |
| emit_join_masks / bucket_candidates / unify_batch_probe | 0 | 0 | 0 |

Whole-file blr=0, TLVP=0. D6 typed remainder (RuleCompiler::compile
RandomState TLS blr + hashbrown rehash_in_place x21) CLOSED — source has 0
HashMap/RandomState hits (grep exit 1). Standing flipped PARTIAL -> ALIVE
(bn3 unify scope) in the p2 census addendum; storage closure cells remain
recorded, out of lane scope, not re-censused.

## Verify

`CARGO_TARGET_DIR=/tmp/bn3d-target cargo test -p cgap-bn3` exit 0:
119 unittests + 2 bgp_hop_courts + 7 lever_courts = 128 passed, 0 failed,
2 ignored (pre-existing), doc-only edits perturbed nothing.

## Standing

ALIVE for the lane scope (doc edits + census + gate). commands/exits above.
