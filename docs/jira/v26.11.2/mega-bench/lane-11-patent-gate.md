# Lane 11 — G7 Patent Gate (mega-bench)

AS OF RUN: 2026-11-02, /Users/sac/cgap, read-only except this receipt.
Landed `ost_report.json` untouched — sha256
`d8ad20358ed66c85818d5e4d2b84da57d6d45d457a5d6dd16655ea2e19d3fd95`, byte-identical
before/after; my run's report went to `/tmp/mb11-report.json`.

## Command

```
python3 scripts/verify_ost_design_around.py \
  --claims docs/jira/v26.11.2/patent/claims.txt \
  --corpus /Users/sac/purrdf/corpora \
  --report /tmp/mb11-report.json
```

## Verbatim gate output (exit 0)

```
US10,817,467 -> DISTINCT-MECHANISM (zero src hits for patented mechanism; landed cite verified)
US11,216,456 -> DISTINCT-MECHANISM (zero src hits for patented mechanism; landed cite verified)
US11,409,698 -> DISTINCT-MECHANISM (zero src hits for patented mechanism; landed cite verified)
  src hits: 1
US11,645,278 -> DISTINCT-MECHANISM (zero src hits for patented mechanism; landed cite verified)
  src hits: 1
G7-PATENT: PASS 4/4 DISTINCT-MECHANISM (exit 0)
```

Shell `EXIT=0` witnessed.

## Per-row table (AS OF THIS RUN)

| Row | Patent | Status | src hits | Cite |
|---|---|---|---|---|
| 1 | US10,817,467 | DISTINCT | 0 (`grep_exit` 1) | mmap_writer.rs + receipt_chain.rs |
| 2 | US11,216,456 | DISTINCT-MECHANISM | 0 (`grep_exit` 1) | crates/cgap-parquet/src/bitslice.rs |
| 3 | US11,409,698 | DISTINCT | 1 (unify.rs:1817 doc — not a semi-join) | cgap-bn3/src/unify.rs |
| 4 | US11,645,278 | DISTINCT | 1 (sigil.rs:757 dyn Error) | cgap-bn3/src/unify.rs |

Row-1 markers verified present: `RingSink`, `drain_to_spill`, `RunOverrun`
(mmap_writer.rs); BLAKE3 prev_hash chain in receipt_chain.rs.

Report JSON: claims_sha256
`963230bbb7167761de5bf0436bf2386a493d7e2b1e2df4bd676f0c53cdad35ac`,
`duplicate_mechanism_groups: []`, `all_distinct: true`, `gate_exit: 0`.

## G7F re-scope (operator order)

Row 1 flipped from the G7R PENDING state to DISTINCT-MECHANISM via the G7F
re-scope to the single-writer append-only RingSink + BLAKE3 receipt chain
(US10,817,467), landed this wave; the re-scope note is carried in the report's
row-1 mechanism field ("re-scoped 2026-11-02 ... per operator order;
partition-sealed delta pages remain a further refinement, not claimed").
Row 2's PLNF cardinality remainder stays a named open item in
`docs/jira/v26.11.2/03-ost-design-around-register.md`, explicitly NOT part of
the DISTINCT claim.

## Non-vacuity (cited, not re-run)

FLIP merged-duplicate mutation, witnessed exiting **1** (the flip contract
exits 1, not 101 — real code recorded here):

- `docs/jira/v26.11.2/waves/w5-g7-patent-gate.md`: mutated /tmp claims
  (merged duplicate mechanism), exit 1, US11,645,278 DISTINCT->NOT-DISTINCT,
  sha256-exact restore True (0ce2db232c00).
- `docs/jira/v26.11.2/waves/w6-g7-remediation.md`:
  /tmp/claims_flipped_g7r.txt, exit 1, rows 3+4 DISTINCT->NOT-DISTINCT,
  restore_exact=True (8387d20824b83705...).

Not re-run this session: the clean 4/4 exit 0 agrees with both receipts.

## Verdict

G7-PATENT **PASS 4/4 DISTINCT-MECHANISM** (exit 0, this session, exact subject
/Users/sac/cgap @ main). Standing: ALIVE. Typed remainder: none on DISTINCT
surfaces; row-2 PLNF cardinality is a named open item outside the DISTINCT
claim. Landed report file unmodified — sha256
`d8ad20358ed66c85818d5e4d2b84da57d6d45d457a5d6dd16655ea2e19d3fd95`.
