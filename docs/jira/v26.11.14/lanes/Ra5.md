# Lane Ra5 - cgap-aoq publish dry-run preflight (no publish)

Standing: BLOCKED (dry-run fails on registry dependency; no publish attempted).

## Commands
- `cargo package -p cgap-aoq --list --allow-dirty` -> exit 0, 56 files (src, tests, Cargo.toml).
- `cargo publish -p cgap-aoq --dry-run --allow-dirty` (before edit) -> exit 101.
  Index fetch succeeded (not a network failure). Error: `cgap-crypto = "^26.11.5"`
  unresolved; crates.io has only 26.11.2, 26.11.1.
- Same command after edit -> exit 101, same cgap-crypto error; the
  "no documentation, homepage or repository" warning is gone.
- No real publish run. Publishes are FROZEN (.claude/rules/publish.md).

## Metadata gaps
- license: license-file inherited from workspace (OK).
- description: present, one-line (OK).
- repository: was missing; FIXED by adding `repository.workspace = true` (siblings do this).
- readme/documentation/homepage/keywords/categories: absent; not required, not edited (ambiguous; no README file in crate).
- path-dep versions: normal dep cgap-crypto has version 26.11.5 (OK form) but that version is
  unpublished -> BLOCKER: cgap-crypto 26.11.5 must publish first (also cgap-audit 26.11.5 dev-dep
  is versioned and likely unpublished; dev-deps without version are stripped, with version they must resolve - UNVERIFIED).
- Other dev path-deps (cgap-arrow, cgap-crypto, wire-shm, cgap-blake3) carry no version; stripped on package.

## Anti-vacuity
Mutation: the repository edit flipped the manifest warning off (observed); the blocker persists, so the
failure is real, not a metadata artefact.

## Fences
Only crates/cgap-aoq/Cargo.toml metadata changed; no code. Workspace build not re-run (UNVERIFIED).
