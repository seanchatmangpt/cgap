# P3-9 Publish cgap-beam then cgap-gpu

Lane: P3-9 PUB-beam-gpu (lane-runner). Repo /Users/sac/cgap.
Lease: this file only. CARGO_TARGET_DIR=/tmp/p39-target (deleted post-run).
Publisher law: one attempt per crate; Retry-After honored; UA cgap-publish-receipt; bounded poll on p3-recon-manifests.md (<=20 min).

## cgap-beam

BLOCKED-for-publish by policy field, not compile (recon verdict honored,
publish not attempted this lane).

- `crates/cgap-beam/Cargo.toml` line 6: `publish = false`.
- Recon: `cargo publish --dry-run -p cgap-beam --allow-dirty` -> exit 101
  ("`package.publish` must be set to `true` or a non-empty list").
- Coordinates for repair: delete the `publish = false` line (or set
  `publish = true`) in crates/cgap-beam/Cargo.toml; add description/license
  metadata (see gpu finding below — the same 400 awaits otherwise).
- Version verified: 26.11.1 via workspace inheritance (root Cargo.toml line 39).

## cgap-gpu

BLOCKED-for-publish at the crates.io upload gate (metadata 400), exit 101.

- Version verified: 26.11.1 (workspace inheritance, root Cargo.toml line 39).
- Bounded poll for p3-recon-manifests.md: present (found within window).
- Dry-run (this lane, this session):
  `CARGO_TARGET_DIR=/tmp/p39-target cargo publish --dry-run -p cgap-gpu --allow-dirty`
  -> exit 0. Packaged 7 files / 6.7KiB; verification compile OK;
  "warning: manifest has no description, license or license-file";
  upload aborted by dry run. Recon's PUBLISHABLE verdict reproduced.
- Publish attempt (ONE attempt per publisher law):
  `CARGO_TARGET_DIR=/tmp/p39-target cargo publish -p cgap-gpu --allow-dirty`
  -> exit 101. Server: "status 400 Bad Request: missing or empty metadata
  fields: description, license."
- Coordinates for repair: `crates/cgap-gpu/Cargo.toml` [package] section —
  add `description` and `license` (siblings use e.g.
  description + `license = "Apache-2.0"`; see crates/cgap-arrow/Cargo.toml
  for the exact house style). One-line metadata fix, then republish.
- Dependency check: cgap-arrow 26.11.1 live on crates.io (API query,
  max_version 26.11.1) — dependency satisfiable.

## cgap-gpu repair + publish (coordinator-authorized re-invocation)

- Coordinator order expanded the lease to authorize the metadata repair and
  one re-invocation of publish (the prior 400 rejected the upload before any
  bytes landed; registry truth checked first: API returned "crate
  `cgap-gpu` does not exist" — no version was created, immutability law not
  at risk).
- Repair: `crates/cgap-gpu/Cargo.toml` [package] gained
  `description = "GPU execution membrane for the cgap sovereign substrate:
  Metal Unified Memory zero-copy buffer bridging."` and
  `license-file = "../../LICENSE.md"` (house style mirrored from
  crates/cgap-arrow/Cargo.toml, which uses license-file, not `license =`).
- Dry-run after repair -> exit 0 (8 files / 7.7KiB, metadata warning gone).
- Publish attempt 2: exit 101 — 429 Too Many Requests; server Retry-After
  Mon, 02 Nov 2026 03:33:33 GMT. Honored: bounded wait to the server-stated
  deadline, then ONE retry.
- Publish attempt 3 (post-deadline): exit 0 — "Published cgap-gpu v26.11.1
  at registry `crates-io`".
- API verify (UA cgap-publish-receipt): max_version 26.11.1,
  num_versions 1, newest 26.11.1. ALIVE.
- cgap-beam stays guarded (`publish = false`, no publish receipt — correct).

## Standing

PARTIAL: cgap-gpu 26.11.1 PUBLISHED and API-verified on crates.io (ALIVE);
cgap-beam BLOCKED-for-publish by `publish = false` (repair is a future
wave). All claims above are this-session command receipts.

## Divergences

- Dispatch named lease `docs/jira/v26.11.1/waves/p3-pub-beam-gpu.md`, but
  the file did not exist at lane start; created it (divergence recorded
  per contract).
- Recon manifest verdict for cgap-gpu ("PUBLISHABLE") is dry-run-scoped:
  cargo publish --dry-run does not enforce the crates.io metadata policy,
  so the verdict did not predict the 400. Publish-day metadata repair
  declined: lease grants this file only, not crates/cgap-gpu/Cargo.toml.
- cgap-beam publish attempt intentionally NOT made (special law: BLOCKED
  recon verdict = record + skip).
