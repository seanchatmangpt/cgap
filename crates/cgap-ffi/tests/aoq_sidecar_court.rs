//! AoQ sidecar end-to-end court (W2-C, wave-2-membrane-plan §1 W2-E).
//!
//! Real sidecar PROCESS over a real path-UDS (Chicago: no mocks; the
//! spawned binary is `env!("CARGO_BIN_EXE_aoq_sidecar")`, so cargo builds
//! it for the test run). Mirrors corridor_join_court.rs J1/J2 hop-for-hop
//! with the sidecar's live serve loop in place of the in-test corridor.
//!
//! Courts (each names its anti-vacuity mutation):
//! - J1 handshake -> established -> one verified AppData round trip.
//!   Mutation A (in J2): a flipped body byte must yield a TYPED refusal
//!   word, never an echo of the window.
//! - J2 tag-gate refusal: a stale tamper-tier tag (mutation B: an
//!   echo-instead-of-refuse sidecar would return the request bytes back —
//!   the court asserts the response is exactly the 4-byte word).
//! - J3 zero-byte/short window: mutation C (an accept-short sidecar would
//!   spin or accept); the court asserts a typed INCOMPLETE_FRAME refusal
//!   for a partial window and a clean exit-0 for a zero-byte close.
//!
//! No-sleep law: the court bounded-polls for the socket path (deadline
//! checked), never sleeps.

// SOURCE-SCAN-EXEMPT: std:: : integration-court-only collaborator
// SOURCE-SCAN-EXEMPT: vec! : integration-court-only collaborator
// SOURCE-SCAN-EXEMPT: Command: spawns the real sidecar binary under test

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Instant;

use cgap_aoq::codec;
use cgap_aoq::header::AoqHeader;
use cgap_aoq::aead::{Nonce96, TAG_LEN};
use cgap_aoq::session::{self as sess, FrameKind, KemSeam, SessionError, CT_BYTES, EK_BYTES};
use cgap_arrow::ipc::{self, AOQ1_TAG_AT, AOQ1_TAG_BYTES, EOS_SIZE, PREFIX_SIZE};
use cgap_crypto::fips202::Shake256;
use cgap_ffi::aoq::status_maps::{
    AOQ_STATUS_DIGEST_MISMATCH, AOQ_STATUS_INCOMPLETE_FRAME, AOQ_STATUS_OK,
};

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

/// The court's own view of the SAME seam the sidecar constructs (deterministic
/// ML-KEM-768 encapsulation seed from the peer ek): identical derivation
/// constants, so the court can recompute the sidecar's encapsulation
/// outcomes locally.
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
// Sidecar process harness: spawn + bounded poll (no sleep)
// ---------------------------------------------------------------------------

struct Sidecar {
    child: Child,
    stream: UnixStream,
    #[allow(dead_code)]
    sock_path: PathBuf,
    /// Open handle to the sidecar's stdin (the stdin-EOF stop signal);
    /// held for the court's lifetime so the sidecar keeps serving.
    _stop_signal: Option<std::process::ChildStdin>,
}

impl Drop for Sidecar {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.sock_path);
    }
}

