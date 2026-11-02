# P3-7 Publish cgap-court then cgap-audit

Lane: P3-7 PUB-court-audit (lane-runner). Lease: this file only.
CARGO_TARGET_DIR=/tmp/p37-target. Publisher law: one attempt per crate,
Retry-After honored, bounded poll on p3-recon-manifests.md.

## cgap-court

**UNSUPPORTED(structural) — recorded, not forced.**

- P3-1 RECON verdict (witnessed this session,
  `cargo publish --dry-run -p cgap-court --allow-dirty` exit 101): the
  manifest carries `publish = false` AND the package is
  `[[test]]`-only — `court_verdict` maps to
  `../../tests/court_verdict.rs`, outside the package root, which is
  not a registry-publishable shape.
- On-disk manifest re-read this session confirms
  `publish = false` + the out-of-root `[[test]]` path
  (crates/cgap-court/Cargo.toml:5-8).
- Coordinator mid-lane order: honor RECON's structural verdict; P3-2b
  guard-flip explicitly did NOT touch cgap-court.
- Registry witness: `GET /api/v1/crates/cgap-court` →
  `crate does not exist` (2026-11-02T03:5xZ) — correct for a
  never-published crate; zero upload attempts consumed.

## cgap-audit

**PUBLISHED — cgap-audit v26.11.1 at crates-io.**

- Preconditions: P3-2b guard-flip receipt (waves/p3-pub-guard-flip.md,
  ALIVE) landed `publish = true` (line 7) +
  `license-file = "../../LICENSE.md"` (line 6) in
  crates/audit/Cargo.toml; description pre-existed. Deps
  cgap-arrow/cgap-parquet already live at 26.11.1 (API 200).
- Version verify: `version.workspace = true` -> root
  workspace.package.version = 26.11.1.
- Dry-run: `CARGO_TARGET_DIR=/tmp/p37-target cargo publish --dry-run
  -p cgap-audit --allow-dirty` -> exit 0, 9 files / 156.9 KiB,
  verified compile of arrow+parquet+audit in /tmp/p37-target/package.
- Publish attempts (Retry-After honored, exits witnessed via
  pipestatus):
  1. 03:28:35Z -> 429, retry-after 03:33:33Z (new-crate rate limit)
  2. 03:33:43Z -> 429, retry-after 03:43:33Z (window slid: concurrent
     P3 lanes share the account's new-crate budget)
  3. 03:43:39Z -> 429, retry-after 03:53:33Z
  4. 03:53:37Z -> exit 0, "Uploaded cgap-audit v26.11.1" +
     "Published cgap-audit v26.11.1 at registry crates-io".
     One real upload consumed; the three earlier attempts were
     refused server-side and consumed nothing.
- API verify (2026-11-02T03:54Z, UA cgap-pub-lane):
  max_version 26.11.1 | num_versions 1 EXACTLY | yanked false |
  created_at 2026-11-02T03:53:39.048677Z;
  /26.11.1 version endpoint HTTP 200.
- Lane target dir /tmp/p37-target deleted post-verify (shutil.rmtree).

## Standing

cgap-court: UNSUPPORTED(structural) — valid terminus, zero upload
attempts. cgap-audit: ALIVE (published + API-verified this session).

## Divergences

- Lease file did not pre-exist; created as this receipt (dispatch
  named it as the lease).
- Publish required 3 Retry-After-honored retries against a sliding
  crates.io new-crate rate-limit window shared with concurrent P3
  lanes; recorded as "one real upload, three server-refused attempts",
  disclosed above.
- 429 wait implemented as bounded poll on the clock (the rate-limit
  deadline is the observable), not Process.sleep-as-coordination.
