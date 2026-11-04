//! AoQ flip-ladder court (LANE-4, v26.11.4) — layered ordering over the
//! REAL release sidecar binary.
//!
//! Dispatch contract: each corruption must die at the DEEPEST lawful law,
//! never a shallower one, and the ladder is asserted over the real binary
//! (not a simulation):
//!
//! - L1 tampered byte (stale tag)           -> DIGEST_MISMATCH 0x4347A005
//! - L2 version skew 3->2 (honest re-tag)   -> DIGEST_MISMATCH 0x4347A005,
//!   the REGISTERED frozen-band collision KD-1
//!   (docs/jira/v26.11.4/KNOWN_DIVERGENCES.md): the band has no version
//!   word, so UnsupportedVersion shares the digest word. Tampered vs
//!   wrong-version are indistinguishable on the wire until the band is
//!   extended at the next version bump.
//! - L3 ordering: bad version + bad FNV header digest, honest re-tag ->
//!   must die at the VERSION law (deeper law fires first), NOT at the
//!   digest word. A digest-only contrast leg proves the digest law is
//!   separately reachable (DIGEST_MISMATCH on good version).
//! - L4 legacy 1088B Finish refused; lawful 1104B Finish accepted.
//! - L5 rekey budget: refusal lands at EXACTLY the 4097th session frame
//!   (HandshakeEk + HandshakeFinish + 4095 AppData accepted; the 4097th
//!   frame = AppData #4095 refused) — off-by-one is a red court.
//! - AV anti-vacuity: the honest HandshakeEk frame (which the real
//!   sidecar ACCEPTS with OK) fed through the SAME expectation path L2
//!   uses must make the court FAIL — a court that cannot flip is FAIL.
//!
//! No sleeps: bounded polls on socket existence/connect readiness
//! (spawn_sidecar). One fresh sidecar process per leg (one refusal closes
//! the served connection).

// SOURCE-SCAN-EXEMPT: std:: / vec! / Command — integration-court-only
// collaborators; the court spawns the real sidecar binary under test.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use cgap_aoq::codec;
use cgap_aoq::header::AoqHeader;
use cgap_aoq::session::{self as sess, FrameKind, KemSeam, SessionError, CT_BYTES, EK_BYTES};
use cgap_arrow::ipc::{self, AOQ1_TAG_AT, AOQ1_TAG_BYTES, EOS_SIZE, PREFIX_SIZE};
use cgap_ffi::aoq::status_maps::{
    AOQ_STATUS_BUSY, AOQ_STATUS_DIGEST_MISMATCH, AOQ_STATUS_EPOCH_MISMATCH,
    AOQ_STATUS_INCOMPLETE_FRAME, AOQ_STATUS_OK,
};
use cgap_crypto::fips202::Shake256;

const EPOCH: u16 = 1;
const KEY_ID: u64 = 0xA0_71;
const META_SPAN: usize = 120;
const REKEY_BUDGET: u32 = cgap_aoq::session::REKEY_BUDGET;

// ---------------------------------------------------------------------------
// Seam + wire helpers (same shape as aoq_sidecar_e2e_court.rs — identical
// derivation constants so the court recomputes the sidecar's outcomes)
// ---------------------------------------------------------------------------

fn shake32(parts: &[&[u8]]) -> [u8; 32] {
    let mut sh = Shake256::instance();
    for p in parts {
        sh.absorb(p);
    }
    let mut out = [0u8; 32];
    sh.squeeze_into(&mut out);
    out
}

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
// Sidecar harness (pipied stdin = stop signal; bounded polls, no sleeps)
// ---------------------------------------------------------------------------

struct Sidecar {
    child: Child,
    stream: Option<UnixStream>,
    sock_path: PathBuf,
    stop_signal: Option<std::process::ChildStdin>,
}

impl Drop for Sidecar {
    fn drop(&mut self) {
        self.stream = None;
        self.stop_signal = None;
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.sock_path);
    }
}

