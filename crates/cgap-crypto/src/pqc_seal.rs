//! pqc_seal — composite seal/open surface (SD-1 phase 6, lane L14).
//!
//! Binds the ML-DSA-65 signer ([`crate::mldsa`]) with the AEAD_CHACHA20_
//! POLY1305 detached layer ([`crate::aead`], RFC 8439 §2.8 var-AAD form)
//! into one fail-closed seal/open pair:
//!
//! - **seal**: AEAD-encrypt the plaintext, then sign the bound message
//!   `ct ‖ aad` with ML-DSA-65 under the sender's `dk`. Wire layout
//!   (length-pinned by const asserts):
//!
//!   ```text
//!   [ ciphertext (n bytes) ][ tag 16B ][ sig 3309B ]
//!   ```
//!
//! - **open**: verify-then-decrypt, fail-closed — length parse, ML-DSA-65
//!   verify over `ct ‖ aad` under the claimed `ek`, and only after the
//!   signature holds does the AEAD tag get checked and the plaintext
//!   decrypted. Any divergence is a typed refusal; the plaintext buffer
//!   is never written on any failure path.
//!
//! # Key binding
//!
//! The signature's `mu` is `H( tr ‖ ct ‖ aad )` where `tr = H(ek)` —
//! seal takes `tr` from the `dk` layout (`dk[64..64+TR_BYTES]`, written
//! by [`crate::mldsa::keygen`]); open re-derives it from the `ek`. A
//! `dk`/`ek` pair mismatch therefore breaks the signature check itself
//! (the wrong `tr` hashes to a different `mu`), not a separate boolean —
//! the binding is structural, not a flag.
//!
//! # v3 wire law (citation)
//!
//! The AEAD body is the RFC 8439 var-AAD detached pair promoted by the
//! AAD ruling Option b (`docs/jira/v26.11.4/receipts/aad-ruling-executed.
//! md`, F1 83207531): the covered span authenticates variable-length
//! AAD — here the caller's context `aad` of any length — under the
//! `AOQ1_VERSION = 3` session law
//! (`docs/jira/v26.11.4/receipts/wire-law-v3-landing.md`). No 36-byte
//! wrapper width is assumed anywhere in this module.
//!
//! `#![no_std]` + `#![forbid(unsafe_code)]` respected (alloc for the
//! returned sealed buffer).

use alloc::vec;
use alloc::vec::Vec;

use crate::aead::{self, Nonce96};
use crate::mldsa::{self, PK_BYTES, SK_BYTES, TR_BYTES};

/// Re-exported wire widths (sealed-layout vocabulary).
pub use crate::aead::TAG_LEN;
/// Sealed-signature span width (ML-DSA-65 `SIG_BYTES`, 3309).
pub use crate::mldsa::SIG_BYTES;

/// ML-DSA-65 context-separator byte: domain-separates the composite
/// seal message `ct ‖ aad` from any other `mu` consumer of the same key.
pub const PQC_SEAL_DOMAIN: u8 = 0x50; // 'P'

/// Composite-seal wire widths (all const-asserted).
pub const CT_OVERHEAD: usize = TAG_LEN + SIG_BYTES;
/// Sealed output length for a plaintext of `n` bytes: `n + CT_OVERHEAD`.
pub const fn sealed_len(plaintext_len: usize) -> usize {
    plaintext_len + CT_OVERHEAD
}

/// Typed refusals — one variant per failure class; no boolean folding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PqcSealError {
    /// Sealed buffer shorter than `TAG_LEN + SIG_BYTES` (open parse).
    MalformedSealed,
    /// `dk`/`ek` pair mismatch detected at seal time (tr derived from the
    /// dk layout != H(ek)) — refused before any signing work.
    KeyPairMismatch,
    /// ML-DSA-65 verification failed (wrong ek, tampered ct/aad, or
    /// forged sig).
    BadSignature,
    /// AEAD tag mismatch (after a PASSING signature — wrong key/nonce,
    /// or a tag forged independently of the signed span).
    AeadMismatch,
    /// ML-DSA-65 signing exceeded its FS-abort budget.
    SignAbortLimit,
}

