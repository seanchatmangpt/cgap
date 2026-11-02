# RFC-0042 Challenger Sales Chicago Test (CSCT)

Condensed specification of the CSCT pipeline: 6 stages, each with a metric
contract, a real collaborator, and an anti-vacuity mutation. Full economics
constants live in `crates/csct/src/lib.rs`.

## The 6 stages

1. **Warmer** — 1M-row ingest ΔS (delta-storage) stage. The stage ingests 1M
   rows through a real collaborator and asserts the delta-storage contract.
2. **Reframe** — amplification ratio ≥ 6.66x with 924,120 channels.
3. **Drowning** — status-quo bleeding: $422,386/mo, annualized
   $5,068,632/yr.
4. **Emotional** — p99 2750 ms and a 600 MB tab OOM witness. The p99 bound
   and the OOM observation are both asserted, never narrated.
5. **New Way** — envelope contract: ≤131072 B footprint, <10 ns hot-path,
   ≤33.3 ms stage budget, no GC / no vectors / no wire formats.
6. **Solution** — BLAKE3-stable receipt, zero-alloc, ≤$1,200/mo ceiling,
   ROI 352x verdict gate.

## The 4 Chicago axioms

1. **Real collaborators.** No mocks: stages run real ingest, real timing,
   real economics. A hand-written real interface implementation is not a
   mock.
2. **Anti-vacuity mutation.** Every court names its mutation; a court that
   cannot flip is FAIL, and a mutation-free green run is VACUOUS (exit 1).
3. **Andon 101.** A stage that cannot be evaluated honestly refuses with
   exit 101 (`AndonRefusal`) — stop the line, loud skip never silent pass.
4. **Economic delta verification.** The final verdict is arithmetic, not
   narrative: net = $5,068,632 − $14,400 annual; verdict = ROI ≥ 352x.

## State-transition flow

`run_suite()` executes the stages strictly in order:

```text
warmer -> reframe -> drowning -> emotional_impact -> new_way -> solution
```

- Each stage returns `StageResult { exit, ... }`.
- First non-`Pass` exit short-circuits: `run_suite()` returns early with a
  Report carrying the failing stage's exit code; later stages do not run.
- On full pass, economic deltas are computed
  (net = 5,068,632 − 14,400 = 5,054,232 USD/yr) and
  `final_commercial_verdict = (roi >= REQUIRED_ROI_X)` with
  `REQUIRED_ROI_X = 352.0`.

## See Also

- `docs/csct/README.md` — crate index, run commands, exit-code table.
- `docs/jira/v26.11.2/waves/csct-lane-01-scaffold.md` — scaffold receipt and
  the 351.15x-vs-352x arithmetic errata.
- `crates/csct/src/lib.rs` — constants and sequencing.
