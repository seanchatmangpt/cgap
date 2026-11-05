# zd-103: cgap-ffi aoq_sidecar warning fix-forward

Standing: ALIVE

Lane zd-103 (fan-out wave, repo `/Users/sac/cgap` @ b6d68517). Ownership:
`crates/cgap-ffi/src/bin/aoq_sidecar.rs` (warning sites only) + this receipt.
XIP untouched; zero Cargo.toml edits.

## Baseline (zd-102 noted 2 pre-existing warnings)

`cargo check -p cgap-ffi` at b6d68517 flagged in the owned bin:

- `PAD_LO` / `PAD_HI` never used (`aoq_sidecar.rs:135-136`) — dead pad-law
  constants; a full-repo grep found zero references (source + tests). Deleted
  both with their doc comment (ERRC: delete outranks generate).
- Cascade: removing them orphaned the `AOQ1_TAG_AT` import (line 58) —
  removed from the `cgap_arrow::ipc` use list.

## Additional clippy lints in the owned file (--all-targets sweep)

- `write_vectored(&mut pieces)` → `&pieces` (clippy::unnecessary_mut_passed);
  dropped the now-unneeded `let mut` on `pieces`.
- `u16::from(EPOCH)` → `EPOCH` (`EPOCH: u16` already; clippy::useless_conversion).

## Verification (ALIVE, this session, exact subject)

- `cargo check -p cgap-ffi` → 0 warning sites in `aoq_sidecar.rs`
  (crate-wide check output still cites cgap-crypto warnings — out of lane).
- `cargo clippy -p cgap-ffi --all-targets` → 0 mentions of `aoq_sidecar.rs`;
  crate-wide `grep -c "^warning"` = **200** remaining, all outside lane
  ownership, typed: sovoreign.rs 20, ffi_stress.rs 5,
  aoq_session_ffi_court.rs 5, receipt_chain_scale.rs 4, xip_tables.rs 4,
  c_header_abi_court.rs 3, oabi_lookup.rs 3, arrow_offset_pins.rs 2,
  aoq_sidecar_court.rs 2, receipt_chain.rs 2, lib.rs 2,
  zero_copy/fa9_commit_gating/aoq_sidecar_e2e_court/alloc_courts 1 each.
  Recorded, not rounded to 0.
- `cargo test -p cgap-ffi --test aoq_sidecar_court --test aoq_sidecar_e2e_court`
  → 3 passed + 3 passed, 0 failed (courts unregressed).

## Diff

`crates/cgap-ffi/src/bin/aoq_sidecar.rs`: -7 lines constants/import,
2 lint fixes. No other file touched.
