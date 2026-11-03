# Lane CALVER-01 — CalVer sweep receipt

Date: 2026-11-02 · Repo: /Users/sac/cgap (main) · Standing: **ALIVE**

## W1 Files written

- `crates/csct/Cargo.toml` — `version = "0.1.0"` → `version = "26.11.2" # CALVER-01: was 0.1.0 (dod-01 F1)`
- `crates/csct/Cargo.lock` — csct entry 0.1.0 → 26.11.2 via `cargo update -p csct`

## W2 Commands / exits

| command | exit |
|---|---|
| `cargo metadata --format-version 1 > /dev/null` (repo root) | 0 |
| `cd crates/csct && cargo update -p csct` ("Updating csct v0.1.0 → v26.11.2") | 0 |
| `cd crates/csct && cargo build` (Finished dev profile) | 0 |
| calver grep sweep `^version = ` over all Cargo.toml under crates/ + wasm-footprint/, excluding 26.11.x | 0 matches |

## W3 Version table (before → after)

| manifest | before | after |
|---|---|---|
| root workspace `[workspace.package]` | 26.11.2 | 26.11.2 (unchanged) |
| 15 `version.workspace = true` members | inherit 26.11.2 | inherit (unchanged) |
| cgap-branchless, cgap-fleet-ops, cgap-ocel2, cgap-verification, wasm-footprint, cli (cgap-cli) | 26.11.2 | 26.11.2 (already CalVer) |
| **csct** | **0.1.0** | **26.11.2** |

## W4 Courts run

Anti-vacuity mutation: the csct version mutation itself is the court subject —
before the edit, the grep sweep matched `0.1.0` (non-calver); after, zero
matches. Verified flipped: pre-edit sweep would have exited 0 with a match;
post-edit sweep exits 1 (no matches). cargo metadata exit 0 confirms no
manifest breakage from the mutation.

## W5 Fences

- Publish law: csct has `publish = false` on disk; version bump does not
  touch the publish freeze (`.claude/hooks/publish-guard.sh` untouched).
- Root `Cargo.lock` contains no csct entry (csct is standalone, own
  workspace, like crates/cli) — root lock untouched, no `-w` regen needed;
  csct's own lock updated targeted (`-p csct`, no dependency drift).
- External pins recorded, not changed: `clap-noun-verb 26.9.1` (external,
  `[patch.crates-io]` in crates/cli), lockfile `version = 4` format, external
  dep `1.1.5` entries in cli lock — external, out of lane scope.

## W6 Divergences

- Dispatch named `dod/dod-01-cargo-hygiene.md`; that file does not exist on
  disk (`ls dod` → not found). F1 (csct at 0.1.0) was corroborated directly
  from `crates/csct/Cargo.toml:3`. The dod doc was likely meant as
  `docs/jira/v26.11.2/dod/...`; not located, recorded here as hearsay-closed.
- csct `cargo build` emits a pre-existing manifest warning (unrelated to the
  version key; not a failure, build exit 0).
