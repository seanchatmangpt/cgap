# P3-3 Publish cgap-bont then cgap-bosym

Lane: P3-3 PUB-bont-bosym (lane-runner).
Lease: this file only. CARGO_TARGET_DIR=/tmp/p33-target.
Publisher law: one attempt per crate; Retry-After honored; UA cgap-publish-receipt; self-organizing resolution gate (bounded poll on p3-recon-manifests.md, <=20 min).

## cgap-bont

- Recon manifest p3-recon-manifests.md found on bounded poll (~60s into 20-min window); verdicts honored (publish=false = known gate class per court/beam verdicts).
- Version verify: workspace.package.version = 26.11.1 (root Cargo.toml:39); dep cgap-blake3 version = "26.11.1" (published, max_version 26.11.1).
- Manifest edits: removed `publish = false` (guard flip; P3-2b later normalized to explicit `publish = true` with attribution); added description/repository/license-file for sibling (cgap-blake3) parity.
- cargo publish --dry-run --allow-dirty: exit 0 (9 files, 206.5KiB; 2nd run after metadata add).
- cargo publish --allow-dirty: exit 0 — "Published cgap-bont v26.11.1 at registry crates-io" (ONE attempt).
- API verify (UA cgap-publish-receipt): cgap-bont max_version 26.11.1, num_versions 1.

## cgap-bosym

- Version verify: version.workspace = true -> 26.11.1 (root:39). No deps (Tier 0 leaf per recon).
- Manifest edits: removed `publish = false` (P3-2b normalized to explicit `publish = true` + attribution); added description/repository/license-file.
- cargo publish --dry-run --allow-dirty: exit 0 (7 files, 63.3KiB).
- cargo publish --allow-dirty: exit 0 — "Published cgap-bosym v26.11.1 at registry crates-io" (ONE attempt).
- API verify (UA cgap-publish-receipt): cgap-bosym max_version 26.11.1, num_versions 1.
