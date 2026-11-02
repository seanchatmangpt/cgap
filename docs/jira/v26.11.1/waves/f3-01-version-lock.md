# F3-01 Version Lock Receipt

- **Lane**: F3 wave item 1 — VERSION-LOCK (lane-runner), v26.11.1,
  repo `/Users/sac/cgap` (one canonical checkout)
- **Date**: 2026-11-01
- **Lease**: root `Cargo.toml`; `crates/cgap{,-arrow,-blake3,-git,-parquet}/Cargo.toml`;
  graph file `crates/cli/ontology/cgap-cli.ttl`; this marker.
  Nothing else written.
- **Verdict**: **ALIVE** — `cargo metadata --no-deps` exit 0 resolves all
  five publishable crates at 26.11.1 with umbrella sibling reqs `^26.11.1`;
  frozen pins byte-identical (grep-cited below); zero residual `26.10.29`
  in leased manifests.

## Edits (in-lease)

| File | Change |
|---|---|
| `Cargo.toml:39` | `[workspace.package] version = "26.10.29"` -> `"26.11.1"` |
| `crates/cgap/Cargo.toml:12-15` | four sibling deps `version = "26.10.29"` -> `"26.11.1"` (arrow/blake3/git/parquet) |
| `crates/cli/ontology/cgap-cli.ttl:6` | `cnv:version "26.10.29"` -> `"26.11.1"` |

`cgap-arrow`/`cgap-blake3`/`cgap-git`/`cgap-parquet` manifests carry
`version.workspace = true` — no per-crate version literal to edit; all
inherit the root bump. Their sibling/dev deps are path-only (no version
req), so per the dispatch they need nothing.

## Commands + exits (this session, cwd repo root)

| # | Command | Exit |
|---|---|---|
| 1 | `grep -rn "cnv:version" --include="*.ttl" /Users/sac/cgap` | 0 — single hit `crates/cli/ontology/cgap-cli.ttl:6` |
| 2 | `grep -rn "26\.10\.29" {root + 5 manifests}` (pre-edit) | 0 — 6 in-lease hits (root:39, umbrella:12-15); 3 out-of-lease hits disclosed below |
| 3 | `shasum -a 256` root Cargo.toml + cgap-cli.ttl (baseline) | 0 — `152ba322…7e28` / `b5025018…e4250c` |
| 4 | Edit root `Cargo.toml:39`, umbrella:12-15, ttl:6 | 0 (Edit tool success x3) |
| 5 | `grep -rn "26\.10\.29" {root + 5 manifests}` (post-edit) | 1 — zero residual in-lease |
| 6 | `grep -n "urn:avec:license:p28-sil:v2.1.0-sovereign" Cargo.toml` | 0 — line 3, byte-identical |
| 7 | `grep -n "https://avec.chatman.dev/ontology/26.10.31" docs/jira/v26.10.31/19-ontology-ttl.md` | 0 — lines 38, 40, byte-identical |
| 8 | `cargo metadata --no-deps --format-version 1` | 0 — packages: cgap 26.11.1, cgap-arrow 26.11.1, cgap-blake3 26.11.1, cgap-git 26.11.1, cgap-parquet 26.11.1; cgap dep reqs `^26.11.1` x4; cgap-parquet dev-dep cgap-arrow req `*` (path-only, lawful) |
| 9 | `git diff Cargo.lock` inspection | 0 — five package version strings 26.10.29 -> 26.11.1 (mechanical `cargo metadata` lockfile sync, pre-existing M state) |

## Frozen-pin proof (byte-identical, grep-cited)

- `urn:avec:license:p28-sil:v2.1.0-sovereign` — `Cargo.toml:3` (comment
  header, untouched by the line-39 edit; grep exit 0 post-edit).
- SHACL corpus IRI `https://avec.chatman.dev/ontology/26.10.31` —
  `docs/jira/v26.10.31/19-ontology-ttl.md:38` (`@prefix avec:` binding)
  and `:40` (owl:Ontology IRI); grep exit 0 post-edit. No v26.11.1 IRI
  was minted and no corpus IRI edited.

## Divergences

1. `crates/cli/Cargo.toml:6` still carries `version = "26.10.29"` —
   out of lease (standalone ggen-generated crate; regen flow is
   coordinator-owned: `rm Cargo.toml && ggen sync run && python3
   manifest_patch.py`). The graph file now says 26.11.1, so the next
   byte-identical-replay regen will pick it up. Recorded, not fixed.
2. `crates/integration/Cargo.toml:3` and
   `crates/wasm-footprint/Cargo.toml:3` still carry `version =
   "26.10.29"` — out of lease (non-published membrane/aux crates, not
   among the five). Recorded for the coordinator.
3. `cargo metadata` mechanically synced `Cargo.lock` (side effect of
   the mandated verification command; Cargo.lock was already `M`
   pre-session). Delta: five package version strings bumped + member
   dep lists for the moonshot crates other lanes have in flight.
   No other lock changes introduced by this lane.
4. Marker self-correction: an intermediate Edit to this file
   accidentally truncated the SHACL pin bullet (26.10.31 -> 25.10.31
   typo); restored byte-identical in the immediately following Edit.
   Final on-disk text re-verified by grep below.

## Post-write pin re-verification (after marker edits)

- `grep -n "urn:avec:license:p28-sil:v2.1.0-sovereign" Cargo.toml` ->
  exit 0, line 3.
- `grep -n "https://avec.chatman.dev/ontology/26.10.31"
  docs/jira/v26.10.31/19-ontology-ttl.md` -> exit 0, lines 38/40.
