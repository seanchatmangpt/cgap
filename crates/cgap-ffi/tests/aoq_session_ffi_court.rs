//! W2-A session C-ABI court (wave-2-membrane-plan section 1 Lane W2-A).
//!
//! Chicago discipline: real session objects behind the real FFI exports, no
//! mocks. Each gate aims its refusal at a mutation of the happy-path bytes,
//! so a vacuous gate (one that can never fire) is structurally impossible.
//!
//! Status-band disclosure (contract-block delta 4): the SessionError
//! taxonomy is 10 variants; the C-ABI band is the frozen 8-word
//! 0x4347A001..A008 set. The 10->8 family-clustered map is pinned in
//! `src/aoq_session.rs::status_of` and asserted band-closed here; the
//! `aoq_status_name` round-trip is asserted for every word the surface
//! returns.

use cgap_aoq::codec;
use cgap_aoq::header::{AoqHeader, SIZE_BYTES};
use cgap_aoq::session::{parse_frame, FrameKind, CT_BYTES, EK_BYTES};
use cgap_crypto::fips202::Shake256;
use cgap_ffi::aoq::status_maps::{
    aoq_status_name, AOQ_STATUS_BUSY, AOQ_STATUS_DIGEST_MISMATCH, AOQ_STATUS_EPOCH_MISMATCH,
    AOQ_STATUS_EPOCH_RETIRED, AOQ_STATUS_INCOMPLETE_FRAME, AOQ_STATUS_KEY_UNKNOWN, AOQ_STATUS_OK,
};

// ---------------------------------------------------------------------------
// Real digest substrate: Shake256 over the same derivation law the FFI host
// seam pins (`b"w2a/mlkem/m"` seed domain). The court re-derives the pending
// ct the engine must echo, so the happy-path court is a REAL round trip and
// the ct-echo mutation court is aimed, not guessed.
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

/// The peer's ML-KEM-768 keypair (deterministic seeds, no RNG in tests).
fn peer_keypair() -> ([u8; EK_BYTES], Vec<u8>) {
    let d = shake32(&[b"w2a/mlkem/d"]);
    let z = shake32(&[b"w2a/mlkem/z"]);
    let mut ek = [0u8; EK_BYTES];
    let mut dk = [0u8; 2400];
    cgap_crypto::kem::keygen(&d, &z, &mut ek, &mut dk);
    (ek, dk.to_vec())
}

/// The pending ct the engine holds after accepting an ek frame: the FFI
/// host seam derives the FIPS-203 message seed deterministically from the
/// peer ek (`b"w2a/mlkem/m"`), so the court can compute it exactly.
fn expected_pending_ct(ek: &[u8; EK_BYTES]) -> [u8; CT_BYTES] {
    let m = shake32(&[b"w2a/mlkem/m", ek]);
    let mut ct = [0u8; CT_BYTES];
    let mut ss = [0u8; 32];
    cgap_crypto::kem::encaps(ek, &m, &mut ct, &mut ss);
    ct
}

// ---------------------------------------------------------------------------
// Wire-frame builders (AOQ1 header law composed via codec::encode, never
// re-declared)
// ---------------------------------------------------------------------------

const EPOCH: u16 = 1;
const KEY_ID: u64 = 0xA0_71;

fn header(ordinal: u32, seq: u32) -> AoqHeader {
    AoqHeader::new(EPOCH, KEY_ID, ordinal, seq)
}

fn ek_frame(ordinal: u32, ek: &[u8; EK_BYTES]) -> Vec<u8> {
    let mut buf = vec![0u8; SIZE_BYTES + 1 + EK_BYTES];
    codec::encode(&header(ordinal, 0), &mut buf).expect("header encode");
    buf[SIZE_BYTES] = FrameKind::HandshakeEk.id();
    buf[SIZE_BYTES + 1..].copy_from_slice(ek);
    buf
}

fn finish_frame(ordinal: u32, ct: &[u8; CT_BYTES]) -> Vec<u8> {
    let mut buf = vec![0u8; SIZE_BYTES + 1 + CT_BYTES];
    codec::encode(&header(ordinal, 0), &mut buf).expect("header encode");
    buf[SIZE_BYTES] = FrameKind::HandshakeFinish.id();
    buf[SIZE_BYTES + 1..].copy_from_slice(ct);
    buf
}

fn app_frame(ordinal: u32, seq: u32, payload: &[u8]) -> Vec<u8> {
    let mut buf = vec![0u8; SIZE_BYTES + 1 + 4 + payload.len()];
    codec::encode(&header(ordinal, seq), &mut buf).expect("header encode");
    buf[SIZE_BYTES] = FrameKind::AppData.id();
    buf[SIZE_BYTES + 1..SIZE_BYTES + 5].copy_from_slice(&seq.to_be_bytes());
    buf[SIZE_BYTES + 5..].copy_from_slice(payload);
    buf
}

