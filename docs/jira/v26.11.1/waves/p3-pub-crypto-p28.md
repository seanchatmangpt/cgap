# p3-pub-crypto-p28

Lane P3-5 PUB-crypto-p28 — publish cgap-crypto then cgap-p28 at 26.11.1.
Repo `/Users/sac/cgap`, branch main, 2026-11-02. Lease honored: this file is
the only lane-owned file written.

## cgap-crypto — PUBLISHED (ALIVE)

- Poll: `p3-recon-manifests.md` landed ~06:12 local; recon verdict honored
  (both crates were not in recon's dry-run set; both carried
  `publish = false`).
- Baseline: crates.io `cgap-crypto` did not exist (typed 404); dep base
  `cgap-blake3` 26.11.1 live (default_version 26.11.1).
- Manifest edits (required, disclosed): `publish = false` -> `publish = true`;
  added `description` + `license-file = "../../LICENSE.md"` matching the
  published sibling pattern (cgap-blake3/cgap; registers as `non-standard`).
  First publish attempt 400: "missing or empty metadata fields: description,
  license" — nothing consumed (validation reject, not a publish).
- Dry-run exit 0 -> `cargo publish -p cgap-crypto --allow-dirty` exit 0:
  "Published cgap-crypto v26.11.1 at registry crates-io".
- API verify: max_version 26.11.1, num_versions 1, deps [] (exit 0).

## cgap-p28 — BLOCKED (crates.io new-crate rate limit)

- LICENSE-P28-SIL clause: N/A — p28 manifest references no
  `LICENSE-P28-SIL.md` (used root LICENSE.md sibling pattern instead).
- Dry-run exit 0 (resolved cgap-crypto from the live registry).
- Four publish attempts, all 429 Too Many Requests ("too many new crates in
  a short period"), Retry-After sliding +10 min each: 03:33:33 -> 03:43:33
  -> 03:53:33 -> 04:13:33 GMT. Every window was waited out in full before
  retry; no hammering. Per-attempt exits 101. NOTHING PUBLISHED: registry
  still returns 404 for cgap-p28; version 26.11.1 is unconsumed.
- Diagnosis: shared per-IP/rolling budget saturated by the concurrent
  publish wave (other P3 lanes publishing new crates). Retry after
  2026-11-02T04:13:33Z, single attempt, no fix needed beyond the wait.

## Commands and exits (abridged)

| command | exit |
|---|---|
| poll loop for p3-recon-manifests.md (10s cadence, <=20 min) | 0 (FOUND) |
| `cargo pkgid -p cgap-crypto` / `-p cgap-p28` | 0 / 0 (#26.11.1 both) |
| curl crates.io cgap-crypto baseline | 404 (absent) |
| `cargo publish --dry-run -p cgap-crypto --allow-dirty` | 0 (after metadata fix; first pre-fix run 400 at upload) |
| `cargo publish -p cgap-crypto --allow-dirty` | 0 PUBLISHED |
| curl API cgap-crypto | 26.11.1 / num 1 |
| `cargo publish --dry-run -p cgap-p28 --allow-dirty` | 0 |
| `cargo publish -p cgap-p28 --allow-dirty` (x4, full-window waits) | 101 (429 each) |
| `python3 shutil.rmtree('/tmp/p35-target')` | deleted (absent confirmed) |

## Standing

cgap-crypto: ALIVE (published, API-verified this session).
cgap-p28: BLOCKED (crates.io 429 new-crate rate limit; Retry-After
2026-11-02T04:13:33Z; version unconsumed, single re-dispatch attempt ready).

## Divergences

- Lease file did not exist at dispatch; created by this lane (owned marker).
- 400 metadata rejection consumed dry-run cycle before first real attempt;
  the 400 rejected the upload server-side — no version consumed.
- CARGO_TARGET_DIR lease cleaned; zero debris.

## Re-dispatch R2 (2026-11-02T04:23Z, lane P3-5)

Coordinator ordered the single authorized re-dispatch after the 04:13:33Z
Retry-After window lapsed. Executed in order, one publish attempt:

| step | command | exit | evidence |
|---|---|---|---|
| 1 registry pre-check | `curl -A cgap-publish-receipt .../crates/cgap-p28` | 0 (404 body: "crate `cgap-p28` does not exist") | absent, 26.11.1 unconsumed |
| 1b sibling check | `curl -A cgap-publish-receipt .../crates/cgap-crypto` | 0 | max_version 26.11.1, num_versions 1, created 03:25:33Z |
| 2 dry-run gate | `CARGO_TARGET_DIR=/tmp/pa5-target cargo publish --dry-run -p cgap-p28 --allow-dirty` | 0 | resolved cgap-crypto ^26.11.1 from live crates.io index; packaged 11 files 207.3KiB; "Verifying cgap-p28 v26.11.1" clean; "aborting upload due to dry run" |
| 2b license pre-check | `cargo package --list -p cgap-p28 --allow-dirty` | 0 | LICENSE.md packaged at basename; manifest line 7 `license-file = "../../LICENSE.md"` — LICENSE-P28-SIL clause N/A (manifest references no LICENSE-P28-SIL.md; no missing-file error; nothing copied by hand) |
| 3 publish ONCE | `CARGO_TARGET_DIR=/tmp/pa5-target cargo publish -p cgap-p28 --allow-dirty` | **0** | "Uploaded cgap-p28 v26.11.1 to registry crates-io" -> "Published cgap-p28 v26.11.1 at registry crates-io" (no 429; single invocation) |
| 4 API verify | `curl -A cgap-publish-receipt .../crates/cgap-p28` | 0 | **max_version 26.11.1, num_versions 1** (exactly one upload, no double), created 2026-11-02T04:24:40.688036Z, published_by seanchatmangpt, yanked false |
| 5 cleanup | `python3 shutil.rmtree('/tmp/pa5-target')` | deleted (absent confirmed) | zero lease debris |

Standing (updated): cgap-crypto ALIVE (03:25:33Z, num 1);
**cgap-p28 ALIVE** (04:24:40Z, num 1) — BLOCKED coordinate cleared by the
authorized re-dispatch; both crates of this lane now live at 26.11.1.
Token never read or printed; --allow-dirty disclosed (tree dirty by
campaign design). One publish invocation consumed per the re-dispatch
authorization; no blind retries existed.
