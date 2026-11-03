# Lane DOD-01FIX — Cargo.toml manifest repairs

Date 2026-11-02 · repo /Users/sac/cgap (canonical checkout, branch main).
Fix lane closing DOD-01 findings F3a/F3b/F3c/F3d + DOD-05 Finding 2
(cgap-cloud RECEIPT.md). Additive manifest keys only; no version bumps,
no dep changes, no publish-flag changes, no git write commands.

## Files written (before -> after)

- `crates/cgap-core/Cargo.toml` — F3a: ADDED
  `license-file = "../../LICENSE.md"` (branchless pattern,
  cgap-branchless/Cargo.toml:5) after the keywords line. Before: no
  license key (DOD-05 FLAG: `--list` had zero license lines). After:
  LICENSE.md in the package list.
- `crates/cgap-ocel2/Cargo.toml:6` — F3b: `../LICENSE.md` ->
  `../../LICENSE.md` (old path resolved to crates/LICENSE.md, absent
  on disk, verified `ls` before edit).
- `crates/csct/Cargo.toml:7` — F3c: same dangling-path fix
  (`../LICENSE.md` -> `../../LICENSE.md`).
- `crates/cgap-fleet-ops/Cargo.toml` — F3d: ADDED
  `license-file = "../../LICENSE.md"` (no license field existed).
- `ontologies/cloud/Cargo.toml` — DOD-05 Finding 2: ADDED
  `exclude = ["RECEIPT.md"]` after `publish = true`. Mechanical and
  reversible (single key; coordinator call implemented per dispatch).

## REFUSED item (hook refusal, not skipped silently)

- `crates/cli/Cargo.toml` (F3d, cli half) — the PreToolUse hook
  `guard-generated.sh` REFUSED the edit: cli's Cargo.toml is a
  ggen-GENERATED surface (`.claude/rules/cli-generated.md`); the
  license-file key must enter via the cnv:Cli graph or
  `crates/cli/manifest_patch.py` (regen cycle `rm Cargo.toml && ggen
  sync run && python3 manifest_patch.py` would clobber a hand edit).
  Neither file is in this lane's owned set — escalated to coordinator
  unchanged. The dispatch named the file; the repo hook is the
  higher authority (doc yields to on-disk law); recorded as
  divergence per contract.

## Escalated-unchanged (per contract: F1/F2 coordinator-owned)

- F1: csct version 0.1.0 vs fleet 26.11.2 — NOT touched (version bump
  coordinator-owned).
- F2: workspace MSRV/rust-version undeclared — NOT touched (policy).

## Courts run (this session, commands + exits)

- For each fixed crate (cgap-core, cgap-ocel2, csct, cgap-fleet-ops,
  cgap-cloud): `CARGO_TARGET_DIR=/tmp/dod01fix-target cargo package
  --list --allow-dirty --manifest-path <crate>/Cargo.toml` — all
  exit 0 with LICENSE.md in the list (grep rc=0 each). DOD-05 flip
  witness LANDED: cgap-core's list gains LICENSE.md
  (DOD-05 Finding 1 FLAG closed in list mode).
- cgap-cloud anti-inclusion: `--list | grep -c '^RECEIPT'` = 0
  (RECEIPT.md excluded from the tarball).
- `cargo metadata --format-version 1 --no-deps` exit 0, 23
  workspace members — unchanged.
- `cargo build -p cgap-core` exit 0 (no behavioral diff).

Anti-vacuity: DOD-05's named mutation surface (add the key ->
LICENSE.md must appear) is exactly the executed flip; the before
state (cgap-core list had zero license lines) is the DOD-05
receipt's own FLAG witness. No skips.

## Fence law (must-not-regress surfaces this diff could break)

- `tests/manifest_court.rs` (cgap-core): the A04 manifest court
  bans SPDX-shaped `license` keys; `license-file` (URN pointer
  form) is the lawful form per lane-dod-01-spdx — no SPDX key
  added. Court not run this lane (no test-runner ownership);
  flagged for the coordinator's gate run.
- G1-PUBLISH / make gate0: manifest-only additive diff; cargo
  metadata resolution verified unchanged (23 members, exit 0).
- cgap-cloud `exclude` affects only the package list, not the
  build (build path untouched).

## Standing

ALIVE for F3a/F3b/F3c/F3d(fleet-ops half)/cgap-cloud exclusion —
every claim above re-run this session with exits cited.
PARTIAL overall: cli half of F3d is REFUSED-by-hook and escalated.
Divergences: cli hook refusal (above); receipt is a
working-tree verdict (--allow-dirty, uncommitted wave diffs), same
caveat as DOD-05.

## JSON terminus

{"lane":"DOD-01FIX",
"files_written":["crates/cgap-core/Cargo.toml",
"crates/cgap-ocel2/Cargo.toml","crates/csct/Cargo.toml",
"crates/cgap-fleet-ops/Cargo.toml","ontologies/cloud/Cargo.toml",
"docs/jira/v26.11.2/lanes/lane-dod-01fix-cargo-manifests.md"],
"commands":["cargo package --list --allow-dirty x5 crates (5x0, LICENSE.md present each)",
"grep -c '^RECEIPT' cgap-cloud list (=0)",
"cargo metadata --no-deps (0, 23 members)",
"cargo build -p cgap-core (0)"],
"courts_run":["DOD-05 flip witness: cgap-core --list gains LICENSE.md after license-file added",
"cgap-cloud RECEIPT.md exclusion flip (0 in list)",
"cargo metadata member-count fence (23, exit 0)"],
"standing":"ALIVE for 5 landed edits; cli F3d REFUSED by guard-generated.sh hook, escalated",
"divergences":["crates/cli/Cargo.toml edit refused by PreToolUse guard-generated.sh — license-file must enter via ggen graph or manifest_patch.py (coordinator)",
"F1 (csct 0.1.0) and F2 (workspace MSRV) escalated-unchanged per contract",
"working-tree verdict under --allow-dirty, not HEAD"]}
