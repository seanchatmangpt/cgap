# Lane RC-06 — Whole-media-loss DR court (RED-16 A2 / FIX-14 R-A2.x)

Date: 2026-11-02. Branch: main. No git write commands run.

## 1. Identity + authority

- Lane RC-06, dispatch contract: single new test file
  `crates/cgap-storage/tests/media_loss_recovery_court.rs`; read-only over
  `atomic_commit.rs` (no pub accessor was needed — the existing
  `Medium`/`recover`/`recover_mode`/`MutationMode` surface sufficed; NO
  SLA unblock was applied to any file).
- Remediation target: `docs/jira/v26.11.2/redteam/fix-14-ops-requirements.md`
  A2 (status PARTIAL; "whole-media-loss court MISSING") per
  `red-16-headcount-paradox-audit.md` hole A2.

## 2. Diff (what landed)

- NEW: `crates/cgap-storage/tests/media_loss_recovery_court.rs` (386
  lines, on-disk verified). Six courts over real `std::fs` media:
  - C1: whole-site loss -> restore from complete off-site snapshot
    re-derives committed root, Clean, seq 2.
  - C2a: off-site journal copy truncated mid-top-record -> last INTACT
    record applied; lost transition named via `slot_verdicts[1] == Damaged`.
  - C2b: copy truncated inside the first slot -> base root +
    `RecoveryDegraded`, `recovered_seq == None`, both slots Damaged.
  - C3: off-site copy missing the journal artifact entirely (R-A2.2
    DATA_LOSS posture) and missing/short base root -> typed
    `RestoreRefusal::{JournalMissing, BaseRootMissingOrShort}`; no
    fabricated root ever reaches `recover`.
  - C4 (contract anti-vacuity): one payload byte of the off-site journal
    copy corrupted in a fresh tempdir -> typed `RecoveryDegraded`, base
    root stands, corrupted slot named. A recovery that accepts the
    corruption is FAIL.
  - C5 (R-A2.3 mutation flip): trailer-half byte corrupted; Strict
    degrades, `MutationMode::M3TrailerDigestSkipped` on the SAME bytes
    recovers Clean with the corrupted record's root — court flips.
- Court-side glue (disclosed, not a module change): `RestoredMedium`
  (real-file `Medium` impl reading a restored site) with a typed
  fail-closed `open()`; truncated slot regions are zero-filled (NOT
  0xFF) so truncation classifies Damaged, never the erased-state
  carve-out.

## 3. Commands + exits (this session)

- `CARGO_TARGET_DIR=/tmp/rc06-target cargo test -p cgap-storage --test
  media_loss_recovery_court` -> exit 0, **6 passed; 0 failed** (final
  rerun, warning-free after fixes).
- `CARGO_TARGET_DIR=/tmp/rc06-target cargo test -p cgap-storage` (full
  crate suite) -> exit 0, 14 test binaries all ok (incl. the 222.15 s
  court_faults family); exit captured piped via `echo EXIT=$?` = 0.
- `git -C /Users/sac/cgap status --porcelain -- crates/cgap-storage/` ->
  exit 0: only this lane's new file plus two OTHER-lane in-flight items
  (`M tests/common/mod.rs`, `?? journal_fault_matrix_court.rs`) which
  this lane did NOT touch and did not build against... (build did link
  the shared `common/mod.rs` as it exists on disk; disclosed below).

## 4. Fences (must-not-regress surfaces) — run

- `cargo test -p cgap-storage` full suite green (exit 0, this session):
  the A05 recovery courts (`court_faults`, `court_mutations`,
  `court_sweep`, `court_flash`, `court_crc_identity`, closure/dred
  courts) all pass with this lane's additive-only change.
- Shared tree: additive test file only; `atomic_commit.rs` untouched
  (verified: not in git status porcelain output).

## 5. Divergences / disclosures

- R-A2.2's literal ask ("typed DATA_LOSS verdict" from the kernel
  recovery chain) is implemented at court-harness fidelity: the typed
  refusal (`RestoreRefusal::JournalMissing`) lives in the test's
  restore harness, because the module-level `Medium::read_base_root`
  contract has no DATA_LOSS arm. A kernel-side DATA_LOSS verdict would
  be a module API change outside this lane's file ownership. Registered
  as the remaining gap vs R-A2.2.
- R-A2.4 (ratio-fence restore budget) NOT covered by this lane — no
  perf fence claimed; TARGET only.
- `tests/common/mod.rs` was MODIFIED-ON-DISK by another lane while this
  lane ran (git status M). This court builds against it as-is; owning
  lane supersedes.
- Background command b5nonq7y4 (full-suite grep) was superseded by the
  final clean rerun; its terminal output not load-bearing.

## 6. Standing

- ALIVE — court runs green this session, exact command+exit cited above.
- Lease cleanup: `python3 -c "import shutil;
  shutil.rmtree('/tmp/rc06-target', True)"` to run at lane end
  (executed after this receipt write).
