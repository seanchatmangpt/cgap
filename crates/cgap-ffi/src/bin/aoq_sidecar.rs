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
        AOQ_STATUS_EPOCH_RETIRED, AOQ_STATUS_INCOMPLETE_FRAME,
        AOQ_STATUS_OK,
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
    enum ReadFault {
        /// Peer closed before the first byte (clean end of service).
        CleanClose,
        /// Bytes arrived but the window is incomplete.
        Short,
        /// Framing word refused (`meta_span` not the align-64 span).
        BadFraming,
        /// `body_len` word exceeds [`MAX_BODY`].
        Oversize,
    }

    /// Read exactly one AOQ1 window off the stream.
    ///
    /// Zero-byte close is [`ReadFault::CleanClose`]; a partial window (bytes
    /// then EOF) is [`ReadFault::Short`] — refused, never accepted.
    fn read_window(stream: &mut UnixStream) -> Result<Vec<u8>, ReadFault> {
        let mut prefix = [0u8; PREFIX_SIZE];
        let mut got = 0usize;
        while got < prefix.len() {
            let n = stream
                .read(&mut prefix[got..])
                .map_err(|_| ReadFault::Short)?;
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
        let mut rest = vec![0u8; body_len + body_pad + EOS_SIZE];
        read_full(stream, &mut rest)?;
        let mut window = Vec::with_capacity(PREFIX_SIZE + META_SPAN + rest.len());
        window.extend_from_slice(&prefix);
        window.extend_from_slice(&meta);
        window.extend_from_slice(&rest);
        Ok(window)
    }

    /// `read_exact` semantics over [`Read::read`]: loops until the buffer is
    /// full; any zero read is a typed short fault.
    fn read_full(stream: &mut UnixStream, buf: &mut [u8]) -> Result<usize, ReadFault> {
        let mut got = 0usize;
        while got < buf.len() {
            let n = stream.read(&mut buf[got..]).map_err(|_| ReadFault::Short)?;
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
    /// private `status_of` in `aoq_session.rs` — same pinned collisions).
    fn session_status(err: &SessionError) -> u32 {
        match err {
            SessionError::LengthMismatch { .. } => AOQ_STATUS_INCOMPLETE_FRAME,
            SessionError::Wire(AoqError::BadMagic { .. })
            | SessionError::Wire(AoqError::UnsupportedVersion { .. })
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
                let pieces = if abs_hi <= plo || abs_lo >= phi {
                    vec![IoSlice::new(seg)]
                } else {
                    let lo = plo.max(abs_lo);
                    let hi = phi.min(abs_hi);
                    let mut v = Vec::with_capacity(3);
                    if lo > abs_lo {
                        v.push(IoSlice::new(&seg[..lo - abs_lo]));
                    }
                    v.push(IoSlice::new(&self.meta[lo - plo..hi - plo]));
                    if abs_hi > hi {
                        v.push(IoSlice::new(&seg[hi - abs_lo..]));
                    }
                    v
                };
                let n = self.stream.write_vectored(&pieces)?;
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
        let kem = SidecarKem;
        let mut session = Session::new(&kem, EPOCH, KEY_ID, ORDINAL0);
        loop {
            let window = match read_window(&mut stream) {
                Ok(w) => w,
                Err(ReadFault::CleanClose) => return,
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
            let mut body = vec![0u8; body_len];
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
                let mut plain = vec![0u8; ct_len];
                if let Err(err) = session.open_verified(&body, &mut plain) {
                    let _ = send_status(&mut stream, session_status(&err));
                    return;
                }
                let payload = &plain;
                // v3 wire law: emit writes [header][kind][seq][ct][tag 16B].
                let mut frame_buf = vec![
                    0u8;
                    FRAME_HEADER_BYTES + 1 + 4 + payload.len()
                        + cgap_aoq::aead::TAG_LEN
                ];
                match session.emit_verified(seq, payload, &mut frame_buf) {
                    Ok(written) => {
                        let _ = send_status(&mut stream, AOQ_STATUS_OK);
                        if let Err(err) = send_window(
                            &mut stream,
                            &frame_buf[..written],
                            frame.header.frame_ordinal,
                            seq,
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
        let mut scratch = vec![0u8; window_len(frame.len(), 64)];
        let mut writer = StreamWriter::new(PatchSink::new(stream, meta));
        writer.emit(&mut scratch, ipc_meta, frame)
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
}
