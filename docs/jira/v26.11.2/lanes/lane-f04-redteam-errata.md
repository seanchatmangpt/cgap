# Lane F-04 — red-13/red-20 ERRATA + fix-14 A2 DATA_LOSS closure

Date: 2026-11-02. Repo: /Users/sac/cgap (canonical checkout, main).
Contract honored: writes limited to the three owned files + this
receipt; no git commands; ERRATA append-only over historical text.

## 1. Files written

- `docs/jira/v26.11.2/redteam/red-13-receipt-desync-exploit.md` —
  appended §6 ERRATA: the §1 "zero shm_open anywhere in crates/"
  sub-claim is REFUTED-AS-FILED by `crates/cgap-beam/src/shm.rs`
  (7 shm_open hits, mtime 2026-10-31 22:42, predating the report's
  2026-11-02 session; Darwin caveat shm.rs:47). Cites fix-19-corpus-
  audit.md F1 as finding receipt. Historical §1-§5 untouched.
- `docs/jira/v26.11.2/redteam/red-20-hardened-survivor-synthesis.md` —
  appended §6 ERRATA: register row 75 marked REFUTED-PARTIALLY (the
  SHM object exists in cgap-beam; Darwin-vs-Linux split still
  unmeasured); Σ_PQC row 79 stands; consolidated count reads 12
  verified + 1 refuted. Historical §1-§5 untouched.
- `docs/jira/v26.11.2/redteam/fix-14-ops-requirements.md` — A2 row
  conditional update (per dispatch): new Status bullet "kernel-side
  verdict LANDED" above the preserved as-filed PARTIAL bullet. Cites
  atomic_commit.rs:1076-1096 (RecoveryVerdict + recover_loss_classified),
  DataLossEvidence accessors :1052-1069, court legs c6/c7 in
  tests/media_loss_recovery_court.rs:409-460,462+, and the owning
  receipt lanes/lane-rc06-media-loss-dr-court.md (exit 0, 6 passed /
  0 failed). Still-missing remainder named (receipt-replay postmortem
  wiring, restore runbook, R-A2.4 ratio fence).

## 2. Verification ladder (this session)

- Read fix-19-corpus-audit.md (F1) — exit 0.
- `grep -c shm_open crates/cgap-beam/src/shm.rs` -> 7; `stat` ->
  Oct 31 22:42:28 2026, 44374 B; shm.rs:40-55 read (Darwin caveat at
  :47 confirmed).
- `grep -n DATA_LOSS atomic_commit.rs media_loss_recovery_court.rs` ->
  kernel arm + court legs on disk; `sed -n '1060,1100p'` read
  (RecoveryVerdict, recover_loss_classified confirmed verbatim).
- Court legs c6/c7 read in full (typed DataLoss(JournalMissing), base
  root preserved-not-fabricated, lost_seq_floor == FIRST_SEQ).
- `ls lanes/ | grep s04` -> lanes/lane-rc06-media-loss-dr-court.md;
  receipt standing re-read from disk: ALIVE, exit 0, 6 passed.
- fix-14 A2 Status + Standing section re-read before edit.

## 3. Fences

- ERRATA-never-rewrite: both redteam appends add a new §6; zero
  modifications to historical lines (verified by Edit old_string
  matching exact tail anchors only).
- fix-14: only the A2 Status block edited; the as-filed PARTIAL bullet
  preserved verbatim beneath the new LANDED bullet, per errata law.
- No code, no gates, no courts affected by this diff — no
  must-not-regress fences exist for prose-only errata.

## 4. Standing

ALIVE — every cite re-read from disk this session with the command and
output above; exit codes 0. No invented state: the rc06 receipt's court
run is cited as-of that receipt, explicitly marked "session of that
receipt", not re-run by this lane (courts are another lane's owned
surface).

```json
{"lane":"F-04",
 "files_written":["docs/jira/v26.11.2/redteam/red-13-receipt-desync-exploit.md",
  "docs/jira/v26.11.2/redteam/red-20-hardened-survivor-synthesis.md",
  "docs/jira/v26.11.2/redteam/fix-14-ops-requirements.md",
  "docs/jira/v26.11.2/lanes/lane-f04-redteam-errata.md"],
 "commands":["grep -c shm_open crates/cgap-beam/src/shm.rs -> 7 (exit 0)",
  "stat crates/cgap-beam/src/shm.rs -> Oct 31 2026, 44374 B (exit 0)",
  "grep -n DATA_LOSS atomic_commit.rs + court file (exit 0)",
  "sed -n '1060,1100p' atomic_commit.rs (exit 0)",
  "ls lanes/ | grep s04 -> lane-rc06 receipt (exit 0)",
  "grep lane-rc06 standing -> ALIVE, exit 0, 6 passed (exit 0)"],
 "courts_run":["none executable — prose errata lane; court legs verified by reading the court file + the owning rc06 receipt's command+exit"],
 "standing":"ALIVE",
 "divergences":["S-04b kernel arm was already landed on disk at errata time (not in flight) — fix-14 A2 updated to LANDED citing lane-rc06 receipt rather than adding the in-flight note",
  "receipt file added per system receipt law (lanes/ receipt mandatory); dispatch named only the three content files"]}
```
