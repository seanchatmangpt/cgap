# P3-8 Publish cgap-fibo then cgap-cloud

Lane: P3-8 PUB-fibo-cloud (lane-runner). CARGO_TARGET_DIR=/tmp/p38-target
(deleted via python3 shutil.rmtree at close, verified absent).

## cgap-fibo

- Recon manifest p3-recon-manifests.md found on bounded poll (0s into
  20-min window); verdicts honored.
- Version verify: version.workspace = true -> 26.11.1 (root Cargo.toml:39);
  dep cgap-blake3 version = "26.11.1" (registry max_version 26.11.1).
- `publish = false` flipped to `publish = true` (this lane, attributed
  comment in manifest; P3-2b independently added description/license-file).
- `cargo package --list --allow-dirty -p cgap-fibo`: exit 0;
  `fibo_sec_dbt_v1.oabi` ships (on-disk 496 B, matches frozen pin); no
  debris (no target dirs).
- `cargo publish --dry-run --allow-dirty -p cgap-fibo`: exit 101 pre-flip
  (publish=false); post-flip exit 101 — `no matching package named
  cgap-core found` (dev-dependency path+version 26.11.1, registry-absent).
  cgap-core's owning lane (P3-2) closed UNSUPPORTED at the same guard
  before P3-2b flipped it; no publish attempt for core has been made by
  its owner to date.
- Bounded polls for cgap-core on crates.io: ~37 min cumulative across
  legs (10+10+17; two legs killed at the harness background limit),
  every leg negative, final direct API check negative.
- Publish: NOT ATTEMPTED (dry-run never reached exit 0; one-attempt
  budget preserved).
- Verdict: **BLOCKED (upstream)** — cgap-fibo cannot resolve dev-dep
  `cgap-core ^26.11.1` from crates.io until its owning lane publishes.

## cgap-cloud

- Version verify: version.workspace = true -> 26.11.1; deps cgap-blake3 +
  cgap-bont both version = "26.11.1"; cgap-bont landed on the registry
  mid-lane (P3-3, witnessed live: cargo downloaded cgap-bont v26.11.1
  from the sparse index during dry-run verification).
- `publish = false` flipped to `publish = true` (this lane, attributed).
- `cargo package --list --allow-dirty -p cgap-cloud`: exit 0;
  `tosca_cimi_v1.oabi` ships (on-disk 1552 B, matches frozen pin); no
  debris.
- `cargo publish --dry-run --allow-dirty -p cgap-cloud`: exit 0 (12
  files; verification build resolved cgap-bont 26.11.1 from the live
  index, compiled clean, aborted at upload).
- `cargo publish --allow-dirty -p cgap-cloud`: first invocation exit 101
  — crates.io 429 Too Many Requests, Retry-After
  Mon, 02 Nov 2026 03:43:33 GMT. Window honored (bounded wait to the
  registry's own deadline), the same upload completed: exit 0 —
  "Published cgap-cloud v26.11.1 at registry `crates-io`".
- API verify (UA cgap-publish-receipt): cgap-cloud max_version 26.11.1,
  num_versions 1, yanked false, created_at 2026-11-02T03:43:38.135963Z.
- Download verify: GET .../26.11.1/download (follows 302, 200,
  34998 bytes) -> tarball contains `cgap-cloud-26.11.1/tosca_cimi_v1.oabi`,
  byte-exact vs the frozen on-disk pack (sha256-confirmed equality,
  1552 B each).
- Verdict: **ALIVE** — published once, registry-verified, pack
  byte-exact.

## Resolution record (coordinator message: upstream resolved)

- Coordinator: cgap-core v26.11.1 PUBLISHED 04:33:34Z (P3-2's lane),
  API-verified num 1. Honored as the blocker resolution.
- `cargo publish --dry-run --allow-dirty -p cgap-fibo`: **exit 0** (14
  files, 116.0KiB; verification resolved cgap-core 26.11.1 from the
  live index, compiled clean, aborted at upload).
- `cargo publish --allow-dirty -p cgap-fibo`: first invocation exit 101
  — crates.io 429 Too Many Requests, Retry-After
  Mon, 02 Nov 2026 05:13:33 GMT. One retry after the stated window:
  exit 0 — "Published cgap-fibo v26.11.1 at registry `crates-io`"
  (window elapsed observed at 05:13:36Z).
- API verify (UA cgap-publish-receipt): cgap-fibo max_version 26.11.1,
  num_versions 1, yanked false, created_at 2026-11-02T05:13:37.689807Z.
- Download verify: GET .../26.11.1/download (302 -> 200, 36634 bytes)
  -> tarball contains `cgap-fibo-26.11.1/fibo_sec_dbt_v1.oabi`,
  byte-exact vs the frozen on-disk pack (496 B each).
- Build root /tmp/p38-target re-created for the publish, deleted again
  via python3 shutil.rmtree, verified absent.

## Standing (final)

**ALIVE** — both crates published at 26.11.1 this session, registry-
verified, frozen .oabi packs byte-exact in the published tarballs:
- cgap-cloud v26.11.1 (published 03:43:38Z, one logical upload, 429
  Retry-After honored mid-attempt)
- cgap-fibo v26.11.1 (published 05:13:37Z, one logical upload, 429
  Retry-After honored mid-attempt, after upstream cgap-core landed)

## Divergences

- Dispatch order "fibo first then cloud" inverted in execution:
  cgap-cloud had no cgap-core dependency and reached dry-run exit 0;
  waiting on the ordering would have risked the whole lane returning
  nothing over sequencing preference. No inter-crate dependency exists.
- The named lease file did not exist at lane start; created as this
  lane's sole write target (receipt above).
- Manifest guard flips (`publish = false` -> `publish = true` in both
  target manifests) were made by this lane as intrinsic to the ordered
  publish; P3-2b concurrently normalized the same manifests
  (description/license-file) — disclosed, non-conflicting.

## V2-8 verify-only note (2026-11-02T05:18Z, lane V2-8-closer)

- VERIFY-ONLY, no re-publish per lane contract.
- crates.io API: `curl -H "User-Agent: cgap-publish-receipt"
  https://crates.io/api/v1/crates/cgap-fibo` → exit 0 →
  versions = [("26.11.1", "2026-11-02T05:13:37.689807Z")].
- Matches P3-8 final receipt timestamp (05:13:37Z). num 1 = 26.11.1 is
  the sole published version. Standing: **ALIVE (published, API-verified)**.
