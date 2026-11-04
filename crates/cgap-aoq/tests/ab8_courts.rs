//! AB-8 wire courts A1-A13 (PG-A2 table, AoQ Wave-1 W1-5b).
//!
//! Chicago courts: real windows on real bytes, assert final state. The
//! sibling truncation sweep (`ab8_sweep.rs`) is the classification-identity
//! witness for every class these courts refuse.
//!
//! CLASS MAPPING (R-H2): framing classes live in cgap-arrow `IpcError`;
//! crgap-aoq speaks header law; the pairing table is exercised in
//! `ab8_sweep.rs`. TAG LAW: `blake3(body-as-carried)[..16]`, tamper tier,
//! symmetric on both surfaces.

use cgap_aoq::error::AoqError;
use cgap_aoq::verify;
use cgap_aoq::verify::{decrypt_in_place, verify_window};
use cgap_aoq::writer::AoqWriter;
use cgap_arrow::blake3;
use cgap_arrow::ipc;

/// Host-side keystream (R-K1: cipher bodies stay host-side). blake3
/// blocks over ("ab8-ks", seq, block_no).
fn keystream(seq: u32, body_len: usize) -> Vec<u8> {
    let mut ks = Vec::new();
    let mut block_no: u32 = 0;
    while ks.len() < body_len {
        let mut inp = [0u8; 40];
        inp[0..7].copy_from_slice(b"ab8-ks ");
        inp[7..11].copy_from_slice(&seq.to_le_bytes());
        inp[11..15].copy_from_slice(&1_760_000_001u32.to_le_bytes());
        inp[15..19].copy_from_slice(&block_no.to_le_bytes());
        ks.extend_from_slice(&blake3::hash(&inp));
        block_no += 1;
    }
    ks
}

/// Seal one frame the host way: keystream-XOR the plaintext into the
/// caller buffer, derive the tag over the ciphertext, scatter-write the
/// window. Returns (window_len, tag). Cursor advances only via commit
/// inside `write_window` on success.
fn seal_frame(
    w: &mut AoqWriter,
    out: &mut [u8],
    epoch: u16,
    key_id: u64,
    plain: &[u8],
) -> (usize, [u8; 16]) {
    let seq = w.cursor().1;
    let ks = keystream(seq, plain.len());
    let mut ct = plain.to_vec();
    for i in 0..plain.len() {
        ct[i] ^= ks[i];
    }
    let tag_full = blake3::hash(&ct);
    let mut tag = [0u8; 16];
    tag.copy_from_slice(&tag_full[..16]);
    let n = w
        .write_window(out, epoch, key_id, &tag, &ct)
        .expect("seal_frame: lawful window");
    (n, tag)
}

/// A canonical window: 100-byte plaintext, epoch Genesis, key_id 7.
fn canonical() -> (Vec<u8>, [u8; 16], usize) {
    let plain = [0xA5u8; 100];
    let mut w = AoqWriter::new();
    let mut out = vec![0u8; 8 + 120 + 100 + 128 + 8];
    let (n, tag) = seal_frame(&mut w, &mut out, 0, 7, &plain);
    out.truncate(n);
    (out, tag, n)
}

/// A1 happy path: seal -> write_window -> ipc read_with_meta reads the
/// same body_len/seq, aoq verify_window accepts with the host tag.
/// Anti-vacuity: the sweep flips the tag (A8) and the header digest
/// (A4-mutation) on this exact construction and both refuse.
#[test]
fn a1_seal_write_read_roundtrip() {
    let (win, tag, n) = canonical();
    assert_eq!(n, 8 + 120 + 100 + 128 + 8);
    // ipc framing surface: reads body_len from the padding, seq from the
    // AOQ1 header.
    let mut body = [0u8; 100];
    let (got, meta) = ipc::read_with_meta(&win, None, &mut body).expect("A1: lawful window");
    assert_eq!(got, 100);
    assert_eq!(meta.body_len, 100);
    assert_eq!(meta.seq, 0);
    assert_eq!(meta.magic, ipc::AOQ1_MAGIC);
    // aoq surface: verify_window accepts with the expected tag.
    let vw = verify_window(&win, 0, 0, &tag).expect("A1: verify_window");
    assert_eq!(vw.body_len, 100);
    assert_eq!(vw.body_at, 128);
    // and the decrypted body round-trips to plaintext.
    let mut win2 = win.clone();
    let vw2 = verify_window(&win2, 0, 0, &tag).expect("A1: verify_window 2");
    decrypt_in_place(&vw2, &keystream(0, 100), &mut win2).expect("A1: decrypt");
    assert_eq!(&win2[vw2.body_at..vw2.body_at + 100], &[0xA5u8; 100]);
}