/// Spawn the real sidecar binary and bounded-poll for its socket (deadline
/// checked, no sleep — the observable is the socket path's existence).
/// Stdin is PIPED and held open: stdin EOF is the sidecar's stop signal
/// (ZD-56), so an inherited-closed stdin must not end the process mid-court.
fn spawn_sidecar(tag: &str) -> Sidecar {
    let sock_path: PathBuf = std::env::temp_dir().join(format!(
        "aoq_sidecar_court_{tag}_{:?}.sock",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&sock_path);
    let mut child = Command::new(env!("CARGO_BIN_EXE_aoq_sidecar"))
        .arg(&sock_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn aoq_sidecar");
    let stop_signal = child.stdin.take().expect("sidecar stdin pipe");
    let deadline = Instant::now() + std::time::Duration::from_secs(10);
    loop {
        if sock_path.exists() {
            break;
        }
        if Instant::now() > deadline {
            panic!("sidecar socket never appeared (bounded poll expired)");
        }
    }
    // Bounded connect retry: macOS can report ECONNREFUSED transiently in
    // the window between the socket file appearing and listen() readiness
    // (observed this session; the poll-then-connect-once pattern refused
    // 5/5 times while the same binary accepted manual connects).
    let deadline = Instant::now() + std::time::Duration::from_secs(10);
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
        stream,
        sock_path,
        _stop_signal: Some(stop_signal),
    }
}

// ---------------------------------------------------------------------------
// Wire helpers: AOQ1 window seal (ipc::emit + pad law) and response read
// ---------------------------------------------------------------------------

/// blake3(body)[..16] — the tamper-tier wire tag (cgap-arrow AOQ1 law).
fn wire_tag_for(body: &[u8]) -> [u8; 16] {
    let d = cgap_arrow::blake3::hash(body);
    let mut t = [0u8; 16];
    t.copy_from_slice(&d[..16]);
    t
}

/// Seal `frame` into one AOQ1 window (align-64): the codec-sealed
/// `AoqHeader` metadata (FNV-1a32 at 60 — the law the AOQ1 tag gate
/// parses), body_len u32 LE at 72..76, tag at 76..92.
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
    // body pad already zero; EOS tail:
    let eos_at = 8 + 120 + frame.len() + ipc::pad_to(frame.len(), 64);
    w[eos_at..eos_at + 4].copy_from_slice(&cgap_arrow::ipc::CONTINUATION.to_le_bytes());
    // EOS second word is the zero u32 already.
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

/// Send one request window; read the 4-byte response word; on OK follow
/// with an optional response window read (returns (word, window_opt)).
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

// ---------------------------------------------------------------------------
// COURT J1 — handshake -> established -> one verified AppData round trip
// ---------------------------------------------------------------------------

#[test]
fn j1_sidecar_handshake_and_verified_appdata_roundtrip() {
    let mut car = spawn_sidecar("j1");

    // KEY HOP: the court generates a real ML-KEM-768 pair and sends its ek;
    // the sidecar encapsulates to it (deterministically recomputable here).
    let d = shake32(&[b"w2c/mlkem/d"]);
    let z = shake32(&[b"w2c/mlkem/z"]);
    let mut ek = [0u8; EK_BYTES];
    let mut dk = [0u8; 2400];
    cgap_crypto::kem::keygen(&d, &z, &mut ek, &mut dk);
    let kem = CourtKem;
    let (ct_c, _ss) = kem.encapsulate(&ek).expect("court encapsulate");

    // HOP 1: HandshakeEk (ordinal 0). Sidecar: HandshakeInit -> HandshakeWait.
    let ek_win = seal_window(&wire_frame(FrameKind::HandshakeEk, 0, 0, &ek), 0, 0);
    let (word, _win) = roundtrip(&mut car.stream, &ek_win, false);
    assert_eq!(word, AOQ_STATUS_OK, "HandshakeEk accepted");

    // HOP 2: HandshakeFinish (ordinal 1) echoing the sidecar's pending ct
    // (deterministic: encapsulate(ek) is seed-derived, so ct_c is exactly
    // what the sidecar holds). Sidecar: -> Established.
    // v3 Finish law (M2a): body = ct echo (1088B) + confirm tag (16B)
    // under recv_key/transcript derived from the same (ss, ek) the sidecar
    // derives at the Ek step (deterministic CourtKem makes this computable).
    let ss = kem.encapsulate(&ek).expect("court encapsulate ss").1;
    let transcript = cgap_aoq::session::transcript_tag(&kem, &ek, &ss, EPOCH);
    let hs_master = cgap_aoq::session::derive(&kem, cgap_aoq::session::CTX_HANDSHAKE, &ss, &ek);
    let recv_key = kem.kdf(&[cgap_aoq::session::CTX_RECV, &hs_master]);
    let confirm = cgap_aoq::session::confirm_tag(&kem, &recv_key, &transcript, &ct_c);
    let mut fin_body = [0u8; CT_BYTES + 16];
    fin_body[..CT_BYTES].copy_from_slice(&ct_c);
    fin_body[CT_BYTES..].copy_from_slice(&confirm);
    let fin = wire_frame(FrameKind::HandshakeFinish, 1, 0, &fin_body);
    let fin_win = seal_window(&fin, 1, 0);
    let (word, _win) = roundtrip(&mut car.stream, &fin_win, false);
    assert_eq!(word, AOQ_STATUS_OK, "HandshakeFinish accepted");

    // HOP 3: AppData (ordinal 2, seq 7) -> OK + one verified response window.
    // v3 wire law: body = [seq 4B][ct][tag 16B] AEAD-sealed under the
    // derived recv_key, AAD = the covered prefix, nonce from header.
    let payload = b"corridor payload w2c";
    let mut prefix = [0u8; cgap_aoq::header::SIZE_BYTES + 5];
    codec::encode(&AoqHeader::new(EPOCH, KEY_ID, 2, 7), &mut prefix).expect("encode");
    prefix[cgap_aoq::header::SIZE_BYTES] = FrameKind::AppData.id();
    prefix[cgap_aoq::header::SIZE_BYTES + 1..cgap_aoq::header::SIZE_BYTES + 5]
        .copy_from_slice(&7u32.to_le_bytes());
    let mut app_body =
        vec![0u8; 4 + payload.len() + TAG_LEN];
    app_body[..4].copy_from_slice(&7u32.to_le_bytes());
    let nonce = Nonce96::new(u32::from(EPOCH), (u64::from(2u32) << 32) | 7)
        .expect("nonce");
    let mut aead_tag = [0u8; TAG_LEN];
    cgap_aoq::aead::seal_detached(
        &mut app_body[4..4 + payload.len()],
        &mut aead_tag,
        payload,
        &prefix,
        &recv_key,
        &nonce,
    );
    app_body[4 + payload.len()..].copy_from_slice(&aead_tag);
    let app = wire_frame(FrameKind::AppData, 2, 7, &app_body);
    let app_win = seal_window(&app, 2, 7);
    let (word, win) = roundtrip(&mut car.stream, &app_win, true);
    assert_eq!(word, AOQ_STATUS_OK, "AppData accepted");
    let win = win.expect("response window present");

    // The response window passes the framing-layer tag gate.
    let blen = u32::from_le_bytes([win[72], win[73], win[74], win[75]]) as usize;
    let mut body = vec![0u8; blen];
    ipc::read_verified(&win, None, &mut body).expect("response window tag gate");

    // The response body is the sidecar's emit_verified echo: an AppData
    // frame with the same seq and the same payload, whose inner kdf tag
    // re-derives with the court's own seam over the shared secret.
    let frame = sess::parse_frame(&body).expect("response frame parses");
    assert_eq!(frame.kind, FrameKind::AppData);
    // minimal-unblock (2026-11-04 soak lane, disclosed): emit_verified
    // writes the seq word LITTLE-endian (session.rs `seq.to_le_bytes()`,
    // pinned lawful by arcus TestAoqDriftSeqEndianness; witnessed
    // 117440512 = 0x07000000 = LE 7 read BE). Was from_be_bytes.
    let rseq = u32::from_le_bytes([
        frame.payload[0],
        frame.payload[1],
        frame.payload[2],
        frame.payload[3],
    ]);
    assert_eq!(rseq, 7, "echoed seq");
    // v3 wire law: the echo body = [seq][ct][tag 16B]; the payload is
    // AEAD-sealed under the sidecar's send_key — open_detached under the
    // court-derived send_key recovers the plaintext (AEAD round trip).
    let send_key = kem.kdf(&[
        cgap_aoq::session::CTX_SEND,
        &cgap_aoq::session::derive(&kem, cgap_aoq::session::CTX_HANDSHAKE, &ss, &ek),
    ]);
    let ct_len = frame.payload.len() - 4 - TAG_LEN;
    let mut plain = vec![0u8; ct_len];
    let aead_tag: [u8; TAG_LEN] = frame.payload[4 + ct_len..].try_into().expect("tag");
    cgap_aoq::aead::open_detached(
        &mut plain,
        &frame.payload[4..4 + ct_len],
        &aead_tag,
        &body[..cgap_aoq::header::SIZE_BYTES + 5],
        &send_key,
        &Nonce96::new(
            u32::from(EPOCH),
            (u64::from(frame.header.frame_ordinal) << 32) | u64::from(rseq),
        )
        .expect("nonce"),
    )
    .expect("echo opens under the derived send_key");
    assert_eq!(&plain, payload, "echoed payload (plaintext round trip)");
    let _ = dk;
}

// ---------------------------------------------------------------------------
// COURT J2 — tag-gate refusal: typed word, NEVER an echo (mutation B)
// ---------------------------------------------------------------------------

#[test]
fn j2_tag_gate_refusal_is_typed_not_echo() {
    let mut car = spawn_sidecar("j2");
    let d = shake32(&[b"w2c/mlkem/d"]);
    let z = shake32(&[b"w2c/mlkem/z"]);
    let mut ek = [0u8; EK_BYTES];
    let mut dk = [0u8; 2400];
    cgap_crypto::kem::keygen(&d, &z, &mut ek, &mut dk);
    let kem = CourtKem;
    let (ct_c, _ss) = kem.encapsulate(&ek).expect("court encapsulate");

    // Mutated window: a lawful HandshakeEk window with the stored tag
    // replaced by a WRONG tag (the tamper tier must refuse at the gate,
    // before the frame is ever parsed or echoed).
    let mut ek_win = seal_window(&wire_frame(FrameKind::HandshakeEk, 0, 0, &ek), 0, 0);
    ek_win[AOQ1_TAG_AT] ^= 0x01;
    let request_len = ek_win.len();
    let (word, win) = roundtrip(&mut car.stream, &ek_win, false);
    assert_eq!(word, AOQ_STATUS_DIGEST_MISMATCH, "tag gate refuses typed");
    assert!(win.is_none(), "no window follows a refusal");
    // Mutation B non-vacuity: an echo-instead-of-refuse sidecar would have
    // sent the request bytes back; the response was exactly 4 bytes and the
    // next read on the (now closed) connection yields EOF, not data.
    let mut probe = vec![0u8; request_len];
    let n = car.stream.read(&mut probe).expect("post-refusal read");
    assert_eq!(n, 0, "refusal was NOT an echo (no request bytes came back)");

    // The sidecar refuses-and-closes; a fresh sidecar completes the same
    // handshake when the tag is intact — the gate is the difference.
    let mut car2 = spawn_sidecar("j2b");
    let ek_win2 = seal_window(&wire_frame(FrameKind::HandshakeEk, 0, 0, &ek), 0, 0);
    let (word2, _) = roundtrip(&mut car2.stream, &ek_win2, false);
    assert_eq!(word2, AOQ_STATUS_OK, "intact tag accepted");
    let _ = kem.decapsulate(&dk, &ct_c); // seam liveness (unused otherwise)
}

// ---------------------------------------------------------------------------
// COURT J3 — zero-byte / short window: refused, never accepted (mutation C)
// ---------------------------------------------------------------------------

#[test]
fn j3_zero_and_short_windows_refused() {
    // Leg 1 — TRUE zero-byte: connect, close with no bytes. The sidecar
    // must exit cleanly (code 0), never accept a phantom window. (Since the
    // ZD-56 sequential-serve change, the stop signal is stdin EOF: the court
    // pipes stdin and closes it after the zero-byte client close.)
    {
        let sock = std::env::temp_dir().join(format!(
            "aoq_sidecar_court_j3zero_{:?}.sock",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&sock);
        let mut child = Command::new(env!("CARGO_BIN_EXE_aoq_sidecar"))
            .arg(&sock)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("spawn sidecar (zero leg)");
        // Hold the stop-signal pipe open until the zero-byte leg completes.
        let stop_signal = child.stdin.take().expect("sidecar stdin pipe");
        let deadline = Instant::now() + std::time::Duration::from_secs(10);
        while !sock.exists() {
            assert!(Instant::now() < deadline, "socket poll expired");
        }
        let deadline = Instant::now() + std::time::Duration::from_secs(10);
        let _client = loop {
            match UnixStream::connect(&sock) {
                Ok(s) => break s,
                Err(e) => assert!(Instant::now() < deadline, "connect (zero leg): {e}"),
            }
        };
        drop(_client); // immediate zero-byte close
        drop(stop_signal); // stdin EOF: the sidecar's stop signal
                           // Bounded poll on the child's exit (observable: try_wait status).
        let deadline = Instant::now() + std::time::Duration::from_secs(10);
        let status = loop {
            match child.try_wait().expect("try_wait") {
                Some(s) => break s,
                None => assert!(Instant::now() < deadline, "sidecar exit poll expired"),
            }
        };
        assert_eq!(status.code(), Some(0), "zero-byte close is a clean exit");
        let _ = std::fs::remove_file(&sock);
    }

    // Leg 2 — SHORT window: a 4-byte prefix then EOF. An accept-short
    // sidecar (mutation C) would spin or treat the prefix as a window;
    // this sidecar must refuse with the typed INCOMPLETE_FRAME word.
    {
        let mut car = spawn_sidecar("j3short");
        car.stream
            .write_all(&[0xFF, 0xFF, 0xFF, 0xFF])
            .expect("prefix write");
        car.stream
            .shutdown(std::net::Shutdown::Write)
            .expect("half-close");
        let mut word_b = [0u8; 4];
        car.stream
            .read_exact(&mut word_b)
            .expect("refusal word read");
        let word = u32::from_le_bytes(word_b);
        assert_eq!(
            word, AOQ_STATUS_INCOMPLETE_FRAME,
            "short window refuses typed"
        );
    }
}
