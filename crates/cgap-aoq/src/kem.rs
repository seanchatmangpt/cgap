//! KEM host-facade seam (ruling R-K1: crypto bodies live in cgap-crypto;
//! cgap-aoq hosts the seam only). No kernel is admitted in this crate:
//! every call is a typed KernelNotAdmitted refusal, loud not quiet.

use crate::error::AoqError;

/// Highest admitted KEM kernel id; none exists yet (seam pre-admission).
pub const ADMITTED_KEM_MAX: u32 = 0;

/// Encapsulate: seam only — refuses until a kernel is admitted via the
/// cgap-crypto facade (R-K1).
///
/// # Errors
/// Always [`AoqError::KemKernelNotAdmitted`] with the caller's `kernel_id`.
pub const fn encapsulate(kernel_id: u32, _peer_pk: &[u8]) -> Result<[u8; 32], AoqError> {
    let _ = _peer_pk;
    Err(AoqError::KemKernelNotAdmitted { kernel_id })
}

/// Decapsulate: seam only — refuses until a kernel is admitted (R-K1).
///
/// # Errors
/// Always [`AoqError::KemKernelNotAdmitted`] with the caller's `kernel_id`.
pub const fn decapsulate(kernel_id: u32, _ct: &[u8]) -> Result<[u8; 32], AoqError> {
    let _ = _ct;
    Err(AoqError::KemKernelNotAdmitted { kernel_id })
}