fn close_frame(ordinal: u32) -> Vec<u8> {
    let mut buf = vec![0u8; SIZE_BYTES + 1];
    codec::encode(&header(ordinal, 0), &mut buf).expect("header encode");
    buf[SIZE_BYTES] = FrameKind::Close.id();
    buf
}

/// Every status word the surface can return must round-trip through the
/// generated name map (closed-band law).
fn assert_in_band(w: u32) {
    assert!(
        aoq_status_name(w).is_some(),
        "word {w:#x} outside frozen band"
    );
    if w != AOQ_STATUS_OK {
        assert_ne!(w, AOQ_STATUS_OK, "refusal must not be OK");
    }
}

/// Drive the session to Established with the REAL ML-KEM handshake.
fn establish() -> u64 {
    let (ek, _dk) = peer_keypair();
    let h = unsafe { cgap_ffi::aoq_session::cgap_aoq_session_new(EPOCH, KEY_ID, 0) };
    assert_ne!(h, 0, "session_new returned 0");
    let ek_wire = ek_frame(0, &ek);
    let st =
        unsafe { cgap_ffi::aoq_session::cgap_aoq_session_step(h, ek_wire.as_ptr(), ek_wire.len()) };
    assert_eq!(st, AOQ_STATUS_OK, "ek step refused: {st:#x}");
    let ct = expected_pending_ct(&ek);
    let fin = finish_frame(1, &ct);
    let st = unsafe { cgap_ffi::aoq_session::cgap_aoq_session_step(h, fin.as_ptr(), fin.len()) };
    assert_eq!(st, AOQ_STATUS_OK, "finish step refused: {st:#x}");
    h
}

fn fingerprint(h: u64) -> [u8; 32] {
    let mut fp = [0u8; 32];
    let st = unsafe { cgap_ffi::aoq_session::cgap_aoq_session_fingerprint(h, fp.as_mut_ptr()) };
    assert_eq!(st, AOQ_STATUS_OK, "fingerprint refused: {st:#x}");
    fp
}

// ---------------------------------------------------------------------------
// COURT 1 — happy path: real ML-KEM handshake to Established, then a
// verified emit round trip through parse_frame with tag re-derivation.
// ---------------------------------------------------------------------------

#[test]
fn court1_happy_path_handshake_and_emit_round_trip() {
    let (ek, _dk) = peer_keypair();
    let h = unsafe { cgap_ffi::aoq_session::cgap_aoq_session_new(EPOCH, KEY_ID, 0) };
    assert_ne!(h, 0, "session_new returned 0");

    // HandshakeInit -> HandshakeWait on the peer ek frame.
    let ek_wire = ek_frame(0, &ek);
    let st =
        unsafe { cgap_ffi::aoq_session::cgap_aoq_session_step(h, ek_wire.as_ptr(), ek_wire.len()) };
    assert_eq!(st, AOQ_STATUS_OK, "ek step refused: {st:#x}");

    // HandshakeWait -> Established on the ct-echo finish frame.
    let ct = expected_pending_ct(&ek);
    let fin = finish_frame(1, &ct);
    let st = unsafe { cgap_ffi::aoq_session::cgap_aoq_session_step(h, fin.as_ptr(), fin.len()) };
    assert_eq!(st, AOQ_STATUS_OK, "finish step refused: {st:#x}");

    // Verified emit: [header 64B][kind 2][seq BE 4B][payload][tag 4B].
    let payload: [u8; 16] = core::array::from_fn(|i| (i as u8) ^ 0x5a);
    let seq: u32 = 42;
    let mut out = [0u8; SIZE_BYTES + 1 + 4 + 16 + 4];
    let mut written = 0usize;
    let st = unsafe {
        cgap_ffi::aoq_session::cgap_aoq_session_emit_verified(
            h,
            seq,
            payload.as_ptr(),
            payload.len(),
            out.as_mut_ptr(),
            out.len(),
            &mut written,
        )
    };
    assert_eq!(st, AOQ_STATUS_OK, "emit refused: {st:#x}");
    assert_eq!(written, out.len(), "emit wrote the full frame");

    // Round trip through the real parser: kind, header fields, tag.
    let frame = parse_frame(&out[..written]).expect("emitted frame must parse");
    assert_eq!(frame.kind, FrameKind::AppData);
    assert_eq!(frame.header.epoch, EPOCH);
    assert_eq!(frame.header.key_id, KEY_ID);
    assert_eq!(
        frame.header.frame_ordinal, 2,
        "emit uses the current ordinal"
    );
    assert_eq!(frame.header.seq, seq);
    assert_eq!(frame.payload[..4], seq.to_be_bytes(), "seq prefix");
    assert_eq!(
        &frame.payload[4..4 + payload.len()],
        &payload,
        "payload identity"
    );

    // Tag law: kdf(header||kind||seq||payload)[0..4] over the WHOLE wire
    // frame minus the trailing 4 tag bytes.
    let covered = &out[..written - 4];
    let tag = shake32(&[covered]);
    assert_eq!(&out[written - 4..written], &tag[0..4], "emit tag law");

    // Close, then free; free of a live handle is OK exactly once.
    let cl = close_frame(2);
    let st = unsafe { cgap_ffi::aoq_session::cgap_aoq_session_step(h, cl.as_ptr(), cl.len()) };
    assert_eq!(st, AOQ_STATUS_OK);
    let st = unsafe { cgap_ffi::aoq_session::cgap_aoq_session_free(h) };
    assert_eq!(st, AOQ_STATUS_OK);
}