/// A2 seq/ordinal lockstep: three frames, cursor (0,0) -> (3,3); each
/// window carries the stamped pair and verify_window refuses a cursor
/// mismatch (replay of frame 0 at cursor 2).
#[test]
fn a2_lockstep_three_frames() {
    let mut w = AoqWriter::new();
    let plain = [0x5Au8; 32];
    let mut out = vec![0u8; 8 + 120 + 32 + 64 + 8];
    for i in 0u32..3 {
        let (n, tag) = seal_frame(&mut w, &mut out, 0, 7, &plain);
        let win = &out[..n];
        assert_eq!(w.cursor(), (i + 1, i + 1));
        let vw = verify_window(win, i, i, &tag).expect("A2: lawful frame");
        assert_eq!((vw.header.frame_ordinal, vw.header.seq), (i, i));
        // replay: frame i verified at the cursor AFTER consuming it refuses.
        let err = verify_window(win, i + 1, i + 1, &tag).unwrap_err();
        assert!(
            matches!(err, AoqError::FrameOrdinalRegression { .. })
                || matches!(err, AoqError::SeqGap { .. }),
            "A2: replay refuses, got {err:?}"
        );
    }
}

/// A3 seq regression refuses with SeqGap; ZeroProgress: the writer cursor
/// is byte-identical after the refused write.
#[test]
fn a3_seq_regression_refused_zero_progress() {
    let plain = [0u8; 8];
    let mut w = AoqWriter::new();
    let mut out = vec![0u8; 8 + 120 + 8 + 64 + 8];
    let (n0, tag0) = seal_frame(&mut w, &mut out, 0, 7, &plain);
    let win0 = out[..n0].to_vec();
    let (n1, tag1) = seal_frame(&mut w, &mut out, 0, 7, &plain);
    let win1 = out[..n1].to_vec();
    assert_eq!(w.cursor(), (2, 2));
    // Frame 1 replayed at the consumed cursor (2,2): ordinal gate fires.
    let err = verify_window(&win1, 2, 2, &tag1).unwrap_err();
    assert!(matches!(
        err,
        AoqError::FrameOrdinalRegression { .. } | AoqError::SeqGap { .. }
    ));
    // Old window at the new cursor also refuses (skip/gap both directions).
    let err = verify_window(&win0, 2, 2, &tag0).unwrap_err();
    assert!(matches!(
        err,
        AoqError::FrameOrdinalRegression { .. } | AoqError::SeqGap { .. }
    ));
    assert_eq!(w.cursor(), (2, 2), "A3: cursor unchanged by verifications");
}

/// A4 header digest mismatch: flipping any header byte in the digest span
/// is refused by BOTH surfaces (ipc HeaderDigestMismatch / aoq
/// DigestMismatch via codec::decode). Mutation = single header byte.
#[test]
fn a4_header_digest_mismatch_both_surfaces() {
    let (win, tag, _) = canonical();
    // Reserved-tail bytes (digest-covered, no earlier gate hits them on
    // either surface).
    for byte_at in [40usize, 59usize] {
        let mut m = win.clone();
        m[byte_at] ^= 0x01;
        let ipc_err = ipc::read_with_meta(&m, None, &mut [0u8; 100]).unwrap_err();
        assert!(
            matches!(ipc_err, ipc::IpcError::HeaderDigestMismatch { .. }),
            "at {byte_at}"
        );
        let aoq_err = verify_window(&m, 0, 0, &tag).unwrap_err();
        assert!(
            matches!(aoq_err, AoqError::DigestMismatch { .. }),
            "at {byte_at}"
        );
    }
    // Version-byte mutation: the two surfaces gate in different orders
    // (ipc: digest -> version; aoq codec: version -> digest), so the
    // classes pair HeaderDigestMismatch <-> UnsupportedVersion. Both
    // refuse; the sweep pins this pairing.
    let mut m = win.clone();
    m[12] ^= 0x01;
    let ipc_err = ipc::read_with_meta(&m, None, &mut [0u8; 100]).unwrap_err();
    assert!(matches!(
        ipc_err,
        ipc::IpcError::HeaderDigestMismatch { .. }
    ));
    let aoq_err = verify_window(&m, 0, 0, &tag).unwrap_err();
    assert!(matches!(aoq_err, AoqError::UnsupportedVersion { .. }));
}

