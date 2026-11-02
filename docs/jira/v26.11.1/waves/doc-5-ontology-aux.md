# DOC-5 receipt — ontology + aux crate READMEs

Lane: DOC-5 (lane-runner), repo `/Users/sac/cgap`, branch `main`.
Date: 2026-11-01. No git writes; lease honored (6 READMEs + this receipt).

## Files written

- `ontologies/fibo/README.md` (new)
- `ontologies/cloud/README.md` (new)
- `crates/cgap-beam/README.md` (new)
- `crates/cgap-gpu/README.md` (new)
- `crates/integration/README.md` (new — package `cgap-integration`)
- `crates/wasm-footprint/README.md` (new)

Archive-if-present: no README.md existed in any of the six target dirs
before this lane (witnessed by `ls` at lane start), so nothing was moved
to `docs/jira/v26.11.1/readme-archive/` (dir does not exist; not
created — nothing to archive).

## Evidence (this session)

- Live registry checks, UA `cgap-publish-receipt`:
  - cgap-cloud: exit 0, max_version 26.11.1, num_versions 1, created
    2026-11-02T03:43:38Z -> PUBLISHED-26.11.1.
  - cgap-gpu: exit 0, max_version 26.11.1, num_versions 1, created
    2026-11-02T03:33:40Z -> PUBLISHED-26.11.1.
  - cgap-fibo, cgap-beam, cgap-integration, wasm-footprint: 404
    "does not exist" -> UNPUBLISHED (fibo has `publish = true` post-P3-8,
    landing in flight; beam guarded `publish = false`; integration and
    wasm-footprint `publish = true` post-P3-10, landing in flight).
- `cargo test` in `ontologies/fibo` -> exit 0; 21 passed / 0 failed
  (wave1 1+6, wave4 9, wave8 1, wave9 4) — matches dispatch's 21/0.
- `cargo test` in `ontologies/cloud` -> exit 0; 13 passed / 0 failed
  (courts 8, raw_consumer 1, wire_parity 4) — matches dispatch's 13/0.
- On-disk artifacts: `fibo_sec_dbt_v1.oabi` 496 B;
  `tosca_cimi_v1.oabi` 1,552 B (`ls -l`).
- beam/gpu/integration/wasm-footprint test+build claims cited from
  `waves/assess-beam.md`, `assess-gpu.md`, `assess-integration.md`,
  `assess-wasm-footprint.md` (all marked run this campaign); per-file
  layouts and Cargo.toml fields re-read on disk this session.

## Divergences (doc yields to reality)

1. Lease named `crates/cgap-integration/`; the crate lives at
   `crates/integration/` (package name `cgap-integration`). README
   written to the real directory.
2. Dispatch said fibo has "8 emit/probe binaries"; on disk,
   `ontologies/fibo/src/bin/` contains exactly one binary
   (`emit-pack.rs`). README states the witnessed count.
3. Dispatch expected fibo/cloud "cascading"; live witness shows
   cgap-cloud has since LANDED (created 2026-11-02T03:43:38Z), fibo
   still 404. README written AS WITNESSED.

## Standing

ALIVE for the lane: every README's claims are either run this session
(registry curls, fibo/cloud test suites, artifact sizes) or cited to a
waves/ receipt with on-disk fields re-read. Six files, all new; no
shared-tree files touched.
