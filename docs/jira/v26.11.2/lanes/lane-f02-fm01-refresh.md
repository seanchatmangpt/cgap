# Lane F-02 — FM-01 residual row refresh (post-adv10fix)

Date: 2026-11-02 | Repo: /Users/sac/cgap (canonical checkout, main).
Contract: ONE owned file —
`docs/jira/v26.11.2/dflss/04-pfmea-risk-matrix.md` (FM-01 row +
STATUS-register entry). No git commands run.

## W1 Subject + identity

- Owned file edited in place; before-state re-read from disk first
  (FM-01 row at :86-107, register at :247, divergences at :257+).
- Repair receipt consumed:
  `lanes/lane-r-adv10fix-spill-gate-rename-toctou.md` (landed;
  (dev,ino) keying + rename/symlink courts, flips witnessed).
- Finding: `redteam/corrosion/adv-10-fmea-risk-laundering.md`
  (verdict B SPOILER-CONFIRMED; its repair-register note is now
  satisfied by the adv10fix receipt + this re-status).

## W2 What changed

1. FM-01 STATUS: CLOSED-RECEIPTED (unchanged text) now cites TWO
   repair layers: fix01 (static-alias re-key) + adv10fix
   ((dev,ino) identity keying, grant-time verify, synchronous
   rebind, final-component symlink refusal).
2. Residuals kept loud (scope line in the row): O_NOFOLLOW|O_EXCL
   intra-syscall swap (kernel-serialization-bound, separate lane,
   adv10fix W2 residual 5) and cross-FS case-variant aliasing
   UNVERIFIED (ADV-10 verdict C). CLOSED-RECEIPTED here means the
   rename/symlink keying variants are court-closed with witnessed
   flips, NOT that the FS-portability frontier is done.
3. Code cites refreshed from disk: `mod spill_gate` :889,
   `canonical_key` :931, `identity_key` :976 (consumed
   :1024/:1056), `path_is_symlink` :995, `verify` :1049,
   `rebind` :1071; court legs alias :275, rename :400, symlink
   :529, rebind :562. red-13's `:863-892` region now `:889+`.
4. STATUS-register FM-01 row: fix01 + adv10fix referents, scope
   line pointer.
5. Scope supplement folded (coordinator mid-flight, E-09 audit):
   (a) divergence bullet "FM-08 remains REGISTERED-DESIGN" fixed
   to REPAIRED-RECEIPTED citing adv12fix + F-03 receipts; (b)
   FM-08 code cite refreshed `ingress_ring.rs:210-212` (was stale
   :101-107); (c) R6 updated — `lanes/lane-dx-06-fixture-races.md`
   now on disk: classifier court + per-process fixes, R6 re-scored
   DET 7->2 / RPN 336->48, rank 1->4, top-3 recomposed.

## W3 Commands + exits (this session)

| command | exit | evidence |
|---|---|---|
| `CARGO_TARGET_DIR=/tmp/f02-target cargo test -p cgap-ffi --test fa9_commit_gating` | 0 | 7 passed; 0 failed (0.84s) — all three new legs green: fa9_rename_race_denied, fa9_symlink_keyflip_denied, fa9_rebind_no_stale_ownership |
| line-length + structure sweep over edited regions (awk) | 0 | new prose lines <= 100 chars (register table rows follow the file's pre-existing long-row house style) |
| on-disk cite verification (grep/sed) | 0 | every line cite in the refreshed row verified against `sovereign.rs` / `fa9_commit_gating.rs` / `ingress_ring.rs` this session |

Mutation battery NOT re-run by F-02: the adv10fix flips (MUTANT-
ADV10 exit 101, MUTANT-ADV17 exit 101) are cited as-of-receipt;
re-running them was not in the F-02 contract. Court green run IS
fresh this session.

## W4 Fences (must-not-regress, stated)

- FM-02/FM-07/FM-08 rows and their cites: FM-08 cite was touched
  (refresh only, verified); FM-02/FM-07 untouched.
- R6/R8/R10 top-3: recomposed; R6 rank change is receipt-backed
  (dx-06 exits), flagged as-of-receipt, not fresh-run.
- Anti-vacuity paragraph of the matrix: untouched, still holds
  (row deletion => RPN rises; ranking falsifiable by receipt
  absence).

## W5 Divergences (dispatch vs disk)

1. Dispatch described FM-01 as reading CLOSED-RECEIPTED-WITH-
   RESIDUAL; disk truth at lane start was plain CLOSED-RECEIPTED
   (the WITH-RESIDUAL intermediate was never landed). This lane
   landed the final state directly with the residual scope line
   in the row body; recorded in the file's divergences block.
2. One interim self-inflicted corruption during editing (a
   RESOLVED-bullet opening line was consumed by an adjacent edit)
   was caught by re-read and repaired in the same session; final
   on-disk structure verified (divergences block reads: Dispatch
   bullet -> F-02 refresh bullet -> RESOLVED bullet -> cgap-git
   transient bullet, all well-formed).
3. Coordinator supplement added `canonical_key` :931 and keying
   :944-952; disk verification puts `canonical_key` at :931 with
   text-key consumption at :1023/:1055/:1075 and identity_key
   consumption at :1024/:1056 — the row cites the verified
   positions.

## W6 Standing

ALIVE for the court run (exit 0, 7 passed, fresh this session,
CARGO_TARGET_DIR=/tmp/f02-target). All doc cite refreshes are
ALIVE (grep/sed against disk this session). R6 re-score and the
adv10fix mutant flips are as-of-receipt evidence, explicitly
NOT fresh-run by this lane. Build-root lease
/tmp/f02-target deleted at lane end.

## W7 Coordinator actions requested (none — terminus lane)

None; FM-01 surface is closed. If a later lane re-keys the spill
gate again, refresh this row's cites per the divergence pattern
already recorded.
