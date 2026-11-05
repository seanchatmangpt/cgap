//! L56 delegation court for the cgap-aoq PQC facade
//! (`crates/cgap-aoq/src/pqc_seal.rs`).
//!
//! Chicago court over real collaborators: real ML-DSA-65 keygen + real
//! AEAD via the cgap-crypto kernel — no mocks. Leg 1 proves the facade
//! is pure delegation (seal/open via the facade is byte-identical to the
//! direct kernel pair); Leg 2 is the negative leg: a bad signature via
//! the facade is a typed [`AoqError::PqcSealRefused`] with class
//! [`PqcSealClass::BadSignature`], and the plaintext buffer is never
//! materialized (sentinel-fill assert, mirroring the kernel's
//! never-written-on-failure law). Anti-vacuity is structural: a tampered
//! leg that passed would fail the court.

#![cfg(feature = "pqc")]

use cgap_aoq::error::AoqError;
use cgap_aoq::pqc_seal as facade;
use cgap_crypto::aead::Nonce96;
use cgap_crypto::mldsa::{PK_BYTES, SK_BYTES};
use cgap_crypto::pqc_seal::CT_OVERHEAD;

const KEY_XI: [u8; 32] = [0x56; 32]; // L56 court xi
const AAD: &[u8] = b"l56/facade/aad";
const KEY: [u8; 32] = [0x17; 32];
const RND: [u8; 32] = [0x42; 32];
const PLAINTEXT: &[u8] = b"cgap l56 pqc facade delegation court plaintext";

#[test]
fn facade_delegation_roundtrip_is_byte_identical_to_direct_kernel() {
    let mut ek = [0u8; PK_BYTES];
    let mut dk = [0u8; SK_BYTES];
    cgap_crypto::mldsa::keygen(&KEY_XI, &mut ek, &mut dk);

    let nonce = Nonce96::new(1, 0x0102_0304_0506_0708).expect("nonce");

    // Leg 1a: facade seal vs direct kernel seal, byte-identical
    // (deterministic inputs: fixed rnd, fixed nonce).
    let via_facade =
        facade::seal(&dk, &ek, PLAINTEXT, AAD, &KEY, &nonce, &RND).expect("facade seal");
    let direct = cgap_crypto::pqc_seal::pqc_seal(&dk, &ek, PLAINTEXT, AAD, &KEY, &nonce, &RND)
        .expect("kernel seal");
    assert_eq!(via_facade, direct, "facade seal must be byte-identical to direct kernel");
    assert_eq!(via_facade.len(), facade::sealed_len(PLAINTEXT.len()));

    // Leg 1b: facade open of the facade seal.
    let mut pt_facade = vec![0u8; via_facade.len() - CT_OVERHEAD];
    facade::open(&ek, &mut pt_facade, &via_facade, AAD, &KEY, &nonce).expect("facade open");
    assert_eq!(pt_facade, PLAINTEXT);

    // Leg 1c: direct kernel open of the facade seal — the buffers are
    // interchangeable, proving zero crypto-logic divergence.
    let mut pt_direct = vec![0u8; via_facade.len() - CT_OVERHEAD];
    cgap_crypto::pqc_seal::pqc_open(&ek, &mut pt_direct, &via_facade, AAD, &KEY, &nonce)
        .expect("kernel open of facade seal");
    assert_eq!(pt_direct, PLAINTEXT);
}

#[test]
fn bad_signature_via_facade_is_typed_refusal_and_plaintext_never_materialized() {
    let mut ek = [0u8; PK_BYTES];
    let mut dk = [0u8; SK_BYTES];
    cgap_crypto::mldsa::keygen(&KEY_XI, &mut ek, &mut dk);

    let nonce = Nonce96::new(2, 0x0A0B_0C0D_0E0F_1011).expect("nonce");
    let sealed = facade::seal(&dk, &ek, PLAINTEXT, AAD, &KEY, &nonce, &RND).expect("facade seal");

    // Tamper INSIDE the signature span — the FIRST sig byte (a
    // data-bearing packed byte). Probed: the last two sig bytes of the
    // ML-DSA-65 packed encoding are padding (flips there decode to the
    // identical signature and verify lawfully passes — NOT a kernel bug;
    // probed against cgap-crypto 92fa5f0b with sig offsets 1..3308 all
    // refusing typed). If this leg ever passes, the court fails.
    let mut tampered = sealed.clone();
    let first_sig = tampered.len() - facade::SIG_BYTES;
    tampered[first_sig] ^= 0x01;

    let mut pt = [0xA5u8; PLAINTEXT.len()];
    assert_eq!(pt.len(), tampered.len() - CT_OVERHEAD);

    match facade::open(&ek, &mut pt, &tampered, AAD, &KEY, &nonce) {
        Err(AoqError::PqcSealRefused { class }) => {
            assert_eq!(class, facade::PqcSealClass::BadSignature, "typed refusal class");
            assert_eq!(class.class_code(), 0x50_03);
        }
        Err(other) => panic!("wrong typed refusal: {other:?}"),
        Ok(()) => panic!("VACUOUS: tampered signature accepted by the facade"),
    }

    // Plaintext never materialized: every byte still the sentinel.
    assert!(
        pt.iter().all(|&b| b == 0xA5),
        "plaintext buffer was written on a refusal path"
    );

    // Second negative leg: short sealed buffer -> MalformedSealed.
    let mut pt2 = [0xA5u8; 4];
    let short = &sealed[..facade::CT_OVERHEAD - 1];
    match facade::open(&ek, &mut pt2, short, AAD, &KEY, &nonce) {
        Err(AoqError::PqcSealRefused { class }) => {
            assert_eq!(class, facade::PqcSealClass::MalformedSealed);
        }
        _ => panic!("short sealed buffer must refuse typed"),
    }
    assert!(pt2.iter().all(|&b| b == 0xA5));
}
