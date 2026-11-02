# F2-7 doc-standing — lane notes (v26.11.1, 2026-11-01)

- **Lane**: F2 item 7 — DOC-STANDING (lane-runner), repo `/Users/sac/cgap`.
- **Task**: reconcile five lane docs' status lines with the witnessed F1
  finisher receipts; status lines ONLY, nothing else.
- **Basis**: receipts read in full this session:
  `waves/a03-finish-receipt.md`, `waves/a07-finish-receipt.md`,
  `waves/a08-finish-receipt.md`, `waves/a12-finish-receipt.md`,
  `waves/a13-finish-receipt.md`. Each edited status line carries a one-line
  cite of its receipt with the witnessed command+exit.

## Files written

1. `docs/jira/v26.11.1/lanes/A03-git-merkle.md`
2. `docs/jira/v26.11.1/lanes/A07-bosym-slab.md`
3. `docs/jira/v26.11.1/lanes/A08-shacl-bitmask.md`
4. `docs/jira/v26.11.1/lanes/A12-simd-unifier.md`
5. `docs/jira/v26.11.1/lanes/A13-bn3-builtins.md`
6. `docs/jira/v26.11.1/waves/f2-7-doc-standing-notes.md` (this file)

## Diffs (per-file `git diff --stat`, run this session)

```
A03-git-merkle.md    | 10 +++++-----   (2 status bullets, doc §10)
A07-bosym-slab.md    | 10 +++++++---   (header status + §Standing line)
A08-shacl-bitmask.md | 12 ++++++++----  (header Status + §11 Standing)
A12-simd-unifier.md  |  7 +++++--      (§Status section only)
A13-bn3-builtins.md  |  4 +++-        (header Status bullet only)
5 files changed, 28 insertions(+), 15 deletions(-)
```

Full diffs were read back line-by-line above the stat; every hunk is a
status-line replacement, zero spec/DoD/ADR text touched.

## Standing recorded per doc

- A03 §10: `UNVERIFIED / no code landed` -> **ALIVE** (receipt: 88/0
  `cargo test -p cgap-git`, three sandbox flips, fail-closed restores).
- A07 header + Standing: `PROPOSED / UNVERIFIED / nothing executed` ->
  **ALIVE** (receipt: `cargo test -p cgap-bosym` 14/0, zero-string gate
  1.4.1 green, Pyright 0/0/0, sandbox flip with fail-closed restore).
- A08 header + §11: `PARTIAL_ALIVE / NOT BUILT / UNVERIFIED` -> **ALIVE**
  (receipt: `cargo test -p cgap-bont` 35/0, T5 asm witness, M1c/M3c flips).
- A12 §Status: `PROPOSED / TARGET — nothing measured` -> **ALIVE** with the
  perf gate kept honestly at TARGET/BLOCKED (receipt D-A12-2: no perf
  counter on this host; no perf number measured).
- A13 header Status: `PROPOSED / does not exist yet` -> **ALIVE**
  (receipt: crate builds, 102/102, builtins 26/26, M-GT flip exit 101).

## Stale-but-untouched (out of "status lines ONLY" scope; doc-owner items)

- A07 line 58 `**Status**: PROPOSED` — that is ADR-002's register status
  (design-authority question), not the lane's implementation standing; no
  receipt I read witnesses ADR-002 acceptance into `22-adr-register.md`.
  Left untouched.
- A07 lines 10, 87, 140, 429 — "PROPOSED" as spec/DFD/future-work text,
  not status declarations.
- A08 line 5 `(crate PROPOSED)` parenthetical, line 66 ADR-008 `Status:
  PROPOSED`, line 123 DFD label, line 258 DoD text, §10 Risks lines
  334-335 (`lane stands PARTIAL_ALIVE (design), not ALIVE`) — spec/ADR/
  DoD/risk text; the receipt's divergence note names only header + §11,
  so §10 was left for the doc owner.
- A12 line 6 (`Crate: cgap-bn3 is PROPOSED in this lane — it does not
  exist yet`) — charter preamble text, not the status section; lines
  357-359 (DoD item 8) are DoD table text, explicitly out of scope.
- A13 line 5 `(PROPOSED)` parenthetical and §2.1 lines 67-75 (the
  zero-indirection claim's `Status: UNVERIFIED` grading) — the §2.1 court
  HAS since run (receipt §2: 0 blr / 0 br xN / 0 brk), but flipping that
  in-spec status paragraph means rewriting spec-argument text, which the
  dispatch excludes. Left untouched, recorded here.
- A03 §10 pre-edit "Sources read this session" bullet directly above the
  status bullets was left verbatim (it describes the design session, not
  standing).

## Verification

- Pre-edit: `git status --porcelain` + `git diff --stat` on the five
  docs -> empty (baseline clean; every post-edit diff line is mine).
- Post-edit: full `git diff` read per file (two runs, one per doc
  group) — only status-line hunks; `git diff --stat` cited above;
  exit 0 each run.
- No git write commands used; no files outside the lease touched.

