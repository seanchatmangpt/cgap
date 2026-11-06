# KD-1 cgap_import_arrow Surface Receipt (lane R-KD1)

Standing: PASS

Date: 2026-11-06. Lane R-KD1, repo `/Users/sac/cgap` (cgap side of KD-1;
arcus untouched by this lane). Contract reads: arcus
`KNOWN_DIVERGENCES.md` KD-1 row, `docs/architecture/cdata-abi.md`
(REGISTERED-ABSENCE note), `docs/jira/v26.11.6/kd2-oabi-contract.md`
(O-ABI wire law §2, schema-identity divergence §3).

## What landed

The `cgap_import_arrow` symbol now exists under `crates/` (grep-zero
registered absence is closed):

- `crates/cgap-arrow/src/import_arrow.rs` (new) — the seal/publish half:
  [`import_arrow`] takes a live C Data Interface pair
  (`&ArrowArray`, `&ArrowSchema`), `schema_digest[32]`, one `u64` length
  per buffer, `ImportParams { alignment, seq }`, and an `out` window;
  concatenates the array's already-decoded buffers per the lens,
  seals an O-ABI "IPC1" `MetaHeader` (magic 0x4950_4331 LE, BLAKE3
  schema digest at 0x08, FNV-1a header digest at 0x38 via
  `ipc::seal_meta`), and `ipc::emit`s the encapsulated window arcus
  `internal/arcus/adapter/bitmaps_ipc.go` consumes. Probe = C-ABI
  `out == NULL` → required window length via `out_len`
  (`import_arrow_window_len`). Typed errors: BadAlignment /
  BadBufferCount / NullBuffer / Truncated.
- `crates/cgap-arrow/src/lib.rs` — `#[cfg(not(target_os = "none"))] pub
  mod import_arrow;` (host surface, mirrors `c_abi` gating).
- `crates/cgap-ffi/src/import_arrow.rs` (new) — the `#[no_mangle] pub
  unsafe extern "C" fn cgap_import_arrow(...)` export in the cdylib
  membrane (home of the `cgap_*` exports), thin wrapper over the
  cgap-arrow fn, typed negative return codes (status module: OK=0,
  NULL_ARG=-1, BAD_ALIGNMENT=-2, BAD_BUFFER_COUNT=-3, NULL_BUFFER=-4,
  TRUNCATED=-5). Alignment gates the probe path too.
- `crates/cgap-ffi/src/lib.rs` — host-gated `pub mod import_arrow;`.

## Contract-fit decision

The registered signature sketch was `(array, schema, schema_digest[32],
buffer_lens[], out_view MAY be NULL = probe)`. Landed: the sketch **plus
two emitter params** (`alignment: u32`, `seq: u32`) the O-ABI seal law
requires — the KD-2 wire contract carries an alignment (8/64) and a
monotonic seq (arcus binds seq to chainfile `Index`), so the sketch
could not seal without them. This is the disclosed signature divergence,
named in both module docs. `schema` is carried for signature parity but
not re-digested: the O-ABI schema identity is the caller-supplied
`schema_digest` (KD-2 §3), so re-digesting the flatbuffer Schema struct
would be wrong-form work. The name `cgap_import_arrow` is used verbatim
(extern "C" — no Rust snake_case divergence). Name: no divergence.
Signature: two disclosed additive divergences.

## Implemented vs registered

**Implemented (ALIVE):** the seal/publish half — already-decoded Arrow
buffers → sealed O-ABI "IPC1" window, probe semantics, typed refusals.
**Registered (not implemented):** the **read half** — decoding a foreign
Arrow IPC file/stream (flatbuffer Schema + record-batch messages) into
buffers. cgap-arrow is a zero-external-dep no_std kernel
(`[dependencies]` empty); a from-scratch IPC reader is out of lane
budget and is the named follow-up: an `arrow-ipc` reader dep or a
hand-law flatbuffer decode, plus arcus-side adapter wiring (arcus-side
work, not this lane). Until the read half lands, `cgap_import_arrow`
seals caller-decoded buffers — honest about that boundary, not silent.

## Verification gate (this session)

- `cargo build -p cgap-arrow -p cgap-ffi` → exit 0.
- `cargo test -p cgap-arrow` → exit 0 (17 suites all ok; lib suite
  154 passed / 0 failed, includes the 3 new import_arrow courts).
- `cargo test -p cgap-ffi` → exit 0 (21 suites all ok; includes the 2
  new import_arrow courts).
- Symbol witness: `nm -gU target/debug/libcgap_ffi.dylib | grep
  cgap_import_arrow` → `T _cgap_import_arrow`.
- Chicago discipline: tests build a real exported C-Data-Interface pair
  (`PrimitiveBuilder` → `export`), seal, and round-trip through the real
  reader `ipc::read_with_meta` (magic/schema digest/seq/alignment and
  byte-exact body verified); probe-vs-truncated distinction tested at
  the symbol level; null-pointer-table input returns typed
  NullBuffer, never UB. No mocks.

## Anti-vacuity notes

- The seal law is exercised through the real reader: `read_with_meta`
  refuses corrupt windows, and the round-trip court fails if the seal
  law drifts from the published KD-2 wire contract (magic, digest
  offsets, LE field order, pad/reserved zero laws all re-checked on
  read).
- A mutation that un-seals (skips `seal_meta`) fails the round trip at
  `HeaderDigestMismatch` — the court flips.

## Follow-up (registered)

1. Read half: Arrow IPC file/stream reader producing the buffer lens +
   pointers `cgap_import_arrow` consumes (cgap lane, future budget).
2. arcus side: adapter/bitmaps_ipc.go-side consumption of the new
   symbol (arcus lane; this lane did not touch arcus).
3. KD-1 full close still requires the arcus switch; when both land,
   arcus `KNOWN_DIVERGENCES.md` KD-1 flips CLOSED with a post-close
   receipt naming this surface diff.
