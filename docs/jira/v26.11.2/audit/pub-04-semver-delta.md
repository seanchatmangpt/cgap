# pub-04-semver-delta

PUB-04 semver delta audit (2026-11-02, read-only). Publish
baseline: five crates at 26.11.1 from SHA cfd085a9f (f3 ledger).
Workspace 26.11.2 (root Cargo.toml:47). Delta window
cfd085a9..HEAD. Cal-ver scheme: "major" = next version digit.

## Verdict table

| crate | last pub | semver class | needed bump |
|---|---|---|---|
| cgap-blake3 | 26.11.1 | fix-patch | 26.11.2 patch |
| cgap-arrow | 26.11.1 | BREAKING | major (or remedy) |
| cgap-git | 26.11.1 | additive-minor | 26.11.2 minor |
| cgap-parquet | 26.11.1 | additive-minor | 26.11.2 minor |
| cgap (umbrella) | 26.11.1 | additive-minor | 26.11.2 minor |

## cgap-blake3 — fix-patch

multi_chunk.rs only (24 lines): AArch64 NEON fix — vst1q_one
wrote through a pointer cast from a SHARED &[u32;4] (UB at
opt-level >= 1: LLVM forwarded the initializer, x2 output
dropped the ^ cv_pre term); replaced with mut arrays +
vst1q_u32 on as_mut_ptr. Correctness fix, zero API change.
+ chain_mutation_court.rs (GAP-02). Verdict: patch.

## cgap-arrow — BREAKING (the one real break)

Additive: new pub mod ipc (unconditional) + validate
(host-gated); default-off feature nested-types; Quad64 kept as
path-compat re-export (no type-move break).
THE BREAK: pub mod timestamps + TimestampMicro/Milli/Nano/Second
re-exports moved from #[cfg(not(target_os = "none"))] to
#[cfg(all(not(target_os = "none"), feature = "nested-types"))]
(lib.rs:80-81, :110-111). nested-types is default OFF (Cargo
comment: audit-surface exclusion). Effect: code compiling
against 26.11.1 default features loses cgap_arrow::Timestamp*
at 26.11.2. Removal of reachable pub API under default
features = BREAKING.
Remedies: (a) accept the break, cut arrow as the major;
(b) nested-types into default features for one cycle, remove
next major. Ledger must pick one before publish.

## cgap-git — additive-minor

15 new lib-mounted pub modules (lib.rs:146-162, FIX-06
"additive pub API"), all unconditional — pure additions,
nothing removed or signature-changed. Wrinkle: new enum variant
GitError::UnsupportedDeltaEncoding — technically breaking for
exhaustive matches, Rust-ecosystem-accepted minor (matches the
26.10.29->26.11.1 error-enum precedent); disclose in notes.
pack.rs #[path] facade: additive. [dependencies] stays empty
(dev-deps only). ~6,200 added src lines, all additive.

## cgap-parquet — additive-minor

mmap_writer std-regroup: RingSink/SpillSync/Sealed ALL
#[cfg(feature = "std")] (std feature pre-existing, default OFF).
Pre-existing pub surface byte-comparable at both SHAs.
Feature-gated additive, minor: zero ungated item changed, zero
removed. SpillSync sealed via crate-private supertrait — no
out-of-crate impl regression. Elsewhere: scan.rs Bounds/
rg_prunes/scan_u64_pred/write_fixture_file_with_bounds
(additive); bitslice semi_join adds (additive).

## cgap umbrella — additive-minor

lib.rs: pub mod pipeline + re-exports (EnclavePipeline,
PipelineError, PipelineStage, StageHandle) + host-only
re-exports behind NEW default-ON feature host. Additive.
Manifest: kernel pins 26.10.29 -> 26.11.2; optional deps
core/storage behind host; ffi dev-dep (host-test-only).
New default feature disclosed; surface only grows. Minor.

## cgap-core — not in the published set

Never published (26.11.1 set = blake3/arrow/git/parquet/
umbrella). tracking_alloc STILL UNEXPORTED (lib.rs mounts only
dma_ring/frame/isomorphism; grep no match) — deliberate per
module doc (E0152 collision avoidance; registration left to
court binaries). No semver action until first publish; then
decide tracking_alloc mount.

## Chain constraint

The umbrella pins all four kernels exact-style (=26.11.2 +
path) — the five publishes move together. Arrow's BREAKING is
the only item changing the version-shape decision.

## See Also

audit/pub-02-package-dryrun.md · waves/f3-publish-ledger.md ·
waves/p4-terminus.md
