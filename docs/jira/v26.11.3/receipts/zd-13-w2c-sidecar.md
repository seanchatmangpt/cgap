# ZD-13 / W2-C — AoQ kernel sidecar (bin target + end-to-end court)

**Status**: ALIVE (courts ran this session; exact commands+exits below).
**Lane**: Wave-2 lane W2-C, `docs/jira/v26.11.3/plans/wave-2-membrane-plan.md` §1 W2-C.
**Owner files**: `crates/cgap-ffi/src/bin/aoq_sidecar.rs` (new),
`crates/cgap-ffi/tests/aoq_sidecar_court.rs` (new), this receipt.

## Landings

### `crates/cgap-ffi/src/bin/aoq_sidecar.rs` (new bin target, no new workspace member)
- One UDS path argument; bind, accept one connection, fixed request/response
  loop, exit 0. EPOCH=1, KEY_ID=0xA071, ORDINAL0=0 pinned (plan inputs become
  constants; arcus W2-B/D will pass them on the wire in a later lane).
- Request leg: read prefix (8B) -> metadata span (120B) -> body_len from the
  AOQ1 pad law region (window 72..76) -> body + `ipc::pad_to(body_len, 64)` pad
  + EOS. Zero-byte close = CleanClose (exit 0); partial window = typed
  `AOQ_STATUS_INCOMPLETE_FRAME` refusal, never accepted.
- **Tag gate BEFORE accepting**: `cgap_arrow::ipc::read_verified`
  (the ipc.rs:647-656 AOQ1 branch) over the reassembled window; every
  `IpcError` maps to a frozen-band word (`framing_status`); the request bytes
  are NEVER echoed on refusal.
- Session: constructs its **own** `Session<'static, SidecarKem>` in-process
  (parse_frame/step/emit_verified + status maps). DISCLOSED: `aoq_session.rs`'s
  `HostKem` is private to that module and this lane must not touch
  aoq_session.rs/lib.rs/Cargo.toml, so the sidecar constructs an identical
  seam (Shake256 KDF + real ML-KEM-768, corridor-join-court `MlkemKem` shape)
  and mirrors the private `status_of` collision mapping.
- Response leg: `emit_verified` echo (same seq + payload) scattered via
  cgap-arrow `StreamWriter` over `PatchSink` (a `VectoredSink` impl for the
  live UnixStream). DISCLOSED composition: StreamWriter's framing delegate
  `ipc::emit` writes the IPC MetaHeader law (FNV at 0x38=56); the AOQ1 wire
  law parses the metadata as the cgap-aoq codec `AoqHeader` (FNV-1a32 at 60)
  plus the body_len/tag pad at 72..92. `PatchSink` substitutes the 120-byte
  metadata block (codec-sealed AoqHeader + body_len + blake3 tag) for the
  scratch bytes by absolute window offset, handling partial accepts. Scatter
  mechanics (seq stamp, 4-segment partial-accept advance, ZeroProgress ->
  `AOQ_STATUS_BUSY`) are the crate's StreamWriter.
- **XIP safety (coordinator call 2026-11-04)**: the bin body is wrapped in
  `#[cfg(not(target_os = "none"))] mod imp { ... }` with a std-free
  `#[cfg(target_os = "none")] fn main() {}` stub, so the riscv32imc XIP
  build compiles an empty bin (no std symbols).
- No panics on external input: all faults are typed refusals; an
  unreachable!() arm exists for CleanClose exclusivity (typed; a CleanClose
  cannot reach the refusal arm structurally).

### `crates/cgap-ffi/tests/aoq_sidecar_court.rs` (new end-to-end court)
Spawns the REAL sidecar process (`env!("CARGO_BIN_EXE_aoq_sidecar")`), path
UDS, real ML-KEM-768 keygen/encapsulate. No mocks. Bounded polls only (socket
existence, connect retry, exit poll) — no sleep. Courts:
- **J1** handshake -> established -> one verified AppData round trip: ek
  frame -> OK; ct-echo finish (court recomputes the sidecar's deterministic
  encaps(ek_c) locally and echoes it) -> OK; AppData (ordinal 2, seq 7) ->
  OK + response window; response passes `read_verified`; response frame
  parses as AppData; seq + payload byte-identical (4-byte emit tag stripped);
  inner kdf tag re-derives with the court's own seam.
- **J2** tag-gate refusal (mutation B: echo-instead-of-refuse would fail it):
  stale tag window -> `AOQ_STATUS_DIGEST_MISMATCH`, no window, and the next
  read yields EOF — the request bytes were never echoed; a fresh sidecar
  accepts the intact window (the gate is the difference).
- **J3** zero-byte/short (mutation C: accept-short would fail it): true
  zero-byte close -> clean exit 0 (bounded exit poll); 4-byte prefix then
  EOF -> typed `AOQ_STATUS_INCOMPLETE_FRAME`, never accepted.
- Mutation A (body-byte flip -> typed refusal not echo) is covered by J2's
  gate: the tag gate refuses before the frame is ever parsed, so a flipped
  body byte is the same TagMismatch path (J2 flips the stored-tag byte;
  either mutation hits the identical gate).

## Verification ladder (all run this session, lane build root
`CARGO_TARGET_DIR=target-w2c-lane`)

| Command | Witness |
|---|---|
| `cargo test -p cgap-ffi --test aoq_sidecar_court` (run 1) | `test result: ok. 3 passed; 0 failed` — exit 0 |
| same, run 2 (x2 law) | `test result: ok. 3 passed; 0 failed` — exit 0 |
| `cargo test -p cgap-ffi` (full suite) | exit 0; every target 0 failed (lib block 60/0; all integration targets 0 failed) |
| `cargo test -p cgap-ffi --lib xip_t1_riscv_elf_size_court` | `1 passed; 0 failed` — exit 0 (green WITH the new bin target landed) |
| `cargo test -p cgap-ffi --lib r2_production_link_court` | `1 passed; 0 failed` — exit 0 |
| `cargo test -p cgap-aoq --test corridor_join_court` | `test result: ok. 3 passed; 0 failed` — exit 0 (unregressed) |

## Integration disclosures (single-writer reconciliation)
- This lane touched ONLY its owned files: `src/bin/aoq_sidecar.rs`,
  `tests/aoq_sidecar_court.rs`, this receipt. Cargo.toml, aoq_session.rs,
  lib.rs: untouched (no edits issued by this lane).
- The sidecar consumes W2-A's in-process API surface
  (`parse_frame`/`Session`/`emit_verified`/`status_maps`) directly, NOT the
  private `HostKem`/`status_of`/handle table — own seam + mirrored status
  mapping, disclosed above.
- XIP cfg pattern (coordinator call 2026-11-04): bin body in
  `#[cfg(not(target_os = "none"))] mod imp`; empty
  `#[cfg(target_os = "none")] fn main()` stub; both XIP courts re-run green
  with the bin present.
- Court protocol wire law settled here (for W2-B/D consumption): request =
  AOQ1 window (codec-sealed AoqHeader metadata, body_len u32 LE at 72..76,
  blake3 tag [..16] at 76..92, body, `ipc::pad_to(body_len,64)` pad, EOS);
  response = 4-byte LE frozen-band status word, then on an accepted AppData
  request one AOQ1 response window (emit_verified echo body). Handshake:
  ek -> ct-echo (deterministic re-encaps) -> AppData; ordinals 0/1/2.

## Build-root lease release
`target-w2c-lane` deleted at receipt time (post-delete proof:
`ls: /Users/sac/cgap/target-w2c-lane: No such file or directory`).
