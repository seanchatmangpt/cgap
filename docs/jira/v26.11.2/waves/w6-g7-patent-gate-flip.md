# w6-g7-patent-gate-flip — lane G7F receipt

Date: 2026-11-02. Repo: /Users/sac/cgap, branch main. Operator order:
re-scope register row 1 (US 10,817,467) design-around to the LANDED
single-ring mechanism; land ROW_EVIDENCE; run gate + flip court.

## Files written (ownership respected)

- docs/jira/v26.11.2/03-ost-design-around-register.md (row 1 wording +
  cross-check row-1 entry + receipt standing block only)
- docs/jira/v26.11.2/patent/claims.txt (row 1 status only)
- scripts/verify_ost_design_around.py (ROW_EVIDENCE row-1 entry only)
- this receipt. No git writes; no shared lib.rs/Cargo.toml edits.

## Marker greps (before landing)

- `RingSink` crates/cgap-parquet/src/mmap_writer.rs -> 9 hits, exit 0
- `drain_to_spill` mmap_writer.rs -> 2 hits, exit 0
- `RunOverrun` mmap_writer.rs -> 30 hits, exit 0
- `prev_hash` crates/cgap-ffi/src/receipt_chain.rs -> 13 hits, exit 0

## Row status transitions

| Patent | Before | After |
|---|---|---|
| US10,817,467 | PENDING (KNOWN-DIVERGENCE) | LANDED-crates/cgap-parquet/src/mmap_writer.rs (+ receipt_chain.rs) |
| US11,216,456 | LANDED-bitslice.rs | unchanged |
| US11,409,698 | LANDED-unify.rs | unchanged |
| US11,645,278 | LANDED-unify.rs | unchanged |

Prior KNOWN-DIVERGENCE text retained in the register as the refinement
note: partition-sealed delta pages are a further refinement, not claimed.

## Gate outputs (verbatim, shared tree, this session)

Run 1 (after edits):
US10,817,467 -> DISTINCT-MECHANISM (zero src hits for patented mechanism; landed cite verified)
US11,216,456 -> DISTINCT-MECHANISM (zero src hits for patented mechanism; landed cite verified)
US11,409,698 -> DISTINCT-MECHANISM (zero src hits for patented mechanism; landed cite verified)
US11,645,278 -> DISTINCT-MECHANISM (zero src hits for patented mechanism; landed cite verified)
G7-PATENT: PASS 4/4 DISTINCT-MECHANISM (exit 0)  — EXIT=0

Run 2 (stability): identical 4/4 verdicts, EXIT=0 (output at
/tmp/g7_run2.txt during session; deleted with sandbox shims after).

## Flip court (anti-vacuity; sandbox copy, shared tree untouched)

Mutation: /tmp/g7_flip_claims.txt — row-1 mechanism text set
normalized-equal to row 2's mechanism (merged-duplicate class).
Result (verbatim):
US10,817,467 -> NOT-DISTINCT (duplicate/near-duplicate mechanism shared across rows ['US10,817,467', 'US11,216,456'])
US11,216,456 -> NOT-DISTINCT (duplicate/near-duplicate mechanism shared across rows ['US10,817,467', 'US11,216,456'])
US11,409,698 -> DISTINCT-MECHANISM (zero src hits for patented mechanism; landed cite verified)
US11,645,278 -> DISTINCT-MECHANISM (zero src hits for patented mechanism; landed cite verified)
G7-PATENT: FAIL (2/4 DISTINCT) failing rows: US10,817,467,US11,216,456
FLIP_EXIT=1

Shim parses to duplicate group [['US10,817,467','US11,216,456']]
before the run. Sandbox shims deleted after the court.

## Divergences / notes

- Two failed shim attempts are recorded for honesty: (a) first shim
  used row 2's SHORT mechanism text — not normalized-equal to the
  full continuation text, gate stayed exit 0 (court did not flip;
  not counted as a pass); (b) second shim embedded literal newlines,
  parse_claims raised (ValueError, exit 1 — abort, not a DISTINCT
  flip). Third shim (newlines collapsed) flips as recorded above.
- git status at session start already showed M on the register and
  claims.txt and ?? on verify_ost_design_around.py — pre-existing
  prior-lane edits in the shared tree; this lane's diffs are limited
  to the row-1/ROW_EVIDENCE/standing edits named above.
- claims.txt sha256 after edits:
  963230bbb7167761de5bf0436bf2386a493d7e2b1e2df4bd676f0c53cdad35ac

## Standing

ALIVE — gate command + exit run this session against the exact
subject (2x exit 0; flip exit 1 on sandbox mutation). Receipt chain:
registers row 1 LANDED; w5-ring-rewitness.md courts remain the
mechanism-level witness for the ring legs.
