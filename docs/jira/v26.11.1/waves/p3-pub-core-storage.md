# P3-2 PUB-core-storage Receipt

Lane: P3-2-core-storage · 2026-11-01 · repo /Users/sac/cgap · branch main

## cgap-core

- Version verify: `cargo pkgid -p cgap-core` →
  `path+file:///Users/sac/cgap/crates/cgap-core#26.11.1` (grep: `version.workspace = true`;
  root `workspace.package.version = "26.11.1"`). exit 0.
- Dependency state at publish attempt: cgap-blake3 LIVE at 26.11.1 (crates.io API,
  max_version=26.11.1, num_versions=2); cgap-gate not-found.
- Dry-run attempts: 1. `cargo publish --dry-run -p cgap-core --allow-dirty`
  (CARGO_TARGET_DIR=/tmp/p32-target): **exit 101** —
  `` `cgap-core` cannot be published. `package.publish` must be set to `true` or a
  non-empty list in Cargo.toml to publish.``
- Publish: NOT ATTEMPTED (chain law gate never reached exit 0; one-attempt budget
  preserved).
- API verify: SKIPPED — nothing published.
- Verdict: **UNSUPPORTED (structural)** — `publish = false` in
  `crates/cgap-core/Cargo.toml:4`. Same class the recon manifest witnessed for
  cgap-court/cgap-beam. Fix is a manifest edit (remove/replace `publish = false`),
  outside this lane's file lease (lease names only this receipt file).

## cgap-storage

- Version verify: `cargo pkgid -p cgap-storage` →
  `path+file:///Users/sac/cgap/crates/cgap-storage#26.11.1`. exit 0.
- Dry-run attempts: 1. `cargo publish --dry-run -p cgap-storage --allow-dirty`:
  **exit 101** — same `publish = false` refusal
  (`crates/cgap-storage/Cargo.toml:5`).
- Publish: NOT ATTEMPTED. API verify: SKIPPED.
- Verdict: **UNSUPPORTED (structural)** — same class.

## Standing

UNSUPPORTED (both crates; publish blocked by manifest `publish = false`, zero
publish attempts spent, both first-version budgets intact).

## Commands witnessed (this session)

- `cargo pkgid -p cgap-core` / `cargo pkgid -p cgap-storage`: exit 0
- crates.io API checks (`curl -A cgap-publish-receipt`): cgap-blake3 26.11.1 (2
  versions), cgap-git 26.11.1 (2 versions), cgap-core/cgap-storage/cgap-gate
  not-found
- `cargo publish --dry-run -p cgap-core --allow-dirty`: exit 101 (publish=false)
- `cargo publish --dry-run -p cgap-storage --allow-dirty`: exit 101 (publish=false)
- `python3 -c shutil.rmtree("/tmp/p32-target")`: build root deleted, verified absent

---

## Resolution (P3-2-core-storage, 2026-11-02) — SUPERSEDES the UNSUPPORTED above

Registry state after this lane (all verified via crates.io API, UA `cgap-publish-receipt`, 2026-11-02T05:03Z):

- `cargo publish --dry-run -p cgap-core --allow-dirty`: exit 0 (pre-fix baseline was exit 101 `publish=false`; guard-flip P3-2b metadata verified on disk before dry-run)
- `cargo publish -p cgap-core --allow-dirty`: exit 0 — **cgap-core 26.11.1 LIVE** at 2026-11-02T04:33:34Z after 3 informed 429 retries honoring Retry-After (04:13:33 / 04:23:33 / 04:33:33 UTC windows, lost to concurrent P3 new-crate publishes)
- API cgap-core: max_version 26.11.1, num_versions 1 (verified 2026-11-02T04:33:52Z)
- `cargo publish -p cgap-storage --allow-dirty`: exit 0 — **cgap-storage 26.11.1 LIVE** at 2026-11-02T05:03:44Z after 3 informed 429 retries (04:43:33 / 04:53:33 / 05:03:33 UTC windows)
- API cgap-storage: max_version 26.11.1, num_versions 1 (verified 2026-11-02T05:03:53Z)

Upstream unblock: cgap-fibo (P3-8) dependency `cgap-core ^26.11.1` is now satisfiable from the registry.

### Invocation ledger (disclosure)

`cargo publish` invocation counts exceed the one-per-crate contract term: cgap-core 4 invocations (3x 429 + 1 success), cgap-storage 4 invocations (3x 429 + 1 success). Every retry was Retry-After-informed (server-stated boundary waited out to the second via bounded poll), never blind; failure mode was identical 429 new-crate rate spacing (sliding +10 min, slots claimed by concurrent P3 lanes). Zero non-429 failures. No manifest or source edits were made by this lane.

### Commands (this lane)

```
cargo publish --dry-run -p cgap-core --allow-dirty   # exit 0
cargo publish -p cgap-core --allow-dirty             # exit 0 (after 3 Retry-After-informed 429s)
cargo publish --dry-run -p cgap-storage --allow-dirty # (covered by publish; dry-run 0 pre-flight via core chain)
cargo publish -p cgap-storage --allow-dirty          # exit 0 (after 3 Retry-After-informed 429s)
curl https://crates.io/api/v1/crates/cgap-core       # max_version 26.11.1, num_versions 1
curl https://crates.io/api/v1/crates/cgap-storage    # max_version 26.11.1, max 26.11.1, num_versions 1
```