// ---------------------------------------------------------------------------
// COURT 2 — mutation A (contract block): flipping ONE ct-echo byte must
// yield TranscriptMismatch -> DIGEST_MISMATCH with fingerprint identity
// (session law 4 across the FFI boundary).
// ---------------------------------------------------------------------------

#[test]
fn court2_ct_echo_flip_refused_fingerprint_unchanged() {
    let (ek, _dk) = peer_keypair();
    let h = unsafe { cgap_ffi::aoq_session::cgap_aoq_session_new(EPOCH, KEY_ID, 0) };
    let ek_wire = ek_frame(0, &ek);
    let st =
        unsafe { cgap_ffi::aoq_session::cgap_aoq_session_step(h, ek_wire.as_ptr(), ek_wire.len()) };
    assert_eq!(st, AOQ_STATUS_OK);
    let fp_before = fingerprint(h);

    // Mutation A: the real pending ct with one byte flipped.
    let mut ct = expected_pending_ct(&ek);
    ct[7] ^= 0x01;
    let fin = finish_frame(1, &ct);
    let st = unsafe { cgap_ffi::aoq_session::cgap_aoq_session_step(h, fin.as_ptr(), fin.len()) };
    assert_eq!(
        st, AOQ_STATUS_DIGEST_MISMATCH,
        "ct flip must refuse: {st:#x}"
    );
    assert_in_band(st);
    assert_eq!(cgap_ffi::aoq_session::cgap_aoq_last_error(), st);

    // Law 4: the refused step left the session byte-identical.
    assert_eq!(fingerprint(h), fp_before, "refusal mutated the session");
}
// ---------------------------------------------------------------------------
// COURT 3 — fingerprint identity after EVERY refused step (session law 4
// across the boundary): a battery of refusals, each asserted
// fingerprint-identical, each word in the frozen band, last-error mirror.
// ---------------------------------------------------------------------------

