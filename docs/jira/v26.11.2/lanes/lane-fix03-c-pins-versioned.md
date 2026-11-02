# Lane FIX-03 receipt — Arrow C-ABI offset pins, versioned (G19/EXP-08 court-persistence repair)

Standing: **ALIVE** (commands + exits run this session, exact subject).

## Subject

- NEW `crates/cgap-ffi/tests/arrow_offset_pins.rs` — Rust-side pin court + C-side
  compile court + two anti-vacuity flip courts.
- This receipt.

## What the court pins (23-pin G19 set, Arrow remainder)

From `docs/jira/v26.11.2/audit/dod-cgap-ffi.md` (G19/EXP-08): 10 ArrowArray/
ArrowSchema LP64 pins lived only in `/tmp/g19_offsets_check.c` (witnessed still
on disk this session, 1876 B, NOT version-controlled). Both sides now versioned:

- **Rust side** (`crates/cgap-arrow/src/c_abi.rs:64-91`, `#[repr(C)]`): compile-time
  static asserts — `ArrowArray` size 80 B, `ArrowSchema` size 72 B, all 10/9 field
  offsets (4-arg `core::mem::offset_of!`) — plus a runtime mirror asserting the
  declaration-order contract (offset == i*8 per field, in order).
- **C side**: the G19 pin block (verbatim port, minus the CgapStatus pin already
  covered by the in-tree `c_header_abi_court`) compiled with `cc -std=c11 -I include/`
  against the **versioned header `include/cgap_c_data.h`** — the Arrow structs ARE
  declared there (`cgap_c_data.h:20-50`). `include/cgap.h` carries NO Arrow
  declarations; documented, per dispatch.

## Anti-vacuity (both legs flip, witnessed)

1. `c_pin_flip_on_header_mutation`: a /tmp COPY of `cgap_c_data.h` with
   length<->null_count swapped (placeholder-based swap, no-op-checked) must FAIL
   the same pin block; the mutated include dir is the only -I path, and the test
   asserts `cgap.h` does not shadow. Shared header untouched.
2. `rust_pin_flip_on_wrong_pin`: a rustc twin crate that `#[path]`-includes the
   REAL `c_abi.rs` (plus bitmap/buffer/primitive siblings) with a deliberately
   wrong pin (`size_of::<ArrowArray>() == 79`) must FAIL to compile; a control
   twin identical except a true pin (`ArrowSchema == 72`) must compile clean —
   failure is the pin, not the scaffold.
3. Both wrong-compile outcomes assert `!status.success()` — a court that cannot
   flip is FAIL.

## Commands + exits (this session)

```
cargo build -p cgap-ffi --tests   # via CARGO_TARGET_DIR=/tmp/fix03-target — clean
cargo test -p cgap-ffi --test arrow_offset_pins
  -> test result: ok. 4 passed; 0 failed; 0 ignored  (exit 0)
```

Tests: `rust_offset_pins`, `c_header_pin_compiles`, `c_pin_flip_on_header_mutation`,
`rust_pin_flip_on_wrong_pin` — all ok.

## Fences (must-not-regress)

- `crates/cgap-ffi/tests/c_header_abi_court.rs` (the existing 13-pin header court)
  shares the include/ surface — rerun guard: `cargo build -p cgap-ffi --tests`
  compiled all ffi tests including it; its own run is owned by its lane, and this
  lane added no lib/ code, so no behavior fence can regress. Full ffi suite not
  re-run here (out of lane scope); recorded as UNVERIFIED for the sibling tests.
- No shared `lib.rs`/`Cargo.toml` edits. No git commands.

## Divergences / notes

- Audit doc lives at `docs/jira/v26.11.2/audit/dod-cgap-ffi.md` (dispatch said
  `audit/dod-cgap-ffi.md` repo-relative; same file).
- The Rust side DID lack its own offset pins (G19 note confirmed: pins lived in
  the C file only); `c_header_abi_court.rs` computed offsets at runtime against a
  parsed header mirror — this lane adds the direct static-pin layer.
- Sandbox law: /tmp copies mutated only; no shared-tree mutation; build root
  `/tmp/fix03-target` is a lease, deleted at lane end.
