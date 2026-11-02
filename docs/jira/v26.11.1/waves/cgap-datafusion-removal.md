# cgap-datafusion removal

Status: REMOVED (coordinator-executed 2026-11-02, per operator order:
"no datafusion code or library imports").

- crates/cgap-datafusion/ — deleted (shutil.rmtree; included CD-1's
  scaffold, CD-3's in-flight listing.rs/predicate.rs, CD-2's stopped
  work).
- root Cargo.toml member line + comment — removed.
- Cargo.lock — pruned via scoped `cargo check -p cgap-blake3`;
  0 datafusion entries remain.
- Verified: cargo metadata --no-deps exit 0, 22 workspace members,
  `datafusion present: False`; grep over crates/ *.toml + *.rs → zero
  references.
- CD-1/CD-2/CD-3/CD-4 lanes stopped pre-removal (CD-1's landed spike is
  superseded by this removal; its courts were green at its own state —
  recorded in waves/cd-1-scaffold.md, retained as history).
- The F3-14 consumer / DIA-3 reference crate-index rows mentioning
  cgap-datafusion are superseded by this receipt (DIA-3's index predates
  the crate's existence; no in-tree reference remains).
