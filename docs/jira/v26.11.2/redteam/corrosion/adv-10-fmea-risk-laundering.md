# ADV-10 — PFMEA residual risk laundering (FM-01 canonical_key corrosion)

Date: 2026-11-02 | Lane ADV-10 (read-only over crates/; /tmp probe only)
Contract file: this receipt is the ONLY repo write.

## W1 Subject + identity

- `crates/cgap-ffi/src/sovereign.rs` `mod spill_gate` (private module;
  `canonical_key` at :898, consumed at :945 `fnv1a(&canonical_key(path))`).
- Probe crate `/tmp/adv10-probe/src/main.rs` carries canonical_key + fnv1a
  extracted VERBATIM from `sovereign.rs:870-935` (module wrapper stripped;
  hash-bytes untouched). spill_gate is not `pub`, so a path-dep consumer
  cannot reach `claim` directly — the verbatim extraction is the faithful
  subject; FS behavior (canonicalize/rename/case) is exercised by real
  std calls against this host's real filesystem.
- PFMEA row under audit: `docs/jira/v26.11.2/dflss/04-pfmea-risk-matrix.md:86-107`
  (FM-01, STATUS CLOSED-RECEIPTED, DET re-score 8->3, RPN 192->72).

## W2 Probe design (4 attack classes)

Each case computes the registry key exactly as `claim` does and compares
keys; equal key = racer DENIED (protected), differing key = racer ADMITTED
(truncation escape).

1. A — symlinked parent (`real/x` vs `link/x`, `link` -> `real`).
2. A2 — pre- vs post-existence key stability (FIX-01's uniform rule).
3. B — rename mid-lease: claim `dir/x`, `rename dir -> dir2`, re-claim
   `dir2/x` while the old lease is held.
4. C — case-insensitive APFS aliasing (`cap/X` vs `cap/x`), plus inode
   confirmation that both texts address the same file.
5. D — nonexistent chain (`/ghost/a` vs `/ghost/./a`) raw-fallback escape.

## W3 Commands + exits (this session)

| command | exit | evidence |
|---|---|---|
| `cd /tmp/adv10-probe && CARGO_TARGET_DIR=/tmp/adv10-target cargo run` | 0 | output below |

```
A symlink-parent-collision real=8560788729585721479 link=8560788729585721479 SAME=true
A2 pre-vs-post-existence SAME=true (must be true)
B rename-mid-lease old=1087395529426552610 owner=1087395529426552610 new=17181722570414078000 COLLIDE_OLD_NEW=false STALE_BLOCK=false
C case-variant upper=13527984850778196721 lower=13527984850778196721 SAME=true host_case_insensitive_stat=true
C inodes equal: Some(1385418787) vs Some(1385418787)
D nonexistent-chain raw-fallback SAME=false (alias escapes key: true)
```

## W4 Verdicts per attack class

### A — symlinked parent: PROTECTED

`canonicalize` resolves the parent symlink; `real/x` and `link/x` hash
identically (8560788729585721479). Symlink/real interleave collapses into
one key. FIX-01's uniform parent+name rule holds across pre/post
existence (A2 SAME=true). No escape.

### B — rename mid-lease: SPOILER-CONFIRMED (escape)

Sequence: owner claims `dir/x` (file exists; key = canonical dir bytes +
x = 1087395529426552610). Attacker `rename dir -> dir2`, then claims
`dir2/x` — key 17181722570414078000, DIFFERENT. The registry sees no
collision, so the racer is ADMITTED while the owner's lease is live, and
both path texts now address the SAME inode (the renamed directory). The
racer's grant `File::create` (O_TRUNC, sovereign.rs:762 call chain)
destroys the owner's drained spill through the new path — the exact
red-13 exit-7 shape via a vector the re-key does not cover. STALE_BLOCK=
false also shows the OLD key does not block the new owner: the registry
is keyed to bytes that no longer resolve, so the stale lease neither
protects nor releases — ownership tracking silently loses the file.
This is a TOCTOU on canonicalize itself: the canonical identity is
snapshot at claim time, not re-validated at grant time.

### C — case-variant aliasing: PROTECTED ON THIS HOST (portability residual)

This host is APFS case-insensitive; `canonicalize` returns the stored
case, so `cap/X` and `cap/x` hash identically (13527984850778196721) and
both stat the same inode (1385418787). The escape channel exists only on
case-INsensitive filesystems whose `canonicalize`/realpath does NOT
normalize case (some Linux overlayfs/vfat setups): there the two texts
address one file but hash differently — racer ADMITTED. UNVERIFIED on
this host by construction (host is case-insensitive); recorded as an FS
portability residual, not a local breach.

### D — nonexistent-chain raw fallback: ESCAPE MATCHES DOCUMENTED BOUND

`/ghost/a` vs `/ghost/./a` hash differently (raw bytes). This is the
residual the code doc names at `sovereign.rs:893-897`: nothing exists to
truncate yet, and the first successful grant re-keys on canonical
bytes. A racer's grant through the alias fails `File::create` anyway
(parent chain absent). Residual stands as documented; no laundering
here, though the bound is asserted, not mechanically enforced.

## W5 Laundering verdict

VERDICT: PARTIAL LAUNDERING CONFIRMED in the FM-01 row.

`04-pfmea-risk-matrix.md:97-107` closes FM-01 as CLOSED-RECEIPTED with
DET 8->3 / RPN 192->72 and cites the `dir/./x` court flip. The row
carries ZERO disclosure of the failure modes this probe witnessed:
(1) rename-mid-lease canonicalize TOCTOU (SPOILER-CONFIRMED on this
host, this session, exit-7-equivalent shape); (2) case-sensitivity FS
portability of the canonicalize identity; (3) the raw-fallback residual
is disclosed in the CODE doc comment (sovereign.rs:893-897) but is NOT
reflected in the PFMEA row's DET/RPN or residual field. Per the laundering
test — does CLOSED-RECEIPTED read as zero-residual? — it does: the row's
only stated residual is the 64-bit FNV-1a collision probability, which
bounds none of the three FS-semantics channels above. The fix01 receipt
(`lanes/lane-fix01-fa9-alias-rekey.md`) likewise contains no rename or
case variant; its court grid (`fa9_alias_race_denied`,
tests/fa9_commit_gating.rs:261) tests a STATIC alias only.

Required repair (registered, not executed — this lane is read-only):
FM-01 STATUS should read CLOSED-RECEIPTED-WITH-RESIDUAL, DET re-scored
for an adversarial FS-semantics grid (rename/symlink-parent/case/
nonexistent-chain), and the rename TOCTOU needs either re-canonicalize
at grant time or an inode-identity (dev,ino) key — both are owner-lane
edits to sovereign.rs + the fa9 court.

## W6 Standing

ALIVE — verdicts above cite the cargo run (exit 0) executed this
session against the verbatim extraction of the on-disk implementation;
the extraction was line-verified against sovereign.rs:870-935 before the
run. Case C's cross-FS claim is UNVERIFIED by construction (single-host
probe); D's non-truncation argument is reasoning from std File::create
semantics, not an executed refusal witness.

## W7 Falsifier

Re-run the probe after any owner-lane change to canonical_key: verdict B
must read COLLIDE_OLD_NEW=true (or the registry must key on (dev,ino))
for the rename escape to be closed; if B still reports differing keys
with the row still marked CLOSED-RECEIPTED zero-residual, this finding
stands and the laundering persists.
