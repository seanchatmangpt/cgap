# Lane E13 — C header struct-alignment court (F8 additive remainder)

**Standing: ALIVE** (commands + exits this session, cited below).
**Files owned/written:** `crates/cgap-ffi/tests/c_header_abi_court.rs` (new),
`docs/jira/v26.11.2/countermeasures/lane-e13-abi-court.md` (this receipt).
No git writes; shared tree untouched (pre-existing `M include/cgap.h` in the
session-start snapshot is not this lane's edit).

## Struct inventory (parsed from the frozen headers)

| Struct | Header | Fields (declaration order) | LP64 size/align |
|---|---|---|---|
| `cgap_frame_view` | include/cgap.h | payload, payload_len, schema_id, rule_count | 24 / 8 |
| `ArrowArray` | include/cgap_c_data.h | length, null_count, offset, n_buffers, n_children, buffers, children, dictionary, release, private_data | 80 / 8 |
| `ArrowSchema` | include/cgap_c_data.h | format, name, metadata, flags, n_children, children, dictionary, release, private_data | 72 / 8 |

## Assert inventory

1. **COURT 1** `header_inventory_matches_canonical_structs` — parsed header
   struct/field inventory equals the canonical inventory (order included).
2. **COURT 2** `rust_twins_match_parsed_header_layout` — per-field offsets
   from the parsed header equal `offset_of!` on the Rust twins
   (`cgap-ffi::sovereign::CgapFrameView`, `cgap-arrow::c_abi::{ArrowArray,
   ArrowSchema}`), plus size/alignment parity.
3. **Compile-time static-assert pair** — 31 `const _: () = assert!(...)` pins
   (size/align/offset for all three twins) at test-crate scope; any drift
   fails the COMPILE of the test crate itself, mirroring the header's C11
   `_Static_assert` pins.
4. **COURT 3** `c_compile_leg_flips_on_field_reorder` — anti-vacuity:
   `cc -std=c99 -ffreestanding` over a /tmp sandbox copy with a swapped
   ArrowArray field pair must FAIL; the pristine copy must compile clean
   (negative-array `offsetof` pins, CL1 gate0 pattern).
5. **COURT 4** `rust_mirror_leg_flips scored over the header's OWN declarators
   via `c_layout_from_decls` + `kind_of` (loud panic on unclassified
   declarator), must flip on the same mutation while the pristine header
   scores EQUAL (leg-wired proof).

## Commands + exits (this session)

- `CARGO_TARGET_DIR=/tmp/e13-target cargo test -p cgap-ffi --test c_header_abi_court`
  -> exit 0, **4 passed; 0 failed**.
- `CARGO_TARGET_DIR=/tmp/e13-target cargo test -p cgap-ffi` -> **exit 0**,
  68 tests total (50 lib + 4 c_header_abi_court + 3 fa9 + 2 lever + 5
  zero_copy + 0 doctest), F8's courts included and green.
- Standalone flip witness (sandbox /tmp/e13-witness-25556, deleted after):
  mutated header -> `MUTATED_CC_EXIT=1` with two negative-size pin errors
  (`pin_length_0`, `pin_null_8`); pristine restore -> baseline cc exit 0.

## Flip witness (anti-vacuity)

Mutation: swap ArrowArray fields 0 and 1 (`length` <-> `null_count`) in the
/tmp sandbox copy only. Effects witnessed:
- C-compile leg: `cc` exit 1, "declared as an array with a negative size" on
  the offsetof pins (real clang output above).
- Rust mirror leg: `c_layout_from_decls` over the mutated header's own
  declarators diverges from `offset_of!` twins -> court fails loudly.

## Divergences from the lane doc

1. ArrowArray/ArrowSchema live in `include/cgap_c_data.h` (ggen-generated),
   not `include/cgap.h`; cgap.h contributes `cgap_frame_view`. Court parses
   BOTH.
2. `cc -std=c99` freestanding is used for the flip leg (its pins are
   negative-array, C99-legal); the pin TU includes cgap.h too, but the
   header's own C11 `_Static_assert` pins are compiled for real in the
   pre-existing wave3 courts (`header_court_c11_compile_real_and_pins_fire_
   on_mutation`, green this session), so nothing is skipped.
3. `size_t` classified as 8-byte Ptr-kind in the LP64 layout model
   (`Kind::I64`); the court is an LP64 host court — pointer-width variants
   remain covered by the header's own `#elif UINTPTR_MAX` pin branches and
   the wave3 courts.
