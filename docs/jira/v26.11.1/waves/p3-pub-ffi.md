# P3-6 Publish cgap-ffi

Lane: P3-6 PUB-ffi (lane-runner).
Lease: this file only. CARGO_TARGET_DIR=/tmp/p36-target.
Publisher law: one publish; Retry-After honored; bounded polls on
registry/file observables; UA cgap-publish-receipt.

## cgap-ffi

Published: **cgap-ffi v26.11.1** — `Published cgap-ffi v26.11.1 at
registry crates-io` (exit 0, 2026-11-02).

API verify: crates.io `max_version: 26.11.1`, `num_versions: 1`.

## Sequence (commands + exits, this session)

1. Bounded poll (<=20 min) for `p3-recon-manifests.md` +
   `p3-pub-guard-flip.md` -> exit 0 (both landed).
2. Version verify: root Cargo.toml:39 `version = "26.11.1"`;
   member line 12; `cargo package --list -p cgap-ffi --allow-dirty` ran.
3. Dry-run loop: 36 attempts, 45-min deadline -> exit 0. Attempts 1-17
   failed `no cgap-bn3`, 18-35 failed `no cgap-core` (deps landing
   mid-loop per P3-4/P3-1 publishers); attempt 36 packaged clean,
   aborted upload by dry-run design.
4. First publish attempt: exit 101, 429 Too Many Requests
   (new-crate rate limit), Retry-After 04:43:33 GMT honored via
   server-named schedule; second retry hit fresh Retry-After
   04:53:33 GMT, also honored; third attempt exit 0.

## Package contents (cargo package --list)

`.cargo_vcs_info.json`, `Cargo.lock`, `Cargo.toml`, `Cargo.toml.orig`,
`README.md`, `src/lib.rs`, `src/oabi_lookup.rs`, `src/sovereign.rs`,
`src/xip_tables.rs`, `tests/alloc_courts.rs`, `tests/zero_copy.rs`,
`xip.ld`.

- `xip.ld` ships. `include/cgap.h` ABSENT from the package (lives at
  repo-root `include/`, outside the crate dir) — recorded, NOT fixed
  (header packaging is an A19-owner decision).
- cargo warns manifest has no license/license-file — recorded, not in
  lease.

## Divergences

- Task named `crypto` in the dep surface; the on-disk manifest has no
  cgap-crypto dep (p28 carries it transitively). Dispatch yielded.
- Background dry-run loop #1 was killed at the harness 30-min default
  background limit (not the 45-min lane deadline); restarted with a
  2-h harness window.
- Guard flip (`publish = false` -> `true`, Cargo.toml line 6) landed
  via lane P3-2b (coordinator-owned manifest), not this lane.

## Standing

ALIVE — publish, package listing, and API verification all carry
this session's command + exit.
