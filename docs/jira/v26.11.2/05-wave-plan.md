# V2 Wave Plan — Three-Wave Overnight Execution

Charter: CHARTER-CGAP-STARDF-v26.11.2. Lane V2-WAVEPLAN (lane-runner).

Lease: this file plus `waves/v2-05-wave-plan.md` (receipt). No git writes.

## Scope

Three waves over the one canonical checkout `/Users/sac/cgap`, branch
`main`, coordinator owns all git transitions. Exit-code contract for all
gates: 0 pass / 1 gate fail / 2 subject missing / 3 loud skip. Chicago
courts: every court names its anti-vacuity mutation; a court that cannot
flip is FAIL; a mutation-free green gate run is VACUOUS (exit nonzero).

## Wave V2-A — Finish the v26.11.1 tail

Objective: close every open v26.11.1 obligation before new surface.

1. `ontologies/fibo` O-ABI pack publish (A09 tail).
2. `integration` crate publish (A11/A19 membrane tail).
3. Final 22-crate sweep: `cargo build --workspace` + per-crate
   `cargo test -p <crate>` census, compile-abort reported separately
   from test failures (census-vs-abort rule).
4. stardf F1 mappings landed (F1 -> cgap frame/bont scalar bindings).
5. License gate: pinned URN
   `urn:avec:license:p28-sil:v2.1.0-sovereign` verified present in the
   P28 attestation path; gate exit 0.
6. std-strip: confirm `no_std` surface of the five kernel crates
   (`cgap-{arrow,blake3,git,parquet}` + bont scalars) with a
   `cargo build` under a `no_std` target probe; divergence = typed FAIL.

Agent arithmetic: 6 structural work items, one lane-runner each = 6
lane-runners; plus 1 court-auditor over the sweep output = 7 agents.

Gate exits required before V2-B: workspace build 0, license gate 0,
sweep census 0 (or loud 3 skips with named reasons).

## Wave V2-B — Five Pareto levers in cgap

Objective: land the five levers, each with anti-vacuity flip and a Divan
PARETO measurement, one lane-runner per lever (5 lane-runners), plus 1
court-auditor (flip verification) and 1 gate-runner (G2-PARETO rollup)
= 7 agents.

Lever -> crate -> root cause -> target -> court -> measurement
(per `02-pareto-levers.md`, the authoritative spec; targets are
TARGETs until a v26.11.2 bench receipt re-measures them):

| Lever | Crate | Root cause (file:line) | Target | Anti-vacuity mutation | Measurement |
|---|---|---|---|---|---|
| L1 ingress digest | cgap-gate | penstock digest | <15 ns/frame | slot-byte corrupt | ns/frame |
| L2 SWAR FNV seal | cgap-parquet | `footer.rs:119-127` | <10 ms @ 1M | window-byte flip | ms @ 1M |
| L3 PLNF semi-join | cgap-bn3 | `unify.rs:11,35-39,274` | <75 ms @ 1M | comparator swap | s @ 1M |
| L4 ring sink | cgap-ffi | `sovereign.rs:706-730` | <4 us/tx | header-byte corrupt | us/tx |
| L5 fused ASC450 | gate + p28 | split passes, re-walk | <1.45 us/row | receipt bit-flip | us/row |

Root-cause detail (all verified on disk in `02-pareto-levers.md`):

- L1: unconditional BLAKE3 payload digest pre-admission for every
  frame; `stage_digest_fold` 93.38 ms, ~91% of the chain mean. Fix
  folds a cheap prefilter into the penstock `FrameView` and escalates
  to BLAKE3 only on a policy flag.
- L2: `crates/cgap-parquet/src/footer.rs:119-127` `fnv1a64_seeded`
  byte-wise (`h ^= b; h *= PRIME`), 10 passes per seal/verify pair.
  Fix: word-wise u64 SWAR FNV, byte-exact FNV-1a semantics preserved.
- L3: `crates/cgap-bn3/src/unify.rs:11,35-39,274` nested-loop unify
  probe (`cur` x `cand` per chunk). Fix: static-footer bit-slice
  semi-join keyed on (s,o), noalloc radix probe scratchpad.
