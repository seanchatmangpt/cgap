# f3-05-auth

Wave F3 item 5 - TOKEN-PRESENCE gate-runner receipt. Repo /Users/sac/cgap.
Redaction rule applied: credential file never read; existence checks and
error classes only; no token material anywhere in this receipt.

## Existence checks

- `test -f /Users/sac/.cargo/credentials.toml` -> exit 0 (present; content
  never read, never hashed)
- legacy `/Users/sac/.cargo/credentials` -> exit 1 (absent)
- `CARGO_REGISTRY_TOKEN` -> unset (value never inspected)
- `/Users/sac/.cargo/config.toml` probe -> exit 2 (file missing)

Primary presence evidence: `credentials.toml` exists.

## Auth-shape probe

- `CARGO_TARGET_DIR=/tmp/f3-5-target cargo publish --manifest-path
  /Users/sac/cgap/Cargo.toml --dry-run -p cgap-blake3 --allow-dirty` -> 0
- Log classes: crates.io index update OK; packaged cgap-blake3 v26.11.1,
  14 files / 109.9KiB; verification compile finished; upload aborted by
  dry-run (1 warning, expected DRY-RUN-ABORT class); 0 errors.
- Precondition: bounded 20s poll for f3-01-version-lock.md -> present;
  workspace version stable at 26.11.1 across reads; no VERSION-LOCK race,
  no retry needed.

## Redaction sweep

- credential-shape sweep (`sk` + hyphen, `token`) over
  /tmp/f3-5-publish.log -> 0 matches (grep exit 1 = zero matches)

## Standing

ALIVE (presence-level): auth material exists in the canonical location and
the publish chain is package-clean against crates.io.

## Divergences

- `--dry-run` aborts before upload, so the probe exercises index
  reachability and packaging, not token acceptance. Token validity on
  crates.io stays UNVERIFIED by lane design; downstream lanes must not read
  this receipt as a validity proof.
- Probe used `--manifest-path` form (cwd-reset discipline), not `cd`.

## Commands (exit codes)

1. `test -f /Users/sac/.cargo/credentials.toml` -> 0
2. `test -f /Users/sac/.cargo/credentials` -> 1
3. env probe `CARGO_REGISTRY_TOKEN` -> unset
4. `grep -q token /Users/sac/.cargo/config.toml` -> 2 (file missing)
5. bounded poll f3-01-version-lock.md -> present within 20s window
6. dry-run publish probe (command above) -> 0
7. redaction sweep -> 0 matches
8. cleanup `python3 shutil.rmtree /tmp/f3-5-target` -> 0 (post-verify)

courts_run: none (presence-probe lane; contract specifies no court)
