# P3-10: publish cgap-integration then wasm-footprint at 26.11.1

Lease: this file. Publisher law: one attempt per crate; Retry-After honored;
UA `cgap-publish-receipt`; self-organizing resolution gate (bounded poll on
`p3-recon-manifests.md`, <=20 min); CARGO_TARGET_DIR=/tmp/p310-target.

## cgap-integration

- [ ] recon verdict honored
- [ ] version 26.11.1 verified
- [ ] cargo publish --dry-run exit 0
- [ ] published once
- [ ] API verify 26.11.1/num 1

## wasm-footprint

- [ ] recon verdict honored
- [ ] version 26.11.1 verified
- [ ] cargo publish --dry-run exit 0
- [ ] published once
- [ ] API verify 26.11.1/num 1

## OUTCOME (2026-11-01) - both crates SKIP, structural

- cgap-integration: `cargo publish --dry-run --allow-dirty` (from
  crates/integration, CARGO_TARGET_DIR=/tmp/p310-target) exit **101** -
  `package.publish` must be true/non-empty. Same structural class as
  cgap-court/cgap-beam in p3-recon-manifests.md. NOT repaired (policy field;
  recon discipline: flagged, not repaired). crates.io API: crate does not
  exist (expected, 0 versions).
- wasm-footprint: same dry-run, exit **101**, same publish=false refusal.
  Additionally standalone-workspace with `../cgap-arrow` path deps outside
  its package root (second blocker, untested - publish=false fires first).
  crates.io API: crate does not exist.
- No publish attempted (one-attempt law preserved; a publish cannot succeed
  through this gate). Version 26.11.1 verified on disk in both manifests
  pre-dry-run. Logs: /tmp/p310-wf-dryrun.log, /tmp/p310-int-dryrun.log.
