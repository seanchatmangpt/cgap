# lane-dod-07-manifests

Lane DOD-07: PUB-01 exact manifest edits (2026-11-02, repo
/Users/sac/cgap, branch main, no git commands).

## 1. Files written

Priority A — `authors.workspace = true` added to [package] of
9 manifests (inserted after `repository.workspace = true`):

- crates/cgap-bont/Cargo.toml
- crates/cgap-bosym/Cargo.toml
- crates/cgap-bn3/Cargo.toml
- crates/cgap-gate/Cargo.toml
- crates/cgap-crypto/Cargo.toml
- crates/cgap-p28/Cargo.toml
- crates/cgap-storage/Cargo.toml
- crates/cgap-ffi/Cargo.toml
- crates/audit/Cargo.toml (crate name cgap-audit)

Priority A2 — crates/cgap-branchless/Cargo.toml: added
`repository.workspace = true` + `authors.workspace = true`
(DOD-02 had NOT done it — read fresh; branchless was
`publish = false`, literal version; during the lane a
concurrent lane flipped it to `publish = true`; my edit is
the two inheritance lines only).

Priority B — crates/csct/Cargo.toml: description,
license-file `../LICENSE.md`, repository, authors literals
(standalone workspace, no inheritance).
crates/cgap-court/Cargo.toml: description added.

## 2. Commands + exits (this session)

- python bulk edit 9 manifests: exit 0, per-crate assert
  (idempotence guard: `authors.workspace` must be absent
  pre-edit).
- `cargo package -p cgap-bn3 --no-verify --allow-dirty`:
  exit 0, warnings=0, "Packaged 17 files, 484.4KiB".
- `cargo package -p cgap-audit --no-verify --allow-dirty`:
  exit 0, warnings=0, "Packaged 10 files, 158.5KiB".
- `cargo package -p cgap-ffi --no-verify --allow-dirty`:
  exit 101 — PRE-EXISTING blocker, not manifest fields:
  `failed to select a version for the requirement
  cgap-audit = "^26.11.2"; candidate versions found which
  didn't match: 26.11.1` (cgap-arrow same class). This is
  the PUB-04/PUB-08 semver-skew surface, not DOD-07 scope.

## 3. Anti-vacuity witness (honest disposition)

Mutation: /tmp sandbox copy of cgap-bn3, `authors.workspace
= true` REMOVED, inheritance de-inherited (version/edition/
repository literalized, path deps absolutized, license-file
absolutized) so it parses standalone. Result: `cargo
package` exit 0, warnings=0 — packaging is UNCHANGED by
authors removal. Recorded honestly per dispatch: authors is
CONFORMANCE, not a gate; the flip witness is the WARNING
ABSENCE both with and without authors (baseline in-tree
re-run exit 0 warnings=0, mutated exit 0 warnings=0). No
gate exists that cargo refuses on missing authors; a court
that could flip would have to be a lint court, not cargo.

## 4. Divergences / refusals

- REFUSED (mechanical): crates/cli/Cargo.toml edit refused
  by `.claude/hooks/guard-generated.sh` PreToolUse hook —
  the file is a GENERATED ggen artifact ("Do not edit").
  PUB-01's Priority B edit for cli must land in the ggen
  pack template, not the generated Cargo.toml. Standing:
  REFUSED, UNVERIFIED for cli fields.
- Divergence: audit pub-01 census lists cgap-ffi spot-check
  as expected-pass; the true blocker is registry skew
  (cgap-audit/cgap-arrow 26.11.2 not on crates.io).
- Concurrent-lane drift observed: cgap-branchless flipped to
  `publish = true` mid-lane by another writer; root
  Cargo.toml gained branchless as member + workspace.dep.
  My A2 edit compatible with both states.

## 5. Standing

PARTIAL. Priority A: ALIVE (9/9 edited, on-disk verified,
2/3 spot packages exit 0 warnings=0). Priority A2: ALIVE.
Priority B: PARTIAL (csct + cgap-court ALIVE; cli REFUSED by
guard hook, needs template-side fix). cgap-ffi dry-run:
BLOCKED pre-existing (registry skew).

## 6. Fences could-break

None touched: no lib.rs, no tests, no gates. Publish-guard
hook untouched. mtime law respected (no sandboxed file
restored into shared tree).

## See Also

audit/pub-01-manifest-census.md · audit/pub-04-semver-delta.md
