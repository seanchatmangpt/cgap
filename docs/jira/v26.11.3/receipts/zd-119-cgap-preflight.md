# ZD-119: cgap pre-publish preflight (ZP-9 Part 3 Stage 0)

Standing: PARTIAL — token present, registry reachable

Date: 2026-11-04. Repo: /Users/sac/cgap, workspace version 26.11.3.
Read-only preflight — no publishes, no manifest edits, no commits.

## Whoami verdict

- `cargo whoami` → `error: no such command: whoami` (exit 101);
  `cargo-whoami` subcommand not installed on this machine.
- `~/.cargo/credentials.toml` exists (57 bytes, mode 0600, mtime 2026-10-29)
  with exactly one `token` entry — a registry token IS configured.
- `cargo search cgap-core --limit 1` → `cgap-core = "26.11.2"` — registry API
  reachable.

Verdict: **PARTIAL — token present, registry reachable, login identity
unverified** (not BLOCKED: a token exists; not ALIVE: whoami unavailable).
Replay path to close the identity check:

```
cargo install cargo-whoami
cargo whoami        # run in /Users/sac/cgap
```

If whoami then names a login, Stage 0 auth is ALIVE. If it reports not
logged in, operator re-runs `cargo login <token>` before publish day.

## Census method

`curl -s -A 'cgap-preflight' https://crates.io/api/v1/crates/<name>` with
1.2 s spacing, 2026-11-04. Registry max read as `versions[0]` (the API's
top-level `max_version` field returned null under this projection;
`versions[0]` is the newest published version and is the load-bearing
value). 404 → NAME-FREE. Census covered 23 crates: the 20 `publish = true`
crates derived mechanically from the tree, plus cgap-beam, cgap-fleet-ops,
and cgap-cli (publish = false, censused because a publish-day flip is
plausible). The task brief's "21+2" count was not reproducible from the
tree; the publish=true set is authoritative for publish day.

## Pre-publish state table

Every local crate resolves to 26.11.3 (workspace `version = "26.11.3"`; the
standalone crates pin 26.11.3 directly).

| crate | publish | registry max | 26.11.3 exists? | collision risk |
|---|---|---|---|---|
| cgap-audit | true | 26.11.2 | no | LOW — 26.11.3 free |
| cgap-aoq | true | NAME-FREE | n/a | LOW — first publish |
| cgap-arrow | true | 26.11.2 | no | LOW — 26.11.3 free |
| cgap-beam | false | NAME-FREE | n/a | none (publish=false) |
| cgap-blake3 | true | 26.11.2 | no | LOW — 26.11.3 free |
| cgap-bn3 | true | 26.11.2 | no | LOW — 26.11.3 free |
| cgap-bont | true | 26.11.2 | no | LOW — 26.11.3 free |
| cgap-bosym | true | 26.11.2 | no | LOW — 26.11.3 free |
| cgap-branchless | true | 26.11.2 | no | LOW — 26.11.3 free |
| cgap-core | true | 26.11.2 | no | LOW — 26.11.3 free |
| cgap-crypto | true | 26.11.2 | no | LOW — 26.11.3 free |
| cgap-ffi | true | 26.11.2 | no | LOW — 26.11.3 free |
| cgap-fleet-ops | false | NAME-FREE | n/a | none (publish=false) |
| cgap-gate | true | 26.11.2 | no | LOW — 26.11.3 free |
| cgap-git | true | 26.11.2 | no | LOW — 26.11.3 free |
| cgap-gpu | true | 26.11.2 | no | LOW — 26.11.3 free |
| cgap-cli | false | NAME-FREE | n/a | none (publish=false) |
| cgap (umbrella) | true | 26.11.2 | no | LOW — 26.11.3 free |
| cgap-parquet | true | 26.11.2 | no | LOW — 26.11.3 free |
| cgap-p28 | true | 26.11.2 | no | LOW — 26.11.3 free |
| cgap-storage | true | 26.11.2 | no | LOW — 26.11.3 free |
| wasm-footprint | true | 26.11.1 | no | LOW — 26.11.3 free (skipped 26.11.2) |
| cgap-integration | true | 26.11.2 | no | LOW — 26.11.3 free |

## Findings

1. Zero 26.11.3 collisions anywhere on the registry — every publishable
   crate can take 26.11.3 on publish day.
2. cgap-aoq is NAME-FREE: it has never been published; publish day is its
   first publish (no version history to conflict with).
3. wasm-footprint's newest registry version is 26.11.1 — it skipped the
   26.11.2 wave; its local 26.11.3 is still free, but this is the one
   surface with a version gap worth noting.
4. Auth is the only open Stage 0 item: identity confirmation via
   `cargo install cargo-whoami && cargo whoami`, else operator `cargo login`.
