//! AEAD host-facade (ruling R-K1: crypto bodies live in cgap-crypto;
//! cgap-aoq hosts the facade — typed delegation, no crypto body here).
//!
//! v3 wire law (Stage 2, 2026-11-04): the two const-refusal stubs are
//! replaced by delegation to cgap-crypto's var-AAD detached seal/open
//! (RFC 8439 §2.8). The covered span is now a variable-length AAD; the
//! old pinned `AAD_LEN = 36` wrapper no longer binds the session layer.
//! The facade maps the kernel's [`mismatch_type::Mismatch`] onto
//! [`AoqError::AeadMismatch`] so every refusal on this surface is a value
//! of the crate's typed error table.

use crate::error::AoqError;
use cgap_crypto::aead as kernel;
pub use cgap_crypto::aead::{EpochExhausted, Mismatch, Nonce96, TAG_LEN};

/// Detached seal (delegation to [`kernel::seal_detached`], RFC 8439
/// §2.8): encrypt `plaintext` into `ciphertext` (equal lengths),
/// authenticate the variable-length `aad` span plus the ciphertext into
/// the 16-byte Poly1305 `tag`.
///
/// Panics (kernel caller-contract, checked at entry): `ciphertext.len()
/// != plaintext.len()`.
pub fn seal_detached(
    ciphertext: &mut [u8],
    tag: &mut [u8; TAG_LEN],
    plaintext: &[u8],
    aad: &[u8],
    key: &[u8; 32],
    nonce: &Nonce96,
) {
    kernel::seal_detached(ciphertext, tag, plaintext, aad, key, nonce);
}

/// Detached open (delegation to [`kernel::open_detached`]): constant-time
/// authenticate `aad || ct`, then decrypt into `plaintext` only on a
/// match.
///
/// # Errors
/// [`AoqError::AeadMismatch`] on any tag divergence (AAD, ct, key or
/// nonce all flow into the tag).
pub fn open_detached(
    plaintext: &mut [u8],
    ciphertext: &[u8],
    tag: &[u8; TAG_LEN],
    aad: &[u8],
    key: &[u8; 32],
    nonce: &Nonce96,
) -> Result<(), AoqError> {
    kernel::open_detached(plaintext, ciphertext, tag, aad, key, nonce)
        .map_err(|kernel::Mismatch| AoqError::AeadMismatch)
}

/// Tag-only verification (delegation to
/// [`kernel::verify_tag_var_aad`]): recompute the var-AAD Poly1305 tag
/// and constant-time compare — the plaintext-free form of
/// [`open_detached`] used by the session machine's receive transition,
/// which has no plaintext buffer at `step` time.
///
/// # Errors
/// [`AoqError::AeadMismatch`] on any tag divergence.
pub fn verify_detached(
    aad: &[u8],
    ciphertext: &[u8],
    tag: &[u8; TAG_LEN],
    key: &[u8; 32],
    nonce: &Nonce96,
) -> Result<(), AoqError> {
    kernel::verify_tag_var_aad(aad, ciphertext, tag, key, nonce)
        .map_err(|kernel::Mismatch| AoqError::AeadMismatch)
}
