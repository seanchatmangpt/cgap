# Lane R-14 / FIX-17 — breach-regression court (tollgate)

Date: 2026-11-02 · Repo: /Users/sac/cgap (main) · Standing: **ALIVE**
Owner file (ONLY): `crates/cgap-git/tests/breach_regression_court.rs`
Read-first receipts: FIX-01 (fa9 alias rekey), FIX-02 (bn3 SRAM gate),
FIX-04 (andon quota), FIX-05 (pack_demux poison).

## W1 Subject + identity

- `crates/cgap-git/tests/breach_regression_court.rs` — final on-disk sha256
  `ce83c746570a1ea35ce8dcd8f0b1d8283ac462b0ab16e48441b8bc24382ee285`
- Shared-tree collaborators (read-only to this lane, sha256 at receipt
  time, post-restore verification): pack_demux.rs
  `8989d3978cbc4d68544c114f3acf3f50fa40cb8752d5d439e4e75eca1bdf9d47`,
  ingress_ring.rs
  `35b4c7bda59bfa44fa5dc8f34448048c561bedffa977f48c266a0c147f38b282`,
  noalloc.rs
  `e039d3ab88b2d1118d38290a6edcece3082a8a1a5d433cda3d11e486afceb483`,
  sovereign.rs
  `4dd08b279acefe916ebb62c52ab47fedb9e3c0edb0e92eaaba6941296bfa312a`,
  verify_cscb_sram.py
  `c95fe63250dc2282a12523f9556f6efcf178c0ed65f159683b87806add2c886c`

## W2 Diff (μ)

- New court `crates/cgap-git/tests/breach_regression_court.rs` (312
  lines), 5 legs encoding the four hard breaches + the gate:
  - LEG 1 (FIX-01/RED-13): alias lease race — typed subprocess running
    FIX-01's `fa9_alias_race_denied` (cargo, target lease
    /tmp/r14-leg1-target so the parent's target lock is never contended;
    asserts exit 0 + the named test ok). Pure-Rust in-process is
    impossible from cgap-git (spill gate lives in cgap-ffi; Cargo.toml
    off-limits).
  - LEG 2 (FIX-02/RC-03/RED-12): bn3 full-capacity init refuses TYPED
    `CapacityExceeded` with exact red-12 numbers (2,162,696 / 4,358,288 /
    6,520,984 vs 131,072), big-stack thread.
  - LEG 3 (FIX-05/RED-09): poison → `TrailingBytes(4)` typed, trailer
    intact, clean-pack positive control, surplus lie → TrailingBytes(26),
    objects_seen pinned.
  - LEG 4 (FIX-04/RED-14 + ADV-12): 1M FrameTooLarge flood → tripped,
    total=1,000,000; real admit+drain byte-exact while sticky; 1024 clean
    admissions clear; lifetime counter preserved.
  - LEG 5 (FIX-02/RED-12): SRAM gate subprocess — `--subject bn3` exit 1
    + `BN3_ENCLAVE_REFUSED`, and the gate's own `--mutate bn3-pin` court
    exit 0 with DIVERGED (in-test anti-vacuity channel).
- No git commands; no Process.sleep (bounded subprocess exit codes, no
  polling); Chicago (real threads/files/subprocesses/byte-built packs).

## W3 Commands + exits (this session, exact subject)

| command | exit | evidence |
|---|---|---|
| `cargo test -p cgap-git --test breach_regression_court` (run_checked, CARGO_TARGET_DIR=/tmp/r14-target) | 101 | FIRST RUN: breach4 failed (stale post-admit `!tripped` assert vs the ADV-12 level-triggered contract that landed mid-lane); breach1 failed (cgap-ffi mid-edit compile break: missing OsStrExt import at sovereign.rs:978/995) |
| same command after ADV-12 contract update + SLA unblock (run4 final) | 0 | 5 passed / 0 failed |
| sandbox baseline, same command in /tmp/r14-sbox mini-workspace (CARGO_TARGET_DIR=/tmp/r14-sbox-target) | 0 | 5 passed / 0 failed |
| M1 (pack_demux `Phase::Done => {}` revert) → breach3 | 101 FLIP | "VACUOUS: post-trailer poison silently absorbed"; restore sha256-exact `8989d397…bdf9d47` + touch |
| M2 (note_refusal escalation deleted) → breach4 | 101 FLIP | "VACUOUS: 1M flood did not escalate the quota"; restore `35b4c7bd…f38b282` + touch |
| M3 (bn3 init_within typed refusal deleted) → breach2 | 101 FLIP | init_enclave admitted; restore `e039d3ab…fceb483` + touch |
| M4 (canonical_key raw + identity_key None) → breach1 | 101 FLIP | LEG1 FLIP/BREACH: alias race admitted, exit 101; restore `4dd08b27…bfa312a` + touch |
| M5 (gate SRAM_BUDGET neutralized to 100000000) → breach5 | 101 FLIP | gate admits bn3 exit 0 → leg5 typed exit-1 assert fails; restore `c95fe632…add2c886c` + touch |
| fence battery: `cargo test -p cgap-git --test andon_liveness_court --test pack_demux_poison_court --test pack_demux_court --test ingress_ring_court --test egress_ring_court -p cgap-bn3 --test capacity_bound_court -p cgap-ffi --test fa9_commit_gating` | 0 | 3 (capacity) + 7 (fa9) + 5 (andon, ADV-12-updated) + 5 (egress) + 7 (ingress) + 11+1ign (pack_demux) + 3 (poison) passed |

