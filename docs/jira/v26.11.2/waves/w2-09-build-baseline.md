# W2-9 build-baseline receipt (gate-runner, G8 pre-witness baseline)

- Subject: /Users/sac/cgap root workspace (post-metadata-repair tree)
- Date: 2026-11-01
- Build root lease: CARGO_TARGET_DIR=/tmp/w2-09-target, deleted at lane end via
  `python3 -c "import shutil; shutil.rmtree('/tmp/w2-09-target', True)"` — deletion
  receipted: `ls -d /tmp/w2-09-target` -> "No such file or directory" (exit 1).
- Scope: baseline only, no repairs, /Users/sac/stardf untouched.

## Ladder

| Gate | Command (cwd=/Users/sac/cgap) | Exit | Evidence |
|---|---|---|---|
| build | `CARGO_TARGET_DIR=/tmp/w2-09-target cargo build --workspace` | 0 | Finished dev profile in 1.84s |
| tests | `CARGO_TARGET_DIR=/tmp/w2-09-target cargo test -p cgap-court` | 0 | 14 passed; 0 failed; 0 ignored (26.05s), court_verdict.rs |

## Classification

- Compile-abort: NONE. Build executed; no census-vs-abort distinction needed.
- Failing crate: none. No errors. Non-failing warnings only (manifest-level,
  pre-existing):
  - crates/bench/Cargo.toml:20-21 unused deps `cgap-ffi`, `cgap-git`
  - crates/integration/Cargo.toml:11 unused dep `cgap-arrow = "26.11.1"`
- Introduced-by-today's-manifest-edits: none observed — build/tests green on
  the post-bump tree (26.11.2), no new warnings attributable to version bump,
  license-file/repository inheritance, version.workspace conversions, or the
  cgap-gpu publish line.
- cgap-court verdict harness: 14/0 green (incl. a19_seal 1m ledger + eval courts).

## JSON footer

```json
{
  "lane": "W2-9",
  "date": "2026-11-01",
  "subject": "/Users/sac/cgap (root workspace, post-metadata-repair)",
  "build_exit": 0,
  "test_exit": 0,
  "tests": {"passed": 14, "failed": 0, "ignored": 0, "binary": "court_verdict.rs"},
  "compile_abort": false,
  "failing_crate": null,
  "classification": "pre-existing-warnings-only",
  "target_dir": "/tmp/w2-09-target",
  "target_dir_deleted": true,
  "repairs_made": 0
}
```