#[test]
fn court3_fingerprint_identity_after_every_refused_step() {
    let (ek, _dk) = peer_keypair();
    let h = unsafe { cgap_ffi::aoq_session::cgap_aoq_session_new(EPOCH, KEY_ID, 0) };

    let mut refused: Vec<u32> = Vec::new();
    let fp0 = fingerprint(h);

    // (a) wrong epoch (legal epoch value, not the active one).
    let mut bad = ek_frame(0, &ek);
    codec::encode(&AoqHeader::new(2, KEY_ID, 0, 0), &mut bad).expect("encode");
    bad[SIZE_BYTES] = FrameKind::HandshakeEk.id();
    // (b) wrong ordinal (replay/gap).
    let bad_ord = ek_frame(9, &ek);
    // (c) unknown frame kind (0x7F outside the closed 5-kind table).
    let mut bad_kind = ek_frame(0, &ek);
    bad_kind[SIZE_BYTES] = 0x7F;
    // (d) truncated frame (header only, no kind tag).
    let truncated = ek_wire_short();
    // (e) garbage bytes (bad magic).
    let garbage = vec![0xa5u8; 512];
    // (f) AppData in HandshakeInit (unexpected (state, frame) cell).
    let app_too_early = app_frame(0, 1, b"nope");

    for (name, frame) in [
        ("wrong-epoch", &bad),
        ("wrong-ordinal", &bad_ord),
        ("unknown-kind", &bad_kind),
        ("truncated", &truncated),
        ("garbage", &garbage),
        ("appdata-in-init", &app_too_early),
    ] {
        let fp_before = fingerprint(h);
        let st =
            unsafe { cgap_ffi::aoq_session::cgap_aoq_session_step(h, frame.as_ptr(), frame.len()) };
        assert_ne!(st, AOQ_STATUS_OK, "{name} must refuse");
        assert_in_band(st);
        refused.push(st);
        // Last-error mirror law (read BEFORE the fingerprint export runs —
        // fingerprint is itself an export and refreshes last_error to OK).
        assert_eq!(cgap_ffi::aoq_session::cgap_aoq_last_error(), st);
        // LAW 4 across the boundary: byte-identical session after refusal.
        assert_eq!(fingerprint(h), fp_before, "{name} mutated the session");
    }

    // The battery actually fires six distinct firing decisions; the words
    // are within the closed band and at least three distinct words appear
    // (vacuity check on the mapping itself).
    assert_eq!(refused.len(), 6);
    let mut distinct = refused.clone();
    distinct.sort_unstable();
    distinct.dedup();
    assert!(distinct.len() >= 3, "mapping collapsed: {refused:?}");

    // Anti-vacuity for law 4 itself: an ACCEPTED step must move the
    // fingerprint (otherwise identity-after-refusal is vacuous).
    let ek_wire = ek_frame(0, &ek);
    let st =
        unsafe { cgap_ffi::aoq_session::cgap_aoq_session_step(h, ek_wire.as_ptr(), ek_wire.len()) };
    assert_eq!(st, AOQ_STATUS_OK, "happy step refused: {st:#x}");
    assert_ne!(
        fingerprint(h),
        fp0,
        "accepted step must move the fingerprint (law-4 vacuity check)"
    );
}

/// Header-only wire bytes (truncated frame: no kind tag).
fn ek_wire_short() -> Vec<u8> {
    let mut buf = vec![0u8; SIZE_BYTES];
    codec::encode(&header(0, 0), &mut buf).expect("encode");
    buf
}

// ---------------------------------------------------------------------------
// COURT 4 — stale-handle refusal: a freed handle is refused by EVERY export
// with the typed KEY_UNKNOWN word; a double free refuses; a fresh handle is
// distinct from the freed one (monotone, never reused).
// ---------------------------------------------------------------------------

#[test]
fn court4_stale_handle_refused_everywhere() {
    let h = establish();
    let st = unsafe { cgap_ffi::aoq_session::cgap_aoq_session_free(h) };
    assert_eq!(st, AOQ_STATUS_OK, "first free must succeed");

    // After free, every export refuses with the same typed word.
    let (ek, _dk) = peer_keypair();
    let wire = ek_frame(0, &ek);
    let st_step =
        unsafe { cgap_ffi::aoq_session::cgap_aoq_session_step(h, wire.as_ptr(), wire.len()) };
    let mut fp = [0u8; 32];
    let st_fp = unsafe { cgap_ffi::aoq_session::cgap_aoq_session_fingerprint(h, fp.as_mut_ptr()) };
    let mut out = [0u8; 256];
    let mut written = 0usize;
    let st_emit = unsafe {
        cgap_ffi::aoq_session::cgap_aoq_session_emit_verified(
            h,
            0,
            out.as_ptr(),
            0,
            out.as_mut_ptr(),
            out.len(),
            &mut written,
        )
    };
    let st_free = unsafe { cgap_ffi::aoq_session::cgap_aoq_session_free(h) };
    assert_eq!(st_step, AOQ_STATUS_KEY_UNKNOWN, "stale step");
    assert_eq!(st_fp, AOQ_STATUS_KEY_UNKNOWN, "stale fingerprint");
    assert_eq!(st_emit, AOQ_STATUS_KEY_UNKNOWN, "stale emit");
    assert_eq!(st_free, AOQ_STATUS_KEY_UNKNOWN, "double free");
    assert_eq!(written, 0, "stale emit must not write");

    // Never-existed handle: refused identically (no existence oracle).
    let st_never =
        unsafe { cgap_ffi::aoq_session::cgap_aoq_session_fingerprint(u64::MAX, fp.as_mut_ptr()) };
    assert_eq!(st_never, AOQ_STATUS_KEY_UNKNOWN);

    // Monotone handles: a fresh session never reuses the freed id.
    let h2 = unsafe { cgap_ffi::aoq_session::cgap_aoq_session_new(EPOCH, KEY_ID, 0) };
    assert_ne!(h2, 0);
    assert_ne!(h2, h, "handle reuse after free");
    let st = unsafe { cgap_ffi::aoq_session::cgap_aoq_session_free(h2) };
    assert_eq!(st, AOQ_STATUS_OK);
}

