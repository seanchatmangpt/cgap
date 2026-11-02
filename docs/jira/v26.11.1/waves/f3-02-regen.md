# F3-02 CLI Regen Receipt

Lane F3-2-cli-regen, 2026-11-01. Repo: /Users/sac/cgap. Lease: crates/cli/ regen flow.

## Gate

Item 1 (f3-01-version-lock) poll: bounded 15s cadence, 300s deadline. Graph
`crates/cli/ontology/cgap-cli.ttl` read `cnv:version "26.11.1"` at t=285s. Proceeded on
graph bump (f3-01 marker never landed during the window; graph hash in sync closure:
`a36a4dab9d202960cff2582a6222e14dd8ee082f1967ae7c92a5888fd45fa567`).

## Commands (exit codes)

- `command -v ggen` -> 0 (ggen 26.10.11 at /Users/sac/.local/bin/ggen)
- pre-regen snapshot: `git diff --stat crates/cli` -> 0 (sole tracked delta: ontology bump)
- `cd crates/cli && rm Cargo.toml && ggen sync run` -> 0
  (written: Cargo.toml, src/generated_cli.rs, tests/zero_code_cli.rs;
  skipped-unchanged: generated_behavior.rs, generated_boundaries.rs, custom_handlers.rs,
  lib.rs, main.rs, docs/SCHEMA_ADMITTED.md, docs/command-reference.md)
- `python3 manifest_patch.py` -> 0 ("patched")
- `CARGO_TARGET_DIR=/tmp/f3-2-target cargo build` -> 0 (cgap-cli v26.11.1, 14.76s)
- `CARGO_TARGET_DIR=/tmp/f3-2-target cargo test` -> 0 (19 passed / 0 failed, 8 binaries:
  12 + 6 + 1 across three non-empty suites; log /tmp/f3-2-test.log)
- `/tmp/f3-2-target/debug/cgap self version` -> 0, output `"26.11.1"`
- `python3 -c shutil.rmtree('/tmp/f3-2-target')` -> 0, dir absent after

## Byte-stability proof

sha256 pre vs post over Cargo.toml, ggen.lock, 6 src files, 2 test files,
docs/command-reference.md: every hash identical except Cargo.toml. Unified diff of
Cargo.toml: single hunk, `version = "26.10.29"` -> `version = "26.11.1"`. Zero other
deltas — regen is byte-stable apart from version (CLAUDE.md generated-artifacts law).
ggen.lock pins packs only, no graph hash; not stale.

## Standing

ALIVE (all commands witnessed this session with exits above).

## Divergences

None. Chartered regen ran exactly as written in CLAUDE.md. f3-01 marker absence during
the poll window resolved per dispatch: graph-level cnv:version bump observed directly.
