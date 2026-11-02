# R13 — Quad64 Dedupe (cgap-arrow pre-publish cleanup)

Standing: ALIVE — dedupe landed, 178 passed / 0 failed, exactly one
`struct Quad64` (layout.rs:36), transpose.rs re-exports via
`pub use crate::layout::Quad64;`. (Standing line added coordinator-side
2026-11-02: the landing receipt omitted the field and read RED-on-form
in R3's gate poll despite green content.)

Lane: R13-quad64 · Date: 2026-11-01 · Lease:
`crates/cgap-arrow/src/transpose.rs` + this receipt.

## Change

Verified duplicate `Quad64` deleted. Canonical definition:
`crates/cgap-arrow/src/layout.rs:36` (`pub`, `#[repr(C, align(64))]`,
derives `Clone, Copy, Debug, PartialEq, Eq, Hash`).

`crates/cgap-arrow/src/transpose.rs`:

- Deleted duplicate `pub struct Quad64` (was transpose.rs:26) and its
  local size/align const-assert block (layout.rs already asserts size 64,
  align 64, all field offsets 0x00-0x20, `[Quad64; 4] == 256`).
- Replaced with `pub use crate::layout::Quad64;` — `pub use` (not private
  `use`) because `crates/bench/src/bin/{cpk_study,ctq_report}.rs` reference
  `cgap_arrow::transpose::Quad64` by path (cpk_study.rs:356,362;
  ctq_report.rs:231,237). Path preserved; no consumer import fix needed.
- Module doc updated: removed the stale "local type, self-contained by
  design, coordination point when layout lane lands" paragraph.
- Field types widen `u32` -> `TermId` (= `u32` alias, same layout);
  derives are a superset (adds `Hash`).

## Verification

- `grep -c "struct Quad64" crates/cgap-arrow/src/*.rs` → exactly 1
  (layout.rs only; transpose.rs = 0).
- `CARGO_TARGET_DIR=/tmp/r13-target cargo test -p cgap-arrow` → exit 0.
  Counts: 134 unit (lib) + 1 + 36 (mutation_kills) + 1 (property) +
  6 (validation) + 0 doc = **178 passed, 0 failed**.
- `CARGO_TARGET_DIR=/tmp/r13-target cargo check -p bench --bins` → exit 0
  (cpk_study + ctq_report compile against the re-export).

No court mutations run: change is a pure dedupe; the crate's own
mutation_kills suite ran green inside the test gate above.

## Cleanup

`rm -rf /tmp/r13-target` DENIED by the permission system (2 attempts) —
the lane build root is still on disk at `/tmp/r13-target` (~debug build
debris). Coordinator must delete it at integration (lane build root =
lease, cleanup law).
