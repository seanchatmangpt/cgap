# lane-fix01-core-license receipt

**Date**: 2026-11-02 · **Lane**: FIX-01 · **Repo**: /Users/sac/cgap (main, no git commands)
**Audit ref**: EXP-02 (audit/dod-cgap-core.md) — cgap-core manifest lacked
license/license-file; license text existed only as a comment.

## Diff

- `crates/cgap-core/Cargo.toml` — added `license-file = "../../LICENSE.md"`
  (house form per cgap-blake3:9, cgap-git:9, cgap-parquet:9). The pre-existing
  P28-SIL URN comment lines are retained beneath the new field.

## Commands + exits (this session)

| Command | Exit |
|---|---|
| `CARGO_TARGET_DIR=/tmp/fix01-target cargo package -p cgap-core --allow-dirty --no-verify` | 0 |
| (rerun, log /tmp/fix01-pkg.log) | 0; 0 license mentions, 0 warnings |
| `tar -tzf /tmp/fix01-target/package/cgap-core-26.11.2.crate | grep -i license` | 0; shows `cgap-core-26.11.2/LICENSE.md` — license file shipped in tarball |

## Sweep (report-only)

All 25 manifests checked. Among `publish = true` crates, cgap-core was the
only manifest missing `license-file`. Off-manifest observations (publish=false,
report-only): `crates/cli`, `crates/csct` lack license fields — separate lanes.

## Standing

ALIVE — commands above run this session against the exact subject.

## Fences

- cgap-core `cargo test`/workspace compile untouched (metadata-only change;
  license-file does not affect build). No other lane's files modified.
- Publish law: metadata fix only; no publish attempt made (publish freeze in
  force).
