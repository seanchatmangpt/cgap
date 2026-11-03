# lane-dod-11-r10

2026-11-02. Repo /Users/sac/cgap @ main. Dispatcher: coordinator
(DOD wave). W7 skeleton per _LANES-TEMPLATE.md.

## 1. Identity

- Lane DOD-11 (R10 closure — the non-flipping zero-tax children).
- Owned files written this lane:
  - crates/csct/src/bin/cscb_c1_c2_ingress.rs (zero-tax arm)
  - crates/csct/src/bin/cscb_c4_alloc.rs (zero-tax arm)
  - crates/csct/src/bin/cscb_c7_receipt_sealing.rs (zero-tax arm)
  - docs/jira/v26.11.2/audit/release-blockers.md (R10 append only)
  - docs/jira/v26.11.2/lanes/lane-dod-11-r10.md (this receipt)

## 2. Authority

Dispatch prompt. C3/C6 NOT edited: verified fresh this session that
they already alias zero-tax (cscb_c3_closure.rs:178
`m == "radix-clamp" || m == "zero-tax"`;
cscb_c6_retraction.rs:53-54). C5 is scripts/verify_cscb_sram.py, a
python script — skipped with disclosure (not an owned Rust bin; also
not env-driven for mutations). No SLA unblocks applied to other lanes'
files.

## 3. Commands+exits

Build: `cd crates/csct && CARGO_TARGET_DIR=/tmp/dod11-target cargo
build --release --bins` -> exit 0 (dead_code warnings pre-existing).

Per-child runs (bins from /tmp/dod11-target/release):

| Child | Clean cmd -> exit | Mutated cmd -> exit |
|---|---|---|
| C1/C2 cscb_c1_c2_ingress | `--self-gen` -> 0 | `CSCB_MUTATE=zero-tax` -> 1 (ANDON line) |
| C4 cscb_c4_alloc | (no args) -> 0 | `CSCB_MUTATE=zero-tax` -> 1 (ANDON line) |
| C7 cscb_c7_receipt_sealing | (no args) -> 7 runs, exit 0 | `CSCB_MUTATE=zero-tax` -> 1 (ANDON line) |
| C3 cscb_c3_closure | ladder row exit 0 | ladder row exit 1 (pre-existing alias) |
| C6 cscb_c6_retraction | ladder row exit 0 | ladder row exit 1 (pre-existing alias) |

Ladder (runner + children fresh-built in lease target
/tmp/dod11-target, CARGO_TARGET_DIR exported so target_bin() resolves
fresh binaries):

- Clean: `cscb_runner --corpus /tmp/cscb_corpus_dod11.bin
  --receipt-out /tmp/dod11_clean3.md` -> exit 0 (all 8 coordinates
  exit 0; blake3-root 96fe88d56360a79c4f8fb725bf84c888a3a6f92d95054196900655ef02a985dc).
- Mutated: `CSCB_MUTATE=zero-tax cscb_runner --corpus
  /tmp/cscb_corpus_dod11m.bin --receipt-out /tmp/dod11_mut2.md` ->
  exit 1; rows C1 1, C2 1, C3 1, C4 gate 1 + child 1, C6 1, C7 1;
  C5 exit 0 (python, not env-driven — disclosed); C8 exit 0 (cargo
  test, not env-driven — disclosed).
- First clean attempt (/tmp/dod11_clean2.md) exit 1 from C6 cold-start
  rate-gate miss (418 ms vs 12 ms warm); re-run warm -> exit 0.
  Disclosed, not hidden.

## 3b. Courts table

| Court | Anti-vacuity mutation | Flip witness | Exit |
|---|---|---|---|
| Per-child zero-tax flip (C1/C2, C4, C7) | CSCB_MUTATE=zero-tax typed-refusal arm | ANDON line + exit 1 vs clean exit 0 | 1 |
| Per-child zero-tax flip (C3, C6) | pre-existing MV-5 aliases | ladder rows exit 1 vs clean exit 0 | 1 |
| Runner clean baseline | none | /tmp/dod11_clean3.md, exit 0 | 0 |
| Runner zero-tax flip | CSCB_MUTATE=zero-tax | /tmp/dod11_mut2.md, exit 1, 6 children refusing | 1 |

The flip IS the court; a child without the arm ran clean under the
mutation (witnessed on stale shared-tree binaries before re-targeting
— the mutated ladder's stale run /tmp/dod11_mut.md shows C1/C7 exit 0
under zero-tax, the exact vacuous state R10 named; now closed).

## 4. Consequence

Diff: three bins gain an early typed-refusal arm (loud ANDON line +
exit 1) before their normal mutation parsing; C3/C6 untouched;
C5/C8 disclosed non-flipping. Fences: runner aggregate semantics
unchanged (GAP-04 gate intact — witnessed in the mutated receipt);
C4's alloc-mutation path, C7's `CSCB_MUTATE=1` bitflip path, and C1/C2's
`branchy` path all still parse their own values (zero-tax is checked
first and is not their value, so no conflation — C7's former
`is_ok()` conflation risk with zero-tax is closed by ordering). The
mutated ladder also flipped on stale-vs-fresh binary ambiguity once
(/tmp/dod11_mut.md vs /tmp/dod11_mut2.md) — resolved by exporting
CARGO_TARGET_DIR; recorded as a fence against silent stale-binary
receipts.

## 5. Standing

- R10: CLOSED by append (every present child flips, both layers
  witnessed this session). Standing ALIVE per commands+exits above.
- C5 (python) and C8 (cargo test) remain non-flipping under zero-tax:
  loud disclosure, not a silent pass — recorded in the R10 append.
- Pre-existing C1/C2 C2-gate flakiness near the 10 ns p99 threshold
  observed once this session (direct run p99 15 ns FAIL; ladder runs
  exit 0) — not introduced by this lane; recorded, not fixed (not
  this lane's file scope for gate tuning).

## 6. Divergences/disclosures

- C5 is a python script: skipped per dispatch with this disclosure.
- C8 is a test-module court, not an env-driven bin: not flippable by
  CSCB_MUTATE=zero-tax; disclosed, not silently counted as flipped.
- Runner-level C4 gate (GAP-04) untouched; both layers now flip.
- Receipts cited from /tmp (dod11_clean3/mut2) are session artifacts;
  the durable witness is this receipt + the R10 append.
