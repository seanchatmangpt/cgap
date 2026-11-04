//! wire-shm — zero-dependency binary-frame shared-memory ring segment.
//!
//! Frame format (big-endian, 16-byte fixed header):
//! `[u32 magic 0x5749_5230 ("WIR0")] [u32 crc32(payload)] [u32 payload_len] [u32 frame_ordinal] [payload]`
//!
//! The segment is an offset-addressed file-backed segment (std::fs +
//! `FileExt`), NOT mmap. Zero-dep law = zero crates, not zero syscalls
//! (decision-d2-zero-dep-reinterpretation.md, 2026-11-04): a bare
//! `extern "C"` libc call is zero-dep lawful. Layout: slot 0 holds `[u64 next_ordinal][u32 capacity]` bookkeeping
//! region (4096 bytes, page-aligned); slots follow at fixed stride.
//!
//! SLOT = 4096-byte stride: 16-byte header + up to 4080 payload bytes.

mod crc32;
pub mod ring;

pub use crc32::crc32_ieee;
pub use ring::WireRing;

pub const MAGIC: u32 = 0x5749_5230; // "WIR0"
pub const HEADER_LEN: usize = 16;
pub const BOOK_LEN: usize = 4096;
pub const SLOT_LEN: usize = 4096;
pub const MAX_PAYLOAD: usize = SLOT_LEN - HEADER_LEN;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireError {
    FrameTooLarge { len: usize, max: usize },
    BudgetTooSmall { need: usize, have: usize },
    CrcMismatch { expected: u32, found: u32 },
    StaleOrdinal { expected: u32, found: u32 },
    BadMagic { found: u32 },
    Io(String),
}

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WireError::FrameTooLarge { len, max } => {
                write!(f, "frame too large: {len} > max {max}")
            }
            WireError::BudgetTooSmall { need, have } => {
                write!(f, "output budget too small: need {need}, have {have}")
            }
            WireError::CrcMismatch { expected, found } => {
                write!(
                    f,
                    "crc mismatch: expected {expected:#010x}, found {found:#010x}"
                )
            }
            WireError::StaleOrdinal { expected, found } => {
                write!(f, "stale ordinal: expected {expected}, found {found}")
            }
            WireError::BadMagic { found } => write!(f, "bad magic: {found:#010x}"),
            WireError::Io(s) => write!(f, "io: {s}"),
        }
    }
}

impl std::error::Error for WireError {}

impl From<std::io::Error> for WireError {
    fn from(e: std::io::Error) -> Self {
        WireError::Io(e.to_string())
    }
}
