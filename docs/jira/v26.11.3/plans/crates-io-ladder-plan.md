# crates.io Ladder Plan — cgap workspace (v26.11.3)

Date: 2026-11-04. Source: YP-1 plan agent, facts read from disk.
Refreshed 2026-11-04 post dry-run (ZD-43/ZD-97, receipt zd-113):
Stage 2 COMPLETE-VERIFIED, nothing published.

## 0. Measured baseline

- **21 publishable crates** in the root workspace — PINNED by live
  `cargo metadata --no-deps` at HEAD 7459848: 25 members, 4 excluded via
  explicit `publish = []` (wire-shm, cgap-beam, bench, cgap-court).
  Supersedes the plan-history counts 17 (YP-1) and 19/20 (pre-sweep
  ladders; ZD-43's "22" was pre-sweep and is corrected to 21 in its
  ZB-16 addendum). The 21: cgap, cgap-arrow, cgap-aoq, cgap-blake3,
  cgap-crypto, cgap-core, cgap-gate, cgap-branchless, cgap-storage,
  cgap-git, cgap-bn3, cgap-parquet, cgap-p28, cgap-ffi, cgap-audit,
  cgap-gpu, cgap-integration, cgap-bont, cgap-bosym, cgap-fibo,
  cgap-cloud.
- No CI publish jobs exist in either repo (zero CRATES_TOKEN /
  `cargo publish` matches in cgap and ggen workflows) → **local publish
  from the maintainer machine is the precedent**; `cargo login` once.
- Workspace inheritance: `version/edition/rust-version/repository/authors`
  via `.workspace = true`; cargo inlines workspace syntax into the
  packaged manifest (cargo ≥1.64 behavior). `license-file` is NOT
  inherited — each member declares `license-file = "../../LICENSE.md"`
  literally, pointing outside the package root. **PREDICTED Stage-1
  blocker — OBSERVED NEVER-FIRED** (see §1).

## 1. License-file "blocker" — FALSIFIED as a gate (historical record)

**Status: struck.** The predicted Stage-1 license fix never fired across
20 dry-runs (ZD-43 + ZD-97, cgap @ 37721e2f and 9582131): every one of
the 20 laddered crates passed packaging metadata, including the
license-file-declaring crates — 7 green full-package builds (cgap-arrow,
cgap-blake3, cgap-bosym, cgap-branchless, cgap-aoq, cgap-git,
cgap-parquet) and 13 failures all in the registry-dep-missing class
(sibling ^26.11.3 reqs unresolvable against the live index — self-healing
at Stage 3 publish order). Zero license-file errors, zero E0583, zero
dirty-worktree blockers after ZD-97. The `../..`-escapes-are-fatal model
(cargo#9432) did not manifest on this toolchain's dry-run verify; any
future license-file work is deferred until a REAL publish or docs.rs
build reproduces it — not a Stage-1 gate.

Historical fix recipe, retained only for that contingency: copy the
root LICENSE.md (sha256 recorded at copy time) into each crate dir +
set `license-file = "LICENSE.md"` per crate, one atomic commit. SPDX
expressions remain **banned** by the root manifest's P28-SIL comment
(license identified by URN §9, no SPDX key) — the fix is never
`license = "..."`.

## 2. Command sequence

```bash
# Stage 0 — preflight
cargo login  # once
for c in <crates>; do curl -s https://crates.io/api/v1/crates/$c | \
  jq -r '.versions[]?.num' | head -3; done   # name/version-collision check

# Stage 1 — license-file fix: STRUCK (falsified as a gate; see §1)
# Stage 2 — dry-run ladder: COMPLETE-VERIFIED. 20/20 accounted across
#   ZD-43 (7 green + 11 registry-dep-missing + 2 dirty-worktree) and
#   ZD-97 (the 2 dirty-worktree crates re-run clean-tree, both
#   registry-dep-missing) → 7 green + 13 registry-dep-missing + 0 other.
for c in $(topo order via cargo tree); do cargo publish -p $c --dry-run; done

# Stage 3 — real ladder (21 crates, §0 pinned census), topo order.
# ADMISSION GATE (restated): 7 green dry-runs (ZD-43) stand as the
# packaging-metadata gate — the 13 registry-dep-missing crates
# self-heal at publish order (L0 first, dependents resolve); after each
# publish, BOUNDED POLL (not sleep) on
# https://crates.io/api/v1/crates/<dep>/26.11.3 until 200, then advance.
```

## 3. Failure modes

| Failure | Handling |
|---|---|
| version-req unresolvable (dep not yet on registry) | bounded poll for index propagation (~3-5 min), retry same crate |
| verification build / doc-test failure | fix in-tree, rerun that crate's dry-run only |
| name already owned by other | typed refusal, STOP ladder |
| mid-ladder defect | `cargo yank --version 26.11.3 -p <crate>`; caret reqs `^26.11.3` match 26.11.4, so restart family at 26.11.4 without re-publishing immutable 26.11.3s |

**Version immutability law**: never re-publish a (crate, 26.11.3) pair
that has touched the registry; defective member → yank + restart at
26.11.4.

## 4. Receipt shape (per crate)

{crate, version, exact HEAD SHA, .crate sha256, registry URL, docs.rs
URL, dry-run exits ×21, preflight outputs, yank receipts if used}.
Replay: scratch-dir `cargo add <crate>
--dry-run` resolves against the registry.

## 5. UNVERIFIED details (falsified/confirmed by Stage-2 dry-runs)

Refresh 2026-11-04 (ZD-43/ZD-97): both §5 items remain UNVERIFIED
as-of-37721e2f/9582131 — the dry-run verify compiles in dev profile, so
doctest behavior and docs.rs rustc-args application are still watch
items; re-test at Stage 3.

1. Whether cargo strips path-only dev-deps at package time (expected
   yes — a dry-run on a crate with path-only dev-deps confirms).
2. Whether publish verification applies docs.rs `rustc-args`
   (expected no — doc tests gated on `#[cfg(docsrs)]` are watch items).

**Falsified by Stage 2**: the §1 license-file Stage-1 blocker (never
fired, 20/20); the 17/19/20 publish-census candidates (pinned at 21, §0).

## 5. UNVERIFIED details (falsified/confirmed by Stage-2 dry-runs)

1. Whether cargo strips path-only dev-deps at package time (expected
   yes — a dry-run on a crate with path-only dev-deps confirms).
2. Whether publish verification applies docs.rs `rustc-args`
   (expected no — doc tests gated on `#[cfg(docsrs)]` are watch items).
