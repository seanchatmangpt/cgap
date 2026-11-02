# Phase-Change Instruments

What the three armed discontinuity metrics measure, and what their
witnessed baselines actually mean.

## Metrics witness, never promise

The charter's law (phase-change-charter.md line 7): "metrics WITNESS,
never promise. A phase change is claimed only retrospectively from an
armed cost curve going vertical." The instruments below are armed
whether or not the discontinuity comes.

## The three instruments

| Transition | Instrument | Direction |
|---|---|---|
| AM->FM | `.rodata` domain strings per crate; `strings_admitted` per receipt | -> 0, stay 0 |
| AC/DC | % tracked bytes regenerable byte-identically (projection_coverage.py) | rises per class |
| Steam | machinery share = 1 - (agent-hops / contract-hops), journal-derived | rises per campaign |

(Panel frozen "BEFORE the run", charter §Metric panel.)

## Reading the witnessed baselines

`waves/a7-phase-change-remeasure.md` (2026-11-01, pre-campaign)
records: projection coverage 0.4386% -> 0.4332% after materializing
index bytes (613 files / 7,408,258 B, fails=[], skips=[]), and
machinery share 0.0 for both readings — reconstructed from
`wf_f175cd34`: 58 agent hops, 0 machinery hops
(`a7-phase-change-remeasure.md:23,55-57`).

What the numbers mean:

- 0.4332% is a near-zero floor: essentially nothing in the repo is
  yet a byte-identical projection of the graph. The AC/DC transition
  has headroom, not proof.
- machinery share 0.0 witnessed *flat* is the honest reading, not a
  failure to report: the charter's own falsifier says "flat machinery
  share across campaigns = a sprint ran, not a phase change" — the
  baseline exists precisely so the next same-shape replay can show
  motion against it, and 0.0 is "the floor every same-shape replay
  exists to move" (`waves/f2-9-hop-ratio-receipt.md:67`).

## Why baselines precede claims

You cannot detect a discontinuity without a pre-registered floor. The
charter's enactment order (§Enactment order) mandates
Baseline-Metrics first, "witnessed with exits, not narrated." A later
curve that moves is only a phase change relative to these numbers;
without them the same movement would be unfalsifiable narrative.
