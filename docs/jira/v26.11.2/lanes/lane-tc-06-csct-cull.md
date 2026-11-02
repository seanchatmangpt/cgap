# Lane TC-06 — csct duplicate-court cull (cull-04)

## W1 Subject

`crates/csct/tests/csct_pipeline_court.rs` (sole owned source file) plus this
receipt. No other file touched; no git commands run.

## W2 Census (pre/post)

Pre (baseline, session-run, exit 0):
- `cargo test` (CARGO_TARGET_DIR=/tmp/tc06-target): **71 passed / 0 failed**
  (41 lib / 1 cscb_c1_c2_ingress / 7 csct_runner / 9 csct_coverage_court /
  7 csct_pipeline_court / 1 posh_batch_court / 5 posh_etag_court / 0 doctests).
Post (same command, exit 0):
- **69 passed / 0 failed**
  (41 / 1 / 7 / 9 / **5** csct_pipeline_court / 1 / 5 / 0).
Delta: −2 tests, both deletions intended by cull-04; zero unintended loss.

## W3 The diff (cull-04 primary finding)

Deleted test (c) `mutation_controls_pass` from csct_pipeline_court.rs — it
duplicated `src/mutation.rs` `mutation_controls_pass` (mutation.rs:66)
verbatim in intent. SURVIVOR = `src/mutation.rs`'s own court (owned by its
lane; untouched by TC-06), which is strictly stronger: the pass assert plus
three flip courts (`missing_entry_flips_gate`,
`duplicate_stage_flips_gate`, `empty_mutation_id_flips_gate`). The now-unused
`use csct::mutation::run_mutation_controls;` import was removed with it.
A comment at the excision site names the subsumer.

## W4 The diff (second finding)

Deleted test (f) `andon_refusal_wire_value_is_101`. Canonical exit-variant
wire pin SURVIVOR (my pick): `tests/csct_coverage_court.rs` COURT 1
`every_exit_variant_encodes_and_renders` (:113-115) — it pins ALL five
variants 0/1/2/3/101, pairwise distinctness, and rendered receipt bytes: a
strict superset of the deleted test's two asserts. Header doc of
csct_pipeline_court.rs updated to match.

## W5 Disclosure (disclose-in-both-directions)

`src/types.rs` `exit_code_wire_values` (:191) ALSO pins all five wire values
and is OUTSIDE TC-06's file ownership (read-only this lane). With the (f)
deletion the wire pin now has TWO surviving sites, not one — types.rs's copy
predates this lane and is disclosed here for the types.rs owning lane to
dedupe (or to bless as the src-side pin; coverage-court copy remains the
integration-side survivor). No SLA unblock was applied to any other lane's
file.

## W6 Flip witness

Survivor court run this session (exit 0, 11 passed / 30 filtered):
`cargo test --lib mutation` → `mutation::tests::mutation_controls_pass`,
`missing_entry_flips_gate`, `duplicate_stage_flips_gate`,
`empty_mutation_id_flips_gate` all ok. Flip battery witnessed on the
survivor; the deleted copies added zero unique bits.

## W7 Standing + notes

Standing: **ALIVE** — all commands run this session against the exact
subject, exits cited above. Verification: post-cull `cargo test` exit 0,
69/69 green; flip witness exit 0. Build root /tmp/tc06-target deleted at
lane end (lease law). Divergences: (1) dispatch flagged
"posh_batch_court's typed FAIL is pre-existing (POSH-2)" — as run this
session posh_batch_court PASSED (1/1, exit 0); recorded as observed, not
rebutted, in case the dispatch referenced a release-profile variant.
(2) W5 two-survivor residue, above.
