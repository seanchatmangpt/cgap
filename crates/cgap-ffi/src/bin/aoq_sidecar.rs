//! AoQ kernel sidecar (W2-C, wave-2-membrane-plan §1 Lane W2-C).
//!
//! One UDS path argument; accepts connections SEQUENTIALLY until stdin EOF
//! (the stop signal); each connection gets a fixed request/response loop
//! over a live UnixStream: read one AOQ1 window, verify its blake3 tag per
//! `cgap-arrow/src/ipc.rs:647-656` (`read_verified`) BEFORE accepting,
//! step the W2-A session state machine in-process, and — for an accepted
//! AppData frame — scatter a verified response window back via
//! [`StreamWriter`] over a [`VectoredSink`] impl for the socket.
//!
//! Disclosed deviations (also in the lane receipt):
//! - **Bin target, not a 26th workspace member.** Lives as
//!   `src/bin/aoq_sidecar.rs` inside cgap-ffi; no manifest edit (the
//!   W2-A lane owns Cargo.toml).
//! - **Own seam, not the private HostKem.** `aoq_session.rs`'s `HostKem`
//!   is private to that module and this lane must not touch it, so the
//!   sidecar constructs its own `Session<'static, SidecarKem>` with an
//!   identical seam (Shake256 KDF + real ML-KEM-768, the corridor join
//!   court's `MlkemKem` shape). Status mapping mirrors the private
//!   `status_of` in `aoq_session.rs` (same collisions pinned).
//! - **Pad patch between framing and wire.** `StreamWriter::emit`
//!   delegates framing to `ipc::emit` (zero pad), which cannot carry the
//!   AOQ1 body_len/tag law bytes at `72..92`, and exposes no post-emit
//!   pre-scatter hook. The scatter sink below substitutes the law bytes
//!   into the metadata segment by absolute window offset, so the bytes on
//!   the wire satisfy `read_verified`'s AOQ1 tag gate.
//! - **EPOCH/KEY_ID/ordinal pinned constants** (the plan's
//!   `cgap_aoq_session_new(epoch, key_id, ordinal)` inputs become pinned
//!   constants here; arcus W2-B/D will pass them on the wire in a later
//!   lane).
//! - **Per-connection I/O timeout (A4 RISK A).** `serve` sets
//!   SO_RCVTIMEO/SO_SNDTIMEO (30s default, `CGAP_SIDECAR_IO_TIMEOUT_MS`
//!   court override) on both legs; a stalled peer is dropped SILENTLY —
//!   no status word (the frozen band is 8/8 allocated, KD-1) — and the
//!   single-tenant accept loop resumes. The deadline refreshes per
//!   read/write call: a slow-but-progressing peer is never dropped.
//!
//! # Named exits (panic=abort rationale, workspace [profile.release])
//!
//! The sidecar has exactly three exits:
//! 1. **abort** — a panic escapes the serve loop: `panic = "abort"` turns
//!    it into SIGABRT; the arcus supervisor reaps the abnormal death and
//!    respawns a fresh generation. This is the process-boundary twin of the
//!    in-process `catch_unwind` -> BUSY surface in
//!    `cgap-ffi/src/aoq_session.rs:202-401`; do not flip to unwind — an
//!    unwind would make a poisoned serve loop keep serving.
//! 2. **stdin EOF** — the lawful stop signal: the watcher thread observes
//!    EOF, unlinks the socket, exits 0.
//! 3. **poison** — an unrecoverable accept/bind fault: exit 2, fail-closed,
//!    never a fallback.

// XIP safety (coordinator call 2026-11-04): the riscv32imc XIP build
// (`target_os = "none"`) compiles bin targets too; std/alloc are unresolved
// there. This bin is HOST-SURFACE ONLY, so the none-target gets an empty
// std-free bin and the whole sidecar body is compiled out.
#[cfg(target_os = "none")]
fn main() {}

#[cfg(not(target_os = "none"))]
fn main() {
    imp::main()
}

#[cfg(not(target_os = "none"))]
mod imp {
    use std::env;
    use std::io::{self, IoSlice, Read, Write};
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::process::exit;

    use cgap_aoq::codec;
    use cgap_aoq::error::AoqError;
    use cgap_aoq::header::{AoqHeader, SIZE_BYTES as FRAME_HEADER_BYTES};
    use cgap_aoq::session::{
        self as sess, KemSeam, Session, SessionError, CT_BYTES, EK_BYTES, SS_BYTES,
    };
    use cgap_arrow::ipc::{
        self, window_len, MetaHeader, AOQ1_MAGIC, AOQ1_TAG_BYTES, EOS_SIZE, PREFIX_SIZE,
    };
    use cgap_arrow::stream_writer::{StreamError, StreamWriter, VectoredSink};
    use cgap_crypto::fips202::Shake256;
    use cgap_ffi::aoq::status_maps::{
        AOQ_STATUS_BUSY, AOQ_STATUS_DIGEST_MISMATCH, AOQ_STATUS_EPOCH_MISMATCH,
        AOQ_STATUS_EPOCH_RETIRED, AOQ_STATUS_INCOMPLETE_FRAME, AOQ_STATUS_OK,
        AOQ_STATUS_UNSUPPORTED_VERSION,
    };

    // ---------------------------------------------------------------------------
    // Host seam: real ML-KEM-768 + Shake256 KDF (corridor-join shape)
    // ---------------------------------------------------------------------------

    /// Order-sensitive 32-byte Shake256 squeeze over concatenated parts.
    fn shake32(parts: &[&[u8]]) -> [u8; 32] {
        let mut sh = Shake256::instance();
        for p in parts {
            sh.absorb(p);
        }
        let mut out = [0u8; 32];
        sh.squeeze_into(&mut out);
        out
    }

    /// The sidecar's own host seam: identical shape to `aoq_session.rs`'s
    /// private `HostKem` (deterministic ML-KEM-768 encapsulation seed derived
    /// from the peer ek, Shake256 KDF).
    struct SidecarKem;

