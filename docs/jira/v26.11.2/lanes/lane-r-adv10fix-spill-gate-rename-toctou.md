# Lane R-ADV10FIX — spill-gate rename-TOCTOU repair (ADV-10 + ADV-17)

Date: 2026-11-02 | Owner files: `crates/cgap-ffi/src/sovereign.rs` (spill_gate region),
`crates/cgap-ffi/tests/fa9_commit_gating.rs`. No git commands run. Shared-tree
mutation only via /tmp swap + sha256 fail-closed restore + mtime bump.

## W1 Subject + identity

- `crates/cgap-ffi/src/sovereign.rs` `mod spill_gate` — ADV-10 finding:
  `docs/jira/v26.11.2/redteam/corrosion/adv-10-fmea-risk-laundering.md` (B verdict,
  SPOILER-CONFIRMED rename-mid-lease escape, exit-7 shape).
- ADV-17 (corordinator scope extension): absent→symlink key-flip,
  `docs/jira/v26.11.2/redteam/corrosion/adv-17-unblock-semantic-collision.md`.
- PFMEA row to re-status: `docs/jira/v26.11.2/dflss/04-pfmea-risk-matrix.md` FM-01
  (NOT edited — not mine; re-status request recorded in the terminus).

## W2 Mechanism (as landed)

1. **(dev, ino) identity keying** (`spill_gate::identity_key`): when
   `fs::metadata(path)` resolves, the registry keys on FNV-1a over
   `"I" + dev(LE) + ino(LE)` — stable across `rename dir -> dir2` (ADV-10
   closed for the whole lease post-grant). Falls back to the existing
   canonical parent+name text key only for the first-claim-while-absent
   case (preserves FIX-01's alias guarantees; `fa9_alias_race_denied`
   stays green 7/7 with it).
2. **Grant-time verify** (`spill_gate::verify`, called in the lease-grant
   arm immediately BEFORE `File::create`): re-derives th+ih and denies
   typed — closes the claim→grant world-change window.
3. **Rebind** (`spill_gate::rebind`, same grant arm, after create):
   swaps the thread's text-key entry for the identity key in the SAME
   synchronous commit call — no stale dual-entry accumulation (witnessed
   by `fa9_rebind_no_stale_ownership`).
4. **ADV-17 symlink refusal** (`spill_gate::path_is_symlink`, enforced in
   claim AND verify): a symlink at the spill path's FINAL component is
   denied typed before any IO (a grant would follow the link and
   O_TRUNC-create the target). Symlinked PARENT components stay legal —
   the FIX-01 alias leg is unaffected.
5. Residual (documented in code, not hidden): verify→create intra-syscall
   swap is kernel-serialization-bound; closing it needs
   O_NOFOLLOW|O_EXCL creation semantics (separate lane). The 64-bit
   FNV-1a collision bound is unchanged.

## W3 Commands + exits (this session)

| command (CARGO_TARGET_DIR=/tmp/radv10fix-target) | exit | evidence |
|---|---|---|
| `cargo test -p cgap-ffi --test fa9_commit_gating` (post-fix) | 0 | 7 passed; 0 failed (1.04s) |
| `cargo test ... fa9_rename_race_denied` under MUTANT-ADV10 (`identity_key` → `return None`) | 101 | leg FAILED — racer ADMITTED under text keying |
| `cargo test ... fa9_symlink_keyflip_denied` under MUTANT-ADV17 (`.is_ok_and(\|_m\| false)` on `path_is_symlink`) | 101 | leg FAILED — owner ADMITTED, got 0x0 |
| `cargo test -p cgap-ffi` (full suite, fence) | 0 | 52+4+4+4+7+2+5+5+0 = 83 tests, 0 failed (103.3s) |
| `cargo build --workspace` (fence) | 0 | Finished dev profile |
| restore sha256 | — | `4dd08b279acefe916ebb62c52ab47fedb9e3c0edb0e92eaaba6941296bfa312a` byte-exact, mtime bumped before each post-restore build |

Mutation swaps were `cp` into the shared tree, `touch` (mtime-bump law),
leg run, restore + sha256 verify + `touch` — never left a mutated byte on
disk at rest. One interim compile-abort (mutant draft with duplicate
`use`) was NOT counted as a flip; the flip witness is the exit-101 run of
the compiling mutant.

## W4 Court grid (fa9_commit_gating.rs, 7 legs)

1. `fa9_gate_register_pin` — 0x4347_0006 exact pin (pre-existing, green).
2. `fa9_race_grid_one_writer_no_truncation` — pre-existing, green.
3. `fa9_release_after_owner_thread_exit` — pre-existing, green.
4. `fa9_alias_race_denied` — FIX-01 alias leg, PRESERVED green (the
   alias guarantee survived the re-key).
5. `fa9_rename_race_denied` — NEW (ADV-10): owner holds `lease/x`;
   test renames `lease -> renamed` mid-lease; racer attacks
   `renamed/x` — all DENIED, spill prefix-preserved via the NEW path.
   Flip: MUTANT-ADV10 exit 101.
6. `fa9_symlink_keyflip_denied` — NEW (ADV-17): dangling `x -> y`
   planted BEFORE y exists; owner attempt on `x` DENIED typed; y is
   NEVER created through the link. Flip: MUTANT-ADV17 exit 101.
   Design note: the present-target variant (y already exists at owner
   claim) is independently denied by the (dev,ino) identity key; this
   leg makes the refusal the SOLE defense (dangling at claim time) so
   the leg pins exactly one mechanism.
7. `fa9_rebind_no_stale_ownership` — NEW (coordinator corollary):
   grant+rebind → steady-state re-entry OK → after owner-thread EXIT a
   fresh thread claims OK — no stale dual-entry deadlock shape.

## W5 Fences (must-not-regress, run)

- FIX-01 alias guarantee (`fa9_alias_race_denied`) — green 7/7 run.
- FA9 denial register pin + race grid + release leg — green (same run).
- Full cgap-ffi suite (all other courts over the crate, incl. the
  stats_block and commit-gating surfaces) — exit 0, 83 tests.
- Root workspace build — exit 0 (only pre-existing bench manifest
  warnings).

## W6 Standing

ALIVE for everything claimed above: every verdict cites a command+exit
run THIS session against the on-disk subject
(sha256 `4dd08b279a…bfa312a` at rest). Divergences from dispatch,
recorded:
1. The dispatch's "re-bind at first flush" was implemented as rebind
   INSIDE the grant arm of the same synchronous commit call (earliest
   sound point; strictly tighter than first-flush).
2. Dispatch named only the rename leg; coordinator's mid-flight scope
   extension added legs 6-7 and the refusal mechanism (4) — both
   variants' flips witnessed.
3. Divergence from a literal reading of ADV-17's repair menu: full-path
   canonicalize at grant time ALONE does not close the dangling-link
   case (canonicalize fails on a dangling link); the typed final-
   component symlink refusal is the mechanism that does.

## W7 Coordinator actions requested (not executed — not my files)

- `docs/jira/v26.11.2/dflss/04-pfmea-risk-matrix.md` FM-01: re-status
  CLOSED-RECEIPTED-WITH-RESIDUAL → residual CLOSED (this receipt);
  rename/symlink/dangling residuals now mechanically enforced, DET may
  drop accordingly.
- ADV-10 receipt (adv-10-fmea-risk-laundering.md) W7 falsifier is now
  satisfiable: verdict B's registry keys collide under the rename
  attack (leg 5 DENIED, exit-0 court).
