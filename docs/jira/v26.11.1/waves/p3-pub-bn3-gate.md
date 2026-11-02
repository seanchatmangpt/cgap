# P3-4 PUB-bn3-gate Receipt

Lane: P3-4-bn3-gate · 2026-11-01 · repo /Users/sac/cgap · branch main.
Lease honored: this marker is the only file written by this lane.

## Standing

BLOCKED — crates.io server-side metadata gate. Both crates verify and
package clean at 26.11.1; the publish attempt for cgap-gate was rejected
HTTP 400 (missing `description` + `license` in the crate manifest). The
two-line manifest fix is outside this lane's file lease.

## Gate sequence (this session)

1. Bounded poll for `p3-recon-manifests.md`: PRESENT-ALIVE at 150s
   (exit 0, 30s cadence, 20 min budget).
2. Version verify: `cargo pkgid -p cgap-gate` and `-p cgap-bn3` ->
   `...#26.11.1` (exit 0); root Cargo.toml:39 `version = "26.11.1"`.
3. crates.io baseline: both crates "does not exist" (clean, no skew).
4. bn3 test gate on the final subject: noalloc.rs residue ABSENT
   (ls exit 1; lib.rs has no noalloc line), `cargo test -p cgap-bn3`
   -> 111 passed / 0 failed / 0 ignored, exit 0 — matches RECON's
   post-removal count exactly.
5. Dry-runs (CARGO_TARGET_DIR=/tmp/p34-target):
   - cgap-gate: exit 0 (6 files, 83.5KiB, verify clean, dry-run abort).
   - cgap-bn3: clean verify + "aborting upload due to dry run"
     (exit code not captured pre-cleanup; clean on content).
6. Publish attempt (cgap-gate, ONE attempt):
   `cargo publish -p cgap-gate --allow-dirty` -> exit 101,
   HTTP 400: "missing or empty metadata fields: description, license".
   NO version minted (crates.io re-read: "does not exist"); the 26.11.1
   slot is still publishable.
7. cgap-bn3 publish NOT attempted — same missing metadata, the attempt
   would consume budget against a known 400.

## Required fix (coordinator / next dispatch)

Add to both manifests, per cgap-blake3 precedent
(crates/cgap-blake3/Cargo.toml:5,9):

```
description = "..."            # one-line crate summary
license-file = "../../LICENSE.md"
```

then re-run: dry-run (step 5) -> publish once -> API verify
max_version == 26.11.1, num_versions == 1.

## Divergences

1. `publish = true` guard flips were landed concurrently by lane P3-2b
   between my first grep (read `false`) and the first gate dry-run
   (refused on `publish = false`). Re-ran on the current subject; the
   stale dry-run refusal was against a superseded subject.
2. Missing description/license metadata was not flagged by P3-1 RECON's
   dry-run pass (it dry-ran court/beam/gpu only, not gate/bn3).

## Resolution record (R2, coordinator re-dispatch, 2026-11-02)

P3-2b landed `description` + `license-file = "../../LICENSE.md"` on both
manifests (verified at gate Cargo.toml:5-6, bn3 Cargo.toml:5-6); registry
re-read confirmed both crates still "does not exist" (budget intact).

- cgap-gate: dry-run exit 0; publish attempts 03:29:44 (400 metadata,
  superseded subject), 03:33:40, 03:43:40, 03:53:44 -> 429 Too Many
  Requests (new-crate rate limit, Retry-After sliding +10 min per
  attempt — sibling lanes publishing new crates concurrently), then
  **PUBLISHED cgap-gate v26.11.1 at 04:03:44Z** (exit 0).
- cgap-bn3: dry-run exit 0; publish attempts 04:03:5x -> 429
  (Retry-After 04:13:33Z), then **PUBLISHED cgap-bn3 v26.11.1 at
  04:13:45Z** (exit 0).

API verify (curl, User-Agent cgap-publish-receipt):
- `cgap-gate`: max_version **26.11.1**, num_versions **1**
- `cgap-bn3`: max_version **26.11.1**, num_versions **1**

All 429s rejected pre-mint; one successful upload per crate, both
versions immutable at 26.11.1. `/tmp/p34-target` deleted post-run
(shutil.rmtree, confirmed absent).

**Standing (updated): ALIVE** — both crates published and API-verified.

Divergence note: 429 Retry-After chasing required ~30 min wall clock per
crate; the per-attempt waits are server-stated scheduling intervals, not
test sleeps. bn3 published against the identical 111/0 test subject
verified earlier this lane (noalloc residue absent).
