# Lane EX4 — cgap-arrow part 1 (types/ABI) exhaustive census

Standing: PARTIAL_ALIVE (read-only census; grep/sed runs, exit 0, 2026-11-02).
Dispatch-named ABSENT files (loud): `schema.rs`, `types.rs` — do NOT exist in
`crates/cgap-arrow/src/`. Real type/ABI surface covered instead.

## c_abi.rs (284 L) — C Data Interface export/import, HOT (FFI boundary)

Fns: `release_slot_swap` (cfg-split ptr-atomic / no-RMW, 41–53), `release_schema`
(95), `release_array::<T>` (122, atomic swap-to-NULL first-winner), `export::<T>`
(145), `ForeignArray::{take, len, is_empty, buffer}` + `Drop` (199–247).
Branches: 1 match (validity Some/None at 149), release-path null guards (2 ifs),
Drop null guard. Hot: export/release; dense-match census: the single match is
2-arm Option — not a dispatch table; rustc lowers as branch, not jump-table
[ANALYTIC]. No dense-match jump-table candidates in this file. Cold: Drop
(error path only). Test bodies at 249–283 (2 tests).

## c_abi_nested.rs (521 L) — struct-of-children export (W3-NESTED)

Fns: `release_swap` (62, same cfg-split), `release_struct_array` (82, cascade),
`release_struct_schema` (118), `export_struct` (162). Branches: release-slot
null guards ×2, child-cascade loop bounds, `export_struct` child-walk.
cfg gates: only `target_has_atomic = "ptr"` split (63/67) — NO cargo feature
`nested-types` gate exists in-file.
Gated state: A2P-era prune-candidate per dispatch, but LIVE consumer edges:
`crates/integration/tests/membranes.rs`, `crates/cgap-ffi/src/sovereign.rs`
(grep -l, exit 0). Not prunable this wave.
Hot: `export_struct`/cascade on nested export; cold: double-release paths.
Tests 225–520: 5 tests incl. bitwise ABI walk + counting-release cascade.
Divergence: dispatch said "nested-types feature"; reality = atomic-target cfg
split only, prune-gating is doc-level (module header), not cfg.

## foreign.rs (255 L) — import-side views, HOT (untrusted boundary)

Fns: `ForeignError` (24) + Display (61), `format_of` (85), `primitive_view::<T>`
(102), `is_valid` (153). Branches: 12 guard ifs — null schema, format-code
equality, n_buffers≥2, negative length, byte-len short-buffer, null slot,
alignment (`is_multiple_of`), validity-side n_buffers≥1/len<0/bitmap-len/
row-bounds. All failures COLD typed `ForeignError` refusals, keep-branchy
(BR3-compliant: refuse-early, no branchless gambling on hostile input).
Tests 181–253: 3 tests (round-trip, format-mismatch, short-buffer refusals).

## validate.rs (155 L) — REP5-era fixed-width boundary checks

Fns: `ValidateError` (13) + Display (28), `FixedWidthColumn` (49),
`validate_fixed_width` (57). Branches: column-count guard, 8-byte
`align_offset` guard, `saturating_mul` overflow guard. All COLD typed refusals
(length overflow / misalignment without dereference) — keep-branchy per BR3;
branchless rewrite would be wrong here (checks are off the hot loop, once per
column). Tests 79–154: 5 tests incl. misalign-without-deref and display law.

## layout.rs (345 L) — Quad64 256B? no: 4-quad quad-lexicon layout

Fns: `Quad64` (36; s/p/o/g TermId ×4, t_ns u64, vector_clock u32,
policy_mask u32, inline_digest [u8;32] = 0x20..0x40), impl fns: `is_bottom`,
`is_wildcard`, `term_matches`, `matches`, `to_le_bytes`, `from_le_bytes`
(122–167). `term_matches`/`matches` documented BRANCHLESS (spec §1.1 wildcard
sentinel compare) — HOT path, correctly branch-free. `to/from_le_bytes` HOT
(wire encode/decode). `from_le_bytes` returns Option (1 branch at boundary).
Tests 185–345: 13 tests (size/alignment/offset laws, LE round-trip, sentinel
predicates). Sentinel constants pinned in-test (bottom/wildcard).

## primitive.rs (238 L) — Scalar column kernel

Fns: `Scalar::write_le` macro-generated per-type impls (21–48), `Primitive<T>`
(50): len/is_empty/format_code/values/validity/values_ptr/validity_ptr/
buffers_aligned_64 (105, alignment law), `PrimitiveBuilder<T>` (130):
new/push/finish + Default. Branches: validity match (97, 2-arm Option),
`push` on `Option<T>` match (154). HOT: push/finish/values accessors;
`buffers_aligned_64` cold-ish (assertion-style). Tests 194–237: 3 tests.

## Coverage ledger (100% of dispatch scope)

| file | status | fns | branches | hot/cold |
|---|---|---|---|---|
| c_abi.rs | PRESENT | 8 | ~6 | export/release HOT, Drop COLD |
| c_abi_nested.rs | PRESENT | 4 | ~6 | export/cascade HOT, double-release COLD |
| foreign.rs | PRESENT | 4 | 12 guards | guards COLD typed refusals (BR3) |
| validate.rs | PRESENT | 3 | 3 guards | COLD typed refusals (BR3) |
| layout.rs | 345 L | 8 | branchless hot | matches/LE codecs HOT |
| primitive.rs | 238 L | 12 | 2 matches | push/finish HOT |
| schema.rs | ABSENT (loud) | — | — | — |
| types.rs | ABSENT (loud) | — | — | — |

Divergences: (1) schema.rs/types.rs absent — covered real files instead;
(2) c_abi_nested.rs has no `nested-types` cargo feature gate — only
`target_has_atomic="ptr"` cfg split; prune-gating is doc-level, with live
consumers (membranes.rs, sovereign.rs) blocking pruning.
No builds run; read-only lane; shared tree untouched.