## W4 Court legs + anti-vacuity

All five legs name their mutation and all five FLIPPED in the /tmp
sandbox this session (M1–M5 above; every restore sha256-exact + mtime
bump; the shared tree was never mutated by any mutation — shared-tree
target hashes re-read post-run and byte-identical to the landed state).

## W5 Divergences (disclosed)

- D1 (contract move mid-lane, ADV-12 repair lane): ingress
  RefusalQuota's `tripped` became LEVEL-TRIGGERED mid-lane (clears only
  after `CLEAN_ADMISSIONS_TO_CLEAR`=1024 consecutive clean admissions;
  `acknowledge()` added). My first-cut leg4 asserted the old one-admit
  clears contract and failed deterministically; updated to the ADV-12
  contract. FIX-04's own receipt (§4 MUTANT tables) describes the
  one-admit-clears contract and its §6.2 unblock added `extern crate
  alloc;` to quota_alloc_purity_court.rs — both now superseded by ADV-12
  semantics; FIX-04's andon court itself was updated by that lane (5
  tests now pass, was 3). Noted as errata candidate against
  `lane-fix04-andon-quota.md` (§2 Diff bullet 3 and §4) — not edited by
  me (ERRATA, never rewrite).
  Andon anti-vacuity note: under ADV-12, the M2 escalation-deletion
  mutation still flips breach4 (witnessed above).
- D2 (disclose-in-both-directions SLA unblock): cgap-ffi's in-flight
  ADV-10 edit left sovereign.rs non-compiling (E0599 `OsStr::from_bytes`
  without the OsStrExt import at :978/:995) — this froze my leg1 and the
  fa9 fence. Minimal unblock: function-local
  `use std::os::unix::ffi::OsStrExt;` added in `identity_key` and
  `path_is_symlink`, commented as R-14 SLA unblock. The ADV-10/ADV-17
  owner lane superseded/expanded the diff on landing (an intermediate
  duplicate-MetadataExt E0252 state was their transient, not mine).
  Owning lane's receipt should cite this unblock; mine cites theirs.
- D3 (first-run breach4/leg1 failures were NOT flake): run1 failed for
  the two reasons in W3; both were deterministic contract/compile states,
  not flaky courts.
- D4 (subprocess legs): leg1/leg5 assert exact exit codes + typed output
  markers of real collaborators (cargo/python3 gates); sandbox copies
  inherit the court unchanged, so sandbox mutation runs exercise the
  sandboxed collaborators (repo_root derives from CARGO_MANIFEST_DIR).
- D5 (M4 scope): reverting `canonical_key` ALONE does not reopen the
  alias breach anymore — the landed ADV-10 inode-identity keying already
  denies the racer via (dev, ino). The flip required the full pre-fix
  shape (raw-text key + identity_key → None). Consequence: LEG 1 today
  is protected by TWO layers; its FIX-01-era mutation (canonical_key
  only) is no longer sufficient — recorded as a fence-relevant finding
  for the ADV-10 lane's receipt.
- D6: sandbox assembly needed three extra crates (cgap-storage, audit,
  cgap-branchless) beyond the dispatch's per-crate copies; mini-workspace
  manifest at /tmp/r14-sbox/Cargo.toml inlines the workspace.package keys.
- D7: first-run compile-aborts observed twice mid-lane (cgap-git ref_store
  mid-edit: 11→4→3 errors; cgap-ffi sovereign mid-edit) — compile-abort
  is not a failure count (census law); every court/fence exit above was
  captured against a compiling tree.

## W6 Standing

ALIVE for the R-14 surface: court file on disk, 5/5 legs green on the
shared tree (exit 0, run4), 5/5 mutation flips witnessed in the /tmp
sandbox with fail-closed restores, fence battery exit 0 (41 tests across
7 fence binaries). Shared tree untouched except the disclosed D2 unblock
lines in sovereign.rs (owner lane supersedes on landing).

## W7 Falsifier

Re-run any of M1–M5 in a fresh /tmp sandbox → the named leg must flip
(exit 101); if any leg stays green under its mutation, this receipt is
FAIL. Also: if the SRAM gate or fa9 court exit codes change shape, legs
5/1 are the typed tripwires.
