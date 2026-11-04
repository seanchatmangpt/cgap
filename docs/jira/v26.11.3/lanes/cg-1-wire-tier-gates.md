# CG-1 lane doc — wire-tier gates (G7 G1 G2 G4 G3 G6 G5)
> v26.11.3, implementation lane CG-1, Wave 1, rank 1. Plan authority: PG-6
> (contract binding, decisions embedded in the dispatch contract).

## Scope executed (PG-6 dependency order)

- **G7** — `just wire-shm`: real test suite + `cargo tree --depth 0`
  zero-dep proof (depth-0 output MUST be exactly one line, the
  `wire-shm` root; any dep edge shows at depth 1+ and RED-refuses).
  `just doctor` rides along INFORMATIONAL (rc printed, never gates the
  recipe). Witnessed GREEN 2026-11-03.
- **G1** — module stays PRIVATE (PG-6 decision). New
  `crates/cgap-arrow/src/generated_arrow_primitives_courts.rs`
  (hand-law, `#![cfg(test)]`), wired by ONE line in `lib.rs`
  (`#[cfg(all(not(target_os = "none"), test))] mod ...`). Courts:
  status lattice pairwise-distinct (co-domain), per-variant mapping
  through the REAL `cgap_import_array`/`cgap_import_schema` entrypoints,
  every branch (null array/schema/format -> INVALID_FORMAT, "q" ->
  FORMAT_MISMATCH, length -1 -> NEGATIVE_LENGTH, n_buffers-short and
  null-slot -> MISSING_BUFFER both sites, +8 addr -> MISALIGNED, valid
  DenseUnionArray ("tu") -> OK with the n_buffers-reduction anti-vacuity
  flip), BUFFER_TOO_SHORT unreachability pinned in the module doc,
  ARRAY_KINDS<->LAYOUT_SPECS consistency for the 3 unambiguous kinds +
  the "i" collision PINNED as named assert (ontology follow-up, not
  CG-1), BitTestLaw kill court, table-shape anti-vacuity assert. 6/6
  green.
- **G2** — `ipc.rs`: typed `BadVersion { got }` / `BadFlags { got: u16 }`
  (integer fields, no String, no_std-safe) checked in the shared gate
  order right after BadMagic. New `read_with_meta` returning
  `(usize, MetaHeader)`; `read` is now the thin wrapper
  (`read_with_meta(...).map(|(n, _)| n)`). New `seq_ok(prev, next)`:
  wrap-safe strict-advance successor law (`next == prev.wrapping_add(1)`),
  seq rides `MetaHeader.seq`. Courts c7-c9: wrap law incl. u32::MAX->0;
  consistently-sealed foreign-version window -> `BadVersion { got: 2 }`
  through read AND read_verified (w8 pattern: header mutated AND FNV-1a
  seal recomputed, so the death is the version gate, never the digest
  gate; version-1 twin control passes); flags 0x0001 consistent-seal ->
  `BadFlags { got: 1 }` both paths. Mutation-kill shape disclosed: the
  /tmp sandbox-copy sha256-restore court shape is used where the surface
  is file-based (G3 scan courts below); the version/flags kills are
  input-mutation courts (the crafted consistently-sealed window IS the
  mutation), which is stronger than a rebuild-kill for gate-order proof.
- **G4** — `parse_meta` seam extracted; `read`/`read_with_meta` and
  `read_verified` ALL delegate. Alignment!=64 -> `BodyDigestNoRoom` now
  answered from the seam (post-parse), so a truncated window classifies
  `Truncated { needed, got }` — killing the pre-seam wrong-class
  `BodyDigestNoRoom { got: 8 }`. Courts c10-c12: seq through
  read_with_meta, N-window classification-identity anti-divergence sweep
  (72 byte mutations + every truncation; read and read_verified must
  return the same VARIANT on every read-reject), exact-class truncation
  pin (needed 192, got 100, both paths).
- **G3** — NO shared digest seam (PG-6 decision). Witness tiers section
  added to `ipc.rs` module doc (checksum tier = transport/accidental;
  blake3 = tamper tier; plain read is NOT a verifying tier — cites the
  module's BODY INTEGRITY CONTRACT). Anti-confusion scan courts:
  `ipc_courts.rs` C13 zero-"crc32" scan over ipc.rs; `wire_courts.rs`
  zero-"blake3" scan over wire-shm sources; each with a sandbox-add-token
  anti-vacuity kill (same scan over a /tmp copy with the token injected
  MUST flip) and a fail-closed restore witness (repo files only read,
  byte-exact before/after). Known vector
  `crc32_ieee(b"123456789") == 0xCBF4_3926` asserted in
  `wire_courts.rs::crc32_known_vectors` (pre-existing, re-witnessed).
- **G6** — `cgap-gate` dev-dep on cgap-core (path dev-dependency,
  disclosed: NOT a workspace.dependencies entry — the workspace table
  carries only cgap-branchless, so the path form is the lawful minimal
  edit) + `tests/frame_abi_mirror_court.rs`: compile-time const asserts
  size/align equality (128B/64, 80B/8) + per-field `offset_of!` equality
  (7 CgapFrameHeader fields, 6 P28 fields) mirror vs canonical, plus a
  runtime witness test so the court reports in `cargo test`. The Wave-1
  "off-limits Cargo.toml" deferral is discharged.
- **G5** — runbook note below.

## Runbook note (G5): mutation_kills 1/36 filter

Observed during this lane: an invocation of `cargo test -p cgap-arrow`
with a test-name filter (e.g. the G1 court-module filter used while
landing G1) applies the filter to EVERY integration-test binary, so
`tests/mutation_kills.rs` reports `1 passed; 35 filtered out` — an
artifact of cargo's per-binary filter semantics, NOT test loss. The full
unfiltered suite runs 36/36:

    cargo test -p cgap-arrow --test mutation_kills
    # witnessed 2026-11-03: 36 passed; 0 failed; 0 filtered

## Receipt

See `receipts/cg-1.md`.
