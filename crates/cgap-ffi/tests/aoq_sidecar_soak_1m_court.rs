//! AoQ sidecar 1M-trip soak court (ZP-11 G1, PLANS row
//! ggen/docs/jira/v26.11.4/PLANS.md "ZP-11 ... G1 = 1M UDS round-trip court
//! (`aoq_sidecar_release soak`, 10 conn x 100k, ... ~60s wall from ab-8's
//! 579k fps baseline)").
//!
//! SHAPE DISCLOSURE (read before citing any number this court prints):
//! The coordinator constraint law stated the soak must measure
//! REFUSAL-SHAPED trips ("the sidecar accepts connections SEQUENTIALLY
//! and Go cannot reach Established (no seeded ML-KEM encaps)"). Premise
//! checked against the substrate per the premise-rot law: that constraint
//! is GO-SIDE truth (arcus cannot finish the sidecar handshake — ZP-11 G4
//! probe disclosure, uds_transit_probe_test.go header). It is NOT
//! Rust-side truth: this court's sibling aoq_sidecar_court.rs J1
//! completes the REAL 3-hop handshake to Established (real ML-KEM-768
//! keygen, deterministic SidecarKem encapsulation, real emit_verified
//! echo) and receives a real verified echo window against the same
//! binary. Accordingly this court measures REAL verified echo round
//! trips — 10 connections x 100k AppData echo trips each, one full
//! handshake per connection — and its throughput number is REAL echo fps.
//! Scoping law: citing these fps as Rust-reachable echo throughput IS
//! lawful; citing them as Go-reachable throughput is NOT (the Go probe's
//! refusal-shaped numbers remain the Go-plane receipt).
//!
//! Fences (ZP-18 timing-fence census law):
//! - profile_guard::assert_release loud-skip compose at the top of the
//!   measured court: a dev-profile run loud-skips (typed reason), a
//!   release run must complete.
//! - Wall budget: the court refuses if 1M trips exceed the disclosed
//!   120 s budget (1M/120 s = 8.3k fps floor; ab-8's measured 579k fps
//!   baseline predicts ~2 s of trip time).
//! - No-sleep law: bounded polls only (bind poll, connect retry, teardown
//!   wait poll on try_wait) — no Process.sleep equivalents.
//!
//! ANTI-VACUITY (flipped-tag mutation leg): mid-stream on an ESTABLISHED
//! connection, one frame carries a corrupted tag -> the sidecar's tag
//! gate must refuse DIGEST_MISMATCH (exact frozen-band word), the
//! connection dies (refusal-closes-connection law). Then a fresh
//! connection resumes clean (full handshake + trips OK), proving the
//! refusal was per-connection and the mutation is detected against the
//! real gate (a sidecar with a bypassed/removed tag gate would return OK
//! + echo here and the court fails).
//!
//! Wire-law note: the response window's tag is recomputed by the court
//! over the echo body via the same cgap-arrow blake3 law the gate uses,
//! so the soak's response leg is itself a per-trip tag re-derivation —
//! no faked verification anywhere in the loop.

// SOURCE-SCAN-EXEMPT: std:: : integration-court-only collaborator
// SOURCE-SCAN-EXEMPT: vec! : integration-court-only collaborator
// SOURCE-SCAN-EXEMPT: Command: spawns the real sidecar binary under test

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use cgap_aoq::aead::{self, Nonce96, TAG_LEN};
use cgap_aoq::codec;
use cgap_aoq::header::AoqHeader;
use cgap_aoq::profile_guard;
use cgap_aoq::session::{self as sess, FrameKind, KemSeam, SessionError, CT_BYTES, EK_BYTES};
use cgap_arrow::ipc::{self, AOQ1_TAG_AT, AOQ1_TAG_BYTES, EOS_SIZE, PREFIX_SIZE};
use cgap_crypto::fips202::Shake256;
use cgap_ffi::aoq::status_maps::{AOQ_STATUS_DIGEST_MISMATCH, AOQ_STATUS_OK};

const EPOCH: u16 = 1;
const KEY_ID: u64 = 0xA0_71;
const META_SPAN: usize = 120;

// ---------------------------------------------------------------------------
// Sidecar process harness (G2 lifecycle law: stdin held open; stdin EOF
// is the stop signal; bounded polls, never sleeps).
// ---------------------------------------------------------------------------