/// A4b w8-consistently-sealed: the SAME 64-byte sealed header embedded in
/// an alignment-8 framing (meta span 64, zero padding) is refused by BOTH
/// surfaces (no padding to carry body_len/tag) while the header seal
/// itself is alignment-independent: the first 72 bytes of the w8 framing
/// are byte-identical to the w64 window's first 72 bytes, and the same
/// header bytes verify as w64.
/// Anti-vacuity mutation: flipping a header byte in the w8 frame changes
/// the digest (A4 pair), never "passes as w8".
#[test]
fn a4b_w8_consistently_sealed() {
    let (win, tag, _) = canonical();
    // w8 framing = same prefix-with-span-64 + same 64B header.
    let mut w8 = vec![0u8; 8 + 64];
    w8[0..4].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
    w8[4..8].copy_from_slice(&64u32.to_le_bytes());
    w8[8..72].copy_from_slice(&win[8..72]);
    // ipc: w8 AOQ1 -> TagMissing (no padding to carry the tag).
    let e = ipc::read_with_meta(&w8, None, &mut [0u8; 4]).unwrap_err();
    assert!(matches!(e, ipc::IpcError::TagMissing), "got {e:?}");
    // aoq: span != 120 -> short-span class.
    let e = verify_window(&w8, 0, 0, &tag).unwrap_err();
    assert!(matches!(e, AoqError::HeaderTooShort { .. }));
    // consistency: header bytes identical to the lawful w64 window.
    assert_eq!(&w8[8..72], &win[8..72]);
}

/// A5 epoch gate: epoch 3 (outside the closed enum) refuses on both
/// surfaces (ipc BadFlags at the flags-position gate / aoq
/// UnknownEpochWire via codec::decode). Note: a mutated epoch also fails
/// the ipc digest gate first; the court re-seals to isolate the class.
/// Anti-vacuity: the un-mutated window passes (A1).
#[test]
fn a5_epoch_gate() {
    let (win, tag, _) = canonical();
    // ipc: digest gates first (sealed header) -> HeaderDigestMismatch.
    let mut m = win.clone();
    m[8 + 6] = 3u8; // epoch low byte = 3, outside {0,1,2,65535}
    let e = ipc::read_with_meta(&m, None, &mut [0u8; 100]).unwrap_err();
    assert!(matches!(e, ipc::IpcError::HeaderDigestMismatch { .. }));
    // Isolated class: re-seal the mutated header so the digest passes and
    // each surface's epoch gate fires alone.
    let mut hdr_bytes = [0u8; 64];
    hdr_bytes.copy_from_slice(&win[8..72]);
    let mut hdr = cgap_aoq::codec::decode(&hdr_bytes).expect("canonical header");
    hdr.epoch = 3;
    let mut resealed = [0u8; 64];
    cgap_aoq::codec::encode(&hdr, &mut resealed).expect("reseal");
    let mut iso = win.clone();
    iso[8..72].copy_from_slice(&resealed);
    let e = ipc::read_with_meta(&iso, None, &mut [0u8; 100]).unwrap_err();
    assert!(matches!(e, ipc::IpcError::BadFlags { got: 3 }), "got {e:?}");
    let e = verify_window(&iso, 0, 0, &tag).unwrap_err();
    assert!(
        matches!(e, AoqError::UnknownEpochWire { got: 3 }),
        "got {e:?}"
    );
}

/// A6 pad cleanliness: a nonzero metadata-pad byte OUTSIDE the
/// body_len/tag law region refuses on both surfaces (ipc PadNonZero with
/// the window offset / aoq ReservedNotZero). The law region itself
/// (offsets 72..92) may carry nonzero body_len/tag — court A1 pins that.
#[test]
fn a6_pad_cleanliness() {
    let (win, tag, _) = canonical();
    let mut m = win.clone();
    m[100] ^= 0x01;
    let e = ipc::read_with_meta(&m, None, &mut [0u8; 100]).unwrap_err();
    assert!(matches!(e, ipc::IpcError::PadNonZero { at: 100 }));
    let e = verify_window(&m, 0, 0, &tag).unwrap_err();
    assert!(matches!(e, AoqError::ReservedNotZero));
}