    impl KemSeam for SidecarKem {
        fn encapsulate(
            &self,
            ek: &[u8; EK_BYTES],
        ) -> Result<([u8; CT_BYTES], [u8; SS_BYTES]), SessionError> {
            let m = shake32(&[b"w2c/mlkem/m", ek]);
            let mut ct = [0u8; CT_BYTES];
            let mut ss = [0u8; SS_BYTES];
            cgap_crypto::kem::encaps(ek, &m, &mut ct, &mut ss);
            Ok((ct, ss))
        }

        fn decapsulate(
            &self,
            dk: &[u8],
            ct: &[u8; CT_BYTES],
        ) -> Result<[u8; SS_BYTES], SessionError> {
            let dk: &[u8; 2400] = dk
                .try_into()
                .map_err(|_| SessionError::KemRefused { op: 1 })?;
            Ok(cgap_crypto::kem::decaps(dk, ct))
        }

        fn kdf(&self, parts: &[&[u8]]) -> [u8; 32] {
            shake32(parts)
        }
    }

    // ---------------------------------------------------------------------------
    // Pinned session constants
    // ---------------------------------------------------------------------------

    /// The sidecar session's epoch (closed enum {0,1,2,65535}).
    ///
    /// LOAD-BEARING LAW: the epoch is PINNED, never derived from the
    /// supervisor's generation counter. Each respawned sidecar generation
    /// re-establishes the same epoch (1 = `AoqEpoch::ROTATED_1` in the
    /// frozen enum `cgap_ffi::aoq::kind_enums::AoqEpoch` — frozen labeled
    /// enums projected from `aoq:Epoch`; the supervisor's generation is a
    /// process-lifecycle counter with NO wire meaning). Deriving the epoch
    /// from the generation would make every respawn land on an epoch the
    /// frozen enum refuses, so the sequential single-tenant serve loop plus
    /// this pinned constant is the law the supervisor's derive-on-respawn
    /// contract rests on.
    const EPOCH: u16 = 1;
    /// The sidecar session's key id.
    const KEY_ID: u64 = 0xA0_71;
    /// The sidecar session's starting frame ordinal.
    const ORDINAL0: u32 = 0;
    /// Upper bound on an accepted request window body (DoS bound; larger
    /// `body_len` words refuse instead of allocating).
    const MAX_BODY: usize = 1 << 20;
    /// Metadata span for alignment-64 AOQ1 windows (`meta_span_for(64)`).
    const META_SPAN: usize = 120;

    // ---------------------------------------------------------------------------
    // Request leg: read one AOQ1 window (typed read faults)
    // ---------------------------------------------------------------------------

    /// Why the request leg refuses to produce a window.
    #[derive(Debug)]
    enum ReadFault {
        /// Peer closed before the first byte (clean end of service).
        CleanClose,
        /// Bytes arrived but the window is incomplete.
        Short,
        /// Framing word refused (`meta_span` not the align-64 span).
        BadFraming,
        /// `body_len` word exceeds [`MAX_BODY`].
        Oversize,
        /// The peer stalled past the per-connection I/O timeout
        /// ([`IO_TIMEOUT`]): the OS read timed out with bytes sent but the
        /// window incomplete. Transport-layer fault; the connection is
        /// dropped WITHOUT a status word (the frozen band is 8/8 allocated
        /// — KD-1 — and a timeout is a transport fault, not a framing
        /// verdict), and the single-tenant accept loop resumes.
        ClientTimeout,
    }