struct Sidecar {
    child: Child,
    #[allow(dead_code)]
    sock_path: PathBuf,
    _stop_signal: Option<std::process::ChildStdin>,
}

impl Drop for Sidecar {
    fn drop(&mut self) {
        let _ = self._stop_signal.take();
        // stdin EOF is the lawful stop; bounded poll on try_wait (the
        // observable is child exit), kill on expiry (fail-closed teardown).
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {
                    if Instant::now() > deadline {
                        let _ = self.child.kill();
                        let _ = self.child.wait();
                        break;
                    }
                    std::thread::yield_now();
                }
                Err(_) => break,
            }
        }
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.sock_path);
    }
}

/// Spawn the real sidecar binary (CARGO_BIN_EXE — cargo builds the exact
/// binary under test for the run), stdin PIPED and held open (G2 law).
fn spawn_sidecar(tag: &str) -> Sidecar {
    let sock_path: PathBuf =
        std::env::temp_dir().join(format!("aoq_soak_1m_{tag}_{:?}.sock", std::process::id()));
    let _ = std::fs::remove_file(&sock_path);
    let mut child = Command::new(env!("CARGO_BIN_EXE_aoq_sidecar"))
        .arg(&sock_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn aoq_sidecar");
    let stop_signal = child.stdin.take().expect("sidecar stdin pipe");
    // Bounded bind poll: observable is the socket path existing.
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if sock_path.exists() {
            break;
        }
        if Instant::now() > deadline {
            panic!("sidecar socket never appeared (bounded poll expired)");
        }
        std::thread::yield_now();
    }
    // Bounded connect retry: macOS ECONNREFUSED window between the socket
    // file appearing and listen() readiness (aoq_sidecar_court precedent).
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match UnixStream::connect(&sock_path) {
            Ok(_) => break,
            Err(_) => {
                if Instant::now() > deadline {
                    panic!("sidecar never accepted within 10s of bind");
                }
                std::thread::yield_now();
            }
        }
    }
    Sidecar {
        child,
        sock_path,
        _stop_signal: Some(stop_signal),
    }
}

// ---------------------------------------------------------------------------
// Wire helpers (same law as aoq_sidecar_court.rs — duplicated because Rust
// integration tests cannot share code except via a common module; kept
// byte-identical to the J1 court's helpers).
// ---------------------------------------------------------------------------

/// blake3(body)[..16] — the tamper-tier wire tag (cgap-arrow AOQ1 law).
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
    // EOS second word stays zero.
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

/// Write one request window; read the 4-byte status word; on OK read the
/// response window and run it through the SAME tag gate the sidecar uses
/// (read_verified). Returns (word, Option<response body>).
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
    assert_eq!(meta_span, META_SPAN, "response meta span");
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

/// Read the response window body out of a raw window Vec (already framed).
fn response_body(win: &[u8]) -> Vec<u8> {
    let blen = u32::from_le_bytes([win[72], win[73], win[74], win[75]]) as usize;
    let mut body = vec![0u8; blen];
    ipc::read_verified(win, None, &mut body).expect("response window tag gate");
    body
}

// ---------------------------------------------------------------------------
// Connection lifecycle: one full real handshake (J1 law) then N echo trips.
// ---------------------------------------------------------------------------

/// The court's own view of the SAME deterministic seam the sidecar
/// constructs (J1 law: encapsulate(ek) is seed-derived, so the court
/// knows the ct the sidecar holds without a side channel).
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

/// BUSY-drain word (PL7 multiplex-refusal law, sidecar main loop).
const AOQ_STATUS_BUSY_WORD: u32 = 0x4347A003;

/// Open one connection, run the real 3-hop handshake to Established,
/// return the stream ready for AppData echo trips.
///
/// Bounded BUSY-drain retry (PL7 law): a dial that queued in the backlog
/// during the PREVIOUS connection's serve gets the frozen-band BUSY word
/// on its first hop and a close. The observable is "handshake accepted";
/// the retry is a bounded poll on it, never a sleep.
fn connect_established(sock_path: &PathBuf) -> (UnixStream, SessionKeys) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match try_connect_established(sock_path) {
            Ok(ks) => return ks,
            Err(e) => {
                if Instant::now() > deadline {
                    panic!("connect_established never succeeded within 10s: {e}");
                }
                std::thread::yield_now();
            }
        }
    }
}

