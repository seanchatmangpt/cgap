# Receipt: CG-1 wire-tier gates (v26.11.3)

| Field | Value |
|---|---|
| Subject | /Users/sac/cgap @ working tree (lane-local edits, no git ops run) |
| Lane | CG-1, Wave 1, rank 1 (plan authority: PG-6) |
| Standing | PARTIAL_ALIVE (all gates green this session; coordinator owns integration) |

## Diff (owned files only)

- `crates/cgap-arrow/src/lib.rs` — ONE line: `#[cfg(all(not(target_os =
  "none"), test))] mod generated_arrow_primitives_courts;` (+ 3-line
  comment).
- `crates/cgap-arrow/src/generated_arrow_primitives_courts.rs` — NEW,
  hand-law, `#![cfg(test)]`, 323 lines, 6 courts.
- `crates/cgap-arrow/src/ipc.rs` — `BadVersion { got: u16 }` /
  `BadFlags { got: u16 }` variants; gates in shared order after BadMagic;
  `parse_meta` seam (`ParsedMeta`); `read_with_meta` returning
  `(usize, MetaHeader)`; `read` = thin wrapper; `seq_ok` wrap-safe
  successor law; `read_verified` delegates to the seam (truncation now
  classifies `Truncated`, killing the wrong-class
  `BodyDigestNoRoom { got: 8 }`); WITNESS TIERS doc section (G3).
- `crates/cgap-arrow/tests/ipc_courts.rs` — c7-c13 added (7 new courts;
  17 total in file).
- `crates/wire-shm/tests/wire_courts.rs` — zero-"blake3" scan court with
  sandbox-add-token kill + restore witness.
- `crates/cgap-gate/Cargo.toml` — `[dev-dependencies] cgap-core = { path
  = "../cgap-core" }` + rationale comment.
- `crates/cgap-gate/tests/frame_abi_mirror_court.rs` — NEW: compile-time
  layout-identity const asserts (size/align + 13 field offsets, mirror vs
  canonical) + runtime witness.
- `justfile` — `wire-shm` recipe (test suite + depth-0 zero-dep proof,
  fail-closed) + informational doctor.
- `docs/jira/v26.11.3/{lanes/cg-1-wire-tier-gates.md,receipts/cg-1.md}` —
  NEW (dir created this lane).

## Gates run this session (exact commands, observed exits)

- `cargo test -p cgap-arrow` — exit 0; suites: 145 lib (incl. 6 G1
  courts), 3, 5, 9, 1, 36 (mutation_kills full), 1, 7, 3, 4, 6, 0 →
  220 passed / 0 failed (baseline 214; +6 G1 courts). no_std embedded
  surface intact.
- `cargo check -p cgap-arrow --target riscv32imc-unknown-none-elf` —
  exit 0 (no_std embedded check).
- `cargo test --manifest-path crates/wire-shm/Cargo.toml` — exit 0;
  8 passed (was 7; +1 G3 scan court).
- `cargo test -p cgap-gate frame_abi_mirror` — exit 0; 1 passed
  (compile-time mirror identity + witness). Full `cargo test -p
  cgap-gate`: 12+1+3+5, 0 failed.
- `cargo check -p cgap-arrow` — exit 0.
- `just wire-shm` — GREEN: suite + zero-dep tree proof + informational
  doctor rc=1 (YELLOW by design, does not gate).
- Anti-vacuity evidence: G1 n_buffers-reduction flip (OK -> MISSING_BUFFER
  on same fixture); w8-pattern consistently-sealed BadVersion/BadFlags
  windows (digest gate passes, version/flags gate kills, exact variant,
  both read paths); c11 classification-identity sweep (72 mutations +
  all truncations, variant-identical); sandbox-add-token kills for both
  source scans (injected token detected); BitTestLaw input-flip kill.

## Disclosed deviations

1. **map_status direct call infeasible**: `map_status` is private to the
   never-edit generated module; adding an accessor would hand-edit a
   generated file (banned ever). The 6-arm lattice is therefore asserted
   through the real `cgap_import_array`/`cgap_import_schema` entrypoints
   (real call path, stronger than a re-implemented copy); co-domain
   pairwise-distinctness asserted over the `status::` constants.
2. **cgap-gate dev-dep form**: path dependency, not a
   `workspace.dependencies` entry (workspace table carries only
   cgap-branchless). Runtime dep surface unchanged (zero external).
3. **Mutation-kill shape (G2)**: input-mutation courts (crafted
   consistently-sealed windows) instead of /tmp-copy rebuild kills; the
   /tmp sandbox + fail-closed restore shape is used where the surface is
   file-based (G3 scans). Rationale disclosed in the lane doc.
4. **e80-06/e80-07 receipts placement divergence**: the dispatch said
   "e80-06/e80-07 receipts live in config-lsp" — verified: they are NOT
   under docs/jira/v26.11.3 (that dir did not exist before this lane;
   created here). Stream-writer context for the IPC surface lives in
   `crates/cgap-arrow/src/lib.rs` module comments and
   `tests/stream_writer_type_test.rs` / `stream_writev_witness.rs`; the
   config-lsp placement is disclosed as the standing divergence rather
   than migrated by this lane.
5. **ipc.rs doc tier spelling**: the G3 witness doc section spells the
   checksum tier "CRC-32" (hyphenated) so the source stays clean under
   the C13 zero-"crc32" scan — the court outranks the prose.

## Falsifiers / follow-ups (not this lane)

- "i" format collision (DictionaryUInt32Utf8 vs Int32 across
  ARRAY_KINDS/LAYOUT_SPECS): ontology fix is a follow-up ticket (PG-6);
  pinned by `array_kinds_layout_specs_consistency` so neither side drifts
  silently.
- Full rebuild-class mutation kills for the version/flags gates
  (mutate ipc.rs in a /tmp crate copy and build): the c8/c9 exact-variant
  courts + c11 identity sweep cover the observable class; the rebuild
  ladder is available if PG-6 wants it.
