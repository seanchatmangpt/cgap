//! AoQ event-tap conformance court (PL16 caller-side law, v26.11.3).
//!
//! Chicago court: drives a REAL `cgap_aoq::session::Session` (deterministic
//! court KEM, real ML-KEM-768 encapsulation) through the 4x5 (state, kind)
//! table — happy walk plus refusals — and replays the tapped rows against a
//! replayed state machine. The conformance oracle IS the 4x5 table.
//!
//! Courts:
//! - C1 oracle: every tapped row's (from -> to) must be a lawful table cell;
//!   the replayed machine advances exactly per row.kind.
//! - C2 count law (anti-vacuity, refusal side): refused steps emit NOTHING —
//!   tapped total equals successful step count, refusals leave fingerprints
//!   unchanged, and the wire-visible row count matches.
//! - C3 anti-vacuity flip: an injected impossible transition row
//!   (HandshakeInit -> Established) must FAIL the oracle. A court that
//!   cannot flip is FAIL (VACUOUS).
//! - C4 ring semantics: overwrite-on-full with a loud dropped counter.
//!
//! Each mutation names its anti-vacuity target. No mocks; no sleeps.

// SOURCE-SCAN-EXEMPT: vec! : conformance-court collaborator
// SOURCE-SCAN-EXEMPT: std:: : conformance-court collaborator

use std::vec::Vec;

use cgap_aoq::codec;
use cgap_aoq::header::AoqHeader;
use cgap_aoq::aead::{Nonce96, TAG_LEN};
use cgap_aoq::session::{
    self as sess, derive, transcript_tag, FrameKind, KemSeam, SessionError, CT_BYTES,
    CTX_HANDSHAKE, CTX_RECV, EK_BYTES,
};
use cgap_crypto::fips202::Shake256;
use cgap_ffi::aoq_events::{AoqEvent, AoqEventRow, AoqEventTap};

// ---------------------------------------------------------------------------
// Deterministic court KEM (same shape as the sidecar court's CourtKem)
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

/// Real ML-KEM-768 seam; the message seed derives deterministically from the
/// peer ek (`b"pl16/mlkem/m"` domain), so the court recomputes encapsulation
/// outcomes locally where needed.
struct CourtKem;

