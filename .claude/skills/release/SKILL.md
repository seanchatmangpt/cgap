---
name: release
description: >-
  Publish the cgap kernel crates to crates.io under the one-attempt law:
  verify tree state, dry-run gate, sequenced publish along the dependency
  chain, API verify, receipt append. Sources: coord-publish-record.md and
  the P3 publisher receipts (v26.11.1, 2026-11-02).
disable-model-invocation: true
---

# Release (crates.io publish law)

Authority: this skill runs ONLY on explicit operator invocation. It performs
unreversible uploads to crates.io. Sources of record:
`docs/jira/v26.11.1/waves/coord-publish-record.md`,
`f3-15-api-verify.md`, `p3-api-verify-all.md`, `p3-pub-*.md`.

## Law (from witnessed receipts)

1. ONE publish invocation per crate is the target; every attempt beyond the
   first must be Retry-After-informed, never blind (p3-pub-core-storage.md:
   "Every retry was Retry-After-informed ... never blind").
2. A Retry-After deadline that slides after you wait it out is a STOP, not a
   re-chase (p3-pub-aux-final.md: "Per coordinator law (slide => STOP), no
   further attempts this pass"). Backoff window: honor the server-stated
   Retry-After; if it slides, stop the pass and re-enter no sooner than
   1 hour later (observed sliding window: +10 min per 429, slots claimed by
   concurrent publishers).
3. Order is the dependency chain: cgap-blake3 -> cgap-arrow -> cgap-git ->
   cgap-parquet -> cgap (umbrella). Aux/new crates follow their own chain
   (core -> storage -> bont/bosym -> bn3 -> gate/crypto/p28 -> ffi ...) with
   each dry-run resolving predecessors from the live sparse index after a
   15 s propagation wait.
4. UA for any registry curl: `cgap-publish-receipt`. Token never printed.
5. `--allow-dirty` is permitted and disclosed (tree dirty by design:
   campaign receipts).

## Steps

1. Verify tree state at the release version (root Cargo.toml
   [workspace.package] reads the target version exactly; G1
   scripts/release_check.py exits 0).
2. Per crate, in chain order: `cargo package --no-verify --allow-dirty -p <crate>`
   (exit 0), then `cargo publish --no-verify --allow-dirty -p <crate>`.
3. On 429: bounded-poll to the server Retry-After second (no blind sleep);
   re-attempt once per stated window; slide => STOP (law 2).
4. After the umbrella publishes, wait a 15 s propagation window, then API
   verify per crate: `curl -A cgap-publish-receipt -s
   https://crates.io/api/v1/crates/<name>` -> max_version == target,
   num_versions == expected exact count (existing crate +1; new crate == 1),
   yanked == false. Witness: p3-api-verify-all.md (num_versions == 2 EXACTLY
   for the five originals after 26.11.1).
5. Append the publish record: per-crate dry-run/publish exits, one-attempt
   accounting, API verify table, timestamp, published_by. Pattern:
   docs/jira/v26.11.1/waves/coord-publish-record.md.

## Fail-closed

- num_versions higher than expected => possible double upload: STOP, type
  DOUBLE-UPLOAD-SUSPECTED in the receipt, do not yank without operator.
- f3-15 precedent: a marker without ALIVE tokens is BLOCKED, not published;
  registry acceptance is the API, never the local exit code.
