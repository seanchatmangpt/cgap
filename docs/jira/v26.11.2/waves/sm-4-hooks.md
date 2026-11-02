# SM-4 HOOKS — Wave Receipt

Lane: SM-4-hooks · repo /Users/sac/cgap · 2026-11-01

## Deliverables

| Hook | Path | Matcher | Behavior |
|---|---|---|---|
| publish-guard | `.claude/hooks/publish-guard.sh` | PreToolUse/Bash | Blocks `cargo publish` (exit 2, JSON decision:block) unless authority marker file exists (default `~/.claude/hooks/publish-authority.token`, overridable `CGAP_PUBLISH_AUTHORITY_FILE`) containing `OPERATOR-PUBLISH-AUTHORITY`. Fail-closed: parse error/absent command exits 2; unknown shape exits 3 (loud skip). |
| cwd-guard | `.claude/hooks/cwd-guard.sh` | PreToolUse/Bash | Warns (exit 0 + stderr) when a cargo command references `/tmp` without a leading `cd /tmp/...` — the silent-canonical-build hazard (witnessed PP-1/VAC-2/P3-13). Fail-closed on malformed stdin (exit 2). |
| mtime-restore-guard | `.claude/hooks/mtime-restore-guard.sh` | PostToolUse/Bash | Reminds (exit 0 + stderr) when a command matches sha256/shasum AND restore — restored bytes match but mtimes do not; touch or clean or cargo may skip rebuilds. Fail-closed on malformed stdin (exit 2). |
| flip tests | `.claude/hooks/flip-tests.sh` | — | Anti-vacuity court for all three; both directions. |

## Court evidence (flip-test run, this session)

Command: `bash /Users/sac/cgap/.claude/hooks/flip-tests.sh` → EXIT=0

- publish-guard FIRES: exit=2, block JSON emitted (mutation = no marker) — PASS
- publish-guard QUIET on `cargo test`: exit=0, no output — PASS
- publish-guard marker-mutation (mutation = marker present): flips block->allow, rc=0 with message — PASS (court can flip both directions)
- cwd-guard FIRES (`cargo test --manifest-path /tmp/...`): warn on stderr — PASS
- cwd-guard QUIET (`cd /tmp/... && cargo test`; plain cargo no /tmp) — PASS
- mtime-restore-guard FIRES (shasum + RESTORE) / QUIET (shasum alone) — PASS
- Syntax gate: `bash -n` all three hooks → SYNTAX_OK

## STATUS: UNWIRED (scripts on disk, NOT activated)

Activation is operator-owned. Wiring snippet (merge into `.claude/settings.json`):

```json
{
  "hooks": {
    "PreToolUse": [
      { "matcher": "Bash", "hooks": [
        { "type": "command", "command": "/Users/sac/cgap/.claude/hooks/publish-guard.sh" },
        { "type": "command", "command": "/Users/sac/cgap/.claude/hooks/cwd-guard.sh" }
      ]}
    ],
    "PostToolUse": [
      { "matcher": "Bash", "hooks": [
        { "type": "command", "command": "/Users/sac/cgap/.claude/hooks/mtime-restore-guard.sh" }
      ]}
    ]
  }
}
```

Note: publish-guard without a marker blocks every `cargo publish` — that is the
intended fail-closed default (coordinator executes publishes). cwd-guard is
warn-only by design; if the operator wants it hard-blocking, change its firing
path to emit the same JSON decision:block shape as publish-guard and exit 2.

## Divergences

- Dispatch named hook scripts + receipt; flip tests placed at
  `.claude/hooks/flip-tests.sh` (inside the lease, not a new top-level dir).
- settings.json untouched per CAUTION clause.
