# BP-2 settings receipt

Lane: BP-2-settings — deterministic enforcement (.claude/settings.json, single writer).
File owned: `/Users/sac/cgap/.claude/settings.json`. No git writes.

## What was added (preserving the pre-existing Agent PreToolUse hook)

1. **Stop hook (fast core gate)**:
   `cd "$CLAUDE_PROJECT_DIR" && cargo check -p cgap-blake3 -p cgap-core -q 1>&2 || exit 2`
   Rationale: cheap sub-second core-subset compile check so a terminus is refused
   when the core doesn't build. The FULL workspace gate
   (`cargo build --workspace` / `cargo test -p cgap-court`) remains the
   pre-terminus check run explicitly before landing — the Stop hook is a fast
   tripwire, not the full ladder.
2. **PostToolUse Edit|Write**: `bash .claude/hooks/fmt-edited.sh` — formats the
   just-edited file (BP-3 owns the script).
3. **PreToolUse Edit|Write** (two hooks):
   - `guard-generated.sh`: blocks hand-edits to crates/cli generated surfaces,
     ontology projections, and frozen-pin files.
   - `guard-secrets.sh`: belt-and-suspenders secret-path guard, pairing with the
     permissions.deny Read rules.
4. **permissions.allow** (witnessed-safe, read-only/test only): cargo
   test/check/metadata --no-deps/bench --no-run; the two gate scripts
   (boundary_gate.py, zero_string_gate.py incl. --self-check); crates.io index
   reads with the cgap-publish-receipt UA; ls/grep/find; git status/log/diff
   (read-only git).
5. **permissions.ask**: `Bash(git push *)` — pushes require explicit approval.
6. **permissions.deny**: Read(./.env), Read(./.env.local), Read(./secrets/**).

## Coordination note (disclosed)

BP-3 creates the hook scripts concurrently
(`.claude/hooks/fmt-edited.sh`, `guard-generated.sh`, `guard-secrets.sh`).
Until those land, the PreToolUse/PostToolUse hooks may error (missing file).
This is acceptable and expected mid-wave; the settings wiring is complete and
correct the moment the scripts exist.

## Validation

- `python3 json.load` on the final file: VALID (run in the same command that
  wrote it, and re-read on disk after).
- Pre-existing keys (hooks.PreToolUse Agent -> ban_generic_agents.py) preserved
  as entry 0 of PreToolUse.

## Standing

ALIVE for the settings file itself (written + JSON-validated this session).
The three BP-3 hook scripts are UNVERIFIED here (not this lane's files).
