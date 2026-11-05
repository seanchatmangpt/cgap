//! PQC composite-seal host-facade (L56, consumer-lane item deferred by
//! L14's receipt): typed delegation to [`cgap_crypto::pqc_seal`] — the
//! crypto body stays in cgap-crypto (ruling R-K1); this module re-frames
//! only the kernel's typed refusals onto the crate's error table.
//!
//! Wire layout and laws (unchanged, cited not restated): see
//! `cgap-crypto/src/pqc_seal.rs` — `[ ct ][ tag 16B ][ sig 3309B ]`,
//! verify-then-decrypt, plaintext never written on any failure path.
//!
//! # Alloc gating
//!
//! `pqc_seal` returns an owned buffer, so this surface is alloc-using and
//! rides the `pqc` feature (same gating precedent as the alloc-using
//! `cert` module: `rfc0089_certificate`). The default embedded surface
//! stays no-alloc / no_std.
//!
//! # Errors
//!
//! Every kernel refusal maps 1:1 onto [`AoqError::PqcSealRefused`] with
//! the [`PqcSealClass`] code carried as an integer field — `Copy`, no
//! heap, no strings, per the [`AoqError`] integer-field law.

use alloc::vec::Vec;

use crate::error::AoqError;
use cgap_crypto::aead::Nonce96;
use cgap_crypto::pqc_seal::PqcSealError;
use cgap_crypto::mldsa::{PK_BYTES, SK_BYTES};
pub use cgap_crypto::pqc_seal::{sealed_len, CT_OVERHEAD, PQC_SEAL_DOMAIN, SIG_BYTES, TAG_LEN};

/// Kernel refusal classes, carried as an integer field on
/// [`AoqError::PqcSealRefused`] (`Copy`, wire/log-safe numeric codes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PqcSealClass {
    /// Sealed buffer shorter than `CT_OVERHEAD` / plaintext-length
    /// mismatch at open parse.
    MalformedSealed,
    /// `dk`/`ek` pair mismatch refused at seal time.
    KeyPairMismatch,
    /// ML-DSA-65 verification failed.
    BadSignature,
    /// AEAD tag mismatch after a passing signature.
    AeadMismatch,
    /// ML-DSA-65 signing exceeded its FS-abort budget.
    SignAbortLimit,
}

impl PqcSealClass {
    /// Stable numeric class code (wire/log-safe).
    #[must_use]
    pub const fn class_code(self) -> u16 {
        match self {
            Self::MalformedSealed => 0x50_01,
            Self::KeyPairMismatch => 0x50_02,
            Self::BadSignature => 0x50_03,
            Self::AeadMismatch => 0x50_04,
            Self::SignAbortLimit => 0x50_05,
        }
    }
}

impl From<PqcSealError> for PqcSealClass {
    fn from(e: PqcSealError) -> Self {
        match e {
            PqcSealError::MalformedSealed => Self::MalformedSealed,
            PqcSealError::KeyPairMismatch => Self::KeyPairMismatch,
            PqcSealError::BadSignature => Self::BadSignature,
            PqcSealError::AeadMismatch => Self::AeadMismatch,
            PqcSealError::SignAbortLimit => Self::SignAbortLimit,
        }
    }
}

/// Composite seal (pure delegation to [`cgap_crypto::pqc_seal::pqc_seal`]):
/// AEAD-encrypt `plaintext`, sign `ct ‖ aad` with ML-DSA-65 under the
/// sender's `dk`. `rnd` is the FIPS 204 sign randomness; `key`/`nonce`
/// are the AEAD stream key/nonce (caller owns stream discipline).
///
/// # Errors
/// [`AoqError::PqcSealRefused`] carrying [`PqcSealClass::KeyPairMismatch`]
/// or [`PqcSealClass::SignAbortLimit`] (all-or-nothing: an empty `Vec` is
/// never returned as a partial seal).
pub fn seal(
    dk: &[u8; SK_BYTES],
    ek: &[u8; PK_BYTES],
    plaintext: &[u8],
    aad: &[u8],
    key: &[u8; 32],
    nonce: &Nonce96,
    rnd: &[u8; 32],
) -> Result<Vec<u8>, AoqError> {
    cgap_crypto::pqc_seal::pqc_seal(dk, ek, plaintext, aad, key, nonce, rnd)
        .map_err(|e| AoqError::PqcSealRefused { class: PqcSealClass::from(e) })
}

/// Composite open (pure delegation to [`cgap_crypto::pqc_seal::pqc_open`]):
/// verify-then-decrypt. `plaintext` must be exactly
/// `sealed.len() - CT_OVERHEAD` bytes and is written ONLY on the success
/// path — the court asserts this with a sentinel fill.
///
/// # Errors
/// [`AoqError::PqcSealRefused`] carrying the matching [`PqcSealClass`].
pub fn open(
    ek: &[u8; PK_BYTES],
    plaintext: &mut [u8],
    sealed: &[u8],
    aad: &[u8],
    key: &[u8; 32],
    nonce: &Nonce96,
) -> Result<(), AoqError> {
    cgap_crypto::pqc_seal::pqc_open(ek, plaintext, sealed, aad, key, nonce)
        .map_err(|e| AoqError::PqcSealRefused { class: PqcSealClass::from(e) })
}
