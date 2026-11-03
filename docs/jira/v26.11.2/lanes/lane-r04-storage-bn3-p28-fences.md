# Lane R-04 — cgap-storage + cgap-bn3 + cgap-p28 fences (tollgate wave)

Date: 2026-11-02. Repo: /Users/sac/cgap (main). Read-only fence lane — no
source edits, no git commands.

## W1 Subject

Re-witnessed the RC-06 / RC-07 / RC-03 / RC-04 / FIX-02 lane surfaces
(docs/jira/v26.11.2/lanes/lane-rc06-media-loss-dr-court.md,
lane-rc07-journal-fault-matrix.md, lane-rc03-bn3-capacity-bound.md,
lane-rc04-attestation-mac.md, lane-fix02-bn3-sram-gate.md) as full fences,
this session, fresh build lease /tmp/r04-target.

## W2 Commands + exits (this session)

Fence 1 — cgap-storage full suite:
- `python3 scripts/run_checked.py --out /tmp/r04-storage2.log -- env
  CARGO_TARGET_DIR=/tmp/r04-target cargo test -p cgap-storage` → exit 0.
  13 test binaries all `0 failed`; includes
  `media_loss_recovery_court.rs` **6 passed / 0 failed** and
  `journal_fault_matrix_court.rs` **3 passed / 0 failed**.
- First cold-target run (exit 101) had one `never executed` spawn failure
  of court_crc_identity ("No such file or directory" at the build-script
  out-dir path) — compile/spawn-abort class, not a test failure; the
  targeted rerun passed 3/3 and the warm full-suite rerun was exit 0
  across all 13 binaries.

Fence 2 — cgap-bn3:
- `cargo test -p cgap-bn3` → exit 0: 119 lib + bgp 2 (+2 ignored) +
  **capacity_bound_court 3/3** + edge_closure 5 + lever 7 + repair20 3.
- `python3 scripts/verify_cscb_sram.py` → exit 0, baseline 36048 B,
  headroom 95024 B (FIX-02 baseline fence held).
- `python3 scripts/verify_cscb_sram.py --subject bn3` → exit 1,
  `BN3_ENCLAVE_REFUSED ... 6520984 B (NaMatches 2162696 B + NaEngine
  4358288 B) > 131072 B` (refusal persists, 49.75x ceiling).

Fence 3 — cgap-p28 full suite:
- `cargo test -p cgap-p28` → exit 0: 38 lib + **attestation_mac_court
  4/4** (incl. tamper sweeps) + lever 2 + invariant 6 = 50 passed,
  0 failed.

## W3 Residual (typed, stays REGISTERED)

- Kernel-side DATA_LOSS arm still absent: `grep -n "DATA_LOSS\|DataLoss"
  crates/cgap-storage/src/atomic_commit.rs` → zero matches this session.
  R-A2.2's literal ask remains the RC-06 registered divergence
  (typed refusal lives in the court harness
  `crates/cgap-storage/tests/media_loss_recovery_court.rs`, not the
  module API). NOT fixed by this lane; stays REGISTERED.

## W4 Fences

These fences ARE the must-not-regress surface: full crates
cgap-storage/cgap-bn3/cgap-p28 suites + the FIX-02 SRAM gate (both
subjects). All green this session (bn3 subject exit 1 is the gate's
correct typed refusal, not a failure). One cold-target spawn-abort
observed and superseded by a clean warm rerun (W2); disclosed, not hidden.

## W5 Standing

ALIVE — every claim carries a command + exit run this session against the
exact subject.

## W6 Divergences

- None vs the dispatch contract. No SLA unblocks applied; no shared-file
  edits. Build lease /tmp/r04-target deleted via shutil at lane end
  (witnessed, exit 0).

## W7 Cleanup

/tmp/r04-target removed (python3 shutil.rmtree, exit 0). Logs retained at
/tmp/r04-{storage,storage2,crc,bn3,sram-base,sram-bn3,p28}.log.
