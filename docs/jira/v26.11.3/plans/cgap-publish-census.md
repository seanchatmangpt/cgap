# cgap crates.io Publish Census (XB-2)

Date: 2026-11-04. Read from disk at main post-sweep; crates.io published-state
UNKNOWN (network-blocked sandbox) — flagged, not proven.

## Ladder — 21 publish=true crates (not 19), 4 dependency tiers

- **L0**: cgap-arrow, cgap-blake3, cgap-bosym, cgap-branchless, cgap-aoq,
  cgap-core, cgap-git, cgap-parquet
- **L1**: cgap-bont, cgap-bn3, cgap-storage, cgap-crypto, cgap-gate,
  cgap-audit, cgap-gpu, cgap-fibo
- **L2**: cgap-p28, cgap-cloud, cgap-ffi
- **L3**: cgap
- **Registry-tier** (after L0-L2 land 26.11.3): cgap-integration
  (pins `"26.11.2"` for arrow/blake3/parquet/git — must exist on the
  registry), wasm-footprint (standalone, pins `"26.11.3"`)
- **Standalone**: wire-shm (publish=false per YD-3), cgap-mcp
  (publish=false per YD-3)

## Per-crate flags

| Crate | Flag |
|---|---|
| cgap-branchless | version/edition hardcoded (26.11.3 consistent, not inherited) |
| cgap-fibo | RECEIPT.md ships in tarball (no exclude — cgap-cloud excludes its own) |
| cgap-integration | registry pins `26.11.2` unverified to exist |
| cgap-gpu | build-dep cc; Metal — docs.rs cross-compile risk only |
| all L0-L3 | license-file outside pkg dir (../../LICENSE.md) — confirm in first dry-run |
| all | no `readme = ...` key anywhere (crates.io pages description-only) |
| cgap-mcp | standalone workspace (empty [workspace], not root member); check from crate dir |
| wire-shm | name-collision risk on crates.io (unverified) |

No `=` exact pins to siblings; all inter-crate reqs `"26.11.3"` (caret).

## Dry-run signals

`cargo publish --dry-run` per crate in ladder order; watch for
license-file-not-included warnings, E0583 (include lists), and
version-req resolution failures on the registry tier.
