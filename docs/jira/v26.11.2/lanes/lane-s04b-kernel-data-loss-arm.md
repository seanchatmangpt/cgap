# Lane S-04b — Kernel-side DATA_LOSS verdict arm (closes RC-06 divergence)

Date: 2026-11-02. Branch: main. No git write commands run.

## 1. Identity + authority

- Contract files (both owned, both landed):
  - `crates/cgap-storage/src/atomic_commit.rs` — ADDITIVE only: two
    default-`true` `Medium` trait methods
    (`journal_present` / `base_root_present`, added S-04b) + the typed
    DATA_LOSS surface (`DataLossClass`, `DataLossEvidence`,
    `RecoveryVerdict`, `recover_loss_classified`). The journal format, the
    validator chain, `recover`, and `RecoveryError` arms are UNCHANGED.
  - `crates/cgap-storage/tests/media_loss_recovery_court.rs` — extended
    with C6/C7/C8 (see §2). RC-06's landed legs C1-C5 untouched.
- Closes the registered divergence in
  `docs/jira/v26.11.2/lanes/lane-rc06-media-loss-dr-court.md:71-74`
  ("kernel-side DATA_LOSS verdict would be a module API change outside
  this lane's file ownership"). RC-06 = court-harness fidelity; S-04b =
  kernel fidelity. FIX-14 R-A2.2 ("base root + journal both lost => typed
  DATA_LOSS verdict, fail-closed, no fabricated root") now holds at
  kernel-API fidelity: `recover_loss_classified(&medium)`.

## 2. Diff (what landed)

- `Medium::journal_present()` / `Medium::base_root_present()`: new trait
  methods, default `true` — every pre-existing impl (FlashMedium, the
  other-lane `FileMedium`, all existing courts) compiles unchanged; the
  shared storage suite stayed green with zero edits outside this lane's
  two files.
- `recover_loss_classified<M: Medium>(&M) -> Result<RecoveryVerdict,
  RecoveryError>`:
  - base-root superblock absent/short => `RecoveryVerdict::DataLoss`
    with `class = BaseRootMissing`, `base_root = None` (never
    fabricated), slot verdicts + `top_journal_seq` decoded from
    surviving journal bytes (C7 asserts `top_journal_seq == Some(2)`).
  - journal artifact absent => `DataLoss` with
    `class = JournalMissing`, surviving base root reported, `slot_
    verdicts = None`, `top_journal_seq = None`,
    `lost_seq_floor = FIRST_SEQ`.
  - both absent => `BaseRootMissing` wins (documented precedence).
  - otherwise: `Recovered(recover(&medium))` — identical to the
    existing chain (Strict), so Degraded/Clean/JournalAmbiguous/
    SeqRegression semantics are untouched.
- Sharpness law (asserted by C8): DATA_LOSS = no lawful recovery
  substrate exists (unrecoverable without off-site copy); Degraded =
  recovery ran and completed WITH losses (rollback to base). Truncation
  never classifies DATA_LOSS — C8 fails the court if a truncation
  scenario ever yields the `DataLoss` arm.
- Court legs added to `media_loss_recovery_court.rs`:
  - C6: off-site copy missing the journal artifact entirely =>
    kernel-side `DataLoss(JournalMissing)` with evidence asserted
    (base root Some, slot verdicts None, lost_seq_floor FIRST_SEQ).
  - C7: base-root superblock lost, journal survives => kernel-side
    `DataLoss(BaseRootMissing)` with journal evidence preserved
    (slot verdicts `[Valid; 2]`, `top_journal_seq Some(2)`).
  - C8: partial truncation (40 bytes, torn first slot) through the
    kernel classifier => `Recovered(RecoveryDegraded)`, NOT DataLoss —
    the Degraded/DATA_LOSS distinction is pinned sharp.
  - RC-06's C1-C5 all still pass (9/9 court legs total).
- `RestoredMedium` gained `open_lossy` (skips harness refusals so the
  KERNEL classifies — RC-06's typed `open()` refusals remain for the
  harness-fidelity legs) and the two presence overrides.

## 3. Commands + exits (this session)

- `CARGO_TARGET_DIR=/tmp/s04b-target cargo build -p cgap-storage` -> exit 0.
- `CARGO_TARGET_DIR=/tmp/s04b-target cargo test -p cgap-storage --test
  media_loss_recovery_court` -> exit 0, **9 passed; 0 failed** (C1-C5
  from RC-06 + C6/C7/C8).
- `CARGO_TARGET_DIR=/tmp/s04b-target cargo test -p cgap-storage` (FULL
  suite) -> exit 0, 14 test binaries all ok (6+3+3+3+3+5+3+6+2+5+3+3+9
  tests; longest court_bitplane_seen 219.75 s).
- Anti-vacuity mutations (M-DL1/M-DL2), /tmp sandbox copy of
  cgap-storage only (deps re-pointed at absolute shared paths; shared
  tree untouched — sha256 of shared atomic_commit.rs identical
  pre/post sandbox work: 1366c20e0395a669...efab92f):
  - M-DL1 (`if false && !medium.journal_present()` — DATA_LOSS
    classification for a missing journal removed): court FAILS,
    exit 101, exactly `c6_journal_artifact_lost_kernel_side_data_loss`.
  - M-DL2 (`if false && !medium.base_root_present()` — base-root
    classification removed): court FAILS, exit 101, exactly
    `c7_base_root_lost_kernel_side_data_loss_with_journal_evidence`.
  - Each mutation kills exactly its target court leg — the flip is
    sharp, not a blanket failure.
  - Shared tree sha256 re-witnessed unchanged after all mutation runs.
- Lease cleanup executed: /tmp/s04b-target, /tmp/s04b-sandbox-target,
  /tmp/s04b-sandbox removed.

## 4. Fences (must-not-regress surfaces) — run

- FULL `cargo test -p cgap-storage` green, exit 0, this session (14
  binaries) — covers the A05 recovery courts (`court_faults`,
  `court_mutations`, `court_sweep`, `court_flash`, `court_crc_identity`),
  the closure/dred courts, and the other-lane in-flight
  `journal_fault_matrix_court.rs` (3/3, exit 0).
- The additive trait methods are default-`true`, so no other crate's
  impls change behavior; `cargo build -p cgap-storage` exit 0.
- `RecoveryError` arms and `recover()`/`recover_mode()` signatures and
  semantics unchanged — RC-06's harness-fidelity legs (C1-C5, C3's
  typed `RestoreRefusal`s) pass unmodified.

## 5. Divergences / disclosures

- In-flight other-lane items observed in git status and NOT touched by
  this lane: `M tests/common/mod.rs` (built against as-is, owning lane
  supersedes), `?? tests/journal_fault_matrix_court.rs`.
- Session defect self-correction, disclosed: two mid-edit corrupted
  tokens were introduced by this lane and repaired in the same session
  (a mangled write in atomic_commit.rs's loss-classification section,
  caught by the write-integrity hook; and a stray placeholder token in
  the court file, self-caught). Final on-disk bytes are the intended
  content; both files compile and 9/9 court legs pass. No prior
  receipt was edited.
- R-A2.4 (ratio-fence restore budget) remains open — not this lane's
  contract; TARGET only, no perf fence claimed.
- The receipt-replay postmortem wiring (R-A2.2 tail, cgap-ffi) remains
  open work for the cgap-ffi owning lane.
- Errata duty (none yet): if RC-06's receipt is later amended, cite
  this file, not a silent edit.

## 6. Standing

- ALIVE — every claim above carries a command + exit run this session
  against the exact on-disk subjects (sha256 witnesses in §3).
- Coordinator note: RC-06's registered divergence -> **CLOSED-RECEIPTED
  pending coordinator integration**. FIX-14 A2 gap "kernel-side
  DATA_LOSS verdict MISSING" is closed at kernel fidelity; remaining
  R-A2.2 residual is the cgap-ffi receipt-replay postmortem wiring.