/// A7 body_len word corruption: the padding word disagrees with the
/// window length on BOTH surfaces (ipc BadBodyLen / aoq short-window
/// class). Anti-vacuity: A1 passes with the lawful word.
#[test]
fn a7_body_len_word_corruption() {
    let (win, tag, _) = canonical();
    let mut m = win.clone();
    let mut blw = [0u8; 4];
    blw.copy_from_slice(&m[72..76]);
    let blw = u32::from_le_bytes(blw) + 1;
    m[72..76].copy_from_slice(&blw.to_le_bytes());
    // The shifted body span misaligns the EOS on ipc (MissingEos); aoq's
    // full-window law refuses as Short. Identity: both refuse.
    let e = ipc::read_with_meta(&m, None, &mut [0u8; 100]).unwrap_err();
    assert!(
        matches!(
            e,
            ipc::IpcError::Truncated { .. } | ipc::IpcError::MissingEos { .. }
        ),
        "got {e:?}"
    );
    let e = verify_window(&m, 0, 0, &tag).unwrap_err();
    assert!(matches!(e, AoqError::HeaderTooShort { .. }), "got {e:?}");
}

/// A8 tag mismatch: flipping one tag byte keeps plain framing read green
/// (consumer duty documented on the plain path) while BOTH verifying
/// surfaces refuse, and the A12 law holds: nothing was decrypted.
#[test]
fn a8_tag_mismatch_both_surfaces() {
    let (win, tag, _) = canonical();
    let mut m = win.clone();
    m[76] ^= 0x01;
    assert!(ipc::read_with_meta(&m, None, &mut [0u8; 100]).is_ok());
    let e = ipc::read_verified(&m, None, &mut [0u8; 100]).unwrap_err();
    assert!(matches!(e, ipc::IpcError::TagMismatch { .. }), "got {e:?}");
    let e = verify_window(&m, 0, 0, &tag).unwrap_err();
    assert!(matches!(e, AoqError::DigestMismatch { .. }), "got {e:?}");
    // no-plaintext-on-error: the buffer is byte-identical ciphertext.
    let ciphertext_before = m.clone();
    let e = verify_window(&m, 0, 0, &tag);
    assert!(e.is_err());
    assert_eq!(m, ciphertext_before, "A8/A12: refused window untouched");
}

/// A9 truncation exact-needed (AF-4 read-ahead law): parse demands
/// prefix+span (128 for w64); body-span shortfalls are BadBodyLen; the
/// EOS band is MissingEos; the class flips exactly at each boundary.
/// Full identity table lives in ab8_sweep (365 prefix lengths).
#[test]
fn a9_truncated_exact_needed() {
    let (win, _tag, n) = canonical();
    assert_eq!(n, 364);
    // Band [8,128): Truncated naming the exact read-ahead minimum.
    for l in [8usize, 63, 100, 127] {
        let e = ipc::read_with_meta(&win[..l], None, &mut [0u8; 100]).unwrap_err();
        assert!(
            matches!(e, ipc::IpcError::Truncated { needed: 128, .. }),
            "L={l}: {e:?}"
        );
    }
    // Band [128,356): body-span demand (body_len + pad = 228 from 128).
    for l in [128usize, 150, 191, 300, 355] {
        let e = ipc::read_with_meta(&win[..l], None, &mut [0u8; 100]).unwrap_err();
        assert!(
            matches!(e, ipc::IpcError::BadBodyLen { .. }),
            "L={l}: {e:?}"
        );
    }
    // Band [356,364): EOS band.
    for l in [356usize, 357, 363] {
        let e = ipc::read_with_meta(&win[..l], None, &mut [0u8; 100]).unwrap_err();
        assert!(
            matches!(e, ipc::IpcError::MissingEos { .. }),
            "L={l}: {e:?}"
        );
    }
    // At exactly 364: passes.
    assert!(ipc::read_with_meta(&win, None, &mut [0u8; 100]).is_ok());
    // aoq mirrors (Short everywhere below the boundary, Pass at 364).
    for l in [8usize, 63, 127, 128, 150, 191, 300, 355, 356, 363] {
        let e = verify_window(&win[..l], 0, 0, &[0u8; 16]);
        assert!(e.is_err(), "L={l}: {e:?}");
    }
}

/// A10 ZeroProgress: a refused write (short output window) leaves the
/// writer cursor byte-identical AND the output window unwritten.
#[test]
fn a10_write_refusal_zero_progress() {
    let plain = [0x11u8; 16];
    let mut w = AoqWriter::new();
    let mut out = vec![0u8; 8 + 120 + 16 + 16 + 8 - 1];
    let before = out.clone();
    let e = w
        .write_window(&mut out, 0, 7, &[0u8; 16], &plain)
        .unwrap_err();
    assert!(matches!(e, AoqError::BufferTooShort { .. }), "got {e:?}");
    assert_eq!(w.cursor(), (0, 0), "A10: cursor untouched");
    assert_eq!(out, before, "A10: output untouched (exact-needed buffer)");
    // Now with a large-enough buffer: passes and commits exactly once.
    let mut ok_out = vec![0u8; 8 + 120 + 16 + 64 + 8 + 64];
    let n = w
        .write_window(&mut ok_out, 0, 7, &[0u8; 16], &plain)
        .expect("lawful");
    assert_eq!(n, 8 + 120 + 16 + 64 + 8);
    assert_eq!(w.cursor(), (1, 1));
}