/// Derived direction keys for one established connection (v3 wire law):
/// the client seals outbound AppData under `recv_key` (the sidecar's
/// receive direction) and opens the sidecar's echo under `send_key`.
struct SessionKeys {
    recv_key: [u8; 32],
    send_key: [u8; 32],
}

/// One connect+handshake attempt; Err carries the fault (retryable).
///
/// The FIRST HOP must not panic on transport faults: the PL7 BUSY-drain
/// law can close a queued connection before the client's first write
/// (witnessed BrokenPipe), so hop 1 does its own write/read and returns
/// typed retryable faults. Once hop 1 is accepted (OK), the connection is
/// being served and hop 2 onward is transport-stable.
fn try_connect_established(sock_path: &PathBuf) -> Result<(UnixStream, SessionKeys), String> {
    let mut stream = UnixStream::connect(sock_path).map_err(|e| e.to_string())?;

    let d = shake32(&[b"w2c/mlkem/d"]);
    let z = shake32(&[b"w2c/mlkem/z"]);
    let mut ek = [0u8; EK_BYTES];
    let mut dk = [0u8; 2400];
    cgap_crypto::kem::keygen(&d, &z, &mut ek, &mut dk);
    let kem = CourtKem;
    let (ct_c, ss) = kem.encapsulate(&ek).expect("court encapsulate");

    let ek_win = seal_window(&wire_frame(FrameKind::HandshakeEk, 0, 0, &ek), 0, 0);
    stream.write_all(&ek_win).map_err(|e| e.to_string())?;
    let mut word_b = [0u8; 4];
    stream.read_exact(&mut word_b).map_err(|e| e.to_string())?;
    let word = u32::from_le_bytes(word_b);
    if word != AOQ_STATUS_OK {
        return Err(match word {
            AOQ_STATUS_BUSY_WORD => "BUSY (backlog drain; retry lawful)".to_string(),
            w => format!("HandshakeEk refused {w:#010x}"),
        });
    }
    // v3 Finish law (M2a): body = ct echo (1088B) + confirm tag (16B)
    // binding recv_key ‖ transcript ‖ ct_echo. Mirrors the migrated
    // aoq_sidecar_court J1 client; the v2 ct-only 1088B body is refused
    // INCOMPLETE_FRAME (0x4347A004) under the v3 law. (bm4 integration
    // disclosure: this court's handshake re-pinned to the M2a v3 law after
    // the v2 shape drew INCOMPLETE_FRAME on the release run above.)
    let transcript = sess::transcript_tag(&kem, &ek, &ss, EPOCH);
    let hs_master = sess::derive(&kem, sess::CTX_HANDSHAKE, &ss, &ek);
    let recv_key = kem.kdf(&[sess::CTX_RECV, &hs_master]);
    let confirm = sess::confirm_tag(&kem, &recv_key, &transcript, &ct_c);
    let mut fin_body = [0u8; CT_BYTES + 16];
    fin_body[..CT_BYTES].copy_from_slice(&ct_c);
    fin_body[CT_BYTES..].copy_from_slice(&confirm);
    let (word, _) = roundtrip(
        &mut stream,
        &seal_window(
            &wire_frame(FrameKind::HandshakeFinish, 1, 0, &fin_body),
            1,
            0,
        ),
        false,
    );
    if word != AOQ_STATUS_OK {
        return Err(format!("HandshakeFinish refused {word:#010x}"));
    }
    let hs_master = sess::derive(&kem, sess::CTX_HANDSHAKE, &ss, &ek);
    let recv_key: [u8; 32] = kem.kdf(&[sess::CTX_RECV, &hs_master]);
    let send_key: [u8; 32] = kem.kdf(&[sess::CTX_SEND, &hs_master]);
    Ok((stream, SessionKeys { recv_key, send_key }))
}

