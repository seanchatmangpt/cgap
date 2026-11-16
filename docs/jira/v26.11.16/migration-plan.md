# cgap migration plan v26.11.16

Standing: UNVERIFIED

## Identity
- path: /Users/sac/cgap
- remote: https://github.com/seanchatmangpt/cgap.git
- branch@sha: main@ede726ff (tracks origin/main)
- last commit: 2026-11-06; commits: 253; size: 4132 MB (of which `.git` = 4.0 GB)
- languages/build: Rust workspace (Cargo.toml, 26 members), Justfile, Makefile
- nested repos: none

## Observed state
- dirty: 0; untracked: 0; ahead/behind: 0/0; stash: 0; unpushed branches: 0
- only one local branch (`main`)
- build/dep dirs: `target/` absent at measurement time
- `.git` is 4.0 GB — history carries large objects (likely benchmark/golden data)
- submodules: none; no .env / *.db / data dirs observed at top level

## What it is
Sovereign `#![no_std]` columnar/deductive substrate ("renamed from avec 2026-10-31"): 23-26
workspace crates published to crates.io at 26.11.x, with C-ABI admit/eval/commit/verify
entrypoints and a `cgap` umbrella facade. CLAUDE.md is authoritative and detailed.

## Classification
ACTIVE (census agrees; v26.11.6 wave closed 2026-11-06, fully pushed).

## Migration risk
Everything is committed and pushed; nothing unique on disk.
Rate: NONE

## Recommended action
CLONE-FRESH
1. `cd /Users/sac/cgap && git status --porcelain` — confirm still empty.
2. `git add docs/jira/v26.11.16 && git commit -m "docs: migration plan v26.11.16" && git push`
3. Do not copy this directory (4 GB git history re-downloads from origin).
4. DECISION: 4.0 GB `.git` — consider `git clone --filter=blob:none` on the new machine, or
   a history rewrite later to drop large blobs.

## First commands on the new machine
`git clone https://github.com/seanchatmangpt/cgap.git && cd cgap && cargo check --workspace && just check`

## See Also
- ../v26.11.6/
- /Users/sac/docs/jira/v26.11.16/README.md
