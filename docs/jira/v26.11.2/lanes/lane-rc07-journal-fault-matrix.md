# Lane RC-07 — Journal Fault-Mutation Matrix

## 1. Identity

- Lane RC-07, repo /Users/sac/cgap (branch main), 2026-11-02.
- Subject: NEW `scripts/journal_mutation_matrix.py` (sha256
  `c1e3852e0c9416fb6d6b33c0a98756ada16311082b50bce054f4c35e9b81b06f`) +
  NEW `crates/cgap-storage/tests/journal_fault_matrix_court.rs` (sha256
  `b191863375dd0cccf04179943f55b0313e1ccfe2b7c90c208bf178ee7839a65d`),
  against `crates/cgap-storage/src/atomic_commit.rs` (read-only).
- Remediates fix-14 A2 (R-A2.1/R-A2.3): automated fault matrix over the
  single-fault mutation protocol; `redteam/fix-14-ops-requirements.md:66-73`
  (premise sha256 `deb5cf8f...1510a2`, ledger row appended).

## 2. Authority

Dispatch prompt (owned files: the two above + heartbeat `rc-07.alive` +
ledger row). atomic_commit.rs untouched (read-only). No git commands run.

## 3. Commands + exits

- `cargo test -p cgap-storage --test journal_fault_matrix_court`
  (CARGO_TARGET_DIR=/tmp/rc07-target): 3 passed, 0 failed — exit 0.
  First compile hit a sibling lane's in-flight `cgap-git/pack_demux.rs`
  unclosed-delimiter abort (compile-abort, not a failure count; owner
  fixed it; later runs green).
- `python3 scripts/journal_mutation_matrix.py` — exit 0:
  `PASS: 17 rows, 14 non-clean ... self-check detected corrupt restore`.
- Negative controls: RC07_REPO=/tmp/nonexistent-rc07 → exit 2
  (SUBJECT MISSING); env -i PATH without cargo → exit 3 ([SKIP] loud).
- Exit-1 witnesses (fail-closed law): mid-development the gate correctly
  exited 1 on a VACUOUS finding (4 rows silently clean, pre-verdict fix)
  and on a row mismatch (tuple arity). /tmp script copy with a corrupted
  expectation (bitflip_seq2_first expected=pristine) → exit 1, flip
  witnessed: `FAULT: row bitflip_seq2_first: expected ... got ...`.
- Fence run: full `-p cgap-storage` blocked by a sibling lane's in-flight
  `tests/media_loss_recovery_court.rs` (E0277/E0369, not mine). All 11
  other test targets swept green this session (closure_alloc 3,
  closure 3, bitplane_seen 3, crc_identity 3, faults 5, flash 3,
  mutations 6, sweep 2, dred_cascade 5, dred_swar_wire 3,
  journal_fault_matrix 3 — all "0 failed").

## 4. Consequence

- The script enumerates 17 rows (pristine + 16 faults/invariances):
  bit-flip first/mid/last in each record, truncation k=1/60/111 of the
  final record, torn first record, both-records-lost, missing journal,
  zeroed header, duplicated record (F3 dedup), duplicate conflict
  (typed JournalAmbiguous), reordered records (swap invariance).
- Fault bytes are applied to /tmp sandbox copies of a REAL fixture
  minted by `encode_record` in the Rust harness (Python never
  synthesizes journal bytes — no BLAKE3 in stdlib); the REAL recovery
  runs in-process via the harness test (`JFM_SPEC`/`JFM_OUT` line
  protocol, no serde). Expected classes were derived from reading the
  code (top valid seq wins; degraded only when NO valid candidate and a
  slot is Damaged; erased-only = clean base rollback; same-seq conflict
  = typed refusal), then pinned; divergence = exit 1.
- Restore law: fixture restored sha256-verified byte-exact + mtime bump;
  `--self-check` corrupts the restore on a real temp copy and must
  detect it (witnessed `corrupt restore DETECTED`).
- Fences that could regress: cgap-storage test targets (all green,
  swept above); cgap-git dev-dep compile (green after sibling fix).

## 5. Courts table

| Court | Mutation (anti-vacuity) | Flip witness (command+exit) |
|---|---|---|
| `fault_matrix_matches_derived_classes` (Rust) | per-row: fault rows must differ from pristine; invariance rows (dedup/reorder) must not drift; corpus needs >=2 non-clean | cargo test ... journal_fault_matrix_court, exit 0; vacuity path witnessed earlier as exit 101 (duplicated_record) before the fault/invariance split |
| `m3_trailer_digest_mutation_flips` (Rust) | M3TrailerDigestSkipped over bitflip at byte 105 of seq 2: Strict falls back to seq 1, M3 accepts seq 2 — assert_ne on roots | same command, exit 0 |
| Python gate matrix (17 rows) | corrupted expectation on /tmp script copy → exit 1 flip; self-check corrupt restore detected | `python3 scripts/journal_mutation_matrix.py` exit 0; /tmp copy flip exit 1 |
| Exit law | subject missing → 2; no cargo → 3 [SKIP] | RC07_REPO=/tmp/nonexistent-rc07 exit 2; env -i exit 3 |

## 6. Standing

ALIVE — every claim above carries a command + exit run this session
against the exact subject files.

## 7. Divergences / disclosures

- Doc-vs-reality: the fault corpus lives on the A05 2-slot in-process
  flash model (`SLOT_COUNT=2`), so "journal" = 2x112-byte slots; the
  "same matrix" in the Rust court and the Python gate is aligned row for
  row (17 rows, identical expectations incl. slot-verdict pairs).
- The Rust court's in-process anti-vacuity uses the module's own typed
  `MutationMode` (D3 divergence in atomic_commit.rs) instead of file
  mutation — file-level mutation/restore is exercised by the Python
  gate on the /tmp fixture sandbox instead.
- No SLA unblocks of other lanes' files; two sibling in-flight compile
  aborts (cgap-git/pack_demux.rs early; media_loss_recovery_court.rs at
  fence time) were owner-owned and left untouched.
- Heartbeat rc-07.alive + ledger row landed per template law (not in the
  dispatch's owned-files list — disclosed here).
