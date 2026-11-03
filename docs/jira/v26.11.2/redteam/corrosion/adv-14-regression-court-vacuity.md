# ADV-14 / F-09 — breach-regression court vacuity attack (corrosion)

Date: 2026-11-02 · Repo: /Users/sac/cgap (main) · Standing: **ALIVE**
Subject: `crates/cgap-git/tests/breach_regression_court.rs` (sha256
`ce83c746…2ee285`, matches R-14 receipt W1) over the landed fix set
FIX-01/02/04/05 + ADV-10 inode keying + ADV-12 level-triggered quota.
Method: every mutation lived in /tmp/adv14-sbox (mini-workspace: 14
member crates + inlined workspace.package keys; the full-repo copy was
REFUSED by the topology guard — divergence D0 below), mutated one file
at a time, sha256 fail-closed restore + mtime bump each leg; shared
tree hashes re-read post-run and byte-identical to the R-14 landed
state (all five, exact W1 values). CARGO_TARGET_DIR=/tmp/adv14-target
(deleted at lane end), leg1 subprocess lease /tmp/r14-leg1-target
(inherited from R-14; content-fingerprinted rebuilds observed per
mutation).

## Control

- Clean sandbox, pre-mutation: `cargo test -p cgap-git --test
  breach_regression_court` → exit 0, 5 passed / 0 failed.
- Post-restore (all five files restored + touched), full court: exit 0,
  5 passed / 0 failed. No vacuous restore: every mutated state produced
  a different outcome before its restore was accepted (flip-or-fail
  closed on both sides).

## Per-leg verdicts (attack = make the leg PASS pre-fix)

| leg | mutation (sandbox file) | vulnerable-copy exit | verdict |
|---|---|---|---|
| 1 | M-L1-FULL: `canonical_key` → raw `path.to_vec()` AND `identity_key` → `None` (sovereign.rs) | 101 | FLIPPED — not vacuous |
| 1a | M-L1-TEXTONLY: text layer only, inode layer intact | 0 | PASSES — leg1 survives |
| 1b | M-L1-INOONLY: `identity_key` → None only, text layer intact | 0 | PASSES — leg1 survives |
| 2 | M-L2: both `init_within` refusals `if false && …` (noalloc.rs) | 101 | FLIPPED — not vacuous |
| 3 | M-L3: `Phase::Done` arm → `{}` (pack_demux.rs) | 101 | FLIPPED — not vacuous |
| 4 | M-L4: `note_admission` self-clears `tripped` on first admit — pre-ADV-12 edge semantics (ingress_ring.rs) | 101 | FLIPPED — the NEW court catches the OLD semantics |
| 5 | M-L5: `SRAM_BUDGET = 100000000` (verify_cscb_sram.py) | 101 | FLIPPED — not vacuous |

Witnessed panic texts: leg1 "LEG1 FLIP/BREACH: alias race admitted —
exit=Some(101) … RED-13 alias must be DENIED"; leg2 line-182/210
CapacityExceeded asserts; leg3 "VACUOUS: post-trailer poison silently
absorbed … TrailingBytes"; leg4 "ADV-12 contract: one admission must
NOT clear the sticky escalation"; leg5 "gate must refuse bn3 with exit
1 (typed refusal), got Some(0)".

## Leg1 two-layer finding (dispatch question 1)

The full two-layer revert flips (exit 101 — racer ADMITTED at the
`dir/./x` alias, fa9 asserts the RED-13 breach). EACH single-layer
revert still PASSES: the surviving layer independently denies the alias
(the inode layer keys (dev,ino) of the same target file regardless of
path text; the canonical-text layer resolves `dir/./x` to the same
bytes regardless of inode state). Neither layer is decorative against
this vector: leg1 has NO single point of failure, and there is NO
spoiler — the weaker-layer-false-confidence condition in the dispatch
is NOT met. R-14's D5 is confirmed and extended: canonical_key-only
revert does not flip (D5), AND identity_key-only revert also does not
flip (new this lane).

Note: leg1's in-court doc comment (breach_regression_court.rs:53-57)
still names canonical_key-only as "the" mutation and claims the flip —
that claim is stale per R-14 D5 and now doubly-confirmed here; owner
(R-14/FIX-17) may update the comment; not edited by this lane (not my
owned file).

## Leg4 old-semantics attack (dispatch question 4)

Attacking the OLD semantics against the NEW court is possible and
FAILS as required: the self-clear mutation (first admission erases
`tripped`) is caught by the level-triggered assert
(breach_regression_court.rs:312). The court is not merely
edge-triggered-immune by accident — the sticky assert is the load-bearing
tripwire and it fired (exit 101).

## Restores (sha256-exact, mtime bumped after each)

| file | restored sha256 |
|---|---|
| crates/cgap-ffi/src/sovereign.rs | `4dd08b27…bfa312a` |
| crates/cgap-bn3/src/noalloc.rs | `e039d3ab…fceb483` |
| crates/cgap-git/src/pack_demux.rs | `8989d397…bdf9d47` |
| crates/cgap-git/src/ingress_ring.rs | `35b4c7bd…f38b282` |
| scripts/verify_cscb_sram.py | `c95fe632…add2c886c` |

Shared tree: never mutated; all five hash-exact post-run.

## Commands + exits (this session, exact subject)

- clean control (pre-mutation): exit 0, 5 passed
- M-L1-FULL breach1: exit 101 (flipped); M-L1-TEXTONLY: 0;
  M-L1-INOONLY: 0 (second attempt; first heredoc was corrupted —
  syntax error before patch, so that run was clean-tree, discarded,
  re-run properly)
- M-L2 breach2: 101; M-L3 breach3: 101; M-L4 breach4: 101;
  M-L5 breach5: 101
- post-restore full court: exit 0, 5 passed
- fence note: no shared-tree diff exists from this lane, so no fence
  battery is required; the R-14 fence battery (41 tests, exit 0) stands
  as landed.

## Divergences

- D0: full-repo /tmp copy REFUSED by the topology guard hook; the
  sandbox is a 14-member mini-workspace (source-only rsync of crates/
  minus cli/csct/target + scripts/ontologies/LICENSE/include + inlined
  workspace keys). Functionally equivalent for this court: control exit
  0 pre-mutation, all five flips witnessed.
- D1: leg1 doc-comment mutation claim is stale (see leg1 finding);
  errata against R-14 W2/W4 wording is an owner-lane item.

## Standing

ALIVE: 5/5 legs flip under their full mutation (all exit 101), 2/2
leg1 single-layer partial reverts correctly do NOT flip (exit 0), clean
controls exit 0 pre- and post-restore, shared tree byte-identical.
Falsifier: re-run any M-L* in a fresh sandbox — the named leg must
flip; if leg1 flips under a single-layer revert, this finding is wrong.
