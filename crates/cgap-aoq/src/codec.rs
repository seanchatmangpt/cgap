//! AOQ1 header codec — 64-byte envelope encode/decode, alloc-free.
//!
//! Wire law (R-1 ruling, landed field table): little-endian per the generated
//! layout; FNV-1a32 digest over the half-open span `[0x00, 0x3C)` stored
//! at `0x3C`; reserved tail `[0x18, 0x3C)` zero on write,
//! ignored-but-digest-covered on read (IPC1 zero-tail discipline).
//! Framing/payload semantics are out of scope here (R-H2: IpcError reuse
//! at the AB-8 integration seam); this codec speaks header law only.

use crate::error::AoqError;
use crate::header::{AoqHeader, Epoch, UnknownEpoch, SIZE_BYTES, VERSION, MAGIC};

/// FNV-1a32 (32-bit FNV-1: multiply then xor). Not a cryptographic
/// primitive — it is the header integrity digest named by the landed graph
/// (`aoq:hf-header-digest`, WireKind::Fnv1a32); cryptographic digest
/// bodies remain cgap-crypto/cgap-blake3 territory (R-K1).
#[must_use]
pub const fn fnv1a32(data: &[u8]) -> u32 {
    let mut hash: u32 = 0x811C_9DC5;
    let mut i = 0;
    while i < data.len() {
        hash ^= data[i] as u32;
        hash = hash.wrapping_mul(0x0100_0193);
        i += 1;
    }
    hash
}

/// Encode the header into `out` (first `SIZE_BYTES` bytes), digest
/// included. The reserved tail is written as zeros (reserved-zero law);
/// the digest covers
/// `[0x00, 0x3C)` and lands at `0x3C`.
///
/// # Errors
/// [`AoqError::BufferTooShort`] when `out` is shorter than the envelope.
pub fn encode(header: &AoqHeader, out: &mut [u8]) -> Result<(), AoqError> {
    if out.len() < SIZE_BYTES {
        return Err(AoqError::BufferTooShort {
            got: out.len() as u16,
        });
    }
    out[0..4].copy_from_slice(&header.magic.to_le_bytes());
    out[4..6].copy_from_slice(&header.version.to_le_bytes());
    out[6..8].copy_from_slice(&header.epoch.to_le_bytes());
    out[8..16].copy_from_slice(&header.key_id.to_le_bytes());
    out[16..20].copy_from_slice(&header.frame_ordinal.to_le_bytes());
    out[20..24].copy_from_slice(&header.seq.to_le_bytes());
    // Reserved-zero write law (aoq:hf-reserved): zeros on write,
    // ignored-but-digest-covered on read.
    out[24..60].fill(0);
    let digest = fnv1a32(&out[0..60]);
    out[60..64].copy_from_slice(&digest.to_le_bytes());
    Ok(())
}

/// Decode a header from `buf` (first `SIZE_BYTES` bytes), verifying magic,
/// version, epoch, reserved tail, and the FNV-1a32 digest.
///
/// # Errors
/// Typed [`AoqError`] refusals, integer fields only.
pub fn decode(buf: &[u8]) -> Result<AoqHeader, AoqError> {
    if buf.len() < SIZE_BYTES {
        return Err(AoqError::HeaderTooShort {
            got: buf.len() as u16,
        });
    }
    let magic = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
    if magic != MAGIC {
        return Err(AoqError::BadMagic { got: magic });
    }
    let version = u16::from_le_bytes([buf[4], buf[5]]);
    if version != VERSION {
        return Err(AoqError::UnsupportedVersion { got: version });
    }
    let epoch_raw = u16::from_le_bytes([buf[6], buf[7]]);
    // Epoch validation against the closed enum; the header struct carries
    // the raw u16 (Epoch::Retired is a read-only sentinel, R-1 law).
    if let Err(UnknownEpoch(raw)) = Epoch::try_from(epoch_raw) {
        return Err(AoqError::UnknownEpochWire { got: raw });
    }
    let key_id = u64::from_le_bytes([
        buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15],
    ]);
    let frame_ordinal = u32::from_le_bytes([buf[16], buf[17], buf[18], buf[19]]);
    let seq = u32::from_le_bytes([buf[20], buf[21], buf[22], buf[23]]);
    // Reserved tail: ignored on read (IPC1 zero-tail discipline) but
    // digest-covered — a tampered tail fails the digest check below.
    let mut reserved = [0u8; 36];
    reserved.copy_from_slice(&buf[24..60]);
    let stored = u32::from_le_bytes([buf[60], buf[61], buf[62], buf[63]]);
    let computed = fnv1a32(&buf[0..60]);
    if stored != computed {
        return Err(AoqError::DigestMismatch {
            expected: stored,
            got: computed,
        });
    }
    Ok(AoqHeader {
        magic,
        version,
        epoch: epoch_raw,
        key_id,
        frame_ordinal,
        seq,
        reserved,
        header_digest: stored,
    })
}