/// One AppData echo trip on an established connection at (ordinal, seq),
/// under the v3 wire law: the client AEAD-seals the payload (detached
/// Poly1305 tag appended after ct) under the connection's recv_key; the
/// echo is opened under send_key (plaintext byte-identity assert).
fn echo_trip(stream: &mut UnixStream, keys: &SessionKeys, ordinal: u32, seq: u32, payload: &[u8]) {
    let mut prefix = [0u8; cgap_aoq::header::SIZE_BYTES + 5];
    codec::encode(&AoqHeader::new(EPOCH, KEY_ID, ordinal, seq), &mut prefix).expect("encode");
    prefix[cgap_aoq::header::SIZE_BYTES] = FrameKind::AppData.id();
    prefix[cgap_aoq::header::SIZE_BYTES + 1..cgap_aoq::header::SIZE_BYTES + 5]
        .copy_from_slice(&seq.to_le_bytes());
    let mut app_body = vec![0u8; 4 + payload.len() + TAG_LEN];
    app_body[..4].copy_from_slice(&seq.to_le_bytes());
    let nonce = Nonce96::new(
        u32::from(EPOCH),
        (u64::from(ordinal) << 32) | u64::from(seq),
    )
    .expect("nonce");
    let mut aead_tag = [0u8; TAG_LEN];
    aead::seal_detached(
        &mut app_body[4..4 + payload.len()],
        &mut aead_tag,
        payload,
        &prefix,
        &keys.recv_key,
        &nonce,
    );
    app_body[4 + payload.len()..].copy_from_slice(&aead_tag);
    let app = wire_frame(FrameKind::AppData, ordinal, seq, &app_body);
    let win = seal_window(&app, ordinal, seq);
    let (word, resp) = roundtrip(stream, &win, true);
    assert_eq!(word, AOQ_STATUS_OK, "AppData accepted at ordinal {ordinal}");
    let body = response_body(resp.as_deref().expect("echo window present"));
    let frame = sess::parse_frame(&body).expect("echo frame parses");
    assert_eq!(frame.kind, FrameKind::AppData);
    // WITNESSED (this court, release run 1, confirmed by the 2026-11-04
    // LE migration): the sidecar's emit_verified echo writes the seq word
    // LITTLE-endian in the payload (session.rs seq.to_le_bytes(), pinned
    // lawful by arcus TestAoqDriftSeqEndianness). This court and the
    // sidecar's inbound read both use LE.
    let rseq = u32::from_le_bytes([
        frame.payload[0],
        frame.payload[1],
        frame.payload[2],
        frame.payload[3],
    ]);
    assert_eq!(rseq, seq, "echoed seq");
    // v3 wire law: the echo body = [seq][ct][tag 16B]; the payload is
    // AEAD-sealed under the sidecar's send_key — open_detached recovers
    // the plaintext (AEAD round trip, byte-identity assert).
    let ct_len = frame.payload.len() - 4 - TAG_LEN;
    let mut plain = vec![0u8; ct_len];
    let echo_tag: [u8; TAG_LEN] = frame.payload[4 + ct_len..].try_into().expect("tag");
    aead::open_detached(
        &mut plain,
        &frame.payload[4..4 + ct_len],
        &echo_tag,
        &body[..cgap_aoq::header::SIZE_BYTES + 5],
        &keys.send_key,
        &Nonce96::new(
            u32::from(EPOCH),
            (u64::from(frame.header.frame_ordinal) << 32) | u64::from(rseq),
        )
        .expect("nonce"),
    )
    .expect("echo opens under derived send_key");
    assert_eq!(&plain, payload, "echoed payload (AEAD open byte-identity)");
}

// ---------------------------------------------------------------------------
// THE SOAK COURT
// ---------------------------------------------------------------------------

const PHASES: usize = 10;
const TRIPS_PER_PHASE: usize = 100_000;
const TOTAL_TRIPS: usize = PHASES * TRIPS_PER_PHASE; // 1,000,000
const WALL_BUDGET: Duration = Duration::from_secs(120);
/// cgap-aoq session.rs REKEY_BUDGET = 4096 frames per session: the 4097th
/// frame draws RekeyRequired -> the sidecar maps it to BUSY and the
/// connection must be renewed (witnessed: trip index 4096 failed with an
/// OK-then-BUSY wire pattern in the first release run). The PLANS row's
/// "10 conn x 100k" is therefore not reachable without the rekey
/// handshake; the lawful adaptation (disclosed) keeps the 10 x 100k phase
/// census and chains fresh connections of <=4000 trips inside each phase.
const TRIPS_PER_CONNECTION: u32 = 4000;

