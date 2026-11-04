# XD-1: cargo-deny config + CI wiring for cgap (XC-5r rec #2)

- **Date**: 2026-11-04
- **Repo/subject**: `/Users/sac/cgap` @ `f561b563` (uncommitted working-tree diff on top)
- **Owned files**: `deny.toml` (new), `.github/workflows/ci.yml` (edit), this receipt
- **Standing**: ALIVE (both checks executed this session on the exact subject tree)

## What landed

1. **`deny.toml`** (new, repo root): `cargo-deny` config shaped after
   `/Users/sac/ggen/deny.toml` (template defaults), trimmed to the two sections
   wired into CI — `[advisories]` and `[licenses]`.
2. **`.github/workflows/ci.yml`**: new `deny` job.
   - Step `cargo deny check advisories` — **blocking**.
   - Step `cargo deny check licenses` — **continue-on-error** by registered
     divergence (below), plus an always-run provenance annotation so the
     disclosed gap is never silent.

## Verification ladder (commands + exits, run on the exact subject)

| command | exit | evidence |
|---|---|---|
| `cargo deny --version` | 0 | `cargo-deny 0.20.2` |
| `cargo deny check advisories` | 0 | `advisories ok` |
| `cargo deny check licenses` | 1 | 21× `error[unlicensed]: cgap-* = 26.11.3 is unlicensed` — all 21 workspace crates, zero external |
| `cargo deny check advisories licenses` | 1 | advisories ok + licenses FAILED (above) |
| `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"` | 0 | jobs: verify, courts, fmt-clippy, deny |

Allowlist was converged empirically (removed 6 unmatched-license warnings across
two passes): final allowlist = `MIT`, `Apache-2.0` — exactly what the external
graph (E=0 kernels; dev-deps) actually carries.

## Registered divergence: `licenses` check FAILS by design

Every cgap workspace crate declares `license-file = LICENSE.md` (the P28-SIL
sovereign license, deliberately no SPDX key per lane-A04 DoD, annotated at
`crates/cgap-core/Cargo.toml:11`), and `licenses.private.ignore = true` does not
suppress it because the crates are not marked `publish = false`. cargo-deny
cannot express a license-file-only crate in an SPDX allowlist, so `cargo deny
check licenses` exits 1 with 21 `unlicensed` errors — all cgap workspace
crates, **zero external** dependencies implicated. This is the shape the task
brief predicted ("advisories should come back clean"), mirrored on the license
side.

**Close condition**: add `publish = false` (or an SPDX `license` key) to the 21
workspace Cargo.toml files, or add per-crate `[[licenses.clarify]]` entries —
all outside this task's file ownership; then delete `continue-on-error` on the
licenses step.

## CI replay path

CI runs the same invocations on `ubuntu-latest` via the new `deny` job:
`taiki-e/install-action@v2 (cargo-deny)` → `cargo deny check advisories`
(blocking) → `cargo deny check licenses` (continue-on-error) → provenance
annotation (`if: always()`), mirroring the repo's existing disclosed-gap
pattern (BEAM gate, fmt-clippy).

## Files not touched

No other file edits. `git status` shows other lanes' in-flight receipts in the
same directory — not this lane's.
