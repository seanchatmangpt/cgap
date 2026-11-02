# SM-2 Settings Receipt

Lane: SM-2-settings · 2026-11-01 · lease: .claude/settings.json,
.claude/settings.local.json, this receipt.

## Outcome: NO EDIT

Both files valid JSON pre-session (`python3 json.load` -> BOTH_VALID, exit 0).
No witnessed need maps to a supported settings key without violating the
conservative law.

## Witnessed needs -> disposition

- (a) cargo cwd-reset sandbox hazard (bare `cargo` invocations silently build
  the canonical /Users/sac/cgap workspace): settings schema supports `env`
  (variables) but no annotations/comments; a permissions allowlist entry would
  be an escalation, not a note. RECEIPT-ONLY. Mitigation in use: absolute
  `--manifest-path` on every cargo call from lane agents.
- (b) topology guard refuses whole-repo rsync + relative-path sandbox copies
  (witnessed repeatedly this session): guard config intentionally untouched.
  RECEIPT-ONLY.
- (c) build-copy stardf crates at /Users/sac/stardf/arrow/crates/ (verified
  on disk: stardf-arrow, stardf-arrow-core, stardf-arrow-bench,
  stardf-arrow-capi, stardf-alloc-probe, stardf-bench, stardf-branchless,
  stardf-cdt, jsonschema, ...): neither settings file carries a project-path
  allowlist; `permissions.allow` entries are command-scoped only. Nothing to
  annotate. RECEIPT-ONLY, read-only observation.

## Diff summary

settings.json: unchanged (hooks.PreToolUse Agent -> ban_generic_agents.py
preserved). settings.local.json: unchanged (10 permissions.allow entries
preserved). Zero keys added, zero removed.

## Proposed (NOT wired — order does not name them)

- Optional `env.CARGO_MANIFEST_DIR_HINT` or a doc-note convention is not a
  real settings mechanism; no hook proposed for (a) beyond existing guard.
- No changes to topology guard.
