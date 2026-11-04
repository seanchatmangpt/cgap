//! AOQ1 codec court: real roundtrips, digest tamper mutations, typed
//! refusals — Chicago style, assert final state, every mutation named.

use cgap_aoq::codec::{decode, encode, fnv1a32};
use cgap_aoq::error::AoqError;
use cgap_aoq::header::{AoqHeader, Epoch};

/// Canonical genesis header for the courts.
fn genesis() -> AoqHeader {
    AoqHeader {
        magic: cgap_aoq::header::MAGIC,
        version: cgap_aoq::header::VERSION,
        epoch: Epoch::Genesis as u16,
        key_id: 0x0102_0304_0506_0708,
        frame_ordinal: 7,
        seq: 42,
        reserved: [0u8; 36],
        header_digest: 0,
    }
}

#[test]
fn roundtrip_genesis_is_byte_stable() {
    let mut h = genesis();
    let mut wire = [0u8; 64];
    encode(&h, &mut wire).expect("encode genesis");
    let back = decode(&wire).expect("decode genesis");
    // Decode returns the STORED digest; pin the expectation to the codec's
    // own digest law, then assert the digest separately below.
    h.header_digest = fnv1a32(&wire[0..60]);
    assert_eq!(back, h, "roundtrip must preserve the header");
    // Digest law: bytes [0,0x3C) digest to the stored digest at [60..64).
    assert_eq!(
        back.header_digest,
        fnv1a32(&wire[0..60]),
        "stored digest must equal FNV-1a32 over the covered span"
    );
}

#[test]
fn digest_tamper_flips_decode() {
    let h = genesis();
    let mut wire = [0u8; 64];
    encode(&h, &mut wire).expect("encode");
    let orig = wire;
    wire[10] ^= 0xFF; // mutate a digest-covered key_id byte
    let err = decode(&wire).expect_err("tampered envelope must refuse");
    assert!(
        matches!(err, AoqError::DigestMismatch { .. }),
        "key_id tamper must be a DigestMismatch, got {err:?}"
    );
    // Reserved tail is digest-covered too (zero-tail discipline).
    let mut wire2 = orig;
    wire2[30] = 0x5A;
    let err2 = decode(&wire2).expect_err("reserved tamper must refuse");
    assert!(
        matches!(err2, AoqError::DigestMismatch { .. }),
        "reserved tamper must be a DigestMismatch, got {err2:?}"
    );
}

#[test]
fn wire_law_refusals_are_typed() {
    let h = genesis();
    let mut wire = [0u8; 64];
    encode(&h, &mut wire).expect("encode");

    // Magic flip (anti-vacuity mutation named: wire[0..4] magic bytes).
    let mut m = wire;
    m[0] = 0x00;
    assert!(matches!(
        decode(&m),
        Err(AoqError::BadMagic { .. })
    ));

    // Version flip (wire[4..6]).
    let mut v = wire;
    v[4] = 9;
    assert!(matches!(
        decode(&v),
        Err(AoqError::UnsupportedVersion { got: 9 })
    ));

    // Unknown epoch flip (wire[6..8]) — outside the closed enum.
    let mut e = wire;
    e[6] = 0x34;
    e[7] = 0x12;
    assert!(matches!(
        decode(&e),
        Err(AoqError::UnknownEpochWire { got: 0x1234 })
    ));

    // Truncated envelope (short input).
    let t = &wire[..63];
    assert!(matches!(
        decode(t),
        Err(AoqError::HeaderTooShort { got: 63 })
    ));

    // Short output buffer at encode.
    let mut small = [0u8; 63];
    assert!(matches!(
        encode(&h, &mut small),
        Err(AoqError::HeaderTooLong { got: 63 })
    ));

    // Digest byte flip (wire[60..64], not covered — detectable only as a
    // stored-vs-computed mismatch).
    let mut d = wire;
    d[61] ^= 0x01;
    assert!(matches!(
        decode(&d),
        Err(AoqError::DigestMismatch { .. })
    ));
}

#[test]
fn retired_epoch_decodes_but_its_digest_law_holds() {
    // R-1 law: Retired is a read-only sentinel — it admits on decode.
    let h = genesis();
    let mut wire = [0u8; 64];
    encode(&h, &mut wire).expect("encode");
    wire[6..8].copy_from_slice(&(Epoch::Retired as u16).to_le_bytes());
    // Recompute the digest over the mutated span so ONLY the epoch flip
    // is under test (isolate the mutation).
    let digest = fnv1a32(&wire[0..60]);
    wire[60..64].copy_from_slice(&digest.to_le_bytes());
    let back = decode(&wire).expect("Retired must decode (read-only sentinel)");
    assert_eq!(back.epoch, Epoch::Retired as u16);
}

#[test]
fn error_class_codes_are_stable() {
    // Wire/log-safe class codes: 11 variants, 0x01..0x0B.
    let codes = [
        AoqError::ReservedNotZero.class_code(),
        AoqError::KemKernelNotAdmitted { kernel_id: 1 }.class_code(),
        AoqError::AeadKernelNotAdmitted { kernel_id: 1 }.class_code(),
    ];
    assert_eq!(codes, [0x00_07, 0x00_0A, 0x00_0B]);
}

#[test]
fn kem_aead_seams_refuse_loudly() {
    // R-K1 law: cgap-aoq hosts seams only; every call is a typed refusal.
    assert!(matches!(
        cgap_aoq::kem::encapsulate(0xBEEF, &[0u8; 32]),
        Err(AoqError::KemKernelNotAdmitted { kernel_id: 0xBEEF })
    ));
    assert!(matches!(
        cgap_aoq::kem::decapsulate(0xBEEF, &[0u8; 48]),
        Err(AoqError::KemKernelNotAdmitted { kernel_id: 0xBEEF })
    ));
    assert!(matches!(
        cgap_aoq::aead::seal(0xCAFE, &[0u8; 32], b"pt"),
        Err(AoqError::AeadKernelNotAdmitted { kernel_id: 0xCAFE })
    ));
    assert!(matches!(
        cgap_aoq::aead::open(0xCAFE, &[0u8; 32], &[0u8; 16]),
        Err(AoqError::AeadKernelNotAdmitted { kernel_id: 0xCAFE })
    ));
}

#[test]
fn fnv1a32_known_vectors() {
    // Standard FNV-1 32-bit vectors (fnv1a32("")=2166136261, "a"=0xE40C292C).
    let empty: [u8; 0] = [];
    assert_eq!(fnv1a32(&empty), 0x811C_9DC5);
    assert_eq!(fnv1a32(b"a"), 0xE40C_292C);
    assert_eq!(fnv1a32(b"foobar"), 0xBF9C_F968); // python cross-checked
}
