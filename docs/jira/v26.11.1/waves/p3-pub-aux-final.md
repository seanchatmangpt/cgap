# P3 AUX-PUBLISH final receipt (lane A-3)

Date: 2026-11-01 · Lane: A-3 AUX-PUBLISH · Repo: /Users/sac/cgap (canonical, branch main)

## wasm-footprint

Structural fix (P3-10): `../cgap-arrow` / `../cgap-parquet` path deps converted to
registry deps (`cgap-arrow = "26.11.1"` etc.), `publish = false` → `true`,
`license-file = "../../LICENSE.md"` added (house style per crates/cgap-arrow/Cargo.toml).

- Publish verification compiles in the DEV profile; the default unwind dev profile
  failed the wasm link (`rust-lld: undefined symbol: memcmp` from precompiled core).
  Fix inside lease: `[profile.dev] panic = "abort"` + `opt-level = 3` mirroring the
  release profile the footprint gate builds with.
- Dry-run: `cargo publish --dry-run --allow-dirty --target wasm32-wasip1`
  → "Packaged 8 files, 39.8KiB ... Finished dev profile [optimized + debuginfo]"
  → "Uploading wasm-footprint v26.11.1 ... aborting upload due to dry run" (clean).
- Publish (ONE invocation, as contracted):
  `cargo publish --allow-dirty --target wasm32-wasip1`
  → **429 Too Many Requests**: crates.io new-crate rate limit, retry after
  **Mon, 02 Nov 2026 03:43:33 GMT**. No crate uploaded; version 26.11.1 still free.
- Standing: **BLOCKED(rate-limit)** — manifest is publish-ready (dry-run exit-clean);
  one retry invocation after the rate-limit window expires completes the publish,
  then API-verify `crates.io/api/v1/crates/wasm-footprint` shows 26.11.1 / num 1.

## cgap-integration

Assessment: doc-only `src/lib.rs` + 4 Chicago test suites (`tests/{admission_e2e,
cross_crate,error_paths,membranes}.rs`). Manifest fixed (publish=true, license-file,
registry deps) — dry-run failed at packaging:

