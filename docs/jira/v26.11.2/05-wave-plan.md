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

Plan authored this session; all wave gates are UNVERIFIED until their
receipts cite command+exit runs. This file is the plan of record;
`waves/v2-05-wave-plan.md` carries the execution receipt.
