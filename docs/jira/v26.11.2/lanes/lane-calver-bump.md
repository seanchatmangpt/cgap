# Lane calver-bump — 26.11.2 → 26.11.3 CalVer bump receipt

Date: 2026-11-02 · Repo: /Users/sac/cgap (main) · Standing: **ALIVE**

## W1 Files written

24 Cargo.toml files bumped (`26.11.2` → `26.11.3`, value-only sed):

- Root `Cargo.toml` (workspace.package version + cgap-branchless ws dep)
- crates/: cgap, cgap-ocel2, cgap-crypto, cgap-bn3, integration, cgap-ffi,
  cgap-bont, wire-shm, cli, cgap-beam, cgap-core, audit,
  cgap-verification, cgap-storage, csct, cgap-gpu, cgap-p28,
  cgap-fleet-ops, wasm-footprint, cgap-court, cgap-branchless
- ontologies/{cloud,fibo}/Cargo.toml
- Lockfiles: root `Cargo.lock` (via `cargo update -w` exit 0 after
  integration dev-dep unpin), `crates/cli/Cargo.lock` (`cargo update -w`
  exit 0), `crates/csct/Cargo.lock` (`cargo update -p csct` exit 0),
  `crates/wire-shm/Cargo.lock` + `crates/cgap-mcp/Cargo.lock` (refreshed
  by their `cargo test` runs)

## W2 Version table (before → after)

- All 23 root members (cargo metadata enumeration): 26.11.2 → 26.11.3
  (15 inherit workspace.package 26.11.3; explicit-version members bumped)
- wire-shm 26.11.2 → 26.11.3 · cgap-mcp already 26.11.3 (verified, unchanged)
- cli, csct, cgap-verification, wasm-footprint: 26.11.2 → 26.11.3
- cgap-ocel2, cgap-fleet-ops: 26.11.2 → 26.11.3 (non-root-member, in scope)

## W3 Commands / exits

| command | exit |
|---|---|
| `cargo metadata --no-deps` (root, enumeration) | 0 |
| sed sweep over 24 Cargo.tomls | 0 |
| `cargo metadata --format-version 1` (root, FULL resolve) | 0 (after fix below; was 101 pre-fix) |
| `cargo update -w` (root) | 0 |
| `cargo check -p cgap-core` (root) | 0 |
| `cargo test` in crates/wire-shm | 0 (1+7 = 8/8 pass) |
| `cargo test` in crates/cgap-mcp | 0 (4+1 = 5/5 pass) |
| `cargo update -p csct` (crates/csct) | 0 |
| `cargo update -w` (crates/cli) | 0 |
| version-value sweep grep | 0 stray `version = "26.11.2"` |

## W4 Named exceptions (lawful, kept at 26.11.2)

cgap-integration registry-only dev-deps (no `path` key; resolved from
crates.io, never compiled in-tree): `crates/integration/Cargo.toml:16,21,23,24`
— cgap-arrow, cgap-blake3, cgap-parquet (features=["std"]), cgap-git all
pinned `"26.11.2"` with `# CALVER-BUMP-26.11.3` comments. Rationale:
caret req ^26.11.2 already admits 26.11.3 post-publish; pinning ^26.11.3
made root FULL `cargo metadata` resolve exit 101 (registry has no
26.11.3 yet, witnessed this session) and would break
`cargo build --workspace` until the publish lift. Each line carries the
annotation.

## W5 Courts / anti-vacuity

Court subject = the bump itself: pre-edit the sweep matched 24 files at
26.11.2 (exit 0 with matches); post-edit the same sweep finds zero
version values at 26.11.2 outside the four annotated integration pins.
The four pins are independently falsifiable: removing the annotation
test — full `cargo metadata` flipped 101 (with ^26.11.3 pins) → 0 (with
minimum-bound pins), witnessed this session, i.e. the pins have a real
observable effect and the court is not vacuous.

## W6 Fences

- Root workspace green: `cargo metadata` full-resolve exit 0 +
  `cargo check -p cgap-core` exit 0 this session. Fence was
  transiently broken (exit 101) by naive ^26.11.3 pinning; repaired
  within the lane.
- Publish law: publish.md states 26.11.3 republishes owed on lift
  (cgap-blake3/arrow/parquet/bn3) — this bump stages exactly that
  version; no publish attempted, nothing published.
- Prior receipt CALVER-01 (csct 0.1.0→26.11.2): superseded value only
  (26.11.2→26.11.3); no errata — its commands remain accurate as-of-date.
