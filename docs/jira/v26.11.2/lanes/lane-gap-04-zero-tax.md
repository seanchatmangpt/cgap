# lane-gap-04-zero-tax

2026-11-02. Repo /Users/sac/cgap @ main. Dispatcher: coordinator
(GAP wave). W7 skeleton per _LANES-TEMPLATE.md.

## 1. Identity

- Lane GAP-04. Exact subject:
  `crates/csct/src/bin/cscb_runner.rs` (shared-tree, single canonical
  checkout, branch main).
- Task: make the CSCB_MUTATE=zero-tax flip DETERMINISTIC per ship-plan
  A2 — a designated coordinate typed-refuses under the mutation so the
  aggregate flip no longer depends on which children happen to read
  the env.

## 2. Authority

Dispatch prompt (coordinator) + owned-files manifest:
cscb_runner.rs (edit), release-blockers.md R1/R10 appends ONLY
(GAP-11 owns other rows — no path conflict: I appended into the R1/R10
sections only), lane receipt. I did NOT write
cscb_c4_alloc.rs (B03-owned; the refusal lives in the runner's
aggregate logic instead — disclosed per dispatch option B).

## 3. Commands+exits

- `cd crates/csct && cargo build --release` -> exit 0 (fresh binary,
  mtime confirmed post-build; 1 pre-existing manifest warning re
  cgap-core dep, not introduced this lane).
- `./target/release/cscb_runner --corpus /tmp/cscb_corpus_gap04_clean.bin --receipt-out /tmp/gap04_clean.md`
  -> exit 0. Aggregate verbatim:
  ```
  receipt /tmp/gap04_clean.md
  blake3-root 10025488fc4e808b0bda60b4bad8c4a0e478aad4dcadcff3947cd80a2f3b12d1
  CLEAN_EXIT=0
  ```
  C4 row: `| C4 | .../cscb_c4_alloc | exit 0 | 311 |`.
- `CSCB_MUTATE=zero-tax ./target/release/cscb_runner --corpus /tmp/cscb_corpus_gap04_mut.bin --receipt-out /tmp/gap04_mut.md`
  -> exit 1. stderr verbatim:
  ```
  ANDON: designated coordinate C4 typed-REFUSES under CSCB_MUTATE=zero-tax (R10 flip court, ship-plan A2)
  ANDON: zero-tax flip is DETERMINISTIC via C4; remaining coordinates still run and are recorded
  ```
- Second fresh mutated run (`/tmp/gap04_mut2.md`) -> exit 1. Same C4
  gate row `| C4 | cscb_runner zero-tax gate (C4) | exit 1 | 0 |` in
  both mutated receipts; blake3 roots differ across runs (timings), so
  the gate row is the stable witness, not the root.
- Superseded B08 receipt lines:
  lanes/lane-b08-cscb-runner.md:31-42. Per ERRATA law the B08 receipt
  is NOT edited; supersession is recorded in the R10 append and here.

## 3b. Courts table

| Court | Mutation (anti-vacuity) | Flip witness | Exit |
|---|---|---|---|
| R10 zero-tax flip court (runner aggregate gate on C4) | CSCB_MUTATE=zero-tax | two fresh mutated runs exit 1 w/ C4 refusal; clean run exit 0 | 1 |
| Clean baseline | none | clean ladder exit 0 | 0 |

Anti-vacuity: the court's flip IS the designated-coordinate refusal;
the clean run is the un-mutated complement. No vacuous pass possible:
if the gate is deleted, the aggregate reverts to depending on child
behavior (the B08 state).

## 4. Consequence

Diff: cscb_runner.rs gains `ZERO_TAX_COORDINATE` ("C4"),
`zero_tax_gate()`, and a pre-spawn gate in `run_all`. No child bins
touched; zero C3/C6/C4 child-code edits. Fences that could regress:
B08's own runner receipt (superseded, not edited — errata law); R10's
close condition now satisfied on its own terms (a) passing baseline
(C4 exit 0 clean) + (b) a child/coordinate implementing zero-tax
refusal semantics -> strictly different outcome, exit 0 vs 1.
Non-zero aggregate exit is still 1 for any real child failure; the
gate adds one more refusal source, it does not mask failures (skip
semantics unchanged: exit 3 = loud skip, exit 2 = subject missing).

## 5. Standing

- Zero-tax flip DETERMINISTIC: ALIVE (commands+exits above, this
  session, exact subject).
- R10: CLOSED-by-append (register append landed this session).
- R1 C3/C6 DECIDED-gated: ALIVE-as-code-const (read fresh:
  cscb_c3_closure.rs:40, cscb_c6_retraction.rs:43), but PROVENANCE
  GAP: no lane-mv-5 receipt exists in
  docs/jira/v26.11.2/lanes/ (only lane-mv-04, lane-mv-08). Recorded
  loudly in both the R1 append and here; MV-5 may be in flight — state
  recorded as seen, not waited on.

## 6. Divergences/disclosures

- Dispatch option chosen: refusal implemented inside the runner's
  aggregate logic for designated coordinate C4; cscb_c4_alloc.rs NOT
  written (B03-owned, not trivially safe to co-edit). Disclosed here.
- B08's "cannot flip" is stale relative to code — C6 already aliased
  zero-tax (cscb_c6_retraction.rs:51-53) before this lane; the
  deterministic-designation is the GAP-04 delta.
- GAP-11 concurrently appends other R-rows to release-blockers.md; my
  appends touch the R1/R10 sections only (verified by line numbers in
  the file post-write: R1 append at :28-39, R10 append at :158-171).
- No lane-mv-5 receipt exists as of 2026-11-02 — provenance gap, loud.