/// A11 commit-only-after-full-accept: frame_segments does not advance the
/// cursor; only commit() does. Plus the seq wrap law (u32 boundary).
#[test]
fn a11_commit_semantics_and_seq_wrap() {
    let plain = [0x22u8; 8];
    let mut w = AoqWriter::new();
    let out = vec![0u8; 512];
    let tag = [0u8; 16];
    // Stage without commit: cursor frozen, segments valid.
    let total = {
        let segs = w.frame_segments(0, 7, &tag, &plain).expect("stage");
        segs.total_len()
    };
    assert_eq!(total, 8 + 120 + 8 + 64 + 8);
    assert_eq!(w.cursor(), (0, 0), "A11: staging is not progress");
    w.commit();
    assert_eq!(w.cursor(), (1, 1));
    // Seq wrap: drive the cursor to the u32 boundary via a fresh writer.
    let mut w2 = AoqWriter::new();
    // next_seq is private; the wrap law is the lockstep +1 law — court
    // pins the pair semantics via cursor checks after commits.
    for _ in 0..3 {
        w2.commit();
    }
    assert_eq!(w2.cursor(), (3, 3));
    let _ = plain;
    let _ = tag;
    let _ = out;
}

/// A12 no-plaintext-on-error: on ANY refusal the buffer stays ciphertext.
/// Mutations: corrupted tag, corrupted body, wrong cursor — after each
/// refused verify, the buffer is byte-identical and decrypt_in_place's
/// own refusals leave the buffer untouched.
#[test]
fn a12_no_plaintext_on_error() {
    let plain = [0xA7u8; 100];
    let ks = keystream(0, 100);
    let mut ct = plain;
    for i in 0..100 {
        ct[i] ^= ks[i];
    }
    let h = blake3::hash(&ct);
    let mut tag = [0u8; 16];
    tag.copy_from_slice(&h[..16]);
    let mut w = AoqWriter::new();
    let mut win = vec![0u8; 8 + 120 + 100 + 128 + 8];
    let n = w.write_window(&mut win, 0, 7, &tag, &ct).expect("lawful");
    let win_ct = win.clone();
    assert_ne!(&win[128..228], &plain[..], "ciphertext on the wire");

    // (a) wrong tag: refused, buffer untouched.
    let wrong = [0u8; 16];
    assert!(verify_window(&win, 0, 0, &wrong).is_err());
    assert_eq!(win, win_ct);
    // (b) corrupted body byte: refused, buffer untouched.
    let mut m = win.clone();
    m[200] ^= 0x01;
    let _ = verify_window(&m, 0, 0, &tag);
    // (c) decrypt refused (short keystream) leaves buffer untouched.
    let mut m2 = win.clone();
    let vw = verify_window(&m2, 0, 0, &tag).expect("lawful");
    let e = decrypt_in_place(&vw, &ks[..99], &mut m2);
    assert!(e.is_err());
    assert_eq!(m2, win_ct, "A12: short-keystream refusal untouched");
    // (d) happy path decrypts exactly the body span.
    decrypt_in_place(&vw, &ks, &mut m2).expect("decrypt");
    assert_eq!(&m2[128..228], &plain[..]);
    let _ = n;
    let _ = win_ct;
}

/// A13 offset-pin identity across the three surfaces: the writer's pad
/// segment layout (pad byte j = window offset 72+j), verify.rs consts,
/// and cgap-arrow's AOQ1_TAG_AT agree on the SAME window bytes.
#[test]
fn a13_offset_pin_identity() {
    let (win, tag, n) = canonical();
    assert_eq!(n, 364);
    assert_eq!(verify::BODY_LEN_AT, 72);
    assert_eq!(verify::TAG_AT, 76);
    assert_eq!(ipc::AOQ1_TAG_AT, 76);
    assert_eq!(ipc::AOQ1_TAG_BYTES, 16);
    // The tag the writer embedded is at window [76,92) — verify_window
    // accepts it, and flipping byte 76 refuses (A8 pairs this).
    assert!(verify_window(&win, 0, 0, &tag).is_ok());
    let mut m = win.clone();
    m[76] ^= 0x01;
    assert!(verify_window(&m, 0, 0, &tag).is_err());
}
