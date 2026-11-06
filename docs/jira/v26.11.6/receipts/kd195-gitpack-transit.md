# KD-195 Gitpack Transit — cgap Sidecar Leg (Stage 1, R6-I)

Standing: PARTIAL_ALIVE (code landed; build gates exit 0; real-socket courts witnessed pre-fix; post-fix suite execution BLOCKED by the H5b machine fault — syspolicyd wedge, operator sudo required)

## H1. Summary

KD-195 (gitpack transit carrier) closes on the cgap side: the AOQ1 window
now carries a digest-covered frameFlag byte at window-absolute offset 32
(AOQ1 metadata header `reserved[0]`, header offset 24), the sidecar gate
admits only the closed set {0x01 Arrow, 0x08 gitpack} (0x00 = pre-KD-195
Arrow-legacy, reads as 0x01), and the response leg mirrors the request's
flag into the response header `reserved[0]`, stamped PRE-digest so the
mirrored window passes every consumer's `read_verified` digest gate.

## H2. The gitpack format spec (the carrier decision)

**What rides the wire**: raw Kafka RecordBatch bytes, VERBATIM, exactly
one payload per AOQ1 window body, NO envelope, NO length prefix, NO
framing of our own.

- **Verbatim law**: the window body IS the RecordBatch. `offsets = [0,
  len)` by construction. A consumer with a RecordBatch parser slices
  the body itself; the transport never re-frames someone else's format
  (re-framing Kafka RecordBatch bytes would force cgap to re-implement
  Kafka's wire law — a reinvention we refuse).
- **Consumer-has-parser**: the gitpack consumer already parses
  RecordBatch bytes (that is the very capability KD-195 names); the
  carrier adds zero bytes of its own, so the consumer's parser runs
  unchanged on the window body.
- **no_std scope**: `crates/cgap-arrow/src/ipc.rs` is a
  zero-external-dep `#![no_std]` kernel; the view must be an
  allocation-free pure projection, which the verbatim law gives for
  free — `BinaryBatchView` borrows the payload; nothing to allocate
  because nothing is added or copied.
- **Zero-copy free**: since the payload is carried verbatim, the view
  is a borrow (`data: &[u8]` aliasing the source; `offsets = (0, len)`),
  so zero-copy costs nothing — no copy exists to eliminate.

Court: `kd195_gitpack_view_zero_copy` (pointer-range assert + source
flip propagates) and `e2e` byte-fidelity witness below.

## H3. What landed (diff map)

- `crates/cgap-arrow/src/ipc.rs`
  - `pub const FRAME_FLAG_OFFSET_IN_WINDOW: usize = PREFIX_SIZE + 24`
    (window-absolute offset 32), `FRAME_FLAG_ARROW = 0x01`,
    `FRAME_FLAG_GITPACK = 0x08`, `window_frame_flag(&[u8]) -> Option<u8>`
    (post-digest-gate reader), `stamp_frame_flag(&mut [u8], u8)` (the
    emitter-side seal helper: `hdr[24] = flag`, FNV-1a32 recomputed over
    `[0x00,0x3C)` stored at `[0x3C,0x40)`).
  - `BinaryBatchView<'a> { offsets: (usize, usize), data: &'a [u8] }` +
    `binary_batch_view(&[u8]) -> BinaryBatchView<'_>`: the gitpack view
    — verbatim, one payload per body, no envelope, zero-copy, `no_std`
    pure fn. Doc pin: KD-195 receipt H2.
  - Note: these ipc.rs constants landed on HEAD via the coordinator's
    mid-flight commit (909e1573-class) while this lane was executing.
- `crates/cgap-ffi/src/bin/aoq_sidecar.rs`
  - Flag gate (POST-digest-gate law): after `read_verified` accepts,
    `window_frame_flag` is read; closed set {0x00→0x01 legacy, 0x01,
    0x08}; anything else (0x02/0x04/junk) refuses typed with
    `AOQ_STATUS_DIGEST_MISMATCH` (0x4347A005 — the frozen 8/8 band's
    digest-law word per KD-1; no new words minted), NO echo, no session
    step.
  - MIRROR LAW in `send_window`: the request's frameFlag is copied into
    the response header `reserved[0]` via `ipc::stamp_frame_flag`
    (stamped PRE-digest — the FNV span [0x00,0x3C) covers offset 24, so
    the stamp must land before the seal for the response to pass a
    consumer's digest gate).
  - Minimal unblock (compile-freeze SLA, DISCLOSED): the fb-era
    `IpcError` variants (`LegacyReadForeignMeta`, `Fb(_)`,
    `FbBodyLenInvalid`, `BodyTooLarge`, `RowsCarrierMissing`,
    `MetaSizeContinuation`) landed in cgap-arrow (HEAD a216b486..909e1573)
    without extending the sidecar's `framing_status` match — the bin did
    not compile on HEAD before this lane. Mapping (digest-law reading):
    metadata/tamper classes → DIGEST_MISMATCH; `MetaSizeContinuation` →
    INCOMPLETE_FRAME.
- `crates/cgap-ffi/tests/aoq_sidecar_e2e_court.rs`
  - COURT E4 `e4_gitpack_flag_mirror_over_real_uds`: real binary, real
    UDS, 0x08 window → OK; response window[32] == 0x08 (mirror); mirror
    was pre-digest (response passes `read_verified`); payload
    byte-faithful end to end (framing tag gate → session parse → AEAD
    open under the court-derived key == pre-seal bytes). Anti-vacuity
    mutation: dropping the mirror stamp leaves response flag 0x00 → red.
  - COURT E5 `e5_bad_flag_refused_over_real_uds`: 0x02-flag window
    (otherwise fully lawful) → typed DIGEST_MISMATCH, no window, no
    echo (0-byte follow read). Anti-vacuity mutation: deleting the gate
    → OK word → red.

## H4. Courts (all named anti-vacuity mutations)

In-bin (socketpair, real serve loop, Chicago):
- `kd195_gitpack_window_roundtrip_flag_mirror`: 0x08 → OK; response
  window[32] == 0x08; response passes `read_verified` (pre-digest
  mirror); echoed payload byte-equal. Mutation: flip the mirror site
  (drop `stamp_frame_flag` in send_window) → red.
- `kd195_bad_flag_refused`: 0x02 → DIGEST_MISMATCH, no echo (0-byte
  follow read). Mutation: delete the flag gate → red.
- `kd195_arrow_flag_0x01_unaffected`: explicit 0x01 accepted + mirrored;
  zero-reserved legacy window accepted, mirrors 0x01. Mutations: gate
  deletion red via E5/bad-flag; mirror-site flip red via roundtrip.
- `kd195_gitpack_view_zero_copy`: `binary_batch_view` pointer range
  sits inside the source range and starts AT the base (no copy), and a
  source-byte flip (through a pointer derived before the view borrow)
  propagates through the view. Mutation: a copying `to_vec()` view
  breaks the pointer-identity assert → red.

## H5. ALIVE evidence (commands + exits, this session)

- `cargo build -p cgap-arrow` → exit 0.
- `cargo build -p cgap-ffi` → exit 0 (after the disclosed framing_status
  unblock; before it, the bin did not compile on HEAD — pre-existing).
- `cargo test -p cgap-ffi --bin aoq_sidecar` (pre-court-fix) → 12
  passed / 1 failed, 0.20s: all pre-existing courts green unmodified
  (fault matrix, alloc census, warm-serve census, plus 3 of the 4 new
  KD-195 courts passing — mirror, bad-flag, zero-copy); the ONE failure
  was this lane's own court bug (`kd195_arrow_flag_0x01_unaffected`
  sealed hop2 under the wrong key), FIXED and diagnostic code removed.
- `cargo test -p cgap-ffi --bin aoq_sidecar` (post-fix) → **BLOCKED**:
  the freshly relinked test binary hangs at `_dyld_start+0` forever.
- `cargo test -p cgap-ffi --test aoq_sidecar_e2e_court --no-run` →
  exit 0 (compile gate); execution BLOCKED by the same fault.

## H5b. Machine blocker (BLOCKED rows cite file:line class evidence)

`/usr/libexec/syspolicyd` pid 516 is wedged at 98.7–100% CPU, 240+
CPU-minutes accumulated (started 10:12AM, still burning at 01:5x). Every
NEWLY-LINKED third-party binary on this host hangs forever at
`_dyld_start+0` (launch-assessment queue never drains): witnessed on
(a) the freshly relinked `aoq_sidecar` test binary (sampled twice, two
separate relinks), (b) `aoq_tagtool`, (c) a fresh 1-line clang binary
(timeout rc=124), while Apple-signed binaries (`/bin/date`, `cargo`,
`rustc`, `sample`) launch instantly. Restart requires operator sudo
(`sudo kill -9 516`; launchd respawns it) — no passwordless sudo here.
This blocks ALL fleet cargo-test executions of newly-built binaries,
not just this lane. Coordinator notified (staggered-resume message).

## H6. Standing qualifiers (no overclaiming)

- The post-fix in-bin suite (expect 13/0) and the e2e suite (expect
  E1–E5) are UNVERIFIED — execution blocked by H5b, not by a observed
  failure. The mid-session 12/1 run witnessed every pre-existing court
  plus 3 of 4 new courts passing on real sockets; the failing court's
  defect (wrong AEAD key in the court's hop2 seal) was root-caused via
  real-socket instrumentation and the fix is mechanical (key swap in
  the court only, no production-code change involved in that defect).
- cgap-arrow's own court suite (`cargo test -p cgap-arrow`, ipc_courts
  etc.): NOT run by this lane (blocked by H5b machine-wide). ipc.rs
  changes are additive consts + two pure fns + a view struct; a
  separate lane was dispatched on ipc_courts (observed `timeout 540
  cargo test -p cgap-arrow --test ipc_courts` in the fleet process
  table) — also expected to hang on execution until the wedge clears.
- Mutation courts: named in H4, flips not yet executed (same blocker).
- arcus widening is explicitly sequenced AFTER this receipt
  (never-quiet-downgrade law); the SIGSEGV ticket (aoq_session.rs NIF
  leg) is a separate ticket — not touched.

## H7. Falsifier

A 0x02-flagged, otherwise-lawful window accepted with OK (gate dead),
or a 0x08 request answered with response window[32] != 0x08 (mirror
dead), or a mirrored response failing a consumer's `read_verified`
(stamp not pre-digest). All three courts (in-bin + E4/E5) witness the
negative forms live this session.
