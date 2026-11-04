# AoQ Wave-2 Membrane Plan (Ruling R-C1)

**Status**: PLAN (UNVERIFIED — nothing on this page has run yet)
**Ruling**: R-C1 — membranes deferred from Wave 1; unblocked by (a) kernel
landing, (b) AF-1 corridor join court green, with AF1-KD-1 (real
ML-KEM-768 round-trip) closed — the precondition this plan assumes and
whose re-verification is Gate 0 below.
**Invariants**: cgap kernel stays zero-external-dep / `no_std`; arcus
stays Go stdlib-only, zero cgo.

## 0. Seam reconciliation (the load-bearing design decision)

The ruling's constraint set looks contradictory (C-ABI membrane vs
zero-cgo consumer) but the existing transport already resolves it:

- **The C-ABI is not the transport; the wire is.** `cgap-ffi`'s existing
  slot exports (`cgap_i32_batch`/`cgap_array_addr`,
  `crates/cgap-ffi/src/lib.rs:43-82`) serve in-process importers
  (PyArrow/DuckDB). Arcus never links that. Arcus's real path to the
  kernel is the **IPC window over UDS**: `cgap-arrow/src/stream_writer.rs`
  `StreamWriter` scatter (continuation / metadata_size / sealed 64-byte
  metadata / body / pad / EOS) → `net.UnixConn` writev on the Go side
  (`internal/arcus/adapter/bitmaps_ipc.go:343-345`, ontology primitive
  `StreamWriter` "writev/sendmsg vectored emission to a Unix domain
  socket").
- **AOQ1 rides the same framing.** `crates/cgap-arrow/src/ipc.rs:418`
  (`if raw.magic == AOQ1_MAGIC`) already forks the parse: BE header,
  version 2, FNV-1a32 over `[0x00,0x3C)`, `body_len` u32 LE + 16-byte
  blake3 tag at `AOQ1_TAG_AT = PREFIX_SIZE + 64 + 4`.
  `emit_verified`/`read_verified` raise body integrity into the framing
  layer. The Go side already holds the IPC1 twin (`OabiHeader` in
  `bitmaps_ipc.go`) — Wave-2 adds the AOQ1 twin as a sibling primitive
  in the same ontology projection, never hand-written Go.
- **Therefore arcus needs no cgo and no linked libcgap.** The membrane
  is: arcus `flight.Boundary` (interface, `flight_boundary.go:87-92`)
  implemented by a **kernel sidecar process** that speaks AOQ1/IPC
  framing over UDS. cgap-ffi gains the C-ABI *session* surface the
  sidecar hosts (today `crates/cgap-ffi/src/aoq/` is wire-law helpers
  only — field getters, no session export exists). Go-side seal/unseal =
  frame bytes in → AOQ1-framed bytes out over the socket; codec logic
  never emulates in Go (the standing law in `flight_boundary.go:9`).

## 1. Lanes and file ownership

### Lane W2-A — cgap-ffi session C-ABI (cgap repo, kernel)

Own: `crates/cgap-ffi/src/aoq_session.rs` (new module, declared in
`lib.rs` next to `pub mod aoq;` at `lib.rs:357`).

Surface (host cdylib only; cfg-gate `not(target_os = "none")` exactly
like the slot exports — the XIP staticlib must stay allocator-free):

- `cgap_aoq_session_new(epoch: u16, key_id: u64, ordinal: u32) -> u64 handle`
- `cgap_aoq_session_step(handle, frame_ptr, frame_len) -> u32 status word`
  (from the existing `status_maps.rs` closed set; typed refusal words,
  never -1)
- `cgap_aoq_session_emit_verified(handle, body_ptr, body_len, out_ptr,
  out_cap, out_len: *mut usize) -> u32`
- `cgap_aoq_session_fingerprint(handle, out32)` — court hook for the
  no-mutation-on-refusal law (`session.rs` law 4)
- `cgap_aoq_session_free(handle)`
- `cgap_aoq_last_error() -> u32` (thread-local last status, mirror of
  `stats.rs` refusal taxonomy)

Rules: session state behind a handle table (BTreeMap/slotmap-style, std
host surface is allowed — `lib.rs:8-16` placement law); the KEM stays a
caller-injected `KemSeam` (`crates/cgap-aoq/src/session.rs:60-92`) — the
FFI registers the host seam (real ML-KEM via cgap-crypto on host builds;
the XIP surface gets no session module at all). Every export returns a
status word from `aoq/status_maps.rs`; no panics across the boundary
(assert-based panics in `lib.rs` `cgap_digest_of` are the anti-pattern
to avoid here — inputs are external).

Courts: `crates/cgap-ffi/tests/aoq_session_ffi_court.rs` — Chicago
discipline, real session objects, each court names its anti-vacuity
mutation (fingerprint-identity assertion after every refused step, per
session law 4).

### Lane W2-B — AOQ1 Go twin + session wrapper (arcus repo, generated + hand law)

Own:

- `packs/arcus-pack/ontology.ttl` — add `AoqHeader` primitive (offsets
  mirrored from `include/cgap_aoq.h` / `cgap-arrow/src/ipc.rs` AOQ1 law,
  status=generated) and `AoqSessionUds` transport primitive. Never
  hand-edit the projections.
- `internal/arcus/adapter/aoq_ipc.go` — **generated** twin + accessors,
  same shape as `bitmaps_ipc.go` `OabiHeader`; plus a `StreamWriter`
  sibling implementing the AOQ1 fork (BE header, FNV-1a32 @0x38, tag at
  PREFIX+64+4) over `*net.UnixConn` writev.
- `internal/arcus/flight/aoq_boundary.go` — **hand-law** (new
  generated-once seam or `unless_exists` file): implements
  `flight.Boundary` (`Seal`/`Unseal`) by framing/deframing AOQ1 windows
  over the UDS conn to the sidecar. Holds a `SetBoundary` call site;
  `ErrBoundaryUnavailable` stays the default until boot wires it.
- `cmd/arcus` boot path — install the boundary; boot refusal on socket
  dial failure (fail-closed, never silent passthrough).
- Layout guards: compile-time twin-size asserts copied from the
  `ArrowArrayTwin` pattern (`cdata/arrow_cdata.go:27-31`) — ontology
  drift is a build break.

### Lane W2-C — kernel sidecar binary (cgap repo)

Own: `crates/cgap-ffi/src/bin/aoq_sidecar.rs` (or a thin
`crates/aoq-sidecar` member — prefer the `bin` target inside cgap-ffi to
avoid a 26th workspace member; disclose the member-count consequence if
the crate is chosen instead).

Behavior: accept one UDS path argument, serve a fixed request/response
loop — read one AOQ1 window (or raw frame), call the W2-A FFI-internal
session functions directly (in-process, no symbol crossing), scatter the
sealed window back via `cgap-arrow::stream_writer::StreamWriter` over a
`VectoredSink` impl for `UnixStream` (the exact pattern
`stream_writer.rs` tests already implement). Std allowed here (host
binary); zero external deps still holds via path deps only.

### Lane W2-D — admission path (arcus, wiring)

Own: `internal/arcus/flight/admit.go` (hand-law) — feed projection →
`LookupFlightOp` (`flight_ops.go`) → `LookupTranscoder` →
`RequireBoundary().Seal(...)` → UDS write. Admission refusal taxonomy
extends `FlightRefusal` with a status-word mirror of cgap's
`status_maps.rs` closed set — the two refusal vocabularies must be
pinned bijection-mapped in a drift court, not free-text.

### Lane W2-E — courts & receipts (both repos)

- cgap: `crates/cgap-ffi/tests/aoq_session_ffi_court.rs`, sidecar
  end-to-end court `crates/cgap-ffi/tests/aoq_sidecar_court.rs` (live
  socketpair, mirrors `corridor_join_court.rs` J1/J2 hop-for-hop).
- arcus: `internal/arcus/flight/aoq_boundary_test.go` (real
  net.Pipe/UDS, no mocks), adapter drift court asserting Go twin offsets
  == `include/cgap_aoq.h` defines (parse the header in the test —
  `cgap_aoq.h` is generated, so the test reads ground truth at test
  time).

## 2. Sequencing

1. **Gate 0 (precondition)**: re-run the cgap-crypto ML-KEM courts —
   AF1-KD-1 close condition (corridor join court on the real KEM). If
   red, W2-A through W2-E are BLOCKED; the corridor stays on the
   disclosed test-KEM.
2. W2-A (FFI session surface) → W2-C (sidecar) can start together after
   Gate 0; W2-C consumes W2-A's modules in-process, so its API is the
   review gate for W2-A.
3. W2-B ontology twin lands before any hand-law Go: `ggen sync run` in
   arcus, verify `written` list contains only the new slots.
4. W2-D last; W2-E courts land per-lane with each lane, not as a tail.

## 3. Falsifiers (per lane)

- **W2-A**: a `step` refusal that changes `fingerprint()` (violates
  session law 4) makes the lane red; an export reachable with a
  stale/freed handle returning success makes it red; a panic escaping an
  `extern "C"` fn makes it red.
- **W2-B**: Go twin offset ≠ `cgap_aoq.h` define → the drift court must
  fire; a hand-written Go AOQ framer (bypassing the ontology projection)
  is a scope violation by construction.
- **W2-C**: sidecar accepting a window whose blake3 tag (recomputed per
  `ipc.rs:647-656`) disagrees → must refuse with the AOQ1 typed status,
  never echo; a zero-byte accept spun instead of refused violates the
  StreamWriter contract.
- **W2-D**: any feed admitted whose op is absent from `flightOpTable`
  (must yield `FlightRefusal`); any seal/unseal logic in Go that
  re-implements admit instead of delegating to the Boundary (the
  `flight_boundary.go:9` law).
- **Whole plan**: a `cgo` import or a linked libcgap in arcus falsifies
  the seam design; an external-crate dep added to cgap-aoq/cgap-arrow
  falsifies E=0.

## 4. Verification ladder

| Rung | Command | Pass witness |
|---|---|---|
| 0 | `cargo test -p cgap-crypto` (ML-KEM courts) + `cargo test -p cgap-aoq --test corridor_join_court` | exit 0 (AF1-KD-1 closed, corridor green on real KEM) |
| 1 | `cargo test -p cgap-ffi` | exit 0, new session courts N passed / 0 failed (exit record law: both counts) |
| 2 | `cargo test -p cgap-ffi --test aoq_sidecar_court` | 3+ passed / 0 failed ×2 (J1/J2 mirror) |
| 3 | `cargo test -p cgap-aoq --test corridor_join_court` | 3 passed / 0 failed (regression — corridor still green) |
| 4 | `cd /Users/sac/arcus && go test ./internal/arcus/flight/... ./internal/arcus/adapter/...` | ok, drift courts included |
| 5 | `cd /Users/sac/arcus && go build ./... && go vet ./...` | exit 0, zero cgo (`go list -deps` shows no runtime/cgo) |
| 6 | `ggen sync run` both repos | determinism: run 2 `written: []`; arcus twin offsets byte-match `include/cgap_aoq.h` |
| 7 | `make gate0` (cgap) | byte-stable replay PASS |
| 8 | E2E: arcus admission → sidecar → corridor verify, one live feed | decrypted == pre-seal bytes, tag gate fires on flipped byte |

Standing words: nothing above is ALIVE until run this session with an
exit record; the corridor's 0.56 µs/frame AF-30 number is a TARGET for
the FFI+sidecar path until measured with a stated method.

## Critical files

- /Users/sac/cgap/crates/cgap-ffi/src/lib.rs (add `pub mod aoq_session;`,
  host-surface cfg law)
- /Users/sac/cgap/crates/cgap-aoq/src/session.rs (Session/KemSeam state
  machine the C-ABI wraps)
- /Users/sac/cgap/crates/cgap-arrow/src/ipc.rs (AOQ1 magic fork +
  emit_verified/read_verified framing law)
- /Users/sac/cgap/crates/cgap-arrow/src/stream_writer.rs (VectoredSink
  scatter — the sidecar's UDS transport)
- /Users/sac/arcus/internal/arcus/flight/flight_boundary.go (Boundary
  seam the Go side implements against)
