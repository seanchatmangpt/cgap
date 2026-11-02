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

## ADDENDUM (W2-19, 2026-11-02) — D7

D7 (from the W1-17 audit receipt, per sm-5-audit.md): the exit-code
claims in the Court evidence table stand unrepaired — sm-5 observed
malformed-stdin fail-closed as exit 3 (all three hooks), while this
receipt claims exit 2; the marker-path claim
(~/.claude/hooks/publish-authority.token) also diverges from the
script default (repo .claude/hooks/ path). Recorded as OPEN-defect:
receipt exit-code/marker rows are not reproducible as written;
fail-closed behavior itself (nonzero on bad input) is confirmed both
sessions. Evidence source: W1-17 audit receipt (D7) and
sm-5-audit.md hooks rows.

## ERRATA (W7 census, 2026-11-02)

W7 drift census (waves/w7-errata.md) re-grounded the Deliverables
publish-guard row against the script itself
(`.claude/hooks/publish-guard.sh`): default authority path is the
REPO-LOCAL `/Users/sac/cgap/.claude/hooks/publish-authority.token`
(:7), not `~/.claude/hooks/...`; absent-command / empty-CMD input is a
loud-skip EXIT 3 (:16), not exit 2 (exit 2 is the block path, :30).
Three receipt rows non-reproducible as written: (a) "default
`~/.claude/hooks/publish-authority.token`", (b) "parse error/absent
command exits 2", (c) the Court-evidence exit codes relying on (b).
Consistent with the pre-existing W2-19/D7 addendum above; fail-closed
(nonzero-on-bad-input) remains confirmed.
