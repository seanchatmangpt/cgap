# w6-dred-relabel-errata

Lane DREDLABEL (honest mechanism relabel per FA5). Repo /Users/sac/cgap
@ main. Date 2026-11-02. No source edits; no git writes. Sole writes:
`15-ctq-tree.md` (DRed CTQ row), `mega-bench/MEGA-REPORT-v26.11.2.md`
(DRed relabel + defect D16), this errata receipt.

## The relabel

`dred::retract` (dred.rs:116-127) is a bit-parallel retraction FILTER
over caller-supplied static masks. It never follows `derived_deps` as
lineage edges; recursive lineage propagation is NOT implemented — an
upstream or explicit edge-walk is required. FA5 proved the N=10,000
root-deletion cascade leaves 9,999 survivors where 0 is correct; no
existing court covers lineage propagation. The 947M facts/s certifies
bitmask tuple-invalidation speed, not end-to-end DRed lineage
unwinding.

## FA5 probe cite (replayable)

Receipt: `adversarial/forensic-lane-05.md`, Interrogation 3.
Command + exit, run 2026-11-02:

    cd /tmp/fa5-probe && CARGO_TARGET_DIR=/tmp/fa5-target cargo run --release
    -> EXIT=0, PROBE-OK

Key output line (verbatim from the FA5 receipt):
`CHAIN N=10000 retract_ns=15750 ns/lane=1.5750 survivors=9999
(expected 0 if cascade)`. Probe source retained at /tmp/fa5-probe for
replay.

## Prior receipts carrying the misleading framing (quotes)

Original lane receipts are IMMUTABLE records; this errata supersedes
their framing, not their numbers.

1. `waves/p1-dred-retraction-antijoin.md:3-4` — "retraction as columnar
   anti-join in `crates/cgap-storage/src/dred.rs` — retracted-delta x
   rule-premises anti-join -> invalidated rows -> negative bitmask
   filter". The word "retraction" over-claims: the surface is the
   bitmask filter step only; the anti-join computes invalidated rows
   from INPUT masks and never propagates along `derived_deps`.
2. `waves/w5-g4-dred-churn.md:54` — "90% churn throughput = 943.5M
   facts/s >= 15M target (63x headroom)." The number is real for the
   filter; presenting it under a "churn throughput" heading against a
   DRed-maintenance target implies system-level incremental
   maintenance, which is absent (FA5 Interrogation 1: each batch is a
   FRESH corpus, no delta over the previous batch).
3. `mega-bench/lane-05-dred-maintenance.md:63-65` — "**ALIVE.** Gate
   G4 shape satisfied ... 15M facts/s MET with ~63x headroom at 90%
   churn". Same scope error: the G4 shape is satisfied by filter
   speed; the maintenance semantics the gate name implies are not
   measured and, per FA5, do not exist on the kernel surface.

## Edits made this session

- `15-ctq-tree.md`: added an honest DRed incremental-maintenance CTQ
  row to S3 (no DRed row existed — divergence from dispatch, which
  assumed a row to reword; added rather than reworded). Rollup updated
  S3 to 4 CTQs / 1 measured / 3 TARGET.
- `MEGA-REPORT-v26.11.2.md`: MB5 verdict GREEN -> YELLOW (relabelled,
  D16); removed the DRed row from the Met table; added a lineage
  propagation row to NOT-met; scope-tagged the two DRed rows in the
  master throughput table; added defect D16 (HIGH,
  dred.rs:116-127, close condition: lineage-edge walk + cascade court
  with root-deletion corpus flipping 9,999 -> 0).

## Receipt: commands + exits this session

| Command | Exit |
|---|---|
| `grep -n -i 'dred\|retract\|incremental' 15-ctq-tree.md` | 0 (0 hits) |
| `sed -n '60,140p' adversarial/forensic-lane-05.md` | 0 |
| `grep -n -i 'retract...' p1-dred / w5-g4 / lane-05 receipts` | 0 x3 |
| `sed -n` quote pulls (p1-dred, w5-g4, lane-05, FA5 tail) | 0 x4 |
| Edits via Edit tool (ctq x2, mega-report x5) | 0 |
| `awk length>100` line gate on all three written files | 0 |