#[test]
fn soak_1m_echo_trips_over_real_sidecar() {
    // ZP-18 fence compose: dev profile loud-skips (AF-25 G2 caller
    // pattern: loud stderr + early return); release must complete.
    if let Err(v) = profile_guard::assert_release() {
        eprintln!("LOUD-SKIP soak_1m_echo_trips_over_real_sidecar: {v}");
        return;
    }

    let car = spawn_sidecar("soak1m");
    let sock = car.sock_path.clone();

    let t0 = Instant::now();
    let mut trips = 0usize;
    let payload = b"soak-payload-v1";
    for phase in 0..PHASES {
        // Phase = 100k trips over chained connections of <=4000 trips
        // (REKEY_BUDGET law above). Every trip fully verified (OK word +
        // tag gate + echo seq/payload identity).
        let mut phase_trips = 0usize;
        while phase_trips < TRIPS_PER_PHASE {
            let (mut stream, keys) = connect_established(&sock);
            let mut on_conn = 0u32;
            while phase_trips < TRIPS_PER_PHASE && on_conn < TRIPS_PER_CONNECTION {
                echo_trip(&mut stream, &keys, 2 + on_conn, 1 + on_conn, payload);
                on_conn += 1;
                phase_trips += 1;
                trips += 1;
            }
            drop(stream);
        }
        eprintln!(
            "soak phase {}/{} complete: {trips} trips, {:.2?} elapsed",
            phase + 1,
            PHASES,
            t0.elapsed()
        );
    }
    let elapsed = t0.elapsed();
    let fps = trips as f64 / elapsed.as_secs_f64();
    drop(car);

    eprintln!(
        "SOAK RECEIPT: {trips} REAL verified echo round trips in {:.3}s = {:.0} fps (wall budget {:?}; per-trip {:.1} us)",
        elapsed.as_secs_f64(),
        fps,
        WALL_BUDGET,
        elapsed.as_secs_f64() * 1e6 / trips as f64
    );
    assert_eq!(trips, TOTAL_TRIPS, "trip census");
    assert!(
        elapsed <= WALL_BUDGET,
        "soak wall budget exceeded: {elapsed:?} > {WALL_BUDGET:?}"
    );
}

// ---------------------------------------------------------------------------
// ANTI-VACUITY: flipped-tag mutation leg + fresh-connection recovery leg
// ---------------------------------------------------------------------------

#[test]
fn soak_flipped_tag_refuses_and_fresh_connection_recovers() {
    if let Err(v) = profile_guard::assert_release() {
        eprintln!("LOUD-SKIP soak_flipped_tag_refuses_and_fresh_connection_recovers: {v}");
        return;
    }

    let car = spawn_sidecar("soakflip");
    let sock = car.sock_path.clone();

    // Leg 1: establish, send 999 clean echo trips, then ONE frame with a
    // corrupted tag byte mid-stream. The gate must refuse the exact
    // DIGEST_MISMATCH word; the connection dies (refusal law).
    let (mut stream, keys) = connect_established(&sock);
    for i in 0..999 {
        echo_trip(&mut stream, &keys, 2 + i, 1 + i, b"flip-leg");
    }
    let ordinal: u32 = 2 + 999;
    let seq: u32 = 1 + 999;
    let mut app_body = Vec::with_capacity(4 + 8);
    app_body.extend_from_slice(&seq.to_le_bytes());
    app_body.extend_from_slice(b"flip-leg");
    let app = wire_frame(FrameKind::AppData, ordinal, seq, &app_body);
    let mut win = seal_window(&app, ordinal, seq);
    let tag_at = AOQ1_TAG_AT;
    win[tag_at] ^= 0x01; // MUTATION: one tag byte flipped
    stream.write_all(&win).expect("flipped-tag write");
    let mut word_b = [0u8; 4];
    stream
        .read_exact(&mut word_b)
        .expect("flipped-tag status read");
    let word = u32::from_le_bytes(word_b);
    assert_eq!(
        word, AOQ_STATUS_DIGEST_MISMATCH,
        "flipped tag must refuse the exact DIGEST_MISMATCH word, got {word:#010x}"
    );
    // The refusal-closes-connection law: the next read hits EOF (or the
    // peer is already gone); either way the session is dead.
    let mut probe = [0u8; 1];
    match stream.read(&mut probe) {
        Ok(0) => {}
        Ok(_) => panic!("peer sent bytes after a refusal (echo-after-refuse mutation)"),
        Err(_) => {}
    }
    drop(stream);

    // Leg 2: fresh connection resumes clean — the refusal was
    // per-connection, the sidecar survives (sequential accept law).
    let (mut fresh, fresh_keys) = connect_established(&sock);
    echo_trip(&mut fresh, &fresh_keys, 2, 1, b"post-flip-recovery");
    drop(fresh);
}