    /// Per-connection I/O timeout applied to BOTH legs (read and write) of
    /// every served connection (A4 RISK A fix: a stalled client must not
    /// park the single-tenant serve loop in read(2) forever).
    ///
    /// N = 30s: generous against lawful handshake latency (the ML-KEM-768
    /// Ek + Finish round trips are sub-millisecond on host hardware; the
    /// budget exists for scheduling stalls, not for the crypto), yet it
    /// bounds the wedge to 30s per stalled connection instead of forever.
    ///
    /// COURT SEAM (disclosed): `CGAP_SIDECAR_IO_TIMEOUT_MS` overrides the
    /// default in milliseconds so the wedge-freedom court can drive the
    /// REAL binary at a 1s timeout instead of waiting 30s. Production
    /// leaves the env unset and gets the 30s law.
    fn io_timeout() -> std::time::Duration {
        let ms = env::var("CGAP_SIDECAR_IO_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(30_000);
        std::time::Duration::from_millis(ms)
    }

    /// The error kind the OS reports for an expired SO_RCVTIMEO/SO_SNDTIMEO
    /// (EAGAIN/EWOULDBLOCK on macOS/BSD, ETIMEDOUT on Linux) — both are the
    /// one [`ReadFault::ClientTimeout`] law.
    fn is_timeout(e: &io::Error) -> bool {
        matches!(
            e.kind(),
            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
        )
    }

    /// Read exactly one AOQ1 window off the stream.
    ///
    /// Zero-byte close is [`ReadFault::CleanClose`]; a partial window (bytes
    /// then EOF) is [`ReadFault::Short`] — refused, never accepted.
    fn read_window(stream: &mut UnixStream) -> Result<Vec<u8>, ReadFault> {
        let mut prefix = [0u8; PREFIX_SIZE];
        let mut got = 0usize;
        while got < prefix.len() {
            let n = stream.read(&mut prefix[got..]).map_err(|e| {
                if is_timeout(&e) {
                    ReadFault::ClientTimeout
                } else {
                    ReadFault::Short
                }
            })?;
            if n == 0 {
                if got == 0 {
                    return Err(ReadFault::CleanClose);
                }
                return Err(ReadFault::Short);
            }
            got += n;
            // Lag single-byte reads behind the first (socketpair/UDS reads
            // usually land the whole prefix in one read; the loop is the
            // partial-accept law).
        }
        let meta_span = u32::from_le_bytes([prefix[4], prefix[5], prefix[6], prefix[7]]) as usize;
        if meta_span != META_SPAN {
            return Err(ReadFault::BadFraming);
        }
        let mut meta = [0u8; META_SPAN];
        if read_full(stream, &mut meta)? == 0 {
            return Err(ReadFault::Short);
        }
        // AOQ1 pad law region lives in the window's metadata padding: body_len
        // u32 LE at 72..76, tag at 76..92 (window-absolute offsets).
        let body_len = u32::from_le_bytes([meta[64], meta[65], meta[66], meta[67]]) as usize;
        if body_len > MAX_BODY {
            return Err(ReadFault::Oversize);
        }
        let body_pad = ipc::pad_to(body_len, 64);
        // SINGLE-ALLOC SHAPE (alloc-elimination lane): one zeroed window
        // allocation; the 128-byte head (prefix + meta) is memcpy'd into
        // place and the body+pad+EOS bytes stream-read directly into the
        // tail. Fault semantics EXACT: the prefix loop above keeps its
        // CleanClose-on-zero-byte-at-offset-0 vs Short law, the meta read
        // and the tail read keep `read_full`'s Short-on-any-zero-read law,
        // and the Oversize gate fired before this allocation, so an
        // oversize body_len still refuses without allocating.
        let mut window = vec![0u8; PREFIX_SIZE + META_SPAN + body_len + body_pad + EOS_SIZE];
        window[..PREFIX_SIZE].copy_from_slice(&prefix);
        window[PREFIX_SIZE..PREFIX_SIZE + META_SPAN].copy_from_slice(&meta);
        read_full(stream, &mut window[PREFIX_SIZE + META_SPAN..])?;
        Ok(window)
    }

    /// `read_exact` semantics over [`Read::read`]: loops until the buffer is
    /// full; any zero read is a typed short fault.
    fn read_full(stream: &mut UnixStream, buf: &mut [u8]) -> Result<usize, ReadFault> {
        let mut got = 0usize;
        while got < buf.len() {
            let n = stream.read(&mut buf[got..]).map_err(|e| {
                if is_timeout(&e) {
                    ReadFault::ClientTimeout
                } else {
                    ReadFault::Short
                }
            })?;
            if n == 0 {
                return Err(ReadFault::Short);
            }
            got += n;
        }
        Ok(got)
    }

    // ---------------------------------------------------------------------------
    // Tag gate: read_verified BEFORE accepting (ipc.rs:647-656 law)
    // ---------------------------------------------------------------------------

    /// Map an [`ipc::IpcError`] to a frozen-band status word.
    fn framing_status(err: &ipc::IpcError) -> u32 {
        use cgap_arrow::ipc::IpcError as E;
        match err {
            E::TagMismatch { .. } | E::HeaderDigestMismatch { .. } => AOQ_STATUS_DIGEST_MISMATCH,
            E::BadMagic { .. }
            | E::BadVersion { .. }
            | E::BadFlags { .. }
            | E::TagMissing
            | E::PadNonZero { .. }
            | E::SchemaMismatch { .. }
            | E::BodyDigestMissing
            | E::BodyDigestMismatch { .. }
            | E::BodyDigestNoRoom { .. } => AOQ_STATUS_DIGEST_MISMATCH,
            E::BadContinuation { .. }
            | E::BadMetaSize { .. }
            | E::Truncated { .. }
            | E::BadBodyLen { .. }
            | E::MissingEos { .. }
            | E::BadAlignment { .. } => AOQ_STATUS_INCOMPLETE_FRAME,
        }
    }

    /// Map a [`SessionError`] to a frozen-band status word (mirrors the
    /// private `status_of` in `aoq_session.rs` — same pinned mapping).
    fn session_status(err: &SessionError) -> u32 {
        match err {
            SessionError::LengthMismatch { .. } => AOQ_STATUS_INCOMPLETE_FRAME,
            // KD-COVERS: KD-1 — version refusals carry their own word since
            // the KD-1 band extension (FM-AOQ-009); no digest-word collision.
            SessionError::Wire(AoqError::UnsupportedVersion { .. })
                => AOQ_STATUS_UNSUPPORTED_VERSION,
            SessionError::Wire(AoqError::BadMagic { .. })
            | SessionError::Wire(AoqError::DigestMismatch { .. })
            // AoqError::AeadMismatch appended mid-wave by lane CG8 (0x00_0D):
            // authenticity failure -> same frozen digest-mismatch status.
            | SessionError::Wire(AoqError::AeadMismatch)
            | SessionError::Wire(AoqError::ReservedNotZero)
            | SessionError::UnknownFrameKind { .. }
            | SessionError::TranscriptMismatch => AOQ_STATUS_DIGEST_MISMATCH,
            SessionError::Wire(AoqError::UnknownEpochWire { .. })
            | SessionError::EpochMismatch { .. }
            | SessionError::UnexpectedFrame { .. } => AOQ_STATUS_EPOCH_MISMATCH,
            SessionError::Wire(AoqError::KemKernelNotAdmitted { .. })
            | SessionError::Wire(AoqError::KemShapeMismatch { .. })
            | SessionError::Wire(AoqError::AeadKernelNotAdmitted { .. })
            | SessionError::KemRefused { .. }
            // AEAD open refused: an authenticity failure (P2 M1 typed landing).
            | SessionError::AeadMismatch
            // Confirm-tag law (v3): authenticity failure, same frozen band.
            | SessionError::ConfirmMismatch => AOQ_STATUS_DIGEST_MISMATCH,
            SessionError::RekeyRequired { .. } => AOQ_STATUS_BUSY,
            SessionError::SessionClosed => AOQ_STATUS_EPOCH_RETIRED,
            SessionError::Wire(AoqError::HeaderTooShort { .. })
            | SessionError::Wire(AoqError::BufferTooShort { .. })
            | SessionError::Wire(AoqError::SlabFull { .. })
            | SessionError::Wire(AoqError::FrameOrdinalRegression { .. })
            | SessionError::Wire(AoqError::SeqGap { .. })
            | SessionError::OrdinalMismatch { .. } => AOQ_STATUS_INCOMPLETE_FRAME,
        }
    }

    // ---------------------------------------------------------------------------
    // Response leg: StreamWriter scatter with a pad-law PatchSink
    // ---------------------------------------------------------------------------

    /// Vectored sink for the response scatter: wraps the live socket and
    /// substitutes the AOQ1 pad-law bytes (body_len u32 LE at 72..76, blake3
    /// tag at 76..92) into the metadata segment by absolute window offset.
    ///
    /// Why a substitution, not a post-emit patch: [`StreamWriter::emit`]
    /// delegates framing to `ipc::emit` (zero pad) and exposes no post-emit
    /// pre-scatter hook, and the scratch buffer is borrowed by the emit call.
    /// The patch region [72,92) is 20 bytes inside the 120-byte metadata
    /// segment; the sink intersects each segment's absolute range with the
    /// patch region and writes a patched copy for the intersection, so
    /// partial accepts inside the region stay correct.
    struct PatchSink<'a> {
        stream: &'a mut UnixStream,
        meta: [u8; 120],
        sent: usize,
    }