- L4: `crates/cgap-ffi/src/sovereign.rs:706-730` per-call
  `std::fs::File::create` (O_TRUNC, call site 722) + fresh boxed
  `FileSink` alloc. Fix: ring sink leased once per lane slot inside
  the 65,376-lane commit lease; spill to file on rotation only.
- L5: sequential P28 validate -> ASC450 metering -> chatman seal;
  each pass re-walks the attestation span. Fix: fuse metering into
  the P28 section-5 span fold; chatman seal stays a distinct tail.

Anti-vacuity: each lever's court runs BASELINE -> MUTANT -> RESTORE
(sha256 fail-closed) over /tmp sandbox copies; flip-or-FAIL; skips are
loud (`[SKIP] <reason>`).

Divan PARETO: each lever benches before/after on the same subject; the
PARETO row is admitted only if the measured delta is in the claimed
direction; otherwise the lever lands UNVERIFIED, never round up.

Loop law (OVERNIGHT-LOOP ITERATION LAW): each overnight iteration lands
exactly ONE lever, flips its court, and records its Divan PARETO row
before the next iteration starts. Loop until G2-PARETO is green
(all five levers: flip witnessed + PARETO row admitted, gate exit 0).
A lever that cannot flip blocks its iteration only; the loop proceeds
to the next lever with a typed BLOCKED row, never a silent skip.

## Wave V2-C — Positioning + release

Objective: like-for-like materialization and version transition.

1. Like-for-like mat bench vs RDFox at the 2-6.1M triple scale, same
   subject hardware, stated method (Divan, min-of-N, sleep-free).
   Perf numbers are TARGETs until this lands.
2. QEMU stack paint: full `no_std` stack exercised under QEMU; paint
   result recorded as receipt, divergence = typed FAIL.
3. position-paper-v2 grounded: every RDFox comparison row cites a
   measured receipt from item 1; no ungrounded capability claims.
4. 26.11.2 version bump across workspace + tiered publish order:
   kernel crates first (`cgap-{arrow,blake3,git,parquet}`), then
   moonshot crates, then membranes (`ffi`, `beam`, `gpu`), then `cgap`
   umbrella. Each tier publishes only after the prior tier's court
   exits 0.

Agent arithmetic: 4 work items; items 1 and 2 are independent
lane-runners (2), item 3 is 1 lane-runner consuming item 1 receipts,
item 4 is 1 lane-runner + 1 gate-runner over the publish ladder = 5
agents.

## Sequencing

V2-A (7) -> V2-B (7, looped per the iteration law) -> V2-C (5).
Waves are serialized; within a wave, lanes run concurrently under
disjoint file ownership. Total concurrent peak: 7.

## Standing

Plan authored this session; `waves/v2-05-wave-plan.md` carries the
execution receipt. Per-lever standing re-read from receipts
2026-11-02 (lane RW-04 sweep; numbers live in
`11-measured-claims.md`, the exclusive numeric source):

- L1 ingress (<15 ns/frame): MET on quiet host — 10.20 fastest /
  10.30 median ns/frame (`waves/w3-l1-ingress.md:63,69-70`).
  Registered divergence under combined contention (median 19.03,
  >50 ns bar crossed at slowest 124.06):
  `waves/redteam/red-10-timing-jitter-exploit.md:43-66`.
  Court flip witnessed (`w3-l1-ingress.md:75-83`).
- L2 SWAR FNV (<10 ms @1M): NOT MET — 42.58-43.1 ms measured;
  chain-share attribution refuted
  (`waves/w3-l2-swar-fnv.md:26-28,61-63`). Court flip witnessed
  (`w3-l2-swar-fnv.md:38-42`).
- L3 PLNF semi-join (<75 ms @1M): NOT MET (LANDED-NOT-MET) — radix
  leg 1.648 s fastest @1M, 0.97-0.98x vs nested baseline; dominant
  residual is the per-chunk candidate walk outside the probe
  (`waves/w5-l3-remeasure.md:23-33,37-49`). Courts 7/7 exit 0
  (`w5-l3-remeasure.md:58-62`).
