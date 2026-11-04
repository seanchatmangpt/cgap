# YD-3 Receipt — explicit publish=false on cgap-mcp and wire-shm (XP-2 Phase 2 hygiene)

Lane: YD-3, implementation lane, /Users/sac/cgap. Date: 2026-11-04.
Contract: XP-2 Phase 2 hygiene — host-process/IPC internals must not be
accidentally publishable to crates.io (cargo defaults `publish = true`).
Owner: `crates/cgap-mcp/Cargo.toml`, `crates/wire-shm/Cargo.toml`,
`docs/jira/v26.11.3/receipts/yd-3-publish-lines.md` (this file, new).

## Status: ALIVE (all commands ran this session, exits witnessed)

## Diff

- `crates/cgap-mcp/Cargo.toml`: added `publish = false` immediately after
  `description` in `[package]`.
- `crates/wire-shm/Cargo.toml`: added `publish = false` immediately after
  `description` in `[package]`.

## Consumer check (nothing external consumes them as published deps)

Workspace grep over all `Cargo.toml` files for `cgap-mcp` / `wire-shm`:
only two references, both path deps (unaffected by publish flag):

- `crates/cgap-mcp/Cargo.toml:15` — `wire-shm = { path = "../wire-shm" }`
- `crates/cgap-aoq/Cargo.toml:21` — `wire-shm = { path = "../wire-shm" }`
  (annotated in that file as "standalone workspace crate; path dev-dep only")

No version-req dependency on either crate anywhere in the workspace.

## Verification (ran this session)

| command | exit |
|---|---|
| `cargo check -p wire-shm` (repo root) | 0 — `Finished dev profile` |
| `cargo check` (in crates/cgap-mcp) | 0 — `Finished dev profile` |
| `cargo metadata --no-deps` (root) | wire-shm publish = `[]` (= false); cgap-mcp absent (own workspace) |
| `cargo metadata --no-deps` (crates/cgap-mcp) | `[(cgap-mcp, [])]` — publish = false |

Note: `cargo check -p cgap-mcp` from the repo root errors ("a package with
a similar name exists: cgap-aoq") because `crates/cgap-mcp` is a standalone
workspace (empty `[workspace]` table in its manifest, not listed in root
members) — the check was run from its own directory instead. The published
dependency surface is unaffected.

## Standing

ALIVE. Both manifests now carry a deliberate `publish = false`; the
accidental crates.io publish surface is closed for both crates.
