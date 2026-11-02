---
paths:
  - "crates/cli/**"
---

# CLI crate is ggen-generated: never hand-edit generated surfaces

`crates/cli` is a STANDALONE ggen crate (own `[workspace]`, NOT a root
workspace member). Generated surfaces regenerate via the graph; hand edits
to them are clobbered at the next sync and read as drift, not intent.

- Never hand-edit: `src/{main,lib,generated_cli,generated_behavior,
  generated_boundaries}.rs`, `tests/zero_code_cli.rs`,
  `docs/command-reference.md`, `ggen.lock`.
- Custom behavior lives ONLY in `src/custom_handlers.rs` (the
  cnv:CustomBehavior seam, FM-WRITE-008 single-writer-per-target).
- Regen (byte-identical replay): `cd crates/cli && rm Cargo.toml && ggen
  sync run && python3 manifest_patch.py`.

Witnessed: `docs/jira/v26.11.1/waves/cli-2-handlers.md` (handlers landed
solely via `custom_handlers*`, every verb smoke-run exit 0).