- L4 ring sink (<4 us/tx): MET at the stage level — 0.755-0.834
  us/tx fastest (drifted +28-42% vs prior 0.589-0.603, typed);
  chain-level 6.19-6.33 us/tx remains >4 us/tx, typed BLOCKED via
  `waves/p2-receipts-ring-receipt-chain.md:62-66`
  (`waves/w5-ring-rewitness.md:36-58`). Lease-edge refusal code
  65,376/0x43470004 re-witnessed byte-identically.
- L5 fused ASC450 (<1.45 us/row): FAILED-BY-MEASUREMENT — 3.04 vs
  2.37 us/row control; emission crypto, not span walk, dominates
  (`waves/w3-l5-fused.md:38-42`; attribution row:
  `11-measured-claims.md` #13). Court flip witnessed
  (`w3-l5-fused.md:44-52`).

V2-A sweep items and V2-C items: publish is FROZEN-BY-OPERATOR
(`waves/publish-halt.md:1-5`); 8 crates live-verified at 26.11.2
before the halt (`publish-halt.md:6-17`), 7 dry-run-READY, 2 BLOCKED
(cgap-ffi pending parquet re-publish; wasm-footprint host-link),
3 already-live repairs owed on lift (`publish-halt.md:31-42`).
MW4 like-for-like bench remains TARGET — RDFox never executed;
zero measured facts/s at consolidation (`11-measured-claims.md:26,66-67`).
QEMU no_std paint and position-paper grounding status: UNVERIFIED
by this sweep (no receipt cited here re-read this session).

## Standing refresh (2026-11-02, lane GOAL-01)

RW-04's Standing block above stands unmodified (ERRATA-never-
rewrite on its cites; its per-lever numbers remain the record).
Refresh, re-read from disk this session:

Per-lever standing UNCHANGED — no new receipt re-measures any
lever:

- L1, L2, L3, L4, L5 statuses and citations above are current
  as of this read; publish-side receipts (PUB-01, PUB-02b, PUB-03,
  PUBLISH_RUNBOOK, prepublish docs) touch no lever measurement.
  The C-10 reconciliation re-ran the full gate battery (4/4 +
  self-check 8/8, `audit/prepublish-reconciliation.md`) — gates,
  not levers.

Publish window preconditions: receipt-complete EXCEPT the freeze
lift (FROZEN-BY-OPERATOR, `waves/publish-halt.md`, unchanged on
disk) and the coordinator commit landing (clean-tree rule,
`audit/prepublish-state.md` §2). Details:
`audit/pub-01-dry-run-matrix.md` (16/20 package-OK, 4
BLOCKED-DEP-ORDER: cgap, cgap-gate, cgap-ffi, cgap-integration),
`dod/PUBLISH_RUNBOOK_v26.11.2.md` (tier-ordered sequence,
preconditions §1).

Microbenchmark promotions row 39/40: NOT added. The dispatch
conditioned the pointer on PROMO-01 having landed at read time;
no PROMO-01 receipt exists on disk (`docs/jira/v26.11.2/lanes/`
contains `lane-r16-mctq-promotion.md` only; zero files cite
PROMO-01 or a 39/40 row). Loud absence, not a silent skip — the
pointer lands with PROMO-01's receipt.

DoD certificate ("FS-18" per dispatch): no on-disk referent
carries that ID (grep zero hits under docs/jira/v26.11.2/ this
session). Nearest landed carriers: `audit/dod-18-ledger-fold.md`
(ledger fold, standing-as-receipted) and
`audit/dod-19-shipgate-final.md` (ship-gate table SPEC awaiting
the coordinator commit). Divergence recorded in the GOAL-LOOP.md
refresh block and the lane receipt.

PROMO-01 pointer (appended on coordinator follow-up, same lane):
LANDED — claims §6.1 rows 39/40 (82.24 GB/s BN3
statement_scan_24b, ~460 ns P28 attestation verify;
consolidation-addendum, doubly witnessed F-06+FS-02) are
registered in `11-measured-claims.md`, receipted at
`lanes/lane-promo01-claims-promotion.md`. The loud absence this
block recorded for PROMO-01/row 39-40 is SUPERSEDED.
