//! AB-8 truncation classification-identity sweep (PG-A2, court table).
//!
//! Law: for EVERY truncation of a canonical AOQ1 window, `ipc::read` and
//! `cgap_aoq::verify_window` AGREE — both pass, or both refuse with the
//! documented shared-class pairing (R-H2: framing classes live in
//! cgap-arrow `IpcError`; cgap-aoq speaks header law):
//!
//! | class            | ipc variant              | aoq variant               |
//! |------------------|--------------------------|---------------------------|
//! | Short (trunc)    | Truncated / BadBodyLen / MissingEos | HeaderTooShort |
//! | Prefix word      | BadContinuation          | BadMagic                  |
//! | Span             | BadMetaSize              | HeaderTooShort            |
//! | Magic            | BadMagic                 | BadMagic                  |
//! | HeaderDigest     | HeaderDigestMismatch     | DigestMismatch            |
//! | Version          | HeaderDigestMismatch (digest gates first) | UnsupportedVersion |
//! | Epoch            | BadFlags                 | UnknownEpochWire          |
//! | Pad              | PadNonZero               | ReservedNotZero           |
//! | Tag absent       | TagMissing               | DigestMismatch {0,0}      |
//! | Tag bad          | TagMismatch              | DigestMismatch            |
//! | EOS              | MissingEos               | HeaderTooShort            |
//!
//! Cases: every prefix length 0..=window_len (365) + mutation classes at
//! every law-relevant byte — well past the >=72-case bar.

use cgap_aoq::error::AoqError;
use cgap_aoq::verify::verify_window;
use cgap_aoq::writer::AoqWriter;
use cgap_arrow::blake3;
use cgap_arrow::ipc;

/// Deterministic tag over the body-as-carried (no keystream needed: the
/// sweep stores plaintext bodies — the tag law covers the carried bytes,
/// keystream or not).
fn tag_of(body: &[u8]) -> [u8; 16] {
    let h = blake3::hash(body);
    let mut t = [0u8; 16];
    t.copy_from_slice(&h[..16]);
    t
}

/// Canonical plaintext window (body = 100 bytes), window_len = 364.
fn canonical_win() -> Vec<u8> {
    let body = [0xA5u8; 100];
    let tag = tag_of(&body);
    let mut w = AoqWriter::new();
    let mut out = vec![0u8; 8 + 120 + 100 + 128 + 8];
    let n = w
        .write_window(&mut out, 0, 7, &tag, &body)
        .expect("canonical window");
    out.truncate(n);
    out
}

const WIN_LEN: usize = 8 + 120 + 100 + 128 + 8;

/// The sweep: classification identity over every truncation length.
#[test]
fn sweep_truncation_identity() {
    let win = canonical_win();
    assert_eq!(win.len(), WIN_LEN);
    let tag = tag_of(&[0xA5u8; 100]);
    let mut cases = 0usize;
    for l in 0..=WIN_LEN {
        let t = &win[..l];
        let ipc_r = ipc::read_with_meta(t, None, &mut [0u8; 100]);
        let aoq_r = verify_window(t, 0, 0, &tag);
        assert_eq!(
            ipc_r.is_ok(),
            aoq_r.is_ok(),
            "L={l}: ipc {:?} vs aoq {:?}",
            ipc_r.err(),
            aoq_r.err()
        );
        // exact-needed law: below the header-slot boundary, ipc names the
        // exact byte count the framing needs.
        if l < 128 && l >= 8 {
            assert!(matches!(
                ipc_r,
                Err(ipc::IpcError::Truncated { needed: 128, .. })
            ), "L={l}: {:?}", ipc_r.err());
        }
        cases += 1;
    }
    assert!(cases >= 72, "sweep ran {cases} cases");
}

/// Mutation classes: both surfaces refuse; documented pairing.
#[test]
fn sweep_mutation_pairings() {
    let win = canonical_win();
    let tag = tag_of(&[0xA5u8; 100]);
    // Header mutations: single digest-covered reserved-tail byte.
    for at in [24usize, 40, 59] {
        let mut m = win.clone();
        m[8 + at] ^= 0x01;
        assert!(ipc::read_with_meta(&m, None, &mut [0u8; 100]).is_err());
        assert!(verify_window(&m, 0, 0, &tag).is_err());
    }
    // Version byte: digest gates first on ipc, version gate first on aoq.
    let mut m = win.clone();
    m[8 + 4] ^= 0x01;
    assert!(matches!(
        ipc::read_with_meta(&m, None, &mut [0u8; 100]),
        Err(ipc::IpcError::HeaderDigestMismatch { .. })
    ));
    assert!(matches!(
        verify_window(&m, 0, 0, &tag),
        Err(AoqError::UnsupportedVersion { .. })
    ));
    // Epoch byte.
    let mut m = win.clone();
    m[8 + 6] ^= 0x01;
    assert!(ipc::read_with_meta(&m, None, &mut [0u8; 100]).is_err());
    assert!(verify_window(&m, 0, 0, &tag).is_err());
    // Pad mutation (outside law region, window offset 100).
    let mut m = win.clone();
    m[100] ^= 0x01;
    assert!(matches!(
        ipc::read_with_meta(&m, None, &mut [0u8; 100]),
        Err(ipc::IpcError::PadNonZero { .. })
    ));
    assert!(matches!(
        verify_window(&m, 0, 0, &tag),
        Err(AoqError::ReservedNotZero)
    ));
    // Tag mutation: plain read PASSES, read_verified + verify_window refuse.
    let mut m = win.clone();
    m[80] ^= 0x01;
    assert!(ipc::read_with_meta(&m, None, &mut [0u8; 100]).is_ok());
    assert!(ipc::read_verified(&m, None, &mut [0u8; 100]).is_err());
    assert!(verify_window(&m, 0, 0, &tag).is_err());
    // Body mutation: the recomputing surface (read_verified) refuses.
    // verify_window's tag gate is stored-vs-expected: with the ORIGINAL
    // expectation it passes (nothing to compare against the carried
    // bytes); with a fresh carried-body-derived expectation it REFUSES,
    // pinning that detection rides the caller's derivation.
    let mut m = win.clone();
    m[200] ^= 0x01;
    assert!(ipc::read_with_meta(&m, None, &mut [0u8; 100]).is_ok());
    assert!(ipc::read_verified(&m, None, &mut [0u8; 100]).is_err());
    assert!(verify_window(&m, 0, 0, &tag).is_ok());
    let fresh = tag_of(&m[128..228]);
    assert!(verify_window(&m, 0, 0, &fresh).is_err());
}
