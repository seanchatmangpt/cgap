//! Body-integrity contract courts for `crates/cgap-arrow/src/ipc.rs`
//! (implementation lane e80-07, v26.11.3; CL-L finding #1 / witness W4).
//!
//! Contract under test: legacy [`emit`]/[`read`] leave body integrity as
//! the consumer's duty (W4 pins that a body-corrupted window passes a
//! digest-bearing `read`); [`emit_verified`]/[`read_verified`] raise body
//! integrity into the framing layer via a blake3 digest carried in the
//! alignment-64 metadata padding, which `read` never inspects.

use cgap_arrow::blake3;
use cgap_arrow::ipc::{
    emit, emit_verified, meta_span_for, read, read_verified, IpcError, MetaHeader, BODY_DIGEST_AT,
};

const ALIGN64: u32 = 64;

fn meta(body_len: u32) -> MetaHeader {
    MetaHeader {
        magic: cgap_arrow::ipc::MAGIC,
        version: cgap_arrow::ipc::LAYOUT_VERSION,
        flags: 0,
        schema_digest: [7u8; 32],
        body_len,
        alignment: ALIGN64,
        seq: 1,
        reserved0: 0,
        header_digest: 0,
        pad_tail: [0; 4],
    }
}

fn clean_verified_window(body: &[u8]) -> Vec<u8> {
    let m = meta(body.len() as u32);
    let mut w = vec![0u8; cgap_arrow::ipc::window_len(body.len(), 64)];
    let n = emit_verified(&mut w, &m, body).expect("emit_verified ok");
    assert_eq!(n, w.len());
    w
}

/// Clean verified window: green through BOTH read paths.
#[test]
fn t1_clean_verified_window_reads_green_through_both_paths() {
    let body = [0xA5u8; 200];
    let w = clean_verified_window(&body);
    let mut out = [0u8; 200];
    let n = read(&w, None, &mut out).expect("legacy read accepts the verified window");
    assert_eq!(n, 200);
    assert_eq!(&out[..n], &body, "legacy read returns the exact body");
    let mut out2 = [0u8; 200];
    let n = read_verified(&w, Some(&[7u8; 32]), &mut out2).expect("verified read green");
    assert_eq!(n, 200);
    assert_eq!(&out2[..n], &body, "verified read returns the exact body");
    // The digest really is where the contract pins it, and the legacy
    // read's framing checks never covered it.
    assert_ne!(
        &w[BODY_DIGEST_AT..BODY_DIGEST_AT + 32],
        &[0u8; 32][..],
        "digest slot is populated"
    );
}

/// Body-corrupted verified window: legacy read passes (documented
/// consumer duty — the W4 law), verified read refuses typed.
#[test]
fn t2_corrupted_body_legacy_passes_verified_refuses() {
    let body = [0x5Au8; 200];
    let mut w = clean_verified_window(&body);
    let body_at = 8 + meta_span_for(64);
    let saved = w[body_at];
    w[body_at] ^= 0xFF;
    assert_ne!(w[body_at], saved, "vacuous court: mutation was a no-op");

    let mut out = [0u8; 200];
    let n = read(&w, Some(&[7u8; 32]), &mut out).expect("legacy read: consumer-duty contract");
    assert_eq!(n, 200);
    assert_ne!(&out[..n], &body, "the corrupted byte reached the caller");

    let err = read_verified(&w, Some(&[7u8; 32]), &mut [0u8; 200]).unwrap_err();
    match err {
        IpcError::BodyDigestMismatch { got } => {
            assert_eq!(
                got,
                blake3::hash(&body),
                "stored digest is the pristine-body digest"
            );
        }
        other => panic!("expected BodyDigestMismatch, got {other:?}"),
    }
}

/// A window from plain `emit` (no digest slot populated) refuses the
/// verified read typed, and the legacy read stays green on it.
#[test]
fn t3_legacy_window_body_digest_missing() {
    let body = [0x01u8; 64];
    let m = meta(body.len() as u32);
    let mut w = vec![0u8; cgap_arrow::ipc::window_len(body.len(), 64)];
    let n = emit(&mut w, &m, &body).expect("emit ok");
    assert_eq!(n, w.len());

    let mut out = [0u8; 64];
    let n = read(&w, None, &mut out).expect("legacy read green on legacy window");
    assert_eq!(n, 64);
    assert_eq!(out[..n], body);

    let err = read_verified(&w, None, &mut [0u8; 64]).unwrap_err();
    assert_eq!(err, IpcError::BodyDigestMissing, "typed absence refusal");
}

/// Verified emit at alignment 8 refuses: 32 digest bytes do not fit in
/// zero metadata padding.
#[test]
fn t4_verified_emit_align8_no_room() {
    let body = [0x02u8; 32];
    let mut m = meta(body.len() as u32);
    m.alignment = 8;
    let mut w = vec![0u8; cgap_arrow::ipc::window_len(body.len(), 8)];
    let err = emit_verified(&mut w, &m, &body).unwrap_err();
    assert_eq!(err, IpcError::BodyDigestNoRoom { got: 8 });

    // And an align-8 legacy window refuses the verified read on the same
    // typed ground (no padding to carry a digest).
    let n = emit(&mut w, &m, &body).expect("legacy align-8 emit ok");
    assert_eq!(n, w.len());
    let err = read_verified(&w, None, &mut [0u8; 32]).unwrap_err();
    assert_eq!(err, IpcError::BodyDigestNoRoom { got: 8 });
}

/// Non-vacuity guard: tampering with the DIGEST bytes (not the body)
/// also flips the verified read, proving the check reads the stored slot
/// rather than recomputing unconditionally.
#[test]
fn t5_digest_slot_tamper_flips() {
    let body = [0x11u8; 100];
    let mut w = clean_verified_window(&body);
    let saved = w[BODY_DIGEST_AT];
    w[BODY_DIGEST_AT] ^= 0x01;
    assert_ne!(
        w[BODY_DIGEST_AT], saved,
        "vacuous court: mutation was a no-op"
    );
    let err = read_verified(&w, None, &mut [0u8; 100]).unwrap_err();
    match err {
        IpcError::BodyDigestMismatch { got } => {
            assert_ne!(
                got,
                blake3::hash(&body),
                "got is the tampered stored digest"
            );
        }
        other => panic!("expected BodyDigestMismatch, got {other:?}"),
    }
}