// ---------------------------------------------------------------------------
// COURT 5 — no-panic-across-the-boundary: hostile pointer/length shapes
// return typed refusal words; the process survives (the test passing IS the
// no-panic witness; catch_unwind containment is the mechanism under test).
// ---------------------------------------------------------------------------

#[test]
fn court5_hostile_inputs_never_panic() {
    let h = establish();

    // Null frame pointer with nonzero length: guarded BEFORE slice
    // formation.
    let st = unsafe { cgap_ffi::aoq_session::cgap_aoq_session_step(h, core::ptr::null(), 64) };
    assert_eq!(st, AOQ_STATUS_INCOMPLETE_FRAME);
    // Zero-length frame: refused before parse.
    let ok_ptr = {
        let (ek, _dk) = peer_keypair();
        let wire = ek_frame(0, &ek);
        // one-byte dangling-ish stand-in: a real allocation of len 1
        let b = [0u8; 1];
        let _ = wire;
        b.as_ptr()
    };
    let st = unsafe { cgap_ffi::aoq_session::cgap_aoq_session_step(h, ok_ptr, 0) };
    assert_eq!(st, AOQ_STATUS_INCOMPLETE_FRAME);

    // Fingerprint into a null buffer: refused.
    let st =
        unsafe { cgap_ffi::aoq_session::cgap_aoq_session_fingerprint(h, core::ptr::null_mut()) };
    assert_eq!(st, AOQ_STATUS_INCOMPLETE_FRAME);

    // Emit with null out / null out_len / too-small out: refused, typed.
    let payload = [0u8; 8];
    let mut out = [0u8; 8]; // far too small for 64+1+4+8+4
    let mut written = 0usize;
    let st = unsafe {
        cgap_ffi::aoq_session::cgap_aoq_session_emit_verified(
            h,
            0,
            payload.as_ptr(),
            payload.len(),
            core::ptr::null_mut(),
            out.len(),
            &mut written,
        )
    };
    assert_eq!(st, AOQ_STATUS_INCOMPLETE_FRAME);
    let st = unsafe {
        cgap_ffi::aoq_session::cgap_aoq_session_emit_verified(
            h,
            0,
            payload.as_ptr(),
            payload.len(),
            out.as_mut_ptr(),
            out.len(),
            core::ptr::null_mut(),
        )
    };
    assert_eq!(st, AOQ_STATUS_INCOMPLETE_FRAME);
    let st = unsafe {
        cgap_ffi::aoq_session::cgap_aoq_session_emit_verified(
            h,
            0,
            payload.as_ptr(),
            payload.len(),
            out.as_mut_ptr(),
            out.len(),
            &mut written,
        )
    };
    assert_eq!(st, AOQ_STATUS_INCOMPLETE_FRAME, "too-small out cap");
    assert_eq!(written, 0);

    // The session survives every hostile call untouched.
    let fp = fingerprint(h);
    assert_ne!(fp, [0u8; 32], "live session still answers");

    let st = unsafe { cgap_ffi::aoq_session::cgap_aoq_session_free(h) };
    assert_eq!(st, AOQ_STATUS_OK);
}

// ---------------------------------------------------------------------------
// COURT 6 — status-band closure over the WHOLE surface: every word this
// module can return (mapped refusals + stale-handle + internal fault) is in
// the frozen 8-word band and round-trips through the generated name map.
// ---------------------------------------------------------------------------

#[test]
fn court6_status_band_is_closed() {
    let band = [
        AOQ_STATUS_OK,
        cgap_ffi::aoq::status_maps::AOQ_STATUS_PENDING,
        AOQ_STATUS_BUSY,
        AOQ_STATUS_INCOMPLETE_FRAME,
        AOQ_STATUS_DIGEST_MISMATCH,
        AOQ_STATUS_EPOCH_MISMATCH,
        AOQ_STATUS_KEY_UNKNOWN,
        AOQ_STATUS_EPOCH_RETIRED,
    ];
    assert_eq!(band.len(), cgap_ffi::aoq::status_maps::AOQ_STATUS_COUNT);
    for w in band {
        let name = aoq_status_name(w).expect("band word must map to a name");
        let back = cgap_ffi::aoq::status_maps::aoq_status_word(name).expect("name round trip");
        assert_eq!(back, w, "round-trip drift for {name}");
    }
    // Outside-band words map to None (typed refusal at call sites).
    assert_eq!(aoq_status_name(0x4347_0001), None);
    assert_eq!(aoq_status_name(0x4347_A009), None);
    assert_eq!(aoq_status_name(0), None);
}
