# crates.io Ladder Plan — cgap workspace (v26.11.3)

Date: 2026-11-04. Source: YP-1 plan agent, facts read from disk. Standing:
UNVERIFIED as a plan — nothing published, no dry-run run.

## 0. Measured baseline

- **17 crates with `publish = true`** in the root workspace (YP-1 count;
  XB-2 counted 21 including ontology-packs/audit/gpu/integration —
  reconcile by re-reading manifests at execution time; both agree the
  non-members wire-shm/cgap-mcp/fleet-ops/ocel2/verification/cli/csct/
  bench/court/beam are excluded; cgap-court and cgap-beam are explicit
  `publish = false`).
- No CI publish jobs exist in either repo (zero CRATES_TOKEN /
  `cargo publish` matches in cgap and ggen workflows) → **local publish
  from the maintainer machine is the precedent**; `cargo login` once.
- Workspace inheritance: `version/edition/rust-version/repository/authors`
  via `.workspace = true`; cargo inlines workspace syntax into the
  packaged manifest (cargo ≥1.64 behavior). `license-file` is NOT
  inherited — each member declares `license-file = "../../LICENSE.md"`
  literally, pointing **outside the package root** — the primary
  publish blocker.

## 1. License-file blocker (Stage 1 fix)

Cargo requires license-file inside the package root; `../..` escapes are
a packaging error class (cargo#9432). SPDX expressions are **banned** by
the root manifest's P28-SIL comment (license identified by URN §9, no
SPDX key) — so the fix is NOT `license = "..."`.

**Fix**: copy the root LICENSE.md (sha256 recorded at copy time) into
each publish=true crate dir + set `license-file = "LICENSE.md"` per
crate. One atomic commit over N manifests + N copies, coordinator-owned.
`license-file.workspace = true` resolves relative to each member root —
equivalent; prefer explicit per-crate literal.

## 2. Command sequence

```bash
# Stage 0 — preflight
cargo login  # once
for c in <crates>; do curl -s https://crates.io/api/v1/crates/$c | \
  jq -r '.versions[]?.num' | head -3; done   # name/version-collision check

# Stage 1 — license-file fix (atomic commit)
# Stage 2 — dry-run ladder (no registry deps needed; builds package + doc tests)
for c in $(topo order via cargo tree); do cargo publish -p $c --dry-run; done
# 17/17 green = admission gate for Stage 3

# Stage 3 — real ladder, topo order, gated per no-sleep law:
# after each publish, BOUNDED POLL (not sleep) on
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
URL, dry-run exits ×17, preflight outputs, yank receipts if used,
license-fix commit SHA}. Replay: scratch-dir `cargo add <crate>
--dry-run` resolves against the registry.

## 5. UNVERIFIED details (falsified/confirmed by Stage-2 dry-runs)

1. Whether cargo strips path-only dev-deps at package time (expected
   yes — a dry-run on a crate with path-only dev-deps confirms).
2. Whether publish verification applies docs.rs `rustc-args`
   (expected no — doc tests gated on `#[cfg(docsrs)]` are watch items).
