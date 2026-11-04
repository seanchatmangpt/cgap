//! AEAD host-facade seam (ruling R-K1: crypto bodies live in cgap-crypto;
//! cgap-aoq hosts the seam only). No kernel is admitted in this crate:
//! every call is a typed KernelNotAdmitted refusal, loud not quiet.

use crate::error::AoqError;

/// Highest admitted AEAD kernel id; none exists yet (seam pre-admission).
pub const ADMITTED_AEAD_MAX: u32 = 0;

/// Seal: seam only — refuses until a kernel is admitted via the
/// cgap-crypto facade (R-K1).
///
/// # Errors
/// Always [`AoqError::AeadKernelNotAdmitted`] with the caller's `kernel_id`.
pub const fn seal(kernel_id: u32, _key: &[u8; 32], _plaintext: &[u8]) -> Result<(), AoqError> {
    let _ = _key;
    let _ = _plaintext;
    Err(AoqError::AeadKernelNotAdmitted { kernel_id })
}

/// Open: seam only — refuses until a kernel is admitted (R-K1).
///
/// # Errors
/// Always [`AoqError::AeadKernelNotAdmitted`] with the caller's `kernel_id`.
pub const fn open(kernel_id: u32, _key: &[u8; 32], _ciphertext: &[u8]) -> Result<(), AoqError> {
    let _ = _key;
    let _ = _ciphertext;
    Err(AoqError::AeadKernelNotAdmitted { kernel_id })
}
