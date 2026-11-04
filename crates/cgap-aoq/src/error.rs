//! AoqError — the 11-variant typed refusal surface (PG-A1 skeleton table,
//! reconciled to the AOQ1 64-byte envelope per R-H1/R-H2).
//!
//! SUPERSESSION NOTE (AB-1 receipt, coordinator R-H2): the original
//! framing-class variants (parent-prefix/payload framing from the AOQ-02
//! prose design) are superseded by IpcError reuse at the AB-8 integration
//! seam. This module stays REGISTERED as the seams-placeholder error
//! surface: header-law refusals plus the kem/aead KernelNotAdmitted
//! refusals. Integer fields only — `Copy`, no heap, no strings.
//!
//! AB-8 owns final integration; do not delete or re-frame here.

#![allow(clippy::module_name_repetitions)]

use core::fmt;

/// Typed refusals for the AoQ surface. Never `todo!()`/`unimplemented!()`:
/// every failure path is a value of this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AoqError {
    /// Input shorter than the fixed 64-byte envelope.
    HeaderTooShort { got: u16 },
    /// Output buffer shorter than the fixed 64-byte envelope.
    HeaderTooLong { got: u16 },
    /// Leading magic mismatch (`aoq:hf-magic`).
    BadMagic { got: u32 },
    /// Wire version outside the admitted set (`aoq:hf-version`).
    UnsupportedVersion { got: u16 },
    /// Wire epoch outside the closed `aoq:Epoch` enum.
    UnknownEpochWire { got: u16 },
    /// Stored header digest does not match recomputation.
    DigestMismatch { expected: u32, got: u32 },
    /// Reserved tail nonzero where the caller's law demands canonical form.
    ReservedNotZero,
    /// Frame ordinal regressed against the caller's cursor.
    FrameOrdinalRegression { expected: u32, got: u32 },
    /// Sequence number gap/regression against the caller's cursor.
    SeqGap { expected: u32, got: u32 },
    /// KEM kernel seam: no kernel admitted for this id (host facade only).
    ///
    /// Seams-placeholder per the XC-5r census: the KEM kernel seam is
    /// hand-law (`aoq:KemParam`) and no attack scenario exercises this
    /// variant yet — it is the typed landing for a not-yet-admitted kernel
    /// id, not a witnessed refusal path. AB-8 integration owns population.
    KemKernelNotAdmitted { kernel_id: u32 },
    /// AEAD kernel seam: no kernel admitted for this id (host facade only).
    ///
    /// Seams-placeholder per the XC-5r census: the AEAD kernel seam is
    /// hand-law and no attack scenario exercises this variant yet — it is
    /// the typed landing for a not-yet-admitted kernel id, not a witnessed
    /// refusal path. AB-8 integration owns population.
    AeadKernelNotAdmitted { kernel_id: u32 },
}

impl AoqError {
    /// Stable numeric class code (wire/log-safe; no formatting needed).
    #[must_use]
    pub const fn class_code(&self) -> u16 {
        match *self {
            Self::HeaderTooShort { .. } => 0x00_01,
            Self::HeaderTooLong { .. } => 0x00_02,
            Self::BadMagic { .. } => 0x00_03,
            Self::UnsupportedVersion { .. } => 0x00_04,
            Self::UnknownEpochWire { .. } => 0x00_05,
            Self::DigestMismatch { .. } => 0x00_06,
            Self::ReservedNotZero => 0x00_07,
            Self::FrameOrdinalRegression { .. } => 0x00_08,
            Self::SeqGap { .. } => 0x00_09,
            Self::KemKernelNotAdmitted { .. } => 0x00_0A,
            Self::AeadKernelNotAdmitted { .. } => 0x00_0B,
        }
    }
}

impl fmt::Display for AoqError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::HeaderTooShort { got } => write!(f, "header too short: {got}B"),
            Self::HeaderTooLong { got } => write!(f, "header too long: {got}B"),
            Self::BadMagic { got } => write!(f, "bad magic: 0x{got:08X}"),
            Self::UnsupportedVersion { got } => write!(f, "unsupported version: {got}"),
            Self::UnknownEpochWire { got } => write!(f, "unknown AOQ epoch on wire: {got}"),
            Self::DigestMismatch { expected, got } => {
                write!(f, "digest mismatch: stored 0x{expected:08X}, computed 0x{got:08X}")
            }
            Self::ReservedNotZero => write!(f, "reserved tail not zero (canonical form)"),
            Self::FrameOrdinalRegression { expected, got } => {
                write!(f, "frame ordinal regression: expected >= {expected}, got {got}")
            }
            Self::SeqGap { expected, got } => {
                write!(f, "seq gap: expected {expected}, got {got}")
            }
            Self::KemKernelNotAdmitted { kernel_id } => {
                write!(f, "KEM kernel not admitted: id 0x{kernel_id:08X}")
            }
            Self::AeadKernelNotAdmitted { kernel_id } => {
                write!(f, "AEAD kernel not admitted: id 0x{kernel_id:08X}")
            }
        }
    }
}

impl core::error::Error for AoqError {}
