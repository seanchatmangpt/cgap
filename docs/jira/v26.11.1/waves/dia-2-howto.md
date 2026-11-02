# dia-2-howto receipt

Lane: DIA-2-howto (lane-runner). Repo /Users/sac/cgap, branch main, 2026-11-01.
Lease: docs/diataxis/how-to-guides/** (new) + this file. No git writes.

## Files written

- docs/diataxis/how-to-guides/run-the-court-suites.md
- docs/diataxis/how-to-guides/run-a-benchmark-chain.md
- docs/diataxis/how-to-guides/flip-a-court.md
- docs/diataxis/how-to-guides/publish-a-crate.md
- docs/diataxis/how-to-guides/add-a-lane.md
- docs/jira/v26.11.1/waves/dia-2-howto.md (this receipt)

## Pack scout (task step 1)

`~/ggen-marketplace/packs/readme-diataxis-pack` and
`sa2a-semantic-diataxis-pack` found. The pack projects docs from an admitted
`rdx:` RDF graph (RunStep/ApiEndpoint/FaqEntry classes) with two gates
(required-properties, ordering-integrity); cgap has no such admitted graph,
so there is no subject for the pack — following it would mean fabricating
ontology facts. Divergence recorded: hand-written guides grounded in
witnessed receipts instead of a pack projection.

## Grounding

Every procedure cites its witnessed receipt:
run-the-court-suites -> waves/f3-04-courts-green.md (13 passed / 0 failed;
kill 19/19 kill_rate 1.0000; inert control exit 1 VACUOUS GATE;
zero-string poison fired=True).
run-a-benchmark-chain -> waves/bench-chains-{bn3,ingress,parquet,seal,
sovereign}.md (five chains, exit 0 each).
flip-a-court -> waves/a6-flip-rerun.md (A6 NEUTER sandbox flip: baseline
exit 0, mutated exit 101, sha256 restore pin).
publish-a-crate -> waves/coord-publish-record.md + waves/p3-api-verify-all.md
(5 publishes exit 0, num_versions==2 exact, negative probe 404/400).
add-a-lane -> lanes/A06-bont-scalars.md shape + campaign dispatch contract.

## Session re-witness (this lane, exit captured directly)

- `python3 scripts/boundary_gate.py --shim-court-only --shim-inert` -> exit 1,
  `VACUOUS GATE: the INERT control was not killed ... (§6.3)`. Exit 1 is the
  required negative-control refusal (matches f3-04 leg 4).
- `awk length>100` sweep over all five guides: only violation was a long
  bash line in run-a-benchmark-chain.md:28, fixed (wrapped command), re-sweep
  clean.

## Standing

ALIVE for the suite of guides as grounded documentation: each named
procedure's receipt exists on disk and the negative-control procedure was
re-executed this session (exit 1). Guides themselves are prose — their
standing is the standing of the cited receipts.

## Divergences

1. Pack not followed (no admitted rdx: graph to project from) — see Pack
   scout above.
2. Task said "13/14 counts"; the witnessed Rust suite is 13 passed / 0
   failed, and no 14th Rust case exists — guides state 13/14 as 13 passing
   courts + the gate-exercised negative control, per f3-04.
3. The pre-existing docs/diataxis/how-to/ dir (kernel-recipes etc.) is
   outside this lease; how-to-guides/ was created new per dispatch.
