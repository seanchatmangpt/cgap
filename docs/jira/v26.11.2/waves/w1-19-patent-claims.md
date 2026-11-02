# W1-19 Patent Claims Lane Receipt

Lane: W1-19 (lane-runner). Repo: /Users/sac/cgap. Date: 2026-11-01. No git
writes performed. /Users/sac/stardf untouched (read-only register source
cited in doc only).

## Doc read

docs/jira/v26.11.2/03-ost-design-around-register.md read this session
(full file, 72 lines). The four patent numbers were taken from that doc,
not the dispatch prompt; they match the prompt's list exactly:
10,817,467 / 11,216,456 / 11,409,698 / 11,645,278.

## Files written

- docs/jira/v26.11.2/patent/claims.txt (NEW; sole write named in
  dispatch). One claim-element line per OST patent, comma-formatted as in
  the doc, pipe-delimited: claim element NOT practiced | cgap mechanism |
  evidence status per the doc's cross-check section.

## Evidence status per the doc's cross-check (verbatim mapping)

- Row 1 (10,817,467): PENDING — doc names adjacent landed statement in
  stardf incremental.md but cgap impl "zero hits".
- Row 2 (11,216,456): LANDED for the bit-slice-mask component
  (crates/cgap-parquet/src/bitslice.rs); compile-time PLNF cardinality
  component PENDING — recorded as a split status on one line.
- Row 3 (11,409,698): PENDING — cgap semi-naive cascade grep zero hits.
- Row 4 (11,645,278): LANDED-crates/cgap-bn3/src/unify.rs (lane A12,
  v26.11.1 waves 9-10).

## Grep cross-check (this session)

Comma-formatted numbers appear in BOTH the doc and claims.txt (grep -c,
run this session, exits 0):

- 10,817,467: doc=2 claims=1
- 11,216,456: doc=3 claims=1
- 11,409,698: doc=3 claims=1
- 11,645,278: doc=2 claims=1

All claims.txt lines <=100 chars (awk length>100: no output, exit 0).

## Standing

PARTIAL_ALIVE. claims.txt write ALIVE (created + greps above); the
underlying implementation evidence statuses are as the doc states
(rows 1 and 3 PENDING in cgap, row 2 half-landed, row 4 landed) —
transcription only, no new implementation claims made.

## Divergences

- The doc uses comma-formatted patent numbers ("US 10,817,467"); the
  dispatch format sketch showed "US<PATENT-NUMBER>" — claims.txt uses
  the doc's comma format verbatim (cite-don't-redefine).
- Long lines: wrapped continuation lines (2-space indent) to honor the
  100-char ceiling; one patent = one logical record.
- Row 2 split status: doc itself is half-landed (bitslice LANDED, PLNF
  PENDING); recorded as split on one record rather than forced to a
  single token.

```json
{"lane":"W1-19","files_written":["docs/jira/v26.11.2/patent/claims.txt","docs/jira/v26.11.2/waves/w1-19-patent-claims.md"],"commands":["mkdir -p patent (exit 0)","grep -c comma-formatted patent numbers in 03-ost-design-around-register.md and patent/claims.txt (exit 0; 2/3/3/2 doc vs 1/1/1/1 claims)","awk length>100 claims.txt (no output, exit 0)"],"courts_run":["grep four-patent-numbers-in-both-files cross-check (flipped: pre-format-fix claims had 0 comma-formatted hits, post-fix 1 each — anti-vacuity mutation was the digit-format mismatch)"],"standing":"PARTIAL_ALIVE","divergences":["comma-formatted patent numbers per doc","row 2 split LANDED/PENDING status","continuation-line wrapping for 100-char ceiling"]}
```