- `error: failed to prepare local package for uploading — no matching package named
  'cgap-audit' found (crates.io index)`. cargo publish resolves DEV-dependencies
  against the registry even though they are never compiled; path+version is not
  enough. `cgap-audit` is **404** on crates.io
  (https://crates.io/api/v1/crates/cgap-audit, witnessed 2026-11-01), despite the
  audit crate's manifest claiming `publish = true` with an A15 ALIVE receipt comment.
- The only fix is publishing cgap-audit (outside this lane's lease) or removing the
  cgap-audit dev-dep (requires editing tests/error_paths.rs, outside the lease).
- Standing: **BLOCKED(structural — unpublished dev-dep cgap-audit)**; the manifest
  edit is in place and the crate publishes the moment cgap-audit lands on crates.io.

## Divergence from lane context

Lane context asserted "All workspace crates it may dep are LIVE at 26.11.1 on
crates.io" — false for cgap-audit (404 witnessed). Everything else in both dep
surfaces (cgap-arrow, cgap-blake3, cgap-parquet, cgap-git) resolves at 26.11.1.

## Retry pass (coordinator unblock, 2026-11-01 post-03:53Z)

Precondition verified: `cgap-audit` now live on crates.io at 26.11.1 (API `newest_version`,
witnessed this pass); `wasm-footprint` still 404 (version free).

- wasm-footprint, ONE invocation: `cargo publish --allow-dirty --target wasm32-wasip1`
  → **429 Too Many Requests**, Retry-After **Mon, 02 Nov 2026 04:33:33 GMT** (a NEW
  window — the earlier 03:43:33Z expiry passed but the rolling new-crate budget had
  not recovered). BLOCKED(rate-limit) until 04:33:33Z; manifest remains publish-ready
  (dry-run clean this session).
- cgap-integration: dry-run with the now-live cgap-audit dev-dep → clean
  ("Uploading cgap-integration v26.11.1 ... aborting upload due to dry run", exit 0;
  only an `unused_dependencies` warning on the doc-only lib's `[dependencies]
  cgap-arrow`). Publish, ONE invocation → **429 Too Many Requests**, same
  Retry-After **Mon, 02 Nov 2026 04:33:33 GMT**. BLOCKED(rate-limit); the dev-dep
  resolution blocker is CLOSED, only the shared crates.io window remains.

Both crates are one invocation from published after 2026-11-02 04:33:33 GMT:
wasm-footprint: `cd crates/wasm-footprint && cargo publish --allow-dirty --target wasm32-wasip1`
cgap-integration: `cd crates/integration && cargo publish --allow-dirty`

## Post-window sequence (coordinator re-dispatch, 2026-11-02 ~04:30Z)

Bounded clock check vs the server-stated 04:33:33Z deadline (162 s margin waited,
no guess-sleep beyond it), then SEQUENCED per the shared new-crate budget:

- wasm-footprint, ONE invocation at 04:33:38Z:
  `cargo publish --allow-dirty --target wasm32-wasip1`
  → **429 Too Many Requests**, NEW Retry-After **Mon, 02 Nov 2026 04:43:33 GMT**
  (the window slid by the exact 10-minute new-crate span — the earlier batch's
  uploads keep the rolling budget consumed at each attempt instant).
  cgap-integration NOT attempted (sequence discipline: first leg failed).
- API state after: both crates still unpublished.
- Standing: **BLOCKED(rate-limit)**, next server-stated window
  **2026-11-02 04:43:33 GMT**; both manifests remain publish-ready and
  dry-run-clean; the same two one-invocation commands apply after that window.

## Final pass (2026-11-02 04:43:39Z) — wasm-footprint PUBLISHED

- Bounded clock check → single attempt at 04:43:39Z:
  `cargo publish --allow-dirty --target wasm32-wasip1`
  → **"Uploaded wasm-footprint v26.11.1 to registry crates-io ... Published
  wasm-footprint v26.11.1 at registry crates-io"** (exit 0).
- API verify: `newest_version = 26.11.1`, `num_versions = 1`. P3-10 CLOSED.
- cgap-integration, ONE attempt immediately after (sequenced, as contracted):
  → **429 Too Many Requests**, Retry-After **Mon, 02 Nov 2026 04:53:33 GMT**.
  Per coordinator law (slide ⇒ STOP), no further attempts this pass.
- Sliding-window evidence: 03:43:33 → 04:33:33 → 04:43:33 → 04:53:33 GMT. The
  window advances by exactly the span to the next attempt each time — the
  429'd attempts themselves count against the rolling new-crate budget.
- **One-hour-wait protocol for the coordinator**: do NOT retry at 04:53:33.
  Wait until ≥ 05:45:00 GMT (last failure + 1 h margin), then make exactly one
  attempt: `cd crates/integration && cargo publish --allow-dirty`. If that
  429s again, repeat only after another full hour. Each extra attempt pushes
  the window forward 10 minutes; zero attempts let it expire.
- Standing: wasm-footprint **ALIVE (published, API-verified)**;
  cgap-integration **BLOCKED(rate-limit)** — publish-ready, dry-run-clean,
  cgap-audit dep resolved, one invocation from published under the protocol above.

## V2-8 verify-only note (2026-11-02T05:18Z, lane V2-8-closer)

- wasm-footprint: VERIFY-ONLY, no re-publish per lane contract.
  crates.io API: `curl -H "User-Agent: cgap-publish-receipt"
  https://crates.io/api/v1/crates/wasm-footprint` → exit 0 →
  versions = [("26.11.1", "2026-11-02T04:43:42.598288Z")]. Matches A-3
  receipt (04:43:42Z). Standing: **ALIVE (published, API-verified)**.
- cgap-integration: bounded clock check at 2026-11-02T05:18:03Z — still
  BEFORE the last Retry-After deadline (05:45:00Z); sliding window has
  NOT recovered. No publish attempt made (one-invocation rule; chasing
  the window pushes it forward). Standing: **BLOCKED(rate-limit)**
  as of 2026-11-02T05:18:03Z, retry eligible at ≥ 05:45:00Z. Overnight
  session owns the single retry per the protocol above.