/// Composite seal: AEAD-encrypt `plaintext`, then sign `ct ‖ aad` with
/// ML-DSA-65.
///
/// `rnd` is the ML-DSA-65 sign randomness (FIPS 204); `nonce`/`key` are
/// the AEAD stream key/nonce (caller owns stream discipline — the
/// [`Nonce96`] sole-constructor law carries the epoch-exhaustion guard).
///
/// # Errors
/// [`PqcSealError::SignAbortLimit`] only (work is all-or-nothing; the
/// returned `Vec` is empty on refusal).
pub fn pqc_seal(
    dk: &[u8; SK_BYTES],
    ek: &[u8; PK_BYTES],
    plaintext: &[u8],
    aad: &[u8],
    key: &[u8; 32],
    nonce: &Nonce96,
    rnd: &[u8; 32],
) -> Result<Vec<u8>, PqcSealError> {
    // 1. AEAD-encrypt (detached) into the output prefix.
    let mut out = vec![0u8; sealed_len(plaintext.len())];
    let (ct, rest) = out.split_at_mut(plaintext.len());
    let (tag_buf, _sig_buf) = rest.split_at_mut(TAG_LEN);
    let mut tag = [0u8; TAG_LEN];
    aead::seal_detached(ct, &mut tag, plaintext, aad, key, nonce);
    tag_buf.copy_from_slice(&tag);

    // 2. Sign ct ‖ aad under the sender key. mu = H(tr ‖ domain ‖ ct ‖ aad),
    //    tr from the dk layout (structural ek-binding, see module docs).
    //    The `ek` is cross-checked here: a dk/ek pair mismatch refuses at
    //    seal time (fail-closed) instead of producing an unverifiable seal.
    let mut tr_dk = [0u8; TR_BYTES];
    tr_dk.copy_from_slice(&dk[64..64 + TR_BYTES]);
    let mut tr_ek = [0u8; TR_BYTES];
    let mut hs = crate::fips202::Shake256::instance();
    hs.absorb(ek);
    hs.squeeze_into(&mut tr_ek);
    if tr_dk != tr_ek {
        out.clear();
        return Err(PqcSealError::KeyPairMismatch);
    }
    let msg = seal_message(ct, aad);
    let mu = hash_mu(&tr_dk, &msg);
    let sig = match mldsa::sign(dk, &mu, rnd) {
        Ok(s) => s,
        Err(mldsa::SignAbortLimit) => {
            // all-or-nothing: the sealed buffer is never partially returned
            out.clear();
            return Err(PqcSealError::SignAbortLimit);
        }
    };
    out[plaintext.len() + TAG_LEN..].copy_from_slice(&sig);
    Ok(out)
}

/// Composite open (fail-closed, verify-then-decrypt): parse the sealed
/// buffer, ML-DSA-65-verify `ct ‖ aad` under `ek`, and only then AEAD-
/// decrypt into `plaintext`.
///
/// `plaintext` must be exactly `sealed.len() - CT_OVERHEAD` bytes; it is
/// written ONLY on the success path (the kernel `open_detached` decrypts
/// into the caller buffer only after a constant-time tag match, and this
/// module returns before calling it on any verify/parse failure).
///
/// # Errors
/// [`PqcSealError::MalformedSealed`] (short buffer / length mismatch
/// with `plaintext`), [`PqcSealError::BadSignature`],
/// [`PqcSealError::AeadMismatch`].
pub fn pqc_open(
    ek: &[u8; PK_BYTES],
    plaintext: &mut [u8],
    sealed: &[u8],
    aad: &[u8],
    key: &[u8; 32],
    nonce: &Nonce96,
) -> Result<(), PqcSealError> {
    // 1. Fail-closed length parse.
    if sealed.len() < CT_OVERHEAD {
        return Err(PqcSealError::MalformedSealed);
    }
    let ct_len = sealed.len() - CT_OVERHEAD;
    if plaintext.len() != ct_len {
        return Err(PqcSealError::MalformedSealed);
    }
    let (ct, tail) = sealed.split_at(ct_len);
    let (tag_bytes, sig_bytes) = tail.split_at(TAG_LEN);
    let mut tag = [0u8; TAG_LEN];
    tag.copy_from_slice(tag_bytes);
    let mut sig = [0u8; SIG_BYTES];
    sig.copy_from_slice(sig_bytes);

    // 2. Verify-then-decrypt: signature gate first.
    let mut tr = [0u8; TR_BYTES];
    let mut hs = crate::fips202::Shake256::instance();
    hs.absorb(ek);
    hs.squeeze_into(&mut tr);
    let msg = seal_message(ct, aad);
    let mu = hash_mu(&tr, &msg);
    if !mldsa::verify(ek, &mu, &sig) {
        return Err(PqcSealError::BadSignature);
    }

    // 3. Signature held — now the AEAD tag; decrypt only on match.
    aead::open_detached(plaintext, ct, &tag, aad, key, nonce)
        .map_err(|aead::Mismatch| PqcSealError::AeadMismatch)
}

/// Bound message: `domain ‖ ct ‖ aad` (domain-separated from any other
/// `mu` consumer of the same ML-DSA key).
fn seal_message(ct: &[u8], aad: &[u8]) -> Vec<u8> {
    let mut m = Vec::with_capacity(1 + ct.len() + aad.len());
    m.push(PQC_SEAL_DOMAIN);
    m.extend_from_slice(ct);
    m.extend_from_slice(aad);
    m
}

/// `mu = H(tr ‖ msg)` via SHAKE256 — the same absorb order
/// [`crate::mldsa::compute_mu`] pins (`tr` then message); kept local so
/// the domain byte rides inside the message, not a `tr` prefix rewrite.
fn hash_mu(tr: &[u8; TR_BYTES], msg: &[u8]) -> [u8; mldsa::MU_BYTES] {
    crate::mldsa::compute_mu(tr, msg)
}

const _: () = assert!(CT_OVERHEAD == TAG_LEN + SIG_BYTES);
const _: () = assert!(SIG_BYTES == 3309);
const _: () = assert!(PK_BYTES == 1952);
const _: () = assert!(SK_BYTES == 4032);
