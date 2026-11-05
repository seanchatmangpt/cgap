# L36 — cgap fmt-clippy CI fence (honest partial landing)

Standing: PARTIAL

## Subject
Repo /Users/sac/cgap @ HEAD 11f8640b (pre-existing local mods: cgap-beam tests by L19,
KNOWN_DIVERGENCES.md/bm6 receipt by other lanes, untracked l27 files — none staged).

## Diff (μ)
Mechanical `cargo fmt` over the workspace EXCLUDING cgap-beam (L19 surface, in-flight:
`crates/cgap-beam/tests/aq_slab_contract.rs` + `bridge_contract.rs` carried uncommitted
L19 edits — excluded per lane contract). 90 source files reformatted (fmt census was
94 non-beam files; per-package `cargo fmt -p <pkg>` loop, bench + integration via
their own manifests; `crates/integration` is not a workspace member for cargo-fmt
purposes — formatted with `cd crates/integration && cargo fmt`).

cgap-beam: 2 files (src/aq_slab.rs, tests/aq_slab_contract.rs) still fail
`cargo fmt --check` — left for L19, named exclusion.

## Commands / exits
- `cargo fmt --check` census: 96 files non-conforming pre-fmt (94 non-beam).
- post-fmt: 0 non-beam failures; only the 2 excluded beam files remain.
- `cargo clippy --workspace --all-targets`: exit nonzero-ish (2 hard errors).
  - 2 × E0599 (`no method named expect` on `&[u8]`) in cgap-integration test
    `cross_crate` — PRE-EXISTING: `crates/integration/tests/cross_crate.rs` was NOT
    touched by this lane's fmt (only admission_e2e.rs in that crate was).
  - ~213 unique clippy warning sites across 74 files. Per-crate (approx, warning
    sites): cgap-git 52, cgap-arrow 25, cgap-crypto 23, cgap-parquet 17, cgap-ffi 14,
    cgap-storage 12, cgap-bn3 12, cgap-p28 8, cgap-blake3 8, cgap-aoq 5, cgap-core 3,
    cgap-branchless 2, cgap-bont 2, cgap 2, audit 2, integration 1, bench 1,
    ontology-packs 4.
- Corridor `cargo test -p cgap-core -p cgap-arrow -p cgap-blake3`: **387 passed,
  0 failed** — fmt broke nothing.

## Named follow-up (registered)
1. Clippy cleanup: ~213 warning sites / 74 files + the 2 pre-existing E0599 errors in
   `crates/integration/tests/cross_crate.rs` (`schema.release.expect` — release type
   moved to `&[u8]` under the test; substrate moved under it, test needs re-pinning).
   This is the gate blocking the CI flip.
2. cgap-beam fmt convergence (2 files) once L19 lands.
3. Then flip `.github/workflows/ci.yml` fmt-clippy job to blocking.

## CI flip
DEFERRED — clippy is not clean, so per contract the fmt-clippy job stays non-blocking;
fmt convergence landed without it.

## Replay
```
git show <this-sha> --stat   # 91 files (90 fmt + receipt)
cargo fmt --check            # clean except cgap-beam aq_slab{.rs,_contract.rs}
cargo test -p cgap-core -p cgap-arrow -p cgap-blake3   # 387/0
```
