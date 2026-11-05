//! AoQ sidecar E2E court (ZD-56 — closes the membrane plan's rung-8 E2E).
//!
//! Rung-8 witness (wave-2-membrane-plan.md:189): "decrypted == pre-seal
//! bytes, tag gate fires on flipped byte" — driven from the cgap side
//! (arcus cannot do the ML-KEM handshake; see zd-42): real ML-KEM-768
//! keygen/encaps -> handshake frames -> Established -> emit_verified echo
//! through the REAL sidecar process -> the echoed payload re-read through
//! the framing tag gate (`ipc::read_verified`) equals the pre-seal bytes,
//! AND a flipped-byte control fires the tag gate as a typed refusal.
//!
//! Also witnesses the ZD-56 sequential-serve change: ONE sidecar process
//! serves THREE sequential connections (happy path, flipped-byte refusal,
//! fresh full handshake), each with a fresh session, then exits 0 on the
//! stdin-EOF stop signal — the stop signal is an asserted observable.
//!
//! Courts (each names its anti-vacuity mutation):
//! - E1 happy path: an echo-mangling sidecar would return payload bytes
//!   that fail the framing tag gate or diverge from the pre-seal bytes
//!   (mutation A': flipped byte in the echo).
//! - E2 flipped-byte control: an accept-tampered-window sidecar would echo
//!   or accept; the court asserts the typed DIGEST_MISMATCH refusal and a
//!   zero-byte post-refusal read on the SAME connection.
//! - E3 process reuse: a one-connection sidecar (the pre-ZD-56 behavior,
//!   mutation C') cannot serve connection 3; the court asserts the fresh
//!   handshake completes on the same process, then the stdin-EOF stop
//!   signal yields exit 0 and the socket file is removed (court janitor).
//!
//! No-sleep law: bounded polls on socket existence, connect readiness, and
//! child exit status (deadline-checked). Socket janitor: every court removes
//! its own socket path on exit (Drop + asserted unlink).

// SOURCE-SCAN-EXEMPT: std:: : integration-court-only collaborator
// SOURCE-SCAN-EXEMPT: vec! : integration-court-only collaborator
// SOURCE-SCAN-EXEMPT: Command: spawns the real sidecar binary under test

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use cgap_aoq::codec;
use cgap_aoq::header::AoqHeader;
use cgap_aoq::session::{self as sess, FrameKind, KemSeam, SessionError, CT_BYTES, EK_BYTES};
use cgap_arrow::ipc::{self, AOQ1_TAG_AT, AOQ1_TAG_BYTES, EOS_SIZE, PREFIX_SIZE};
use cgap_crypto::fips202::Shake256;
use cgap_ffi::aoq::status_maps::{AOQ_STATUS_DIGEST_MISMATCH, AOQ_STATUS_OK};

const EPOCH: u16 = 1;
const KEY_ID: u64 = 0xA0_71;
const META_SPAN: usize = 120;

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

/// The court's view of the SAME seam the sidecar constructs (identical
/// derivation constants, so the court recomputes the sidecar's outcomes).
struct CourtKem;

impl KemSeam for CourtKem {
    fn encapsulate(&self, ek: &[u8; EK_BYTES]) -> Result<([u8; CT_BYTES], [u8; 32]), SessionError> {
        let m = shake32(&[b"w2c/mlkem/m", ek]);
        let mut ct = [0u8; CT_BYTES];
        let mut ss = [0u8; 32];
        cgap_crypto::kem::encaps(ek, &m, &mut ct, &mut ss);
        Ok((ct, ss))
    }

