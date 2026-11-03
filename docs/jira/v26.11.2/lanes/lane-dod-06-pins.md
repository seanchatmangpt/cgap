# lane-dod-06-pins

pub-08 version unification: 18 manifests + root Cargo.lock bumped
26.11.1 -> 26.11.2 (2026-11-02, lane DOD-06). No git commands run;
edits landed for coordinator commit.

## 1. Falsifier greps (after)

`grep -rn '26\.11\.1' --include='Cargo.toml' .` (minus target/):
all 16 survivors are prose comments only (publish-guard receipts,
lane citations, bench description strings). Zero requirement lines.
`grep -B2 '26\.11\.1' Cargo.lock | grep 'name ='` -> empty
(exit 1). Final pin-scoped grep
`grep -rn 'cgap-.*version = "26\.11\.1"'` -> empty (exit 1).

## 2. Per-manifest summary (61 requirement lines, 18 manifests)

cgap-court 10, cgap-ffi 10, integration 5, ontologies/fibo 4,
cgap-beam 4, cgap-core 4, audit 3, ontologies/cloud 3, cgap-arrow 1
(implicit via workspace audit? none needed - arrow had 0 requirement
lines; counted 1 prose hit), cgap-bn3 2, cgap-bont 2, cgap-bosym 1,
cgap-crypto 2, cgap-gate 1, cgap-gpu 1, cgap-p28 2, cgap-storage 3,
bench 0 (its 2 audit-counted hits were prose).

## 3. Cargo.lock

`cargo update -w` exit 0; diff 25 lines (arrow/blake3/git/parquet
dual workspace+dep entries now 26.11.2). Lock falsifier empty.
crates/cli standalone: `cargo metadata --offline` exit 101 - see
divergence D2.

## 4. Verify command (honest standing)

`CARGO_TARGET_DIR=/tmp/dod06-target cargo package --no-verify
--allow-dirty -p cgap-ffi -p cgap-court -p integration` -> EXIT 101
(BLOCKED, not pass): packaging resolution strips path deps and
resolves against crates.io, where siblings exist only at 26.11.1
(`failed to select a version for the requirement
cgap-audit = "^26.11.2"` / `candidate versions found which didn't
match: 26.11.1` / `location searched: crates.io index`). Expected
pre-publish; unblocks at sequenced registry push (pub-03 sequence).
The pin invariant itself is grep-verified in section 1.

## 5. Anti-vacuity court

/tmp/dod06-av sandbox copy of cgap-bont/Cargo.toml, one pin reverted
to 26.11.1 -> falsifier grep FIRES (exit 0, requirement line 14).
Court flips; shared tree untouched; sandbox deleted after.

## 6. Divergences

- D1: bench's 2 audit-counted "pins" are prose comment lines, not
  requirement lines (pub-08 section 2 overcounted bench; actual
  requirement-line count is 61 across 18 manifests as the doc's own
  total states).
- D2: dispatch verify step (cargo package exit 0) unreachable
  pre-publish; recorded BLOCKED with exact error, not rounded up.
- D3 (disclose): concurrent lanes have in-flight edits to
  cgap-branchless/Cargo.toml and csct/Cargo.toml observed in
  git status; this lane did NOT touch either (csct untouched per
  dispatch; branchless not in my sed file list, verified via its
  diff content being publish-metadata only).

## Standing: PARTIAL (pins + lock ALIVE; package-verify BLOCKED on
registry state, exact error in section 4)
