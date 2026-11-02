# dod-cgap-core

EXP-02 read-only audit lane, crates/cgap-core, v26.11.2.
Only permitted write: this file. No source edits.

## Task 1 - allocator interface (tracking_alloc.rs)

- cgap-core/src/lib.rs:2-4 exports only dma_ring, frame, isomorphism.
- `tracking_alloc` is deliberately NOT a pub export. Evidence:
  - src/tracking_alloc.rs:2 (module doc, "AE7 ... countermeasure
    lane 07") and src/tracking_alloc.rs:14-18 (REGISTRATION DIVERGENCE:
    `#[global_allocator]` is deliberately left to each court binary;
    a lib-level registration would collide via rustc E0152 with the
    per-binary counting allocators already landed in cgap-gate/
    penstock.rs, cgap-parquet/rle.rs, cgap-bn3/unify.rs and six
    crates/cgap/tests/* files).
  - src/tracking_alloc.rs:41-42 (SOURCE-SCAN-EXEMPT comment: the
    module is "#[path]-included ... tests-only collaborator, never
    part of the shipped kernel surface").
  - crates/csct/src/bin/cscb_c4_alloc.rs:20-23: "cgap-core does not
    export `tracking_alloc` from lib.rs and lib.rs is off-limits to
    this lane" (lane-freeze, not a design rejection).
  - crates/cgap/tests/alloc_boundary.rs:50-53: "lib.rs is lane-frozen
    (off limits)"; include becomes plain `use cgap_core::tracking_alloc`
    if a later lane lands the pub mod.
- WHY #[path]: a combination of (a) intentional non-export per the
  module doc (tests-only collaborator) and (b) lane freeze on lib.rs
  during v26.11.2, which forced the work-around rather than an
  interface decision.
- Recommendation (no edit made): ship a pub export pre-release. The
  module is packaged into the .crate today (see Task 3 file list) but
  unusable by external consumers, so the shipped surface is dead code
  externally. Landing `pub mod tracking_alloc;` in lib.rs is safe: it
  does NOT add `#[global_allocator]` (no E0152 risk) and both
  downstream #[path] users already document the one-line migration.
  Coordinate with the SOURCE-SCAN-EXEMPT marker if lib.rs freeze
  lifts. P1, pre-release.

## Task 2 - zero-alloc invariants

- `grep -rEn '(Vec|String|Box)' crates/cgap-core/src/` -> exit 1
  (zero matches). No heap-allocating std types anywhere in core src;
  src uses `core::alloc`, `core::sync::atomic` only
  (src/tracking_alloc.rs:43-44).
- Steady-state courts corroborate: cscb_c4_alloc.rs arms a 0-alloc /
  0-byte budget around the ingress hot loop and passes (per its doc,
  crates/csct/src/bin/cscb_c4_alloc.rs:1-8).

## Task 3 - packaging (cargo package)

- Command: CARGO_TARGET_DIR=/tmp/exp02-target cargo package
  -p cgap-core --allow-dirty --no-verify
  (note: cargo has no --dry-run flag; --no-verify used as the dry-run
  equivalent, exit captured on a re-run.)
- EXIT: 0. Packaged 21 files, 167.6KiB (47.8KiB compressed).
- Warning (non-blocking): "manifest has no license or license-file"
  — crates/cgap-core/Cargo.toml has license text in a comment block
  (P28-SIL v2.1.0 URN) but no `license-file` key. Every sibling crate
  uses `license-file = "../../LICENSE.md"`; workspace root
  Cargo.toml:46 defines `license-file = "LICENSE.md"` and several
  crates use `license-file.workspace = true`.
- Metadata vs house style (docs/diataxis/how-to-guides/
  publish-a-crate.md):
  - description: one-liner present, complies (crates/cgap-core/
    Cargo.toml: "128-byte sovereign CgapFrameHeader codec ...").
  - publish: `publish = true` with P3-2b guard-flip receipt cited
    inline; note docs/jira/v26.11.2/waves/publish-halt.md exists —
    check the halt applies/does not apply before any real publish.
  - license-file: MISSING. Add `license-file = "../../LICENSE.md"`
    (or `.workspace = true`). P1, pre-publish.
- Packaged crate contains src/tracking_alloc.rs but it is not reachable
  via any public path — reinforces Task 1 recommendation.
- Cleanup note: `rm -rf /tmp/exp02-target` was denied twice by the
  permission system in this session; /tmp/exp02-target still exists
  and needs manual deletion.