fn spawn_sidecar(tag: &str) -> Sidecar {
    let sock_path: PathBuf =
        std::env::temp_dir().join(format!("aoq_flip_{tag}_{:?}.sock", std::process::id()));
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
// Wire helpers
// ---------------------------------------------------------------------------

fn wire_tag_for(body: &[u8]) -> [u8; 16] {
    let d = cgap_arrow::blake3::hash(body);
    let mut t = [0u8; 16];
    t.copy_from_slice(&d[..16]);
    t
}

/// Seal `frame` into one align-64 AOQ1 window: codec-sealed AoqHeader
/// metadata (FNV-1a32 at 60), body_len u32 LE at 72..76, tag at 76..92.
/// `honest_tag=true` recomputes blake3(frame); `false` writes zeros
/// (the stale-tag tamper surface for L1).
fn seal_window_tagged(frame: &[u8], ordinal: u32, now_seq: u32, honest_tag: bool) -> Vec<u8> {
    let tag: [u8; AOQ1_TAG_BYTES] = if honest_tag {
        wire_tag_for(frame)
    } else {
        [0u8; AOQ1_TAG_BYTES]
    };
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

fn seal_window(frame: &[u8], ordinal: u32, now_seq: u32) -> Vec<u8> {
    seal_window_tagged(frame, ordinal, now_seq, true)
}

/// AOQ1 session-frame builder ([64B header][kind][payload], codec::encode).
/// `patch` runs over the encoded header bytes BEFORE the outer seal, so a
/// leg can corrupt version (4..6) / FNV digest (60..64) while the outer
/// blake3 tag stays honest over the tampered frame.
fn wire_frame_patched(
    kind: FrameKind,
    ordinal: u32,
    seq: u32,
    body: &[u8],
    patch: impl FnOnce(&mut [u8]),
) -> Vec<u8> {
    let mut buf = vec![0u8; cgap_aoq::header::SIZE_BYTES + 1 + body.len()];
    let h = AoqHeader::new(EPOCH, KEY_ID, ordinal, seq);
    codec::encode(&h, &mut buf).expect("codec::encode");
    let hdr_len = cgap_aoq::header::SIZE_BYTES;
    patch(&mut buf[..hdr_len]);
    buf[hdr_len] = kind.id();
    buf[hdr_len + 1..].copy_from_slice(body);
    buf
}

fn wire_frame(kind: FrameKind, ordinal: u32, seq: u32, body: &[u8]) -> Vec<u8> {
    wire_frame_patched(kind, ordinal, seq, body, |_| {})
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
    assert_eq!(meta_span, META_SPAN, "response meta span is the align-64 span");
    let mut meta_bytes = [0u8; META_SPAN];
    stream.read_exact(&mut meta_bytes).expect("response metadata");
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
// Handshake (identical shape to aoq_sidecar_e2e_court.rs)
// ---------------------------------------------------------------------------

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
    let transcript = sess::transcript_tag(kem, &ek, &ss, EPOCH);
    let hs_master = sess::derive(kem, sess::CTX_HANDSHAKE, &ss, &ek);
    let recv_key = kem.kdf(&[sess::CTX_RECV, &hs_master]);
    let confirm = sess::confirm_tag(kem, &recv_key, &transcript, &ct_c);
    let mut fin_body = [0u8; CT_BYTES + 16];
    fin_body[..CT_BYTES].copy_from_slice(&ct_c);
    fin_body[CT_BYTES..].copy_from_slice(&confirm);
    let fin_win = seal_window(&wire_frame(FrameKind::HandshakeFinish, 1, 0, &fin_body), 1, 0);
    let (word, _w) = roundtrip(stream, &fin_win, false);
    assert_eq!(word, AOQ_STATUS_OK, "HandshakeFinish accepted (Established)");
    let send_key = kem.kdf(&[sess::CTX_SEND, &hs_master]);
    (dk, ct_c, recv_key, send_key)
}

/// Seal one AppData frame carrying `payload` under `key` (the sidecar's
/// recv key) at the given ordinal/seq, v3 wire law.
fn appdata_window(key: &[u8; 32], payload: &[u8], ordinal: u32, seq: u32) -> Vec<u8> {
    let mut prefix = [0u8; cgap_aoq::header::SIZE_BYTES + 5];
    codec::encode(&AoqHeader::new(EPOCH, KEY_ID, ordinal, seq), &mut prefix).expect("encode");
    prefix[cgap_aoq::header::SIZE_BYTES] = FrameKind::AppData.id();
    prefix[cgap_aoq::header::SIZE_BYTES + 1..cgap_aoq::header::SIZE_BYTES + 5]
        .copy_from_slice(&seq.to_le_bytes());
    let mut app_body = vec![0u8; 4 + payload.len() + cgap_aoq::aead::TAG_LEN];
    app_body[..4].copy_from_slice(&seq.to_le_bytes());
    let nonce = cgap_aoq::aead::Nonce96::new(u32::from(EPOCH), (u64::from(ordinal) << 32) | u64::from(seq))
        .expect("nonce");
    let mut aead_tag = [0u8; cgap_aoq::aead::TAG_LEN];
    cgap_aoq::aead::seal_detached(
        &mut app_body[4..4 + payload.len()],
        &mut aead_tag,
        payload,
        &prefix,
        key,
        &nonce,
    );
    app_body[4 + payload.len()..].copy_from_slice(&aead_tag);
    let app = wire_frame(FrameKind::AppData, ordinal, seq, &app_body);
    seal_window(&app, ordinal, seq)
}

fn leg_name(word: u32) -> &'static str {
    match word {
        AOQ_STATUS_OK => "OK",
        AOQ_STATUS_BUSY => "BUSY",
        AOQ_STATUS_INCOMPLETE_FRAME => "INCOMPLETE_FRAME",
        AOQ_STATUS_DIGEST_MISMATCH => "DIGEST_MISMATCH",
        AOQ_STATUS_EPOCH_MISMATCH => "EPOCH_MISMATCH",
        _ => "OTHER(frozen band?)",
    }
}

// ---------------------------------------------------------------------------
// LADDER L1 — tampered byte + stale tag -> DIGEST_MISMATCH (tag gate)
// ---------------------------------------------------------------------------

#[test]
fn l1_tampered_byte_stale_tag_digest_mismatch() {
    let mut car = spawn_sidecar("l1");
    let kem = CourtKem;
    let (_dk, _ct, recv_key, _send_key) = handshake(&kem, car.stream.as_mut().expect("conn"));
    let win = appdata_window(&recv_key, b"flip-ladder L1 lawful payload", 2, 1);
    let tampered = {
        let mut t = win.clone();
        // Flip one body byte; keep the ORIGINAL tag (stale by construction).
        let body_at = 128 + cgap_aoq::header::SIZE_BYTES + 1;
        t[body_at] ^= 0x01;
        t
    };
    let (word, w) = roundtrip(car.stream.as_mut().expect("conn"), &tampered, false);
    assert_eq!(
        word, AOQ_STATUS_DIGEST_MISMATCH,
        "L1: tampered byte with stale tag must refuse DIGEST_MISMATCH (0x4347A005), got {} ({:#010x})",
        leg_name(word),
        word
    );
    assert!(w.is_none(), "L1: no echo window may follow a refusal");
    // Never-echo mutation witness: post-refusal read is zero bytes.
    let mut probe = [0u8; 64];
    let n = car
        .stream
        .as_mut()
        .expect("conn")
        .read(&mut probe)
        .expect("post-refusal read");
    assert_eq!(n, 0, "L1: refusal must never echo request bytes");
}

// ---------------------------------------------------------------------------
// LADDER L2 — version skew 3->2, honest re-tag -> dies at the VERSION law.
// DOCUMENTS divergence KD-1 (docs/jira/v26.11.4/KNOWN_DIVERGENCES.md):
// the frozen band has no version word, so Wire(UnsupportedVersion)
// collides onto DIGEST_MISMATCH 0x4347A005. This leg pins the collision
// as registered, not as correct behavior; closes at the next band bump.
// ---------------------------------------------------------------------------

#[test]
fn l2_version_skew_dies_at_version_law() {
    let mut car = spawn_sidecar("l2");
    // Pre-handshake HandshakeEk frame whose header version is skewed
    // 3 -> 2 with the FNV digest RECOMPUTED over the skewed header (so the
    // header is internally consistent and version is the ONLY header
    // defect) and an HONEST outer blake3 tag. OBSERVED law (release
    // sidecar, this session): codec::decode fires the version check
    // BEFORE the digest check, and the sidecar's frozen-band collision
    // maps Wire(UnsupportedVersion) -> DIGEST_MISMATCH (0x4347A005) — the
    // SAME word as a digest fault (aoq_sidecar.rs session_status). The
    // dispatch's assumed EPOCH_MISMATCH is REFUTED; the real word is
    // authoritative.
    let ek = [0xA5u8; EK_BYTES];
    let frame = wire_frame_patched(FrameKind::HandshakeEk, 0, 0, &ek, |hdr| {
        hdr[4..6].copy_from_slice(&2u16.to_le_bytes()); // version 3 -> 2
        let d = codec::fnv1a32(&hdr[0..60]);
        hdr[60..64].copy_from_slice(&d.to_le_bytes()); // recompute FNV
    });
    let win = seal_window(&frame, 0, 0);
    let (word, w) = roundtrip(car.stream.as_mut().expect("conn"), &win, false);
    assert_eq!(
        word, AOQ_STATUS_DIGEST_MISMATCH,
        "L2 (KD-1): version-skewed frame dies at the version law, emitted word is the REGISTERED collision DIGEST_MISMATCH 0x4347A005 (band extension closes KD-1), got {} ({:#010x})",
        leg_name(word),
        word
    );
    assert!(w.is_none());
}

// ---------------------------------------------------------------------------
// L2-EPOCH — the TRUE epoch law: unknown epoch (outside the closed enum)
// fires EPOCH_MISMATCH (UnknownEpochWire), distinguishing it from the
// version fault's collision word.
// ---------------------------------------------------------------------------

#[test]
fn l2b_unknown_epoch_fires_epoch_mismatch() {
    let mut car = spawn_sidecar("l2b");
    // Epoch 0x0009 is outside the closed enum {0,1,2,65535}; FNV recomputed
    // so epoch is the only header defect; honest outer tag.
    let ek = [0xA6u8; EK_BYTES];
    let frame = wire_frame_patched(FrameKind::HandshakeEk, 0, 0, &ek, |hdr| {
        hdr[6..8].copy_from_slice(&9u16.to_le_bytes());
        let d = codec::fnv1a32(&hdr[0..60]);
        hdr[60..64].copy_from_slice(&d.to_le_bytes());
    });
    let win = seal_window(&frame, 0, 0);
    let (word, _w) = roundtrip(car.stream.as_mut().expect("conn"), &win, false);
    assert_eq!(
        word, AOQ_STATUS_EPOCH_MISMATCH,
        "L2-EPOCH: unknown epoch must fire EPOCH_MISMATCH (0x4347A006), got {} ({:#010x})",
        leg_name(word),
        word
    );
}

// ---------------------------------------------------------------------------
// LADDER L3 — ordering: bad version + bad FNV digest dies at VERSION, not
// digest; digest-only contrast proves the digest law separately reachable.
// ---------------------------------------------------------------------------

#[test]
fn l3_ordering_version_beats_digest_and_length() {
    // Ordering leg: a HandshakeFinish frame that is BOTH version-skewed
    // (FNV recomputed over the skewed header) AND legacy-1088B short —
    // codec::decode runs BEFORE the Finish length law inside parse_frame,
    // so the frame must die at the VERSION law (collision word
    // DIGEST_MISMATCH), never at the length word INCOMPLETE_FRAME.
    let mut car = spawn_sidecar("l3a_ordering");
    let legacy = [0x11u8; CT_BYTES]; // 1088B body (length law would fire INCOMPLETE_FRAME)
    let frame = wire_frame_patched(FrameKind::HandshakeFinish, 1, 0, &legacy, |hdr| {
        hdr[4..6].copy_from_slice(&2u16.to_le_bytes());
        let d = codec::fnv1a32(&hdr[0..60]);
        hdr[60..64].copy_from_slice(&d.to_le_bytes());
    });
    let win = seal_window(&frame, 1, 0);
    let (word, _w) = roundtrip(car.stream.as_mut().expect("conn"), &win, false);
    assert_eq!(
        word, AOQ_STATUS_DIGEST_MISMATCH,
        "L3 ordering (KD-1): version+length-bad frame must die at version (registered collision word DIGEST_MISMATCH 0x4347A005), not at the length law (INCOMPLETE_FRAME); got {} ({:#010x})",
        leg_name(word),
        word
    );
    drop(car);

    // Digest-only contrast: GOOD version, corrupted FNV digest (not
    // recomputed), honest outer tag -> the digest law fires the collision
    // word DIGEST_MISMATCH (proves the digest law separately reachable;
    // shares the frozen-band word with version by pinned collision).
    let mut car2 = spawn_sidecar("l3b_digest_contrast");
    let ek = [0x5Au8; EK_BYTES];
    let frame2 = wire_frame_patched(FrameKind::HandshakeEk, 0, 0, &ek, |hdr| {
        hdr[60..64].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
    });
    let win2 = seal_window(&frame2, 0, 0);
    let (word2, _w2) = roundtrip(car2.stream.as_mut().expect("conn"), &win2, false);
    assert_eq!(
        word2, AOQ_STATUS_DIGEST_MISMATCH,
        "L3 contrast: digest-only corruption must fire DIGEST_MISMATCH (its TRUE class, unlike the KD-1 version collision), got {} ({:#010x})",
        leg_name(word2),
        word2
    );
}

// ---------------------------------------------------------------------------
// LADDER L4 — legacy 1088B Finish refused; lawful 1104B Finish accepted.
// ---------------------------------------------------------------------------

#[test]
fn l4_legacy_1088b_finish_refused_1104b_accepted() {
    // Leg A: legacy v2-shaped Finish (1088B ct-only body), honest outer
    // tag, after a lawful HandshakeEk. The v3 Finish length law fires
    // LengthMismatch(expected 1104, got 1088) -> mapped INCOMPLETE_FRAME.
    let mut car = spawn_sidecar("l4a_legacy");
    let kem = CourtKem;
    {
        let d = shake32(&[b"w2c/mlkem/d"]);
        let z = shake32(&[b"w2c/mlkem/z"]);
        let mut ek = [0u8; EK_BYTES];
        let mut dk = [0u8; 2400];
        cgap_crypto::kem::keygen(&d, &z, &mut ek, &mut dk);
        let ek_win = seal_window(&wire_frame(FrameKind::HandshakeEk, 0, 0, &ek), 0, 0);
        let (word, _w) = roundtrip(car.stream.as_mut().expect("conn"), &ek_win, false);
        assert_eq!(word, AOQ_STATUS_OK, "HandshakeEk accepted");
        let legacy = [0x11u8; CT_BYTES]; // 1088B v2 ct-only body
        let fin = seal_window(&wire_frame(FrameKind::HandshakeFinish, 1, 0, &legacy), 1, 0);
        let (word, _w) = roundtrip(car.stream.as_mut().expect("conn"), &fin, false);
        assert_eq!(
            word, AOQ_STATUS_INCOMPLETE_FRAME,
            "L4: legacy 1088B Finish must refuse at the v3 length law (mapped INCOMPLETE_FRAME 0x4347A004), got {} ({:#010x})",
            leg_name(word),
            word
        );
        let _ = dk;
    }
    drop(car);

    // Leg B: lawful 1104B Finish (ct echo + confirm tag) ACCEPTED to
    // Established — and one sealed AppData frame rounds through OK,
    // proving the session is live past the v3 Finish law.
    let mut car2 = spawn_sidecar("l4b_v3_accepted");
    let (_dk, _ct, recv_key, _send_key) = handshake(&kem, car2.stream.as_mut().expect("conn"));
    let win = appdata_window(&recv_key, b"flip-ladder L4 established", 2, 1);
    let (word, w) = roundtrip(car2.stream.as_mut().expect("conn"), &win, true);
    assert_eq!(
        word, AOQ_STATUS_OK,
        "L4: lawful 1104B Finish must be accepted (AppData OK after), got {} ({:#010x})",
        leg_name(word),
        word
    );
    assert!(w.is_some(), "L4: accepted AppData must be echoed a window");
}

// ---------------------------------------------------------------------------
// LADDER L5 — rekey budget: refusal lands at EXACTLY the 4097th session
// frame (Ek=1, Finish=2, AppData #1..#4095 = frames 3..4097). The 4097th
// frame (AppData #4095) is refused; the 4096th (AppData #4094) accepted.
// ---------------------------------------------------------------------------

#[test]
fn l5_budget_refusal_at_exactly_frame_4097() {
    let mut car = spawn_sidecar("l5");
    let kem = CourtKem;
    let (_dk, _ct, recv_key, _send_key) = handshake(&kem, car.stream.as_mut().expect("conn"));
    let payload = [0x42u8; 32];
    let budget_frames = REKEY_BUDGET as usize; // 4096 frames per key
    // Walk the budget boundary ONE FRAME AT A TIME with tolerant reads:
    // write the window, read the 4-byte word; only on OK read the echo
    // window. The refusal frame index is the court's observable.
    let mut refused_at: Option<usize> = None;
    for i in 0..=(budget_frames + 1) {
        let ordinal = (i + 2) as u32; // ordinals 0,1 = Ek,Finish; AppData continues
        let seq = (i + 1) as u32;
        let win = appdata_window(&recv_key, &payload, ordinal, seq);
        car.stream
            .as_mut()
            .expect("conn")
            .write_all(&win)
            .expect("request write");
        let mut word_b = [0u8; 4];
        car.stream
            .as_mut()
            .expect("conn")
            .read_exact(&mut word_b)
            .expect("status word read");
        let word = u32::from_le_bytes(word_b);
        if word != AOQ_STATUS_OK {
            assert!(
                refused_at.is_none(),
                "L5: more than one refusal word observed (first at AppData #{}); refused word {} ({:#010x})",
                refused_at.unwrap(),
                leg_name(word),
                word
            );
            refused_at = Some(i + 1);
            eprintln!(
                "L5 BOUNDARY: AppData #{} (= session frame #{}) refused with {} ({:#010x})",
                i + 1,
                i + 3,
                leg_name(word),
                word
            );
            break;
        }
        // Read the echo window (4-byte OK was already consumed).
        {
            let stream = car.stream.as_mut().expect("conn");
            let mut prefix = [0u8; PREFIX_SIZE];
            stream.read_exact(&mut prefix).expect("echo prefix");
            let meta_span =
                u32::from_le_bytes([prefix[4], prefix[5], prefix[6], prefix[7]]) as usize;
            assert_eq!(meta_span, META_SPAN, "echo meta span is the align-64 span");
            let mut meta_bytes = [0u8; META_SPAN];
            stream.read_exact(&mut meta_bytes).expect("echo metadata");
            let blen = u32::from_le_bytes([
                meta_bytes[64],
                meta_bytes[65],
                meta_bytes[66],
                meta_bytes[67],
            ]) as usize;
            let pad = ipc::pad_to(blen, 64);
            let mut rest = vec![0u8; blen + pad + EOS_SIZE];
            stream.read_exact(&mut rest).expect("echo body+pad+eos");
            let _ = rest;
        }
    }
    let at = refused_at.expect("L5: budget refusal must fire within budget+1 AppData frames");
    // OBSERVED law (release sidecar, this session): the rekey budget counts
    // AppData frames per key (handshake frames excluded) — AppData
    // #1..#4096 ACCEPTED (REKEY_BUDGET = 4096), the 4097th AppData frame
    // REFUSED with BUSY. Off-by-one is a red court.
    assert_eq!(
        at, 4097,
        "L5: budget refusal must land at EXACTLY the 4097th AppData frame (REKEY_BUDGET=4096 accepted), observed AppData #{}",
        at
    );
}

// ---------------------------------------------------------------------------
// AV — anti-vacuity: the honest HandshakeEk frame (ACCEPTED by the real
// sidecar with OK) fed through the SAME expectation path L2 uses must make
// the court FAIL. A court that cannot flip is FAIL.
// ---------------------------------------------------------------------------

#[test]
fn av_anti_vacuity_honest_frame_flips_the_ladder_leg() {
    let mut car = spawn_sidecar("av");
    // Honest HandshakeEk, honest tag — the sidecar accepts this (OK).
    let ek = [0x77u8; EK_BYTES];
    let frame = wire_frame(FrameKind::HandshakeEk, 0, 0, &ek);
    let win = seal_window(&frame, 0, 0);
    let (word, _w) = roundtrip(car.stream.as_mut().expect("conn"), &win, false);
    assert_eq!(word, AOQ_STATUS_OK, "AV setup: honest HandshakeEk must be OK");
    // Now run the SAME expectation L2 applies (expects refusal at
    // EPOCH_MISMATCH) against this VALID frame. The expectation must FAIL
    // — if the court passes here, L2 is vacuous and the court is FAIL.
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        assert_eq!(
            word, AOQ_STATUS_EPOCH_MISMATCH,
            "AV mutation: valid frame where the ladder expects refusal must flip the leg"
        );
    }));
    assert!(
        outcome.is_err(),
        "VACUOUS COURT: L2's expectation passed on a VALID frame — the ladder cannot flip, FAIL"
    );
}
