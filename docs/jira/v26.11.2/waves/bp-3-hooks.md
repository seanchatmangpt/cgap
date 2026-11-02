# BP-3 Hook Scripts Receipt

Lane: BP-3 HOOK-SCRIPTS (lane-runner) · 2026-11-01 · repo /Users/sac/cgap

Implements the three hook scripts BP-2's settings.json references
(.claude/settings.json lines 18/27/48). Executable bits set via
`python3 os.chmod` (all 0o755, witnessed).

## Scripts + logic

- `.claude/hooks/fmt-edited.sh` — PostToolUse. Reads stdin JSON
  (`tool_input.file_path`) via python3 (jq assumed absent). If path ends
  .rs and exists on disk: `rustfmt --edition 2021 <path>`. Exit 0 always —
  format, never block. Empty/malformed stdin: exit 0 (a formatter has no
  basis to block).
- `.claude/hooks/guard-generated.sh` — PreToolUse. Exit 2 (block) when the
  path is a generated surface per CLAUDE.md: `crates/cli/src/generated_*.rs`,
  `crates/cli/Cargo.toml`, `crates/cli/docs/command-reference.md`,
  `crates/cli/tests/zero_code_cli.rs`, `ontology/cgap.ttl`. Fail-closed:
  malformed JSON -> exit 2. Empty stdin -> exit 0 (nothing targeted).
- `.claude/hooks/guard-secrets.sh` — PreToolUse. Exit 2 when basename or
  path matches `.env`/`.env.*`, `secrets/**`, or `*credentials*`.
  Fail-closed: malformed JSON -> exit 2.

## Fixture table (all in /tmp/bp3-fixtures, removed after witness)

| Script | Input (file_path) | Expected | Witnessed exit |
|---|---|---|---|
| fmt-edited.sh | /tmp/bp3-fixtures/ugly.rs (unformatted) | reformatted on disk, exit 0 | 0 (disk verified: expanded) |
| fmt-edited.sh | /tmp/bp3-fixtures/notes.txt | untouched, exit 0 | 0 |
| fmt-edited.sh | `not json` | exit 0 (never blocks) | 0 |
| guard-generated.sh | crates/cli/src/generated_cli.rs | block | 2 |
| guard-generated.sh | crates/cli/Cargo.toml | block | 2 |
| guard-generated.sh | crates/cgap-core/src/lib.rs | pass | 0 |
| guard-generated.sh | `not json` | block (fail-closed) | 2 |
| guard-secrets.sh | .env | block | 2 |
| guard-secrets.sh | secrets/api_key.txt | block | 2 |
| guard-secrets.sh | crates/cgap-core/src/frame.rs | pass | 0 |
| guard-secrets.sh | `not json` | block (fail-closed) | 2 |

Both guard hooks flip (2 vs 0); fmt hook flips on disk state, not exit.
Anti-vacuity: each guard has a witnessed fire AND no-fire fixture; fmt's
fire is proven by on-disk bytes (ugly.rs reformatted), not exit code.

Standing: ALIVE (all fixtures run this session, exits above).