    impl<'a> PatchSink<'a> {
        fn new(stream: &'a mut UnixStream, meta: [u8; 120]) -> Self {
            Self {
                stream,
                meta,
                sent: 0,
            }
        }
    }

    impl VectoredSink for PatchSink<'_> {
        type Error = io::Error;

        fn write_vectored(&mut self, segs: &[&[u8]]) -> Result<usize, io::Error> {
            let mut written = 0usize;
            for seg in segs {
                if seg.is_empty() {
                    continue;
                }
                let abs_lo = self.sent;
                let abs_hi = abs_lo + seg.len();
                // The window's metadata segment (absolute [8,128)) is replaced
                // wholesale with the sealed AOQ1 metadata block: StreamWriter's
                // ipc::emit framing writes an IPC MetaHeader whose digest sits
                // at 0x38, but the AOQ1 tag gate parses the metadata as the
                // cgap-aoq codec AoqHeader (FNV-1a32 at 60) plus the
                // body_len/tag pad. Everything outside [8,128) is verbatim.
                let (plo, phi) = (PREFIX_SIZE, PREFIX_SIZE + 120usize);
                // Stack-resident piece array (alloc-elimination lane):
                // write_vectored pieces are built in place — never a heap
                // Vec — so the per-segment scatter stays alloc-free.
                let mut pieces = [
                    IoSlice::new(&[][..]),
                    IoSlice::new(&[][..]),
                    IoSlice::new(&[][..]),
                ];
                let mut n_pieces = 0usize;
                if abs_hi <= plo || abs_lo >= phi {
                    pieces[0] = IoSlice::new(seg);
                    n_pieces = 1;
                } else {
                    let lo = plo.max(abs_lo);
                    let hi = phi.min(abs_hi);
                    if lo > abs_lo {
                        pieces[n_pieces] = IoSlice::new(&seg[..lo - abs_lo]);
                        n_pieces += 1;
                    }
                    pieces[n_pieces] = IoSlice::new(&self.meta[lo - plo..hi - plo]);
                    n_pieces += 1;
                    if abs_hi > hi {
                        pieces[n_pieces] = IoSlice::new(&seg[hi - abs_lo..]);
                        n_pieces += 1;
                    }
                }
                let n = self.stream.write_vectored(&pieces[..n_pieces])?;
                self.sent += n;
                written += n;
                if n < seg.len() {
                    // Partial accept: report what was taken; StreamWriter's
                    // advance loop re-scatters the remainder (piece state
                    // recomputes from self.sent, so a partial inside the
                    // region stays correct).
                    return Ok(written);
                }
                let _ = (abs_hi, abs_lo);
            }
            Ok(written)
        }
    }

    // ---------------------------------------------------------------------------
    // Serve loop
    // ---------------------------------------------------------------------------

    /// One served connection: the fixed request/response loop.
    ///
    /// Refusal law: every fault maps to ONE frozen-band status word written as
    /// a 4-byte LE word — the request bytes are never echoed. `ZeroProgress`
    /// (a zero-byte sink accept) maps to [`AOQ_STATUS_BUSY`], the pinned
    /// internal-fault word, via the StreamWriter error surface.
    fn serve(mut stream: UnixStream) {
        // A4 RISK A: bound both legs. A stalled client (connects, sends a
        // partial prefix, holds) must NOT park serve() in read(2) forever —
        // the accept loop is single-tenant, so that wedge stalls every
        // subsequent peer and the PL7 BUSY drain never runs. Both timeouts
        // fire per read/write call, so the deadline refreshes on progress:
        // a slow-but-progressing lawful peer is never dropped.
        let t = io_timeout();
        if let Err(e) = stream.set_read_timeout(Some(t)) {
            eprintln!("set_read_timeout: {e}");
            return;
        }
        if let Err(e) = stream.set_write_timeout(Some(t)) {
            eprintln!("set_write_timeout: {e}");
            return;
        }
        let kem = SidecarKem;
        let mut session = Session::new(&kem, EPOCH, KEY_ID, ORDINAL0);
        // Per-connection reused buffers (alloc-elimination lane): allocated
        // on first use and RESIZED (capacity retained) per frame — the
        // steady-state frame path performs zero heap allocations. Split
        // into caller slices downstream (read_verified / open_verified /
        // emit_verified / StreamWriter all write caller buffers).
        let mut body = Vec::new();
        let mut plain = Vec::new();
        let mut frame_buf = Vec::new();
        let mut scratch = Vec::new();
        loop {
            let window = match read_window(&mut stream) {
                Ok(w) => w,
                Err(ReadFault::CleanClose) => return,
                Err(ReadFault::ClientTimeout) => {
                    // Transport-layer fault: drop the stalled connection
                    // SILENTLY (no status word — the frozen band is 8/8
                    // allocated per KD-1) and return to the accept loop.
                    // The sidecar is NOT killed; the wedge is bounded, the
                    // serve loop resumes (wedge-freedom law).
                    eprintln!("client timeout: stalled peer dropped (no status word)");
                    return;
                }
                Err(e) => {
                    let word = match e {
                        ReadFault::CleanClose => unreachable!(),
                        _ => AOQ_STATUS_INCOMPLETE_FRAME,
                    };
                    let _ = send_status(&mut stream, word);
                    return;
                }
            };

            // TAG GATE (BEFORE accepting): recompute blake3(body)[..16] over the
            // carried body and refuse unless it equals the stored tag.
            let body_len =
                u32::from_le_bytes([window[72], window[73], window[74], window[75]]) as usize;
            body.clear();
            body.resize(body_len, 0u8);
            if let Err(err) = ipc::read_verified(&window, None, &mut body) {
                let _ = send_status(&mut stream, framing_status(&err));
                return;
            }

            // Session step: parse_frame + step, typed refusals mapped to the
            // frozen band; on refusal the body is NEVER echoed back.
            let frame = match sess::parse_frame(&body) {
                Ok(f) => f,
                Err(err) => {
                    let _ = send_status(&mut stream, session_status(&err));
                    return;
                }
            };
            if let Err(err) = session.step(&frame) {
                let _ = send_status(&mut stream, session_status(&err));
                return;
            }

            // Accepted. An accepted AppData frame gets one verified response
            // window: the session's own emit_verified echo (same seq, same
            // payload) scattered via StreamWriter over the pad-law sink.
            if frame.kind == sess::FrameKind::AppData {
                let seq = u32::from_le_bytes([
                    frame.payload[0],
                    frame.payload[1],
                    frame.payload[2],
                    frame.payload[3],
                ]);
                // v3 wire law: body = [seq 4B][ct][tag 16B]. The echo
                // carries the DECRYPTED payload (rung-8 witness: pre-seal
                // bytes survive to echo) — open under recv_key at accept.
                let ct_len = frame.payload.len() - 4 - cgap_aoq::aead::TAG_LEN;
                plain.clear();
                plain.resize(ct_len, 0u8);
                if let Err(err) = session.open_verified(&body, &mut plain) {
                    let _ = send_status(&mut stream, session_status(&err));
                    return;
                }
                let payload = &plain;
                // v3 wire law: emit writes [header][kind][seq][ct][tag 16B].
                frame_buf.clear();
                frame_buf.resize(
                    FRAME_HEADER_BYTES + 1 + 4 + payload.len() + cgap_aoq::aead::TAG_LEN,
                    0u8,
                );
                match session.emit_verified(seq, payload, &mut frame_buf) {
                    Ok(written) => {
                        let _ = send_status(&mut stream, AOQ_STATUS_OK);
                        if let Err(err) = send_window(
                            &mut stream,
                            &frame_buf[..written],
                            frame.header.frame_ordinal,
                            seq,
                            &mut scratch,
                        ) {
                            let word = match &err {
                                StreamError::ZeroProgress => AOQ_STATUS_BUSY,
                                StreamError::Sink(_) => AOQ_STATUS_BUSY,
                                StreamError::Framing(_) => AOQ_STATUS_BUSY,
                            };
                            let _ = send_status(&mut stream, word);
                            return;
                        }
                    }
                    Err(err) => {
                        let _ = send_status(&mut stream, session_status(&err));
                        return;
                    }
                }
            } else {
                let _ = send_status(&mut stream, AOQ_STATUS_OK);
            }
        }
    }

    /// 4-byte LE status word (the refusal surface; never an echo).
    fn send_status(stream: &mut UnixStream, word: u32) -> io::Result<()> {
        stream.write_all(&word.to_le_bytes())
    }

    /// Scatter one sealed AOQ1 response window (body = an emit_verified frame)
    /// via [`StreamWriter`] over the metadata-substituting [`PatchSink`].
    ///
    /// Disclosed composition: StreamWriter's framing delegate (`ipc::emit`)
    /// writes the IPC MetaHeader law (FNV at 0x38); the AOQ1 wire law needs a
    /// codec-sealed [`AoqHeader`] (FNV at 60) plus the body_len/tag pad. The
    /// sink swaps the 120-byte metadata block for a correctly sealed one; the
    /// scatter mechanics (seq stamp, four-segment partial-accept advance loop,
    /// ZeroProgress refusal) are the crate's StreamWriter.
    fn send_window(
        stream: &mut UnixStream,
        frame: &[u8],
        ordinal: u32,
        seq: u32,
        scratch: &mut Vec<u8>,
    ) -> Result<usize, StreamError<io::Error>> {
        let tag = {
            let d = cgap_arrow::blake3::hash(frame);
            let mut t = [0u8; AOQ1_TAG_BYTES];
            t.copy_from_slice(&d[..AOQ1_TAG_BYTES]);
            t
        };
        let mut meta = [0u8; 120];
        let h = AoqHeader::new(EPOCH, KEY_ID, ordinal, seq);
        codec::encode(&h, &mut meta[..64])
            .map_err(|_| StreamError::Framing(ipc::IpcError::TagMissing))?;
        let blen = frame.len() as u32;
        meta[64..68].copy_from_slice(&blen.to_le_bytes());
        meta[68..84].copy_from_slice(&tag);
        let ipc_meta = MetaHeader {
            magic: AOQ1_MAGIC,
            version: 2,
            flags: EPOCH,
            schema_digest: [0; 32],
            body_len: blen,
            alignment: 64,
            seq,
            reserved0: 0,
            header_digest: 0,
            pad_tail: [0; 4],
        };
        scratch.clear();
        scratch.resize(window_len(frame.len(), 64), 0u8);
        let mut writer = StreamWriter::new(PatchSink::new(stream, meta));
        writer.emit(scratch, ipc_meta, frame)
    }

    /// Binary entry: one UDS path argument; bind, then accept connections
    /// SEQUENTIALLY until the stop signal — stdin EOF. Each accepted
    /// connection gets a FRESH session ([`serve`] constructs its own), so a
    /// multi-hop court reuses one process without cross-connection state;
    /// every fault still refuses and closes THAT connection only (the
    /// fail-closed-per-window law is unchanged). Exit codes: 0 on stdin EOF,
    /// 2 on bind/accept fault (fail-closed, never a fallback).
    pub(super) fn main() {
        let mut args = env::args_os();
        let _ = args.next();
        let path = match args.next() {
            Some(p) => p,
            None => {
                eprintln!("usage: aoq_sidecar <uds-path>");
                exit(2);
            }
        };
        let listener = match UnixListener::bind(&path) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("bind {path:?}: {e}");
                exit(2);
            }
        };
        // Stop signal: stdin EOF. A blocking read in a watcher thread — the
        // runtime owns time (no spin, no sleep; the thread parks in read(2)
        // until the operator closes the pipe). `std::process::exit` from the
        // watcher unwinds the accept loop from any state, including a live
        // accept(2) in the main thread. The socket path is unlinked BEFORE
        // the exit (the sidecar cleans up its own socket; the zd-42 stale-
        // socket class starts here).
        let unlink_path = path.clone();
        std::thread::spawn(move || {
            let stdin = io::stdin();
            let mut lock = stdin.lock();
            let mut buf = [0u8; 256];
            loop {
                match lock.read(&mut buf) {
                    Ok(0) | Err(_) => {
                        let _ = std::fs::remove_file(&unlink_path);
                        exit(0)
                    }
                    Ok(_) => continue,
                }
            }
        });
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    serve(stream);
                    // MULTIPLEX-REFUSAL DRAIN (PL7): a connection that
                    // queued in the backlog DURING the sequential serve is
                    // refused, not silently queued for a next round — a
                    // silently-queued peer would wait unbounded for a turn
                    // the single-tenant loop never grants. Nonblocking
                    // accept until the backlog drains (WouldBlock = empty);
                    // every queued peer gets the frozen-band BUSY word
                    // (0x4347A003) and a close. The listener returns to
                    // blocking mode for the next serve round.
                    if let Err(e) = listener.set_nonblocking(true) {
                        eprintln!("listener nonblocking: {e}");
                        exit(2);
                    }
                    loop {
                        match listener.accept() {
                            Ok((mut queued, _)) => {
                                // The peer may have already closed (the
                                // connect-then-instant-close churn class):
                                // the BUSY write is best-effort, the close
                                // (drop) is the law.
                                let _ = send_status(&mut queued, AOQ_STATUS_BUSY);
                            }
                            // WouldBlock = backlog drained; re-block.
                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                            // A peer that aborted between connect and
                            // accept is a per-connection fault, not poison:
                            // skip it and keep draining.
                            Err(ref e) if e.kind() == io::ErrorKind::ConnectionAborted => continue,
                            Err(e) => {
                                eprintln!("accept: {e}");
                                exit(2);
                            }
                        }
                    }
                    if let Err(e) = listener.set_nonblocking(false) {
                        eprintln!("listener re-blocking: {e}");
                        exit(2);
                    }
                }
                Err(e) => {
                    eprintln!("accept: {e}: poisoned listener");
                    exit(2);
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // Courts (#[cfg(test)]): fault matrix + alloc census, same-binary
    // (the sidecar functions are private to this bin; driving them over a
    // real socketpair keeps the court Chicago — real collaborator, final
    // state asserted, no mocks).
    // -------------------------------------------------------------------------

    #[cfg(test)]
    mod tests {
        use super::*;

        fn socketpair() -> (UnixStream, UnixStream) {
            let (a, b) = std::os::unix::net::UnixStream::pair().expect("socketpair");
            (a, b)
        }

        /// blake3(body)[..16] — the tamper-tier AOQ1 wire tag.
        fn wire_tag(body: &[u8]) -> [u8; 16] {
            let d = cgap_arrow::blake3::hash(body);
            let mut t = [0u8; 16];
            t.copy_from_slice(&d[..16]);
            t
        }

        /// One align-64 AOQ1 window carrying `frame` (header law bytes at
        /// 72..92: body_len u32 LE, tag at 76..92).
        fn seal_window(frame: &[u8], ordinal: u32, seq: u32) -> Vec<u8> {
            let tag = wire_tag(frame);
            let mut w = vec![0u8; 8 + 120 + frame.len() + ipc::pad_to(frame.len(), 64) + 8];
            w[0..4].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
            w[4..8].copy_from_slice(&120u32.to_le_bytes());
            let mut hdr = [0u8; 64];
            codec::encode(&AoqHeader::new(EPOCH, KEY_ID, ordinal, seq), &mut hdr)
                .expect("codec::encode");
            w[8..72].copy_from_slice(&hdr);
            w[72..76].copy_from_slice(&(frame.len() as u32).to_le_bytes());
            w[76..92].copy_from_slice(&tag);
            w[128..128 + frame.len()].copy_from_slice(frame);
            let eos_at = 8 + 120 + frame.len() + ipc::pad_to(frame.len(), 64);
            w[eos_at..eos_at + 4].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
            w
        }

        /// [64B header][kind][body] session-frame builder.
        fn wire_frame(kind: sess::FrameKind, ordinal: u32, seq: u32, body: &[u8]) -> Vec<u8> {
            let mut buf = vec![0u8; FRAME_HEADER_BYTES + 1 + body.len()];
            let h = AoqHeader::new(EPOCH, KEY_ID, ordinal, seq);
            codec::encode(&h, &mut buf).expect("codec::encode");
            buf[FRAME_HEADER_BYTES] = kind.id();
            buf[FRAME_HEADER_BYTES + 1..].copy_from_slice(body);
            buf
        }

        // ---------------------------------------------------------------------
        // FAULT MATRIX — every ReadFault arm, over a real socketpair
        // ---------------------------------------------------------------------

        /// Ok path: a lawful window round-trips byte-identical.
        #[test]
        fn fault_matrix_ok_window() {
            let (mut a, mut b) = socketpair();
            let win = seal_window(
                &wire_frame(sess::FrameKind::HandshakeEk, 0, 0, &[7u8; 8]),
                0,
                0,
            );
            a.write_all(&win).expect("write window");
            let got = read_window(&mut b).expect("read_window Ok");
            assert_eq!(got, win, "window round-trips byte-identical");
        }

        /// Zero bytes at offset 0 -> CleanClose (never Short).
        #[test]
        fn fault_matrix_clean_close() {
            let (a, mut b) = socketpair();
            drop(a); // immediate EOF, zero bytes written
            match read_window(&mut b) {
                Err(ReadFault::CleanClose) => {}
                other => panic!("expected CleanClose, got {other:?}"),
            }
        }

        /// Partial prefix then EOF -> Short.
        #[test]
        fn fault_matrix_short_prefix() {
            let (mut a, mut b) = socketpair();
            a.write_all(&[0xFFu8; 4]).expect("partial prefix");
            drop(a);
            match read_window(&mut b) {
                Err(ReadFault::Short) => {}
                other => panic!("expected Short, got {other:?}"),
            }
        }

        /// Partial prefix then a STALL (peer holds, never closes) ->
        /// ClientTimeout, not Short, once the per-connection deadline
        /// expires. The court drives the fault function directly with a
        /// 200ms SO_RCVTIMEO on the server-side socket (the same OS
        /// deadline serve() installs; a bounded poll, not a sleep — the
        /// kernel owns the wait).
        #[test]
        fn fault_matrix_client_timeout() {
            let (mut a, mut b) = socketpair();
            b.set_read_timeout(Some(std::time::Duration::from_millis(200)))
                .expect("set court read timeout");
            a.write_all(&[0xFFu8; 4]).expect("partial prefix");
            // Peer deliberately does NOT close and does NOT send more.
            match read_window(&mut b) {
                Err(ReadFault::ClientTimeout) => {}
                Err(ReadFault::Short) => {
                    panic!("deadline expired but fault read as Short (timeout not mapped)")
                }
                other => panic!("expected ClientTimeout, got {other:?}"),
            }
            // A FRESH read_window after the timeout sees the peer's later
            // close at offset 0 of the new call: CleanClose. (The deadline
            // refreshes per read call; the partial 4 bytes from the timed-
            // out call were discarded with it.)
            drop(a);
            match read_window(&mut b) {
                Err(ReadFault::CleanClose) => {}
                other => panic!("expected CleanClose after peer close, got {other:?}"),
            }
        }

        /// Full prefix + meta with body_len > MAX_BODY -> Oversize (no
        /// window allocation — the gate fires before the single alloc).
        #[test]
        fn fault_matrix_oversize() {
            let (mut a, mut b) = socketpair();
            let mut w = vec![0u8; 8 + 120];
            w[0..4].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
            w[4..8].copy_from_slice(&120u32.to_le_bytes());
            w[72..76].copy_from_slice(&((MAX_BODY + 1) as u32).to_le_bytes());
            a.write_all(&w).expect("prefix+meta");
            match read_window(&mut b) {
                Err(ReadFault::Oversize) => {}
                other => panic!("expected Oversize, got {other:?}"),
            }
        }

        /// Prefix with a non-align-64 meta_span -> BadFraming.
        #[test]
        fn fault_matrix_bad_framing() {
            let (mut a, mut b) = socketpair();
            let mut w = vec![0u8; 8];
            w[0..4].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
            w[4..8].copy_from_slice(&119u32.to_le_bytes());
            a.write_all(&w).expect("bad prefix");
            match read_window(&mut b) {
                Err(ReadFault::BadFraming) => {}
                other => panic!("expected BadFraming, got {other:?}"),
            }
        }

        /// Oversize is measured under the counting allocator: the gate
        /// refuses BEFORE any window allocation (0 allocs on the fault path).
        #[test]
        fn oversize_refuses_alloc_free() {
            let (mut a, mut b) = socketpair();
            let mut w = vec![0u8; 8 + 120];
            w[0..4].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
            w[4..8].copy_from_slice(&120u32.to_le_bytes());
            w[72..76].copy_from_slice(&((MAX_BODY + 1) as u32).to_le_bytes());
            a.write_all(&w).expect("prefix+meta");
            let (res, report) =
                cgap_audit::alloc_counter::with_alloc_tracking(|| read_window(&mut b));
            assert!(matches!(res, Err(ReadFault::Oversize)));
            assert_eq!(
                report.count, 0,
                "Oversize refusal must not allocate (gate before alloc)"
            );
        }

        // ---------------------------------------------------------------------
        // ALLOC CENSUS — the single-alloc read_window shape
        // ---------------------------------------------------------------------

        /// One window == ONE heap allocation (the window Vec itself; the
        /// 8+120-byte head is stack + memcpy into place). Before the
        /// alloc-elimination lane this was 2 allocs + 3 memcpys (a body
        /// Vec, then three extend_from_slice copies into a second Vec).
        #[test]
        fn read_window_single_alloc_census() {
            let (mut a, mut b) = socketpair();
            let win = seal_window(
                &wire_frame(sess::FrameKind::HandshakeEk, 0, 0, &[9u8; 40]),
                0,
                0,
            );
            a.write_all(&win).expect("write window");
            let (got, report) =
                cgap_audit::alloc_counter::with_alloc_tracking(|| read_window(&mut b));
            assert_eq!(got.expect("Ok"), win);
            assert_eq!(
                report.count, 1,
                "read_window: exactly ONE allocation per window (report: {report:?})"
            );
            assert_eq!(report.bytes, win.len(), "the alloc IS the window");
        }

        // ---------------------------------------------------------------------
        // WARM SERVE CENSUS — steady-state frames are alloc-free
        // ---------------------------------------------------------------------

        /// Full client leg over one serve() connection: real handshake
        /// (Ek + Finish under the sidecar's own deterministic seam) then
        /// `n` same-size AppData frames, then clean close. Returns the
        /// allocation count observed on the SERVE thread.
        fn serve_census(n_appdata: usize) -> usize {
            let (server, mut client) = socketpair();
            let handle = std::thread::spawn(move || {
                let ((), report) =
                    cgap_audit::alloc_counter::with_alloc_tracking(move || serve(server));
                report.count
            });

            // Handshake (mirrors J1 in aoq_sidecar_court.rs; same pinned
            // constants — the sidecar seam is deterministic).
            let d = shake32(&[b"w2c/mlkem/d"]);
            let z = shake32(&[b"w2c/mlkem/z"]);
            let mut ek = [0u8; EK_BYTES];
            let mut dk = [0u8; 2400];
            cgap_crypto::kem::keygen(&d, &z, &mut ek, &mut dk);
            let kem = SidecarKem;
            let (ct_c, ss) = kem.encapsulate(&ek).expect("encapsulate");
            let ek_win = seal_window(&wire_frame(sess::FrameKind::HandshakeEk, 0, 0, &ek), 0, 0);
            client.write_all(&ek_win).expect("ek write");
            let mut word = [0u8; 4];
            client.read_exact(&mut word).expect("ek word");
            assert_eq!(u32::from_le_bytes(word), AOQ_STATUS_OK);

            let transcript = sess::transcript_tag(&kem, &ek, &ss, EPOCH);
            let hs_master = sess::derive(&kem, sess::CTX_HANDSHAKE, &ss, &ek);
            let recv_key = kem.kdf(&[sess::CTX_RECV, &hs_master]);
            let confirm = sess::confirm_tag(&kem, &recv_key, &transcript, &ct_c);
            let mut fin_body = [0u8; CT_BYTES + 16];
            fin_body[..CT_BYTES].copy_from_slice(&ct_c);
            fin_body[CT_BYTES..].copy_from_slice(&confirm);
            let fin_win = seal_window(
                &wire_frame(sess::FrameKind::HandshakeFinish, 1, 0, &fin_body),
                1,
                0,
            );
            client.write_all(&fin_win).expect("fin write");
            client.read_exact(&mut word).expect("fin word");
            assert_eq!(u32::from_le_bytes(word), AOQ_STATUS_OK);

            // n same-size AppData frames (seq/ordinal advance per frame).
            let payload = [0xA5u8; 64];
            for i in 0..n_appdata {
                let ordinal = 2 + i as u32;
                let seq = 7 + i as u32;
                let mut prefix = [0u8; FRAME_HEADER_BYTES + 5];
                codec::encode(&AoqHeader::new(EPOCH, KEY_ID, ordinal, seq), &mut prefix)
                    .expect("encode");
                prefix[FRAME_HEADER_BYTES] = sess::FrameKind::AppData.id();
                prefix[FRAME_HEADER_BYTES + 1..FRAME_HEADER_BYTES + 5]
                    .copy_from_slice(&seq.to_le_bytes());
                let mut app_body = vec![0u8; 4 + payload.len() + cgap_aoq::aead::TAG_LEN];
                app_body[..4].copy_from_slice(&seq.to_le_bytes());
                let nonce = cgap_aoq::aead::Nonce96::new(
                    u32::from(EPOCH),
                    (u64::from(ordinal) << 32) | u64::from(seq),
                )
                .expect("nonce");
                let mut aead_tag = [0u8; cgap_aoq::aead::TAG_LEN];
                cgap_aoq::aead::seal_detached(
                    &mut app_body[4..4 + payload.len()],
                    &mut aead_tag,
                    &payload,
                    &prefix,
                    &recv_key,
                    &nonce,
                );
                app_body[4 + payload.len()..].copy_from_slice(&aead_tag);
                let app_win = seal_window(
                    &wire_frame(sess::FrameKind::AppData, ordinal, seq, &app_body),
                    ordinal,
                    seq,
                );
                client.write_all(&app_win).expect("app write");
                client.read_exact(&mut word).expect("app word");
                assert_eq!(u32::from_le_bytes(word), AOQ_STATUS_OK);
                // Drain the response window (status OK precedes it).
                let mut pfx = [0u8; 8];
                client.read_exact(&mut pfx).expect("resp prefix");
                let span = u32::from_le_bytes([pfx[4], pfx[5], pfx[6], pfx[7]]) as usize;
                assert_eq!(span, 120);
                let mut meta = [0u8; 120];
                client.read_exact(&mut meta).expect("resp meta");
                let blen = u32::from_le_bytes([meta[64], meta[65], meta[66], meta[67]]) as usize;
                let mut rest = vec![0u8; blen + ipc::pad_to(blen, 64) + 8];
                client.read_exact(&mut rest).expect("resp body");
            }

            drop(client); // CleanClose: serve returns
            let _ = dk;
            handle.join().expect("serve thread")
        }

        /// Steady-state law: each additional same-size AppData frame costs
        /// EXACTLY ONE heap allocation — the single window Vec of
        /// [`read_window`] (the alloc-elimination target shape; one alloc
        /// plus two head memcpys, body streamed in place). The response leg
        /// (open/emit/send_window/PatchSink) contributes ZERO: buffers are
        /// hoisted per connection and resized in place, so frames 2..n
        /// never allocate. Before the lane, the per-frame cost was 2
        /// allocs in read_window plus 4 per-frame `vec!`s in the
        /// request/response path; the delta here proves the warm path is
        /// window-alloc-only.
        #[test]
        fn serve_warm_frames_window_alloc_only() {
            let one = serve_census(1);
            let eight = serve_census(8);
            assert_eq!(
                eight,
                one + 7,
                "per-frame delta must be EXACTLY the 1 window alloc (1 frame: {one}, 8 frames: {eight})"
            );
        }
    }
}