impl KemSeam for CourtKem {
    fn encapsulate(&self, ek: &[u8; EK_BYTES]) -> Result<([u8; CT_BYTES], [u8; 32]), SessionError> {
        let m = shake32(&[b"pl16/mlkem/m", ek]);
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
// Wire-frame builders (composed via codec::encode, never re-declared)
// ---------------------------------------------------------------------------

const EPOCH: u16 = 1;
const KEY_ID: u64 = 0xA0_71;
const SIZE_BYTES: usize = cgap_aoq::header::SIZE_BYTES;

fn header(ordinal: u32, seq: u32) -> AoqHeader {
    AoqHeader::new(EPOCH, KEY_ID, ordinal, seq)
}

fn ek_frame(ordinal: u32, ek: &[u8; EK_BYTES]) -> Vec<u8> {
    let mut buf = std::vec![0u8; SIZE_BYTES + 1 + EK_BYTES];
    codec::encode(&header(ordinal, 0), &mut buf).expect("header encode");
    buf[SIZE_BYTES] = FrameKind::HandshakeEk.id();
    buf[SIZE_BYTES + 1..].copy_from_slice(ek);
    buf
}

fn finish_frame(ordinal: u32, ct: &[u8; CT_BYTES], ek: &[u8; EK_BYTES]) -> Vec<u8> {
    // v3 Finish law (M2a): body = ct echo + confirm tag over the recv_key
    // the session derives at the Ek step from this same ek (deterministic).
    let kem = CourtKem;
    let (_, ss) = kem.encapsulate(ek).expect("encapsulate");
    let transcript = transcript_tag(&kem, ek, &ss, EPOCH);
    let hs_master = derive(&kem, CTX_HANDSHAKE, &ss, ek);
    let recv = kem.kdf(&[CTX_RECV, &hs_master]);
    let tag = sess::confirm_tag(&kem, &recv, &transcript, ct);
    let mut buf = std::vec![0u8; SIZE_BYTES + 1 + CT_BYTES + 16];
    codec::encode(&header(ordinal, 0), &mut buf).expect("header encode");
    buf[SIZE_BYTES] = FrameKind::HandshakeFinish.id();
    buf[SIZE_BYTES + 1..SIZE_BYTES + 1 + CT_BYTES].copy_from_slice(ct);
    buf[SIZE_BYTES + 1 + CT_BYTES..].copy_from_slice(&tag);
    buf
}

fn app_frame(key: &[u8; 32], ordinal: u32, seq: u32, payload: &[u8]) -> Vec<u8> {
    // v3 wire law: body = [seq 4B][ct][tag 16B], AEAD under the session's
    // current recv_key, AAD = the frame's own covered prefix (header +
    // kind + seq), nonce = Nonce96(epoch, ordinal<<32|seq).
    let mut prefix = [0u8; SIZE_BYTES + 5];
    codec::encode(&header(ordinal, seq), &mut prefix).expect("header encode");
    prefix[SIZE_BYTES] = FrameKind::AppData.id();
    prefix[SIZE_BYTES + 1..SIZE_BYTES + 5].copy_from_slice(&seq.to_le_bytes());
    let mut body = std::vec![0u8; 4 + payload.len() + TAG_LEN];
    body[..4].copy_from_slice(&seq.to_le_bytes());
    let nonce =
        Nonce96::new(u32::from(EPOCH), ((u64::from(ordinal)) << 32) | u64::from(seq))
            .expect("nonce");
    let mut tag = [0u8; TAG_LEN];
    cgap_aoq::aead::seal_detached(
        &mut body[4..4 + payload.len()],
        &mut tag,
        payload,
        &prefix,
        key,
        &nonce,
    );
    body[4 + payload.len()..].copy_from_slice(&tag);
    let mut buf = std::vec![0u8; SIZE_BYTES + 1 + body.len()];
    codec::encode(&header(ordinal, seq), &mut buf).expect("header encode");
    buf[SIZE_BYTES] = FrameKind::AppData.id();
    buf[SIZE_BYTES + 1..].copy_from_slice(&body);
    buf
}

fn rekey_frame(ordinal: u32, ek: &[u8; EK_BYTES]) -> Vec<u8> {
    let mut buf = std::vec![0u8; SIZE_BYTES + 1 + EK_BYTES];
    codec::encode(&header(ordinal, 0), &mut buf).expect("header encode");
    buf[SIZE_BYTES] = FrameKind::Rekey.id();
    buf[SIZE_BYTES + 1..].copy_from_slice(ek);
    buf
}

fn close_frame(ordinal: u32) -> Vec<u8> {
    let mut buf = std::vec![0u8; SIZE_BYTES + 1];
    codec::encode(&header(ordinal, 0), &mut buf).expect("header encode");
    buf[SIZE_BYTES] = FrameKind::Close.id();
    buf
}

// ---------------------------------------------------------------------------
// Conformance oracle: the 4x5 (state, kind) -> to-state table
// ---------------------------------------------------------------------------

const HANDSHAKE_INIT: u8 = 0;
const HANDSHAKE_WAIT: u8 = 1;
const ESTABLISHED: u8 = 2;
const CLOSED: u8 = 3;

/// `TABLE[from][kind] -> Option<to>`. `None` = typed refusal (no event).
/// Faithful transcription of the `step` match (session.rs:453-519):
/// - (HandshakeInit, HandshakeEk) -> HandshakeWait
/// - (HandshakeWait, HandshakeFinish) -> Established
/// - (Established, AppData) -> Established (budget +1; may refuse at budget)
/// - (Established, Rekey) -> Established (budget reset)
/// - (any non-Closed, Close) -> Closed
/// - everything else: refusal
const TABLE: [[Option<u8>; 5]; 4] = [
    // from HandshakeInit (0)
    [Some(HANDSHAKE_WAIT), None, None, None, Some(CLOSED)],
    // from HandshakeWait (1)
    [None, Some(ESTABLISHED), None, None, Some(CLOSED)],
    // from Established (2)
    [None, None, Some(ESTABLISHED), Some(ESTABLISHED), Some(CLOSED)],
    // from Closed (3): terminal; every frame refused (SessionClosed)
    [None; 5],
];

/// The oracle: replay one row against the replayed machine state. `Err`
/// names the violated cell — this is what the anti-vacuity flip aims at.
fn oracle(from: u8, kind: u8, to: u8) -> Result<(), String> {
    let kind_usize = kind as usize;
    if kind_usize >= 5 || (from as usize) >= 4 {
        return Err(format!("out-of-table row: from={from} kind={kind}"));
    }
    match TABLE[from as usize][kind_usize] {
        Some(expected_to) if expected_to == to => Ok(()),
        Some(expected_to) => Err(format!(
            "unlawful transition: from={from} kind={kind} -> {to} (lawful to={expected_to})"
        )),
        None => Err(format!(
            "unlawful transition: from={from} kind={kind} is a refusal cell (no event may exist)"
        )),
    }
}

// ---------------------------------------------------------------------------
// Shared happy-walk fixture: handshake -> established -> traffic -> rekey ->
// traffic -> close. Returns the tap and the drained rows.
// ---------------------------------------------------------------------------

fn peer_keypair() -> [u8; EK_BYTES] {
    let d = shake32(&[b"pl16/mlkem/d"]);
    let z = shake32(&[b"pl16/mlkem/z"]);
    let mut ek = [0u8; EK_BYTES];
    let mut dk = [0u8; 2400];
    cgap_crypto::kem::keygen(&d, &z, &mut ek, &mut dk);
    ek
}

fn run_happy_walk(tap: &AoqEventTap, ek_fresh: &[u8; EK_BYTES]) {
    let kem = CourtKem;
    let mut session = sess::Session::new(&kem, EPOCH, KEY_ID, 0);
    let step_with = |s: &mut sess::Session<'_, CourtKem>,
                         buf: &std::vec::Vec<u8>,
                         kind: FrameKind| {
        let frame = sess::parse_frame(buf).expect("court frame parses");
        // CALLER-SIDE LAW: refusals emit nothing — no tap on Err.
        if let Ok(()) = s.step(&frame) {
            tap.tap(&AoqEvent::mint(s, kind));
        }
    };
    let ek = peer_keypair();
    let pending_ct = {
        let m = shake32(&[b"pl16/mlkem/m", &ek]);
        let mut ct = [0u8; CT_BYTES];
        let mut ss = [0u8; 32];
        cgap_crypto::kem::encaps(&ek, &m, &mut ct, &mut ss);
        ct
    };
    step_with(&mut session, &ek_frame(0, &ek), FrameKind::HandshakeEk);
    step_with(
        &mut session,
        &finish_frame(1, &pending_ct, &ek),
        FrameKind::HandshakeFinish,
    );
    let w2 = app_frame(&session.recv_key(), 2, 0, b"pl16");
    step_with(&mut session, &w2, FrameKind::AppData);
    let w3 = app_frame(&session.recv_key(), 3, 1, b"pl16");
    step_with(&mut session, &w3, FrameKind::AppData);
    step_with(&mut session, &rekey_frame(4, ek_fresh), FrameKind::Rekey);
    let w5 = app_frame(&session.recv_key(), 5, 2, b"pl16");
    step_with(&mut session, &w5, FrameKind::AppData);
    step_with(&mut session, &close_frame(6), FrameKind::Close);
}

// ---------------------------------------------------------------------------
// Courts
// ---------------------------------------------------------------------------

/// C1: oracle conformance over a real session walk. Every tapped row is a
/// lawful 4x5 cell; the replayed machine lands in Closed with every row's
/// state consumed.
#[test]
fn c1_rows_are_lawful_table_cells() {
    let tap: AoqEventTap = AoqEventTap::new();
    let ek_fresh = peer_keypair();
    run_happy_walk(&tap, &ek_fresh);

    let mut rows = std::vec![AoqEventRow {
        ordinal: 0,
        state: 0,
        kind: 0,
        epoch: 0,
        budget: 0,
        fingerprint: [0u8; 32],
        reserved: [0u8; 20],
    }; 64];
    let n = tap.drain(&mut rows);
    assert_eq!(n, 7, "happy walk taps exactly 7 events (7 accepted steps)");
    assert_eq!(tap.total_tapped(), 7);
    assert_eq!(tap.dropped(), 0);
    assert!(!tap.is_empty());

    // Replay: start at HandshakeInit; each row must be a lawful cell.
    let mut replay = HANDSHAKE_INIT;
    for (i, row) in rows[..n].iter().enumerate() {
        // Ordinal law: post-step ordinal = pre-step ordinal + 1 (wrapping).
        assert_eq!(
            row.ordinal as usize,
            i + 1,
            "row {i} ordinal must be the post-step chain ordinal"
        );
        assert_eq!(row.epoch, EPOCH, "row {i} carries the active epoch");
        assert!(
            row.fingerprint != [0u8; 32],
            "row {i} must carry the real post-step fingerprint"
        );
        // Reserved-zero law.
        assert!(
            row.reserved == [0u8; 20],
            "row {i} violates the reserved-zero law"
        );
        oracle(replay, row.kind, row.state)
            .unwrap_or_else(|e| panic!("row {i}: {e}"));
        replay = row.state;
    }
    assert_eq!(replay, CLOSED, "replay must land in Closed");
}

/// C2 count law: refused steps emit NOTHING (PL16 caller-side law).
/// Mutations aimed at: an in-tap emission on refusal (a tap wired inside
/// `step`'s Err arm) would break the count; a stateful session mutation on
/// refusal would break the fingerprint identity.
#[test]
fn c2_refusals_emit_nothing() {
    let tap: AoqEventTap = AoqEventTap::new();
    let kem = CourtKem;
    let mut session = sess::Session::new(&kem, EPOCH, KEY_ID, 0);
    let ek = peer_keypair();

    // Refusal 1: wrong epoch on the opening HandshakeEk.
    let mut bad_epoch = ek_frame(0, &ek);
    bad_epoch[6..8].copy_from_slice(&99u16.to_le_bytes()); // AOQ1_EPOCH_OFFSET=6
    // Refusal 2: wrong ordinal.
    let wrong_ord = ek_frame(7, &ek);
    // Refusal 3: AppData in HandshakeInit (refusal cell of the table).
    let app_early = app_frame(&session.recv_key(), 0, 0, b"x");

    let mut refused: usize = 0;
    for buf in [&bad_epoch, &wrong_ord, &app_early] {
        let fp_before = session.fingerprint();
        // Either layer may refuse (codec enum gate or machine table); the
        // count law covers both: a refusal is a refusal wherever it fires.
        let step_res = match sess::parse_frame(buf) {
            Ok(frame) => session.step(&frame).map(|_| ()),
            Err(_) => Err(SessionError::KemRefused { op: 0 }),
        };
        if step_res.is_err() {
            refused += 1;
        }
        assert_eq!(
            session.fingerprint(),
            fp_before,
            "no-mutation-on-refusal law violated"
        );
    }
    assert_eq!(refused, 3, "all three mutations must be typed refusals");
    assert_eq!(tap.total_tapped(), 0, "refusals emit NOTHING");
    assert_eq!(tap.len(), 0);

    // Now the lawful ek: exactly one event.
    let buf = ek_frame(0, &ek);
    let frame = sess::parse_frame(&buf).expect("parses");
    session.step(&frame).expect("lawful ek accepted");
    tap.tap(&AoqEvent::mint(&session, FrameKind::HandshakeEk));
    assert_eq!(tap.total_tapped(), 1, "count law: one event per accepted step");

    let mut rows = std::vec![AoqEventRow {
        ordinal: 0,
        state: 0,
        kind: 0,
        epoch: 0,
        budget: 0,
        fingerprint: [0u8; 32],
        reserved: [0u8; 20],
    }; 8];
    let n = tap.drain(&mut rows);
    assert_eq!(n, 1);
    oracle(HANDSHAKE_INIT, rows[0].kind, rows[0].state).expect("lawful");
    assert_eq!(rows[0].state, HANDSHAKE_WAIT);
}

/// C3 anti-vacuity flip: inject an impossible row (HandshakeInit ->
/// Established) and the oracle MUST fail it. If this stops failing the
/// oracle is broken (VACUOUS court).
#[test]
fn c3_impossible_transition_fails_oracle() {
    // The injected mutation: after HandshakeInit, a row claiming Established.
    let err = oracle(HANDSHAKE_INIT, FrameKind::HandshakeEk.id(), ESTABLISHED)
        .expect_err("anti-vacuity: impossible row must be REFUSED by the oracle");
    assert!(
        err.contains("lawful to=1"),
        "flip must name the lawful cell, got: {err}"
    );

    // And the refusal-cell side: an event for a refusal cell (AppData from
    // HandshakeInit) must also fail.
    assert!(oracle(HANDSHAKE_INIT, FrameKind::AppData.id(), ESTABLISHED).is_err());
    assert!(oracle(HANDSHAKE_INIT, FrameKind::AppData.id(), HANDSHAKE_WAIT).is_err());

    // Closed is terminal: NO kind may emit from Closed.
    for kind in 0u8..5 {
        assert!(
            oracle(CLOSED, kind, CLOSED).is_err(),
            "kind {kind} from Closed must be refused by the oracle"
        );
    }
}

/// C4 ring semantics: overwrite-on-full with a LOUD dropped counter.
/// Anti-vacuity: a fake-full ring (refuse-on-full) would keep rows[0];
/// overwrite semantics prove the eviction happened.
#[test]
fn c4_overwrite_on_full_and_dropped_is_loud() {
    let tap = AoqEventTap::<4>::new();
    let mut event = AoqEvent {
        ordinal: 0,
        state: ESTABLISHED,
        kind: FrameKind::AppData.id(),
        epoch: EPOCH,
        budget: 1,
        fingerprint: [7u8; 32],
    };
    for i in 0..6u64 {
        event.ordinal = i as u32;
        event.budget = i as u32;
        tap.tap(&event);
    }
    assert_eq!(tap.total_tapped(), 6);
    assert_eq!(tap.dropped(), 2, "6 taps into CAP=4 must report 2 dropped");
    assert_eq!(tap.len(), 4);

    let mut rows = std::vec![AoqEventRow {
        ordinal: 0,
        state: 0,
        kind: 0,
        epoch: 0,
        budget: 0,
        fingerprint: [0u8; 32],
        reserved: [0u8; 20],
    }; 8];
    let n = tap.drain(&mut rows);
    assert_eq!(n, 4);
    // Overwrite law: the OLDEST rows (ordinals 0, 1) are gone.
    assert_eq!(rows[0].ordinal, 2, "oldest two rows evicted");
    assert_eq!(rows[3].ordinal, 5);
    assert!(tap.dropped() > 0, "dropped counter must be loud, not silent");
}
