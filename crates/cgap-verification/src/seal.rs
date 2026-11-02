
//! A16 chatman keyed sealing of the grounded context.
//!
//! `cgap_crypto::chatman::keyed_digest` is loud-fail-closed above
//! `KEYED_MAX_INPUT_LEN` (1024 B); grounded contexts exceed that, so this
//! module binds the context with keyed chunk leaves under a Merkle root
//! built by the real `admissions_root`: leaf_i = keyed_digest(key,
//! chunk_i), root = admissions_root(leaves). The seal is a keyed commitment:
//! verifying requires the key, and any byte flip moves a leaf, hence the
//! root (BLAKE3 collision resistance).

use cgap_crypto::chatman::{admissions_root, keyed_digest, DigestError, KEYED_MAX_INPUT_LEN};
use std::fmt;
use std::vec::Vec;

/// Chunk size for keyed leaves; must be <= [`KEYED_MAX_INPUT_LEN`].
pub const SEAL_CHUNK_LEN: usize = KEYED_MAX_INPUT_LEN;

const _: () = assert!(
    SEAL_CHUNK_LEN <= KEYED_MAX_INPUT_LEN,
    "seal chunk exceeds the chatman keyed-digest loud bound",
);

/// The keyed seal over a grounded context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SealedContext {
    /// Merkle root over keyed chunk leaves.
    pub root: [u8; 32],
    /// Context length (bytes).
    pub len: u64,
    /// Number of keyed leaves.
    pub leaves: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SealError {
    Digest(DigestError),
    /// Context bytes do not match the seal.
    Tampered {
        expected_root: [u8; 32],
        actual_root: [u8; 32],
    },
    /// Length mismatch between context and seal.
    LengthMismatch { expected: u64, actual: u64 },
}

impl fmt::Display for SealError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SealError::Digest(e) => write!(f, "seal digest error: {e:?}"),
            SealError::Tampered { expected_root, actual_root } => write!(
                f,
                "seal integrity FAILED: expected root 0x{}, got 0x{}",
                hex(expected_root),
                hex(actual_root)
            ),
            SealError::LengthMismatch { expected, actual } => {
                write!(f, "seal length mismatch: expected {expected}, got {actual}")
            }
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::new();
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn compute_root(key: &[u8; 32], context: &[u8]) -> Result<([u8; 32], u64), DigestError> {
    if context.is_empty() {
        // Typed zero-root for the empty context (admissions_root law).
        return Ok(([0u8; 32], 0));
    }
    let mut leaves: Vec<Vec<u8>> = Vec::new();
    for chunk in context.chunks(SEAL_CHUNK_LEN) {
        leaves.push(keyed_digest(key, chunk)?.to_vec());
    }
    let n = leaves.len() as u64;
    let refs: Vec<&[u8]> = leaves.iter().map(|l| l.as_slice()).collect();
    Ok((admissions_root(&refs), n))
}

/// Seal the grounded context under `key`.
pub fn seal_grounded(key: &[u8; 32], context: &[u8]) -> Result<SealedContext, SealError> {
    let (root, leaves) = compute_root(key, context).map_err(SealError::Digest)?;
    Ok(SealedContext { root, len: context.len() as u64, leaves })
}

/// Verify context bytes against a seal: length + keyed root.
pub fn verify_sealed(
    key: &[u8; 32],
    context: &[u8],
    sealed: &SealedContext,
) -> Result<(), SealError> {
    if sealed.len != context.len() as u64 {
        return Err(SealError::LengthMismatch {
            expected: sealed.len,
            actual: context.len() as u64,
        });
    }
    let (root, _) = compute_root(key, context).map_err(SealError::Digest)?;
    if root != sealed.root {
        return Err(SealError::Tampered {
            expected_root: sealed.root,
            actual_root: root,
        });
    }
    Ok(())
}
