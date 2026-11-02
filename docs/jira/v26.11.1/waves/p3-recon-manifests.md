# P3-1 RECON-MANIFEST Receipt

Lane: P3-1-recon · 2026-11-01 · repo /Users/sac/cgap · branch main

## Census (assessed on disk, pre-edit)

All 17 leased manifests scanned programmatically (python dep-line scan +
manual read). Findings:

- Every leased manifest already normalized: every intra-workspace path dep
  carries `version = "26.11.1"` alongside path. Path-deps-without-version
  count: **0**. Wrong-version count: **0**.
- `crates/integration` and `crates/wasm-footprint`: both already at literal
  `version = "26.11.1"` — the PUB-A1 prior session completed the bump before
  crashing. No bump needed.
- Root `Cargo.toml` `workspace.package.version = "26.11.1"` (line 39);
  all `version.workspace = true` crates resolve to 26.11.1.
- Frozen pins (license URN, SHACL IRIs) untouched.

## Fixes applied this session

1. **cgap-bn3 W2-9 crash residue removed.** `crates/cgap-bn3/src/noalloc.rs`
   (partial noalloc module, 1178 lines) failed to compile: 1 lib error
   (E0106 missing lifetime, noalloc.rs:417) + 4 test errors (E0382 borrow-of-
   moved at 849). Removed file + its `pub mod noalloc;` line from lib.rs.
   Removed content sha256 (recorded for later redo):
   `f7adfd11ddc553f6e2de832a54e6eaca57932698e45f389afddb1c957bf29000`
   Archive copy: `/tmp/noalloc_w2_9_residue.rs`.
   Post-removal `cargo test -p cgap-bn3`: 111 passed, 0 failed.
2. No manifest edits required — normalization was already complete.

## Dependency tiers (cargo metadata --no-deps, exit 0)

- Tier 0 (leaf, zero deps): cgap-blake3, cgap-gate, cgap-git, cgap-parquet,
  cgap-bosym, cgap-core (runtime), cgap-court (runtime; dev-heavy)
- Tier 1 (dep only on Tier 0): cgap-bont (blake3), cgap-bn3 (blake3),
  cgap-crypto (blake3), cgap-storage (blake3), cgap-gpu (arrow)
- Tier 2: cgap-p28 (blake3+crypto), cgap-audit (arrow+parquet)
- Tier 3 (membranes): cgap-ffi (arrow, blake3, bn3, core, gate, p28,
  parquet[std]), cgap-beam (arrow, blake3, git, parquet)
- Consumers: cgap (facade), bench, cgap-integration, cgap-fibo, cgap-cloud,
  wasm-footprint (standalone workspace)

## Publishability verdicts (cargo publish --dry-run --allow-dirty)

- cgap-court: exit 101 — **structural-unsupported**: `publish = false` in
  manifest; cargo refuses ("`package.publish` must be set to `true` or a
  non-empty list"). [[test]] target `court_verdict` pointing at
  `../../tests/court_verdict.rs` would additionally fail packaging (path
  outside package root) — unverified beyond the publish=false gate.
- cgap-beam: exit 101 — same `publish = false` gate. **BLOCKED-for-publish
  by policy field, not compile.**
- cgap-gpu: exit 0 — packaged 7 files, verified compile, aborted upload
  (dry-run). **PUBLISHABLE** (PROPOSED crate passes packaging cleanly;
  note it lacks the `publish = false` field the siblings carry).
- Prior-session state: PUB-A1's work was already landed (integration +
  wasm-footprint at 26.11.1, all path deps versioned). Crash residue was
  bn3-only.

## Commands witnessed

- cargo test -p cgap-bn3 (pre-removal): compile errors in noalloc.rs only
- rm noalloc.rs + lib.rs edit; cargo test -p cgap-bn3: 111 passed / 0 failed
- cargo publish --dry-run -p cgap-court --allow-dirty: exit 101
- cargo publish --dry-run -p cgap-beam --allow-dirty: exit 101
- cargo publish --dry-run -p cgap-gpu --allow-dirty: exit 0
- cargo metadata --no-deps --format-version 1: exit 0

## Standing

ALIVE (all claims backed by the commands above, this session).

## Divergences

- Task said integration + wasm-footprint "were 26.10.29" — on disk both were
  already 26.11.1 (prior session landed the bump). Assessed, not assumed.
- cgap-gpu has no `publish = false` while its PROPOSED siblings do — flagged,
  not repaired (do-NOT-repair instruction).
