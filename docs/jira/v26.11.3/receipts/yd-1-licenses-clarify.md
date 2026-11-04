# YD-1 Receipt: licenses gate closed via `[[licenses.clarify]]` (LicenseRef-P28-SIL)

Date: 2026-11-04
Lane: YD-1 (XD-1 close condition)
Repo: /Users/sac/cgap
Status: ALIVE (commands + exits below, run this session against the exact subject)

## What was done

XD-1 registered the licenses divergence: 21 workspace crates declare
`license-file = LICENSE.md` (P28-SIL sovereign license, deliberately no SPDX
key per lane-A04 DoD 10), which cargo-deny's SPDX allowlist cannot express, so
`cargo deny check licenses` exited 1-class and CI ran it NON-BLOCKING
(`continue-on-error: true`) per
`docs/jira/v26.11.3/receipts/xd-1-deny-toml.md`.

This receipt closes it WITHOUT changing any crate's licensing intent:

1. **No Cargo.toml was touched.** Every workspace crate still declares
   `license-file = LICENSE.md`; the URN
   `urn:avec:license:p28-sil:v2.1.0-sovereign` remains the only lawful
   instrument pointer; no SPDX-shaped keys were introduced into any manifest
   (lane-A04 DoD 10 intact; `tests/manifest_court.rs` unprovoked).
2. **deny.toml only.** cargo-deny's `[[licenses.clarify]]` mechanism teaches
   the scanner, not the manifests:
   - `licenses.allow` gains `"LicenseRef-P28-SIL"` (the SPDX-compliant
     `LicenseRef-` prefix form for a non-SPDX custom license).
   - 24 `[[licenses.clarify]]` entries, one per license-file workspace crate,
     each `expression = "LicenseRef-P28-SIL"` with
     `license-files = [{ path = "../../LICENSE.md", hash = 0x8273ca50 }]`.
     The hash pins the exact LICENSE.md text cargo-deny computed
     (0x8273ca50, uniform across all license-file crates — verified by
     collecting the gather warnings from the baseline RED run). A license-text
     edit re-breaks the check loudly instead of passing silently.
   - 21 crates were in the baseline unlicensed error set (cgap, cgap-aoq,
     cgap-arrow, cgap-audit, cgap-blake3, cgap-bn3, cgap-bont, cgap-bosym,
     cgap-branchless, cgap-cloud, cgap-core, cgap-crypto, cgap-ffi, cgap-fibo,
     cgap-gate, cgap-git, cgap-gpu, cgap-integration, cgap-p28, cgap-parquet,
     cgap-storage). 3 additional clarify entries (bench, cgap-fleet-ops,
     cgap-ocel2) cover the workspace-inherited crates that the baseline error
     list did not name; they are inert-but-consistent and keep the table
     uniform. `cgap-mcp` and `wire-shm` are Apache-2.0 (SPDX) and need none.
3. **ci.yml flipped to BLOCKING.** `cargo deny check licenses` step lost
   `continue-on-error: true` and the provenance-annotation side-step; step
   renamed to `cargo deny check licenses (BLOCKING, clarified via
   LicenseRef-P28-SIL)`. Precondition honored: `git status` showed NO
   uncommitted changes to `.github/workflows/ci.yml` from other lanes at flip
   time (other lanes own crates/cgap-aoq/*, crates/cgap-mcp/Cargo.toml,
   crates/wire-shm/Cargo.toml, docs/jira/v26.11.3/README.md — untouched by
   this lane).

## Commands / exits (this session)

Baseline (pre-fix), subject = working tree at deny.toml pre-edit:

- `cargo deny check licenses` → **exit 4**, `licenses FAILED`, 21x
  `error[unlicensed]` (counted via `grep -c '^error\[unlicensed\]'` = 21).

Post-fix:

- `cargo deny check licenses` → **exit 0**, `licenses ok`.
- `cargo deny check advisories` → **exit 0** (blocking gate unaffected).
- Non-vacuity shim-court: `sed -i '' 's/0x8273ca50/0xdeadbeef/' deny.toml`
  → `cargo deny check licenses` **exit 4** (gate can flip);
  restore from /tmp/deny.toml.bak → `cmp` byte-exact,
  re-run **exit 0**. A clarify entry with a wrong hash FAILS — the gate is
  not vacuously green.
- TOML structural validation: `python3 tomllib` parse OK; 24 clarify entries,
  allow = [MIT, Apache-2.0, LicenseRef-P28-SIL].

## Files owned/changed (all absolute)

- /Users/sac/cgap/deny.toml — allow + 24 clarify entries + header rewritten
  (XD-1 divergence marked historical)
- /Users/sac/cgap/.github/workflows/ci.yml — licenses step flipped to
  blocking (no continue-on-error, annotation step removed)
- /Users/sac/cgap/docs/jira/v26.11.3/receipts/yd-1-licenses-clarify.md — this
  receipt

NOT changed (explicitly): any crates/*/Cargo.toml `license-file` field; the
P28-SIL license intent; `licenses.private.ignore` (still true).

## Standing

XD-1 licenses divergence: CLOSED (clarify mechanism, not allow-list laxity).
CI licenses gate: BLOCKING. Falsifier if it regresses: a new workspace crate
added with `license-file` and no clarify entry re-breaks `cargo deny check
licenses` loudly (exit nonzero) — this is the intended tripwire, not a defect.
