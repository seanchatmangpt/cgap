# Sovereign C-ABI

Authoritative source: `crates/cgap-ffi/src/sovereign.rs`,
C header `include/cgap.h`. All line numbers witnessed 2026-11-01.

## ABI version

- `CGAP_ABI_VERSION = 1u` — `include/cgap.h:47`;
  Rust twin `crates/cgap-ffi/src/sovereign.rs:55`
  (`const _: () = assert!(CGAP_ABI_VERSION == 1)` at :57).

## Four sovereign entrypoints

Rust-side frozen fn-pointer signatures (sovereign.rs): `CgapAdmitFrameFn`
(:168), `CgapEvalBn3Fn` (:181), `CgapCommitParquetFn` (:192),
`CgapVerifyP28Fn` (:201). Exported `#[no_mangle]` bodies:
`cgap_verify_p28` (:292), `cgap_admit_frame` (:412), `cgap_eval_bn3`
(:521), `cgap_commit_parquet` (:674).

| Entrypoint | Signature | Semantics |
|---|---|---|
| `cgap_admit_frame(bytes: *const u8, len: usize, out_view: *mut CgapFrameView) -> CgapStatus` | sovereign.rs:168, body :412 | magic/format/SHACL gate; `out_view` may be NULL (probe-only); `out_view.payload` borrows caller bytes, zero copy |
| `cgap_eval_bn3(view: *const CgapFrameView, bn3_rodata: *const u8, bn3_len: usize, out_lanes: *mut i32, lanes_cap: usize, out_lane_count: *mut usize) -> CgapStatus` | :181, body :521 | deterministic eval against flash-resident `.bn3` network; only copy is the `out_lanes` write-out |
| `cgap_commit_parquet(lanes: *const i32, lane_count: usize, path: *const c_char) -> CgapStatus` | :192, body :674 | Parquet commit; host targets only (omitted by cfg on XIP/no_std) |
| `cgap_verify_p28(evidence: *const u8, len: usize, out_digest: *mut u64) -> CgapStatus` | :201, body :292 | P28 spoliation verification over the 80-B attestation; read-only, no allocation; `out_digest` may be NULL |

## include/cgap.h types

| Type | Definition | Layout |
|---|---|---|
| `CgapStatus` | `typedef uint32_t` (cgap.h:56); `_Static_assert(sizeof == 4)` | 4 B on every target |
| `cgap_frame_view` | cgap.h:73-79 | LP64: 24 B, 8-aligned; ILP32 form pinned separately |
| — `payload` | `const uint8_t*` (offset 0) | points INTO caller bytes |
| — `payload_len` | `size_t` | |
| — `schema_id` | `uint32_t` | |
| — `rule_count` | `uint32_t` | |

## Layout pins (C11 `_Static_assert`, cgap.h Wave-3/4 block)

- `sizeof(CgapStatus) == 4` on every target.
- LP64: `sizeof(cgap_frame_view) == 24`, `_Alignof == 8`, with per-field
  `offsetof` pins (payload at 0) — a field-type change surviving via tail
  padding still fails the compile.
- ILP32 (ESP32-S3 / ESP32-C3 rv32imc) form pinned explicitly.
- Rust-side twin: `size_of::<CgapStatus>() == 4`
  (sovereign.rs:85).