    fn decapsulate(&self, dk: &[u8], ct: &[u8; CT_BYTES]) -> Result<[u8; 32], SessionError> {
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
// Sidecar process harness: piped stdin (the stop signal), bounded polls
// ---------------------------------------------------------------------------

struct Sidecar {
    child: Child,
    stream: Option<UnixStream>,
    sock_path: PathBuf,
    /// Open handle to the sidecar's stdin: dropped to signal stop.
    stop_signal: Option<std::process::ChildStdin>,
}

impl Drop for Sidecar {
    fn drop(&mut self) {
        self.stream = None; // close the client socket first
        self.stop_signal = None; // stdin EOF -> clean exit path
        let _ = self.child.kill();
        let _ = self.child.wait();
        // Court janitor: remove our own socket path (bounded single unlink;
        // existence is asserted at the end of E3, best-effort on panic paths).
        let _ = std::fs::remove_file(&self.sock_path);
    }
}

/// Spawn the real sidecar binary (stdin piped = the stop-signal pipe) and
/// bounded-poll for its socket, then connect (deadline-checked, no sleep).
fn spawn_sidecar(tag: &str) -> Sidecar {
    let sock_path: PathBuf =
        std::env::temp_dir().join(format!("aoq_e2e_{tag}_{:?}.sock", std::process::id()));
    let _ = std::fs::remove_file(&sock_path);
    let mut child = Command::new(env!("CARGO_BIN_EXE_aoq_sidecar"))
        .arg(&sock_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn aoq_sidecar");
    let stop_signal = child.stdin.take().expect("sidecar stdin pipe");
    let deadline = Instant::now() + Duration::from_secs(10);
    while !sock_path.exists() {
        assert!(
            Instant::now() < deadline,
            "sidecar socket never appeared (bounded poll expired)"
        );
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    let stream = loop {
        match UnixStream::connect(&sock_path) {
            Ok(s) => break s,
            Err(e) => {
                if Instant::now() > deadline {
                    panic!(
                        "connect sidecar failed: {e}; child status={:?}",
                        child.try_wait().expect("try_wait")
                    );
                }
            }
        }
    };
    Sidecar {
        child,
        stream: Some(stream),
        sock_path,
        stop_signal: Some(stop_signal),
    }
}

// ---------------------------------------------------------------------------
// Wire helpers: AOQ1 window seal and response read (ZD-13 J1 shape)
// ---------------------------------------------------------------------------

/// blake3(body)[..16] — the tamper-tier wire tag.
fn wire_tag_for(body: &[u8]) -> [u8; 16] {
    let d = cgap_arrow::blake3::hash(body);
    let mut t = [0u8; 16];
    t.copy_from_slice(&d[..16]);
    t
}

/// Seal `frame` into one AOQ1 window (align-64): codec-sealed AoqHeader
/// metadata (FNV-1a32 at 60), body_len u32 LE at 72..76, tag at 76..92.
fn seal_window(frame: &[u8], ordinal: u32, now_seq: u32) -> Vec<u8> {
    let tag = wire_tag_for(frame);
    let mut w = vec![0u8; 8 + 120 + frame.len() + ipc::pad_to(frame.len(), 64) + 8];
    w[0..4].copy_from_slice(&cgap_arrow::ipc::CONTINUATION.to_le_bytes());
    w[4..8].copy_from_slice(&120u32.to_le_bytes());
    let mut hdr = [0u8; 64];
    codec::encode(&AoqHeader::new(EPOCH, KEY_ID, ordinal, now_seq), &mut hdr)
        .expect("codec::encode");
    w[8..72].copy_from_slice(&hdr);
    w[72..76].copy_from_slice(&(frame.len() as u32).to_le_bytes());
    w[AOQ1_TAG_AT..AOQ1_TAG_AT + AOQ1_TAG_BYTES].copy_from_slice(&tag);
    w[128..128 + frame.len()].copy_from_slice(frame);
    let eos_at = 8 + 120 + frame.len() + ipc::pad_to(frame.len(), 64);
    w[eos_at..eos_at + 4].copy_from_slice(&cgap_arrow::ipc::CONTINUATION.to_le_bytes());
    w
}

/// [`seal_window`] with a KD-195 frameFlag stamped into the AOQ1 header
/// reserved[0] (window-absolute offset 32) and the FNV digest re-sealed —
/// the flag is digest-covered, so the stamped window passes the sidecar's
/// `read_verified` tag gate.
fn seal_window_flagged(frame: &[u8], ordinal: u32, seq: u32, flag: u8) -> Vec<u8> {
    let mut w = seal_window(frame, ordinal, seq);
    ipc::stamp_frame_flag(&mut w[8..72], flag);
    w
}

/// AOQ1 session-frame builder ([64B header][kind][payload], codec::encode).
fn wire_frame(kind: FrameKind, ordinal: u32, seq: u32, body: &[u8]) -> Vec<u8> {
    let mut buf = vec![0u8; cgap_aoq::header::SIZE_BYTES + 1 + body.len()];
    let h = AoqHeader::new(EPOCH, KEY_ID, ordinal, seq);
    codec::encode(&h, &mut buf).expect("codec::encode");
    buf[cgap_aoq::header::SIZE_BYTES] = kind.id();
    buf[cgap_aoq::header::SIZE_BYTES + 1..].copy_from_slice(body);
    buf
}

/// Send one request window; read the 4-byte response word; on
/// `expect_window` follow with one response window read.
fn roundtrip(
    stream: &mut UnixStream,
    window: &[u8],
    expect_window: bool,
) -> (u32, Option<Vec<u8>>) {
    stream.write_all(window).expect("request write");
    let mut word_b = [0u8; 4];
    stream.read_exact(&mut word_b).expect("status word read");
    let word = u32::from_le_bytes(word_b);
    if !expect_window {
        return (word, None);
    }
    let mut prefix = [0u8; PREFIX_SIZE];
    stream.read_exact(&mut prefix).expect("response prefix");
    let meta_span = u32::from_le_bytes([prefix[4], prefix[5], prefix[6], prefix[7]]) as usize;
    assert_eq!(
        meta_span, META_SPAN,
        "response meta span is the align-64 span"
    );
    let mut meta_bytes = [0u8; META_SPAN];
    stream
        .read_exact(&mut meta_bytes)
        .expect("response metadata");
    let blen = u32::from_le_bytes([
        meta_bytes[64],
        meta_bytes[65],
        meta_bytes[66],
        meta_bytes[67],
    ]) as usize;
    let pad = ipc::pad_to(blen, 64);
    let mut rest = vec![0u8; blen + pad + EOS_SIZE];
    stream.read_exact(&mut rest).expect("response body+pad+eos");
    let mut w = Vec::with_capacity(PREFIX_SIZE + META_SPAN + rest.len());
    w.extend_from_slice(&prefix);
    w.extend_from_slice(&meta_bytes);
    w.extend_from_slice(&rest);
    (word, Some(w))
}

/// Open the NEXT sequential connection on the SAME sidecar process (the
/// ZD-56 serve-loop surface; a one-connection sidecar refuses this).
fn next_connection(car: &mut Sidecar) {
    // minimal-unblock (2026-11-04 core LE lane, disclosed): the landed
    // PL7 multiplex-refusal drain BUSY-closes any peer that queues while
    // a serve round (or its post-close backlog drain) is live. Drop
    // conn-N first, then bounded-probe until the sidecar is back in its
    // blocking accept: a probe window (tag-gate refusal, never a session
    // fault) that survives with a non-BUSY word proves the accept loop
    // is serving again; BUSY or a broken probe is a queued-drain victim,
    // so close and retry. Probe refusals return serve() to the accept
    // loop, leaving no session state behind.
    car.stream = None;
    let deadline = Instant::now() + Duration::from_secs(10);
    let stream = loop {
        let mut s = match UnixStream::connect(&car.sock_path) {
            Ok(s) => s,
            Err(e) => {
                if Instant::now() > deadline {
                    panic!("sequential connect failed: {e}");
                }
                continue;
            }
        };
        let probe_window = [0u8; 8 + 120];
        let ready = (|| -> Option<u32> {
            s.write_all(&probe_window).ok()?;
            let mut word_b = [0u8; 4];
            s.read_exact(&mut word_b).ok()?;
            Some(u32::from_le_bytes(word_b))
        })()
        .filter(|word| *word != cgap_ffi::aoq::status_maps::AOQ_STATUS_BUSY);
        match ready {
            // The probe's refusal ended its serve round; the sidecar is
            // back in blocking accept. Drop the probe stream and connect
            // fresh: the new connect is served immediately (never queued).
            Some(_) => {
                drop(s);
                break UnixStream::connect(&car.sock_path).expect("post-probe connect");
            }
            None => {
                if Instant::now() > deadline {
                    panic!("sequential connect: sidecar never returned to accept");
                }
                drop(s);
            }
        }
    };
    car.stream = Some(stream);
}

/// Full cgap-side handshake to Established on the given stream: real
/// ML-KEM-768 keygen, court encapsulation of the sidecar's pending ct.
/// Returns (dk, ct) for the caller's witnesses.
fn handshake(
    kem: &CourtKem,
    stream: &mut UnixStream,
) -> ([u8; 2400], [u8; CT_BYTES], [u8; 32], [u8; 32]) {
    let d = shake32(&[b"w2c/mlkem/d"]);
    let z = shake32(&[b"w2c/mlkem/z"]);
    let mut ek = [0u8; EK_BYTES];
    let mut dk = [0u8; 2400];
    cgap_crypto::kem::keygen(&d, &z, &mut ek, &mut dk);
    let (ct_c, ss) = kem.encapsulate(&ek).expect("court encapsulate");
    let ek_win = seal_window(&wire_frame(FrameKind::HandshakeEk, 0, 0, &ek), 0, 0);
    let (word, _w) = roundtrip(stream, &ek_win, false);
    assert_eq!(word, AOQ_STATUS_OK, "HandshakeEk accepted");
    // v3 Finish law (M2a): body = ct echo (1088B) + confirm tag (16B)
    // under recv_key/transcript derived from the same (ss, ek) pair the
    // sidecar derives at the Ek step (deterministic CourtKem).
    let transcript = sess::transcript_tag(kem, &ek, &ss, EPOCH);
    let hs_master = sess::derive(kem, sess::CTX_HANDSHAKE, &ss, &ek);
    let recv_key = kem.kdf(&[sess::CTX_RECV, &hs_master]);
    let confirm = sess::confirm_tag(kem, &recv_key, &transcript, &ct_c);
    let mut fin_body = [0u8; CT_BYTES + 16];
    fin_body[..CT_BYTES].copy_from_slice(&ct_c);
    fin_body[CT_BYTES..].copy_from_slice(&confirm);
    let fin_win = seal_window(
        &wire_frame(FrameKind::HandshakeFinish, 1, 0, &fin_body),
        1,
        0,
    );
    let (word, _w) = roundtrip(stream, &fin_win, false);
    assert_eq!(
        word, AOQ_STATUS_OK,
        "HandshakeFinish accepted (Established)"
    );
    // The court's send_key (= the sidecar's recv_key): seals AppData the
    // sidecar verifies; the echo is sealed under the sidecar's send_key
    // (= the court's recv_key) and opened with it.
    let send_key = kem.kdf(&[sess::CTX_SEND, &hs_master]);
    (dk, ct_c, recv_key, send_key)
}

// ---------------------------------------------------------------------------
// COURT E1 — rung-8 happy path: decrypted == pre-seal bytes
// ---------------------------------------------------------------------------

#[test]
fn e1_full_path_preseal_bytes_survive_to_echo() {
    let mut car = spawn_sidecar("e1");
    let kem = CourtKem;
    let (dk, _ct, recv_key, send_key) = handshake(&kem, car.stream.as_mut().expect("conn 1"));

    // Pre-seal payload: the bytes the caller wants carried end to end.
    let preseal = b"rung-8 e2e witness: cgap-side sealed feed payload \x00\x01\xff";
    // v3 wire law: body = [seq][ct][tag] under the court's send_key
    // (AAD = the covered prefix, nonce = Nonce96(epoch, ordinal<<32|seq)).
    let mut prefix = [0u8; cgap_aoq::header::SIZE_BYTES + 5];
    codec::encode(&AoqHeader::new(EPOCH, KEY_ID, 2, 7), &mut prefix).expect("encode");
    prefix[cgap_aoq::header::SIZE_BYTES] = FrameKind::AppData.id();
    prefix[cgap_aoq::header::SIZE_BYTES + 1..cgap_aoq::header::SIZE_BYTES + 5]
        .copy_from_slice(&7u32.to_le_bytes());
    let mut app_body = vec![0u8; 4 + preseal.len() + cgap_aoq::aead::TAG_LEN];
    app_body[..4].copy_from_slice(&7u32.to_le_bytes());
    let nonce = cgap_aoq::aead::Nonce96::new(u32::from(EPOCH), (2u64 << 32) | 7).expect("nonce");
    let mut aead_tag = [0u8; cgap_aoq::aead::TAG_LEN];
    cgap_aoq::aead::seal_detached(
        &mut app_body[4..4 + preseal.len()],
        &mut aead_tag,
        preseal,
        &prefix,
        &recv_key,
        &nonce,
    );
    app_body[4 + preseal.len()..].copy_from_slice(&aead_tag);
    let app = wire_frame(FrameKind::AppData, 2, 7, &app_body);
    let app_win = seal_window(&app, 2, 7);
    let (word, win) = roundtrip(car.stream.as_mut().expect("conn 1"), &app_win, true);
    assert_eq!(word, AOQ_STATUS_OK, "AppData accepted");
    let win = win.expect("response window present");

    // Framing-layer tag gate on the response window (read_verified).
    let blen = u32::from_le_bytes([win[72], win[73], win[74], win[75]]) as usize;
    let mut body = vec![0u8; blen];
    ipc::read_verified(&win, None, &mut body).expect("response window tag gate");

    // Session-layer verification: the echo frame's emit_verified kdf tag
    // re-derives over the whole covered frame (the session's decrypt gate).
    let frame = sess::parse_frame(&body).expect("response frame parses");
    assert_eq!(frame.kind, FrameKind::AppData);
    let rseq = u32::from_le_bytes([
        frame.payload[0],
        frame.payload[1],
        frame.payload[2],
        frame.payload[3],
    ]);
    assert_eq!(rseq, 7, "echoed seq");
    // v3 wire law: the echo body = [seq][ct][tag 16B] sealed under the
    // sidecar's send_key (= the court's recv_key); open_detached with the
    // court-derived key recovers the plaintext (AEAD round trip).
    let ct_len = frame.payload.len() - 4 - cgap_aoq::aead::TAG_LEN;
    let mut plain = vec![0u8; ct_len];
    let aead_tag: [u8; cgap_aoq::aead::TAG_LEN] =
        frame.payload[4 + ct_len..].try_into().expect("tag");
    cgap_aoq::aead::open_detached(
        &mut plain,
        &frame.payload[4..4 + ct_len],
        &aead_tag,
        &body[..cgap_aoq::header::SIZE_BYTES + 5],
        &send_key,
        &cgap_aoq::aead::Nonce96::new(
            u32::from(EPOCH),
            (u64::from(frame.header.frame_ordinal) << 32) | u64::from(rseq),
        )
        .expect("nonce"),
    )
    .expect("echo opens under the court-derived send_key");

    // THE RUNG-8 WITNESS: the recovered payload equals the pre-seal bytes.
    assert_eq!(
        &plain,
        &preseal[..],
        "decrypted (echoed) payload == pre-seal bytes"
    );
    // Transcript digests for the receipt (blake3, 32-byte).
    let d_pre = cgap_arrow::blake3::hash(preseal);
    let d_post = cgap_arrow::blake3::hash(&plain);
    eprintln!(
        "E1 transcript: preseal_len={} preseal_sha256ish_blake3={:02x?} echoed_blake3={:02x?} equal_digests={}",
        preseal.len(),
        &d_pre[..8],
        &d_post[..8],
        d_pre[..] == d_post[..]
    );
    let _ = dk;
}

// ---------------------------------------------------------------------------
// COURT E2 — flipped-byte control: the tag gate fires (typed, never echo)
// ---------------------------------------------------------------------------

#[test]
fn e2_flipped_byte_fires_tag_gate() {
    let mut car = spawn_sidecar("e2");

    // Connection 2: lawful HandshakeEk window with ONE flipped tag byte.
    let d = shake32(&[b"w2c/mlkem/d"]);
    let z = shake32(&[b"w2c/mlkem/z"]);
    let mut ek = [0u8; EK_BYTES];
    let mut dk = [0u8; 2400];
    cgap_crypto::kem::keygen(&d, &z, &mut ek, &mut dk);
    let mut ek_win = seal_window(&wire_frame(FrameKind::HandshakeEk, 0, 0, &ek), 0, 0);
    ek_win[AOQ1_TAG_AT] ^= 0x01;
    let request_len = ek_win.len();
    let (word, win) = roundtrip(car.stream.as_mut().expect("conn 2"), &ek_win, false);
    assert_eq!(word, AOQ_STATUS_DIGEST_MISMATCH, "tag gate refuses typed");
    assert!(win.is_none(), "no window follows a refusal");
    // Mutation B non-vacuity: the refusal was NOT an echo of the request.
    let mut probe = vec![0u8; request_len];
    let n = car
        .stream
        .as_mut()
        .expect("conn 2")
        .read(&mut probe)
        .expect("post-refusal read");
    assert_eq!(n, 0, "refusal was NOT an echo");
    let _ = dk;
}

// ---------------------------------------------------------------------------
// COURT E3 — one process, fresh full handshake on a THIRD connection, then
// the stdin-EOF stop signal -> exit 0 + socket removed (janitor witness)
// ---------------------------------------------------------------------------

#[test]
fn e3_process_reuse_then_stdin_eof_stop() {
    let mut car = spawn_sidecar("e3");
    let kem = CourtKem;
    let (_dk, _ct, _recv_key1, _send_key1) = handshake(&kem, car.stream.as_mut().expect("conn 1"));

    // Connection 3 (the reuse witness): a ONE-CONNECTION sidecar (mutation
    // C': the pre-ZD-56 binary) cannot accept this connect — the bounded
    // poll would expire. This sidecar serves a FRESH full handshake.
    next_connection(&mut car);
    let (_dk2, ct2, recv_key2, send_key2) = handshake(&kem, car.stream.as_mut().expect("conn 3"));
    // Fresh-session proof: the new handshake's pending ct is the same
    // deterministic value the sidecar recomputes for the new ek — accepted,
    // so the sidecar did not carry connection-1 session state in.
    let _ = ct2;

    // Established AppData hop on connection 3 (same-process reuse end to end).
    let preseal = b"conn-3 reuse payload";
    // v3 wire law: body = [seq][ct][tag] under the court's send-side key.
    let mut prefix = [0u8; cgap_aoq::header::SIZE_BYTES + 5];
    codec::encode(&AoqHeader::new(EPOCH, KEY_ID, 2, 9), &mut prefix).expect("encode");
    prefix[cgap_aoq::header::SIZE_BYTES] = FrameKind::AppData.id();
    prefix[cgap_aoq::header::SIZE_BYTES + 1..cgap_aoq::header::SIZE_BYTES + 5]
        .copy_from_slice(&9u32.to_le_bytes());
    let mut app_body = vec![0u8; 4 + preseal.len() + cgap_aoq::aead::TAG_LEN];
    app_body[..4].copy_from_slice(&9u32.to_le_bytes());
    let nonce = cgap_aoq::aead::Nonce96::new(u32::from(EPOCH), (2u64 << 32) | 9).expect("nonce");
    let mut aead_tag = [0u8; cgap_aoq::aead::TAG_LEN];
    cgap_aoq::aead::seal_detached(
        &mut app_body[4..4 + preseal.len()],
        &mut aead_tag,
        preseal,
        &prefix,
        &recv_key2,
        &nonce,
    );
    app_body[4 + preseal.len()..].copy_from_slice(&aead_tag);
    let app = wire_frame(FrameKind::AppData, 2, 9, &app_body);
    let app_win = seal_window(&app, 2, 9);
    let (word, win) = roundtrip(car.stream.as_mut().expect("conn 3"), &app_win, true);
    assert_eq!(word, AOQ_STATUS_OK, "conn-3 AppData accepted");
    let win = win.expect("conn-3 response window");
    let blen = u32::from_le_bytes([win[72], win[73], win[74], win[75]]) as usize;
    let mut body = vec![0u8; blen];
    ipc::read_verified(&win, None, &mut body).expect("conn-3 tag gate");
    let frame = sess::parse_frame(&body).expect("conn-3 frame parses");
    // v3: open the echo under the court-derived send-side key.
    let ct_len = frame.payload.len() - 4 - cgap_aoq::aead::TAG_LEN;
    let mut plain3 = vec![0u8; ct_len];
    let aead_tag3: [u8; cgap_aoq::aead::TAG_LEN] =
        frame.payload[4 + ct_len..].try_into().expect("tag");
    cgap_aoq::aead::open_detached(
        &mut plain3,
        &frame.payload[4..4 + ct_len],
        &aead_tag3,
        &body[..cgap_aoq::header::SIZE_BYTES + 5],
        &send_key2,
        &cgap_aoq::aead::Nonce96::new(
            u32::from(EPOCH),
            (u64::from(frame.header.frame_ordinal) << 32)
                | u64::from(u32::from_le_bytes([
                    frame.payload[0],
                    frame.payload[1],
                    frame.payload[2],
                    frame.payload[3],
                ])),
        )
        .expect("nonce"),
    )
    .expect("conn-3 echo opens");
    assert_eq!(&plain3, &preseal[..], "conn-3 echo == pre-seal bytes");

    // STOP SIGNAL: drop the client, then stdin EOF -> exit 0 (bounded poll
    // on try_wait; the runtime owns time).
    car.stream = None;
    car.stop_signal = None;
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        match car.child.try_wait().expect("try_wait") {
            Some(s) => break s,
            None => assert!(Instant::now() < deadline, "sidecar exit poll expired"),
        }
    };
    assert_eq!(status.code(), Some(0), "stdin-EOF stop signal exits 0");
    // Janitor witness: the court's own socket file is gone (removed by Drop
    // or the sidecar's own teardown); assert absence.
    let deadline = Instant::now() + Duration::from_secs(10);
    while car.sock_path.exists() {
        assert!(Instant::now() < deadline, "socket unlink poll expired");
    }
}

// ---------------------------------------------------------------------------
// COURT E4 — KD-195 gitpack transit over the REAL binary: a frameFlag-0x08
// (gitpack) AppData window is accepted end to end, the response window
// mirrors 0x08 in header reserved[0] (window offset 32), and the echoed
// payload is byte-faithful to the pre-seal bytes.
// ANTI-VACUITY MUTATION: dropping the mirror stamp in the sidecar's
// send_window leaves the response flag 0x00 (red); deleting the frameFlag
// gate lets out-of-set flags through (E5 covers the refusal form).
// ---------------------------------------------------------------------------

#[test]
fn e4_gitpack_flag_mirror_over_real_uds() {
    let mut car = spawn_sidecar("e4");
    let kem = CourtKem;
    let (_dk, _ct, recv_key, send_key) = handshake(&kem, car.stream.as_mut().expect("conn e4"));

    let preseal = b"kd195 gitpack transit: raw RecordBatch payload \x08\x00\xff";
    let ordinal = 2u32;
    let seq = 11u32;
    // v3 wire law: body = [seq][ct][tag] under the court's send_key.
    let mut prefix = [0u8; cgap_aoq::header::SIZE_BYTES + 5];
    codec::encode(&AoqHeader::new(EPOCH, KEY_ID, ordinal, seq), &mut prefix).expect("encode");
    prefix[cgap_aoq::header::SIZE_BYTES] = FrameKind::AppData.id();
    prefix[cgap_aoq::header::SIZE_BYTES + 1..cgap_aoq::header::SIZE_BYTES + 5]
        .copy_from_slice(&seq.to_le_bytes());
    let mut app_body = vec![0u8; 4 + preseal.len() + cgap_aoq::aead::TAG_LEN];
    app_body[..4].copy_from_slice(&seq.to_le_bytes());
    let nonce = cgap_aoq::aead::Nonce96::new(u32::from(EPOCH), (u64::from(ordinal) << 32) | u64::from(seq))
        .expect("nonce");
    let mut aead_tag = [0u8; cgap_aoq::aead::TAG_LEN];
    cgap_aoq::aead::seal_detached(
        &mut app_body[4..4 + preseal.len()],
        &mut aead_tag,
        preseal,
        &prefix,
        &recv_key,
        &nonce,
    );
    app_body[4 + preseal.len()..].copy_from_slice(&aead_tag);
    let app = wire_frame(FrameKind::AppData, ordinal, seq, &app_body);
    let app_win = seal_window_flagged(&app, ordinal, seq, ipc::FRAME_FLAG_GITPACK);
    let (word, win) = roundtrip(car.stream.as_mut().expect("conn e4"), &app_win, true);
    assert_eq!(word, AOQ_STATUS_OK, "gitpack (0x08) AppData accepted");
    let win = win.expect("response window present");

    // MIRROR WITNESS over the real binary: response window[32] == 0x08.
    assert_eq!(
        win[ipc::FRAME_FLAG_OFFSET_IN_WINDOW],
        ipc::FRAME_FLAG_GITPACK,
        "response mirrors the gitpack frameFlag"
    );

    // Payload byte-fidelity: framing tag gate, session parse, AEAD open.
    let blen = u32::from_le_bytes([win[72], win[73], win[74], win[75]]) as usize;
    let mut body = vec![0u8; blen];
    ipc::read_verified(&win, None, &mut body).expect("response tag gate (mirror was pre-digest)");
    let frame = sess::parse_frame(&body).expect("response frame parses");
    assert_eq!(frame.kind, FrameKind::AppData);
    let ct_len = frame.payload.len() - 4 - cgap_aoq::aead::TAG_LEN;
    let mut plain = vec![0u8; ct_len];
    let aead_tag: [u8; cgap_aoq::aead::TAG_LEN] =
        frame.payload[4 + ct_len..].try_into().expect("tag");
    cgap_aoq::aead::open_detached(
        &mut plain,
        &frame.payload[4..4 + ct_len],
        &aead_tag,
        &body[..cgap_aoq::header::SIZE_BYTES + 5],
        &send_key,
        &cgap_aoq::aead::Nonce96::new(
            u32::from(EPOCH),
            (u64::from(frame.header.frame_ordinal) << 32) | u64::from(seq),
        )
        .expect("nonce"),
    )
    .expect("echo opens under the court-derived key");
    assert_eq!(&plain, &preseal[..], "gitpack payload byte-faithful end to end");
}

// ---------------------------------------------------------------------------
// COURT E5 — KD-195 out-of-set frameFlag over the REAL binary: a 0x02-flag
// window (otherwise fully lawful) refuses typed (AOQ_STATUS_DIGEST_MISMATCH,
// the frozen 8/8 band's digest-law word per KD-1), with no echo and no
// session step. ANTI-VACUITY MUTATION: deleting the frameFlag gate accepts
// the window (OK word) and this court goes red.
// ---------------------------------------------------------------------------

#[test]
fn e5_bad_flag_refused_over_real_uds() {
    let mut car = spawn_sidecar("e5");
    let kem = CourtKem;
    let (_dk, _ct, _recv_key, send_key) = handshake(&kem, car.stream.as_mut().expect("conn e5"));
    let ordinal = 2u32;
    let seq = 13u32;
    let mut prefix = [0u8; cgap_aoq::header::SIZE_BYTES + 5];
    codec::encode(&AoqHeader::new(EPOCH, KEY_ID, ordinal, seq), &mut prefix).expect("encode");
    prefix[cgap_aoq::header::SIZE_BYTES] = FrameKind::AppData.id();
    prefix[cgap_aoq::header::SIZE_BYTES + 1..cgap_aoq::header::SIZE_BYTES + 5]
        .copy_from_slice(&seq.to_le_bytes());
    let preseal = b"should never be read";
    let mut app_body = vec![0u8; 4 + preseal.len() + cgap_aoq::aead::TAG_LEN];
    app_body[..4].copy_from_slice(&seq.to_le_bytes());
    let nonce = cgap_aoq::aead::Nonce96::new(u32::from(EPOCH), (u64::from(ordinal) << 32) | u64::from(seq))
        .expect("nonce");
    let mut aead_tag = [0u8; cgap_aoq::aead::TAG_LEN];
    cgap_aoq::aead::seal_detached(
        &mut app_body[4..4 + preseal.len()],
        &mut aead_tag,
        preseal,
        &prefix,
        &send_key,
        &nonce,
    );
    app_body[4 + preseal.len()..].copy_from_slice(&aead_tag);
    let app = wire_frame(FrameKind::AppData, ordinal, seq, &app_body);
    let app_win = seal_window_flagged(&app, ordinal, seq, 0x02);
    let request_len = app_win.len();
    let (word, win) = roundtrip(car.stream.as_mut().expect("conn e5"), &app_win, false);
    assert_eq!(
        word, AOQ_STATUS_DIGEST_MISMATCH,
        "out-of-set frameFlag (0x02) refuses typed"
    );
    assert!(win.is_none(), "no window follows a refusal");
    let mut probe = vec![0u8; request_len];
    let n = car
        .stream
        .as_mut()
        .expect("conn e5")
        .read(&mut probe)
        .expect("post-refusal read");
    assert_eq!(n, 0, "refusal was NOT an echo");
}
