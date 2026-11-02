# p1-repair-unify-repair

Lane p1-repair. Subject: `crates/cgap-bn3/src/unify.rs` (2687 lines on disk,
+548/-100 vs HEAD of 6801affa — the mid-edit state left by the killed lane).

## Finding (differs from dispatch)

The dispatch claimed an unclosed `(` at line 990 (`fn eval_chunk_into`)
blocking compilation. Observed at lane start: the file was already in a
compiling, fully-tested state. Witness: `/tmp/p1-repair-target` was populated
with a complete green build at 23:45 local, immediately before this lane's
first build — an earlier instance of this lane (or a concurrent repairer)
landed the repair between task computation and this run.

Bracket-balance check on the raw file reports `paren -1`; this is inside
string literals / doc comments (rustc parses the file clean), not a syntax
defect.

## Verification (forced recompile, not cache trust)

`touch crates/cgap-bn3/src/unify.rs` before each gate so the fingerprint
invalidates and rustc re-parses the exact on-disk bytes.

- `CARGO_TARGET_DIR=/tmp/p1-repair-target cargo build -p cgap-bn3` — exit 0,
  "Compiling cgap-bn3 v26.11.2" + "Finished dev profile".
- `CARGO_TARGET_DIR=/tmp/p1-repair-target cargo test -p cgap-bn3` — exit 0:
  119 unit tests passed (incl. `differential_court_matches_naive_oracle_over_
  100_witnessed_corpora`, `stress_court_1m_rows_converges_chunk_stable_...`),
  7/7 `tests/lever_courts.rs` passed (l3_emit_join_masks_court,
  l3_noalloc_radix_capacity_refusal, l3_noalloc_radix_differential,
  l3_antivacuity_probe_drop_flips, l3_differential_join_court,
  l3_radix_vs_nested_differential, l3_radix_vs_nested_100k), 0 doc-tests.
- `CARGO_TARGET_DIR=/tmp/p1-repair-target cargo build --workspace` — exit 0
  (only a pre-existing `cgap-integration` manifest warning).

## Standing

ALIVE — all three gates ran this session against the exact on-disk subject
with exit 0. No edits made by this lane instance; the repair itself is
UNVERIFIED-as-to-author (attributed to the earlier lane instance, receipt
absent at lane start).

```json
{"lane":"p1-repair","files_written":["docs/jira/v26.11.2/waves/p1-repair-unify-repair.md"],"commands":["cargo build -p cgap-bn3 -> exit 0 (forced recompile)","cargo test -p cgap-bn3 -> exit 0, 119 unit + 7 lever_courts passed","cargo build --workspace -> exit 0"],"courts_run":["tests/lever_courts.rs x7 all green","unify differential/stress/zero-alloc unit courts green"],"standing":"ALIVE","divergences":["dispatch claimed unclosed '(' at line 990 blocking compile; file already compiled and tested at lane start (prior instance repaired it); paren imbalance -1 is inside strings/comments, rustc-clean"]}
```
