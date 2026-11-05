//! KEM host-facade seam (ruling R-K1: crypto bodies live in cgap-crypto;
//! cgap-aoq hosts the seam only). Admission flip (2026-11-04, v26.11.6
//! PQC lane 13): kernel id 1 = the REAL ML-KEM-768 kernel
//! (`cgap_crypto::kem`, FIPS 203 final — deterministic `keygen(d,z)` /
//! `encaps(ek,m)` / infallible cmov decaps; widths EK 1184 / DK 2400 /
//! CT 1088 / SS 32). `ADMITTED_KEM_MAX` was 0: every call was a typed
//! [`AoqError::KemKernelNotAdmitted`] refusal. It is now 1 and the two
//! calls delegate through the kernel; wrong-width inputs refuse typed —
//! loud, never quiet.

use crate::error::AoqError;
use cgap_crypto::kem as kernel;

/// Highest admitted KEM kernel id: 1 = ML-KEM-768 (FIPS 203 final,
/// `cgap_crypto::kem`).
pub const ADMITTED_KEM_MAX: u32 = 1;

/// Encapsulate to a peer encapsulation key (`peer_pk`: ML-KEM-768 ek,
/// exactly 1184 bytes). Deterministic `m` discipline:
/// `m = shake256(b"w2a/mlkem/m" ‖ ek)` — the same order-sensitive
/// discipline the FFI host seam (AB-5) uses; this facade is pure
/// delegation, no crypto logic in this crate.
///
/// # Errors
/// [`AoqError::KemKernelNotAdmitted`] for any `kernel_id` above
/// [`ADMITTED_KEM_MAX`]; [`AoqError::KemShapeMismatch`] for a
/// non-1184-byte `peer_pk` (checked before the kernel is touched).
pub fn encapsulate(kernel_id: u32, peer_pk: &[u8]) -> Result<[u8; 32], AoqError> {
    if kernel_id > ADMITTED_KEM_MAX {
        return Err(AoqError::KemKernelNotAdmitted { kernel_id });
    }
    let ek: &[u8; kernel::EK_LEN] =
        peer_pk.try_into().map_err(|_| AoqError::KemShapeMismatch { got: peer_pk.len() })?;
    let m = shake32(&[b"w2a/mlkem/m", ek]);
    let mut ct = [0u8; kernel::CT_LEN];
    let mut ss = [0u8; kernel::SS_LEN];
    kernel::encaps(ek, &m, &mut ct, &mut ss);
    Ok(ss)
}

/// Decapsulate a ciphertext (`ct`: exactly 1088 bytes) under a
/// decapsulation key (`dk`: exactly 2400 bytes). Infallible FO
/// semantics: a corrupted ct yields the deterministic rejection
/// pseudosecret (FIPS 203 §7.3), never an error.
///
/// # Errors
/// [`AoqError::KemKernelNotAdmitted`] above [`ADMITTED_KEM_MAX`];
/// [`AoqError::KemShapeMismatch`] for a non-2400 dk or non-1088 ct.
pub fn decapsulate(kernel_id: u32, dk: &[u8], ct: &[u8]) -> Result<[u8; 32], AoqError> {
    if kernel_id > ADMITTED_KEM_MAX {
        return Err(AoqError::KemKernelNotAdmitted { kernel_id });
    }
    let dk: &[u8; kernel::DK_LEN] = dk.try_into().map_err(|_| AoqError::KemShapeMismatch { got: dk.len() })?;
    let ct: &[u8; kernel::CT_LEN] = ct.try_into().map_err(|_| AoqError::KemShapeMismatch { got: ct.len() })?;
    Ok(kernel::decaps(dk, ct))
}

/// Order-sensitive 32-byte Shake256 squeeze over concatenated parts.
fn shake32(parts: &[&[u8]]) -> [u8; 32] {
    let mut sh = cgap_crypto::fips202::Shake256::instance();
    for p in parts {
        sh.absorb(p);
    }
    let mut out = [0u8; 32];
    sh.squeeze_into(&mut out);
    out
}
