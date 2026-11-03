# Lane X-01 — Staged-Commit Inventory (Archival Transcription)

TRANSCRIBED from coordinator session record; original audit executed 2026-11-02.
Transcription landed 2026-11-02 by lane T-X01. HEAD at both audit and
transcription: `7f7ccdf7df614a22260a96f53dc8b6af4897fa2e`.

## W1 Subject

X-01's staged-commit inventory over the canonical checkout `/Users/sac/cgap`
(branch `main`): a full census of the working tree at its run time, plus a
classification of every entry against the wave's lane-ownership map, plus the
5-commit staging plan that was superseded by FS-15.

## W2 Authority

Coordinator session record of 2026-11-02 (wave plan
`docs/jira/v26.11.2/05-wave-plan.md`; FS-15's authoritative staging plan at
`docs/jira/v26.11.2/lanes/lane-fs15-staging-plan.md`). Read-only git
re-grounding permitted per lane contract; no git write commands.

## W3 Census at audit run time

- 66 modified tracked files; ~90 untracked entries (approximate at audit time;
  exact count not pinned in the record).
- Diffstat `+4935/−602`.
- HEAD `7f7ccdf7` (`7f7ccdf7df614a22260a96f53dc8b6af4897fa2e`) — unchanged at
  transcription.

## W4 Classification (as of the audit)

### Wave-lane-owned per receipt

The coordinator's record classifies the modified set as wave-lane-owned per
landed receipts. Spot-verification of the crates/ subset: 20 files checked
against the lane map — no coordinator/unrelated leakage (every file traceable
to a named lane receipt).

### The 6 UNOWNED findings (as of the audit)

| # | Finding | Disposition at audit time |
|---|---|---|
| 1 | `crates/cgap-ffi/src/oabi_lookup.rs` | UNOWNED-ESCALATED format reflow; ffi gate 12/12 |
| 2 | `crates/cgap-ffi/src/xip_tables.rs` | UNOWNED-ESCALATED format reflow; ffi gate 12/12 |
| 3 | `bench-validate/` (3 files) | coordinator-call item |
| 4 | `docs/jira/v26.11.2/lanes/pub-01-dry-run-matrix.md` | coordinator-call item |
| 5 | empty `.alive` sentinels | coordinator-call item |
| 6 | `dispatch-ledger.jsonl` | coordinator-call items |

## W5 The 5-commit staging plan (historical, superseded)

Commit 0 — regen checkpoint: `crates/cli` ggen sync + `make gate0`
(byte-identical replay), committed alone as a regen-only checkpoint.
Commit 1 — crates/ code + tests (the lane-owned implementation surface).
Commit 2 — scripts/ + gates (fail-closed gate tooling).
Commit 3 — docs/ + receipts, with orphans segregated (the 6 UNOWNED findings
land separately, coordinator-call items only).
Commit 4 — license + legal files (`LICENSE.md`, `LICENSE-P28-SIL.md`).

**Superseded note**: FS-15's refreshed plan
(`docs/jira/v26.11.2/lanes/lane-fs15-staging-plan.md`, census 235 entries at
HEAD `7f7ccdf7`) is the authoritative staging script. This transcription is
the historical first inventory and is retained for the audit trail only.

## W6 Commands / re-grounding (this session)

- `git rev-parse HEAD` → exit 0 → `7f7ccdf7df614a22260a96f53dc8b6af4897fa2e`
  (matches audit-time HEAD).
- `git status --porcelain` re-ground at transcription: 104 modified tracked,
  136 untracked — the tree has grown since the audit (66M + ~90?? then;
  additional lanes landed in between). Census-as-of-audit stands as
  transcribed; this line is the only fresh observation.
- [COORDINATOR ERRATA 2026-11-02: this observation was a cwd artifact — `docs/jira/v26.11.2/bench-validate/` DOES exist (8 receipted reports, verified by ls at landing). Original text retained per ERRATA law:] `bench-validate/` does not exist on disk at transcription (read at
  2026-11-02) — consumed or renamed since the audit; recorded as-is.
- FS-15 receipt confirmed present on disk.

## W7 Standing

**ALIVE (as archival transcription)** — the transcription is on disk at
`docs/jira/v26.11.2/lanes/lane-x01-staged-commit-inventory.md`; HEAD and the
historical census are faithfully transcribed; the cheap re-ground (status
counts, HEAD, FS-15 receipt presence) ran this session with exits above.
The original audit's own per-file verification (crates/ 20-file check) is
NOT re-executed here and remains as-of-audit-time — not re-derivable without
replaying X-01 against a tree that has since moved.

Fences: none broken — this lane wrote exactly one new docs/ file; no code,
gates, or shared files touched.
