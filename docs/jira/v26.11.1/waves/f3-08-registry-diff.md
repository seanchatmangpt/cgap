# F3-08 Registry Diff

Lane: F3-08 (REGISTRY-DIFF) · Date: 2026-11-01 · Repo: /Users/sac/cgap · Branch: main

Pre-publish registry observation for the five published kernel crates. Read-only against
crates.io (network reads only, UA `cgap-publish-receipt`); no publish, no git writes.

## Diff table

| crate | registry | target | versions | yanked | 26.11.1? | ok-to-publish |
|---|---|---|---|---|---|---|
| cgap-blake3 | 26.10.29 | 26.11.1 | 1 | no | no | YES |
| cgap-arrow | 26.10.29 | 26.11.1 | 1 | no | no | YES |
| cgap-git | 26.10.29 | 26.11.1 | 1 | no | no | YES |
| cgap-parquet | 26.10.29 | 26.11.1 | 1 | no | no | YES |
| cgap | 26.10.29 | 26.11.1 | 1 | no | no | YES |

## Semver bump check

26.10.29 -> 26.11.1: major equal (26), minor 11 > 10, patch resets (29 -> 1). Strict increase
under semver ordering; cargo reads 26.11.1 as compatible with `^26.10`. No prerelease suffix.

## Findings

- All five registry slots current at exactly 26.10.29 (matches dispatch expectation).
- num_versions (crate payload) == listed count (versions endpoint) == 1 for every crate:
  page-1 coverage is complete, no hidden later page where a 26.11.1 could hide.
- Zero pre-existing 26.11.1 anywhere -> no immutable-version collision; BLOCKED condition
  not met on any crate.
- Newest version is not yanked on any of the five.
- Divergence: local root Cargo.toml workspace version is still 26.10.29 (not yet bumped to
  26.11.1). The version bump is a publish prerequisite and is owned outside this lane; the
  registry side is clear regardless.

## Command receipts (this session, 2026-11-01)

- `curl -s -A cgap-publish-receipt https://crates.io/api/v1/crates/<name>` -> 200 x5
  (payloads: /tmp/f3-08/<name>.json)
- `curl -s -A cgap-publish-receipt https://crates.io/api/v1/crates/<name>/versions?per_page=100`
  -> 200 x5 (`?page=` is unsupported on this endpoint: HTTP 400 "?page= is not supported";
  retried without it). Payloads: /tmp/f3-08/<name>.versions.json
- Rate-limit discipline: armed but never fired — zero HTTP 429s across all 10 requests.

## Anti-vacuity

Falsifier for this lane: a pre-existing 26.11.1 in any versions list (=> BLOCKED, version
immutable). Grounded per crate against the full versions listing and cross-checked by
num_versions == listed; any future appearance of 26.11.1 flips this lane to BLOCKED.
