# ZD-134 Receipt — CLAUDE.md publish-law paragraph re-derived from the zd-119 live census
Standing: ALIVE

Lane: ZD-134, docs lane, /Users/sac/cgap @ 09e8c7c7 (pulled, already
up to date with origin). Date: 2026-11-04.
Files owned: `CLAUDE.md` (publish-law paragraph only) and this receipt
(new). No other file touched.

## Status: ALIVE (all commands ran this session)

## Trigger

CLAUDE.md's publish-law paragraph still said "19 of 20 publish-intended
member crates exist on crates.io" (header paragraph) and "23/23 members
declare an explicit flag (20 true, 3 false) as of 2026-11-02" (publish-law
section). zd-119's live registry census (2026-11-04, 23 crates probed,
`zd-119-cgap-preflight.md`) supersedes the registry-state claims; the
YD-3 publish=false lines changed the flag table.

## Re-derivation (ran this session, exits witnessed)

Registry truth (cited, from zd-119's live 2026-11-04 census):

- cgap-aoq: NAME-FREE on crates.io — first-publish candidate.
- wasm-footprint: registry max 26.11.1 (skipped 26.11.2 — the one
  version-gap surface).
- All other censused publish=true crates: registry max 26.11.2.
- 26.11.3 free everywhere: zero collisions.

Flag table truth (re-run this session, `cargo metadata --no-deps` +
per-manifest `grep '^publish'` over `crates/*/Cargo.toml` and
`ontology-packs/*/Cargo.toml`):

- Root workspace = 25 members (23 `crates/` + 2 `ontology-packs/`),
  confirmed by `cargo metadata` member count = 25.
- 25/25 members declare an explicit `publish` flag: 21 true, 4 false
  (false: bench, cgap-beam, cgap-court, wire-shm).
- The prior "23/23 (20 true, 3 false)" does NOT still hold: wire-shm
  joined the member list (root `Cargo.toml` members, AoQ Wave-1) and
  carries YD-3's `publish = false`, moving the table to 21/4. cgap-mcp
  also got a YD-3 `publish = false` line but is a standalone workspace,
  NOT a member (its YD-3 receipt states this) — so the member count
  moves by one crate (wire-shm), not two.

## Diff

`CLAUDE.md` publish-law section only:

- New registry-census bullet citing zd-119 (2026-11-04): cgap-aoq
  NAME-FREE / wasm-footprint 26.11.1 gap / 26.11.3 free everywhere;
  keeps the historical 2026-11-02 table reference
  (`docs/jira/v26.11.2/01-release-gates.md:325ff`).
- Flag-count sentence updated 23/23 (20 true, 3 false) → 25/25
  (21 true, 4 false) as of 2026-11-04, with the prior table named as
  superseded.
- Non-member pointer list updated: cgap-mcp added (YD-3 publish=false,
  standalone workspace), wire-shm moved to the member-false list.
- `re-derived` date bumped 2026-11-02 → 2026-11-04 for the publish=false
  list.

## Out-of-lane disclosure (not edited)

The CLAUDE.md header paragraph (line 5) still carries the stale "19 of 20
publish-intended member crates exist on crates.io" sentence with the stale
registry-absent list ("cgap-branchless/cgap-integration ... registry-absent"
— both are published at 26.11.2 per zd-119). This is outside the
publish-law-paragraph lane boundary and was left untouched; it needs the
same zd-119 re-derivation applied by whoever owns the header paragraph.

## Verification

| command | exit | result |
|---|---|---|
| `git pull --ff-only` | 0 | Already up to date @ 09e8c7c7 |
| `cargo metadata --no-deps` member count | 0 | 25 members |
| per-manifest `grep '^publish'` sweep (crates/ + ontology-packs/) | 0 | 25/25 members flagged: 21 true / 4 false |
| `cargo metadata --no-deps` (crates/cgap-mcp, per YD-3) | 0 | cgap-mcp standalone, publish=false |

## Standing

ALIVE for the publish-law paragraph: every claim in it now traces to a
2026-11-04 observation (zd-119 census or this session's grep/metadata
run). Header paragraph drift disclosed, not repaired (lane boundary).
