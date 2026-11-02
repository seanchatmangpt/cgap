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

Lever -> crate -> court -> measurement:

| Lever | Crate | Anti-vacuity flip | Measurement |
|---|---|---|---|
| Gate escalation | cgap-gate | mutate a verdict level; court flips FAIL | Divan: gate eval ns |
| SWAR FNV | cgap-core hash path | corrupt one SWAR lane; hash differs | Divan: hash throughput |
| PLNF semi-join + radix probe | bont | drop radix probe; result changes | Divan: join latency |
| Ring sink | storage sink | overwrite a ring slot; tail reads it | Divan: sink push ns |
| Fused governance | gate + p28 | mutate fused branch; verdict differs | Divan: fused vs split |

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
