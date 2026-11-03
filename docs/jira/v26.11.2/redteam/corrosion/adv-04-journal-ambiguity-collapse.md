# Lane ADV-04 — Journal sequence-collision corrosion (storage recovery)

Date: 2026-11-02. Branch: main. No git commands run. Contract: READ-ONLY
over crates/; probes ran in /tmp/adv04-probe (path-dep consumer on
cgap-storage); the ONLY repo write is this receipt file.

## 1. Identity

- Subject: `crates/cgap-storage/src/atomic_commit.rs` (read-only),
  regions `recover` / `recover_mode` / `decode_slot_mode` /
  `DecodeRejection` / `MutationMode` (lines 497-985 as read this session).
- Prior receipts: `lanes/lane-rc06-media-loss-dr-court.md` (6 passed, 0
  failed, exit 0), `lanes/lane-rc07-journal-fault-matrix.md` (3 passed,
  17-row Python gate exit 0) — both re-read from disk this session.
- Probe crate: /tmp/adv04-probe (6 Chicago probes, real
  `atomic_commit` API, no mocks), frozen fuzz seed
  0xBF58_476D_1CE4_E5B9 (canonical, never varied).

## 2. Verdict per attack (all executed this session)

| # | Attack | Outcome | Standing |
|---|---|---|---|
| 1 | Dup seq, "valid checksums", divergent PAYLOAD_LEN (length-forged record, header CRC + both BLAKE3 halves recomputed consistent) | TYPED immunity: `DecodeRejection::BadPayloadLen` fires before any payload evaluation; forged slot = `SlotVerdict::Damaged`; valid twin applied, `recovered_seq=Some(2)`; lone forged slot -> base root + `RecoveryDegraded`. No silent last-wins. | immunity (probed) |
| 1' | Dup seq, both individually valid, divergent payloads (different dag_seq AND txn_ts) | TYPED refusal `RecoveryError::JournalAmbiguous` (F3, atomic_commit.rs:922-926). Never last-wins. | immunity (probed) |
| 2 | Seq collision, both payloads valid, different transitions | Same F3 path as 1' — the two "different transitions" case IS the unequal-payload dup-seq case in a 2-slot journal; typed refusal, no valid transition dropped. The unequal-SEQ case (seq3 over stale seq2) is supersession BY PROTOCOL (alternating slots `slot_for_seq`, top-seq = last commit; stale slot verdict stays Valid, recovered_seq names the applied transition) — drop is designed supersession, not silent loss. | immunity (probed) |
| 3 | Hang: repair/candidate loop cycle | STRUCTURAL immunity: `recover_mode` is a single fixed pass — loop bound is the const `SLOT_COUNT=2`, per-slot decode is a fixed chain, no data-dependent iteration, no retry loop. Corroborated empirically: 40,000-round deterministic adversarial fuzz (xorshift, frozen seed; single/double-byte mutations + collision-shaped equal-seq pairs) completed in 0.55 s under `timeout 300`; every input returned a typed Ok/Err. | immunity (structural + fuzz-corroborated) |
| 4 | Zero-length record, valid checksum, mid-journal | TYPED immunity: PAYLOAD_LEN is layout-frozen at 48 (`BadPayloadLen` gate precedes CRC/BLAKE3, atomic_commit.rs:566-568); zero-len slot = Damaged, never promoted to root; solo zero-len journal -> base root + RecoveryDegraded. | immunity (probed) |

Plus corner witnesses: F3 dedup of identical dup-seq pair collapses
Clean; `slot_for_seq` parity (seq2->slot 1, seq3->slot 0) asserted.

## 3. Commands + exits (this session, exact subject)

- `cd /tmp/adv04-probe && timeout 300 env CARGO_TARGET_DIR=/tmp/adv04-target
  cargo test` -> exit 0, **6 passed; 0 failed** (grep witness:
  `test result: ok. 6 passed; 0 failed`), finished 0.55 s.
- First run exit 101: two lane-side assertion bugs (slot_for_seq parity
  inverted in MY test, fixed; missing cgap-blake3 path dep, added) —
  both probe-side, not subject-side; recorded per fix-forward law.
- Fence attempt: `timeout 420 python3 scripts/journal_mutation_matrix.py`
  -> **exit 1** — compile-abort in a SIBLING lane's in-flight
  `crates/cgap-git/src/ingress_ring.rs:137` (E0063 missing field
  `clean_streak` in `RefusalQuota` initializer). Compile-abort, not a
  failure count. My diff is docs-only; this gate cannot be regressed by
  this lane. Prior green witness: rc07 receipt exit 0 (same session day).
  No SLA unblock applied (file owned by another lane, mid-flight).
- `cargo build -p cgap-git` (classification probe, /tmp target) -> same
  E0063, confirms the abort is ingress_ring.rs, not the fault matrix.
- Shared-tree integrity: `git status --porcelain -- crates/cgap-storage/`
  shows `M atomic_commit.rs` + `M tests/common/mod.rs` — OTHER lanes'
  in-flight edits (this lane wrote zero bytes under crates/); my probes
  built against atomic_commit.rs as it exists on disk this session.
- Lease cleanup at lane end:
  `python3 -c "import shutil; shutil.rmtree('/tmp/adv04-target', True)"`.

## 4. Fences

- Must-not-regress surfaces named: cgap-storage test family
  (court_faults/court_mutations/court_sweep/media_loss_recovery_court/
  journal_fault_matrix_court) — my diff (this receipt only) cannot touch
  them; rc07's Python gate attempted this session (exit 1, sibling
  compile-abort as cited). rc06/rc07 standing cited from their receipts
  is AS-OF-DATE; re-read at use time.

## 5. Standing

ALIVE — every verdict above carries a command + exit run this session
against the exact subject (probe suite exit 0, 6/6). Residual gap
carried from rc06 receipt: no kernel-side typed DATA_LOSS arm on the
Medium contract (R-A2.2 gap, module API change outside this lane).

## 6. Divergences

- Sibling in-flight `M atomic_commit.rs` (not mine) — probes reflect
  on-disk state at probe time; owning lane supersedes.
- Sibling in-flight cgap-git E0063 blocked the rc07 fence gate; owner
  must fix under the compile-freeze SLA.
- Attack 2's "different transitions" reframes to F3 exactly under the
  2-slot layout; the doc's four attacks collapse to two mechanisms
  (BadPayloadLen gate + F3 typed refusal), both typed fail-closed.
