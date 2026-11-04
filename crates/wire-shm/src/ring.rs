//! WireRing - offset-addressed file-backed ring segment.
//!
//! Segment layout: [bookkeeping 4096B][slot 0][slot 1]...[slot capacity-1]
//! Bookkeeping: bytes 0..8 = u64 BE next_ordinal, 8..12 = u32 BE capacity.
//! Slot: 16-byte BE header (magic, crc32, payload_len, ordinal) + payload.

use super::crc32::crc32_ieee;
use super::{WireError, BOOK_LEN, HEADER_LEN, MAGIC, MAX_PAYLOAD, SLOT_LEN};
use std::fs::{File, OpenOptions};
use std::os::unix::fs::FileExt;
use std::path::Path;

/// Wire-speed ring segment: fixed SLOT_LEN-byte slots in a file-backed segment.
pub struct WireRing {
    file: File,
    capacity: u32,
}

const NEXT_ORD_OFF: u64 = 0;
const CAP_OFF: u64 = 8;

impl WireRing {
    fn slot_offset(ordinal: u32) -> u64 {
        BOOK_LEN as u64 + (ordinal as u64) * (SLOT_LEN as u64)
    }

    /// Create (or truncate) a POSIX-style segment file at `path` with
    /// `frames_capacity` slots.
    pub fn create(path: &Path, frames_capacity: u32) -> Result<Self, WireError> {
        if frames_capacity == 0 {
            return Err(WireError::Io("capacity must be >= 1".into()));
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        let total = (BOOK_LEN + frames_capacity as usize * SLOT_LEN) as u64;
        // Zero-fill to full segment size.
        file.set_len(total)?;
        let zeros = [0u8; SLOT_LEN];
        let mut off = 0u64;
        while off < total {
            file.write_all_at(&zeros, off)?;
            off += zeros.len() as u64;
        }
        let seg = WireRing {
            file,
            capacity: frames_capacity,
        };
        seg.write_book(NEXT_ORD_OFF, &0u64.to_be_bytes())?;
        seg.write_book(CAP_OFF, &frames_capacity.to_be_bytes())?;
        Ok(seg)
    }

    /// Open an existing segment file; validates the recorded capacity.
    pub fn open(path: &Path) -> Result<Self, WireError> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        let mut buf = [0u8; 8];
        file.read_exact_at(&mut buf, CAP_OFF)?;
        let capacity = u32::from_be_bytes(buf[0..4].try_into().unwrap());
        let len = file.metadata()?.len() as usize;
        let expected = BOOK_LEN + capacity as usize * SLOT_LEN;
        if len != expected {
            return Err(WireError::Io(format!(
                "segment size mismatch: {len} != {expected}"
            )));
        }
        Ok(WireRing { file, capacity })
    }

    fn write_book(&self, off: u64, bytes: &[u8]) -> Result<(), WireError> {
        self.file.write_all_at(bytes, off)?;
        Ok(())
    }

    fn read_book(&self, off: u64, buf: &mut [u8]) -> Result<(), WireError> {
        self.file.read_exact_at(buf, off)?;
        Ok(())
    }

    /// Current next ordinal (strictly increasing per writer).
    ///
    /// Zero-alloc law (S12): the 8 bookkeeping bytes land on the stack.
    pub fn next_ordinal(&self) -> Result<u64, WireError> {
        let mut buf = [0u8; 8];
        self.read_book(NEXT_ORD_OFF, &mut buf)?;
        Ok(u64::from_be_bytes(buf))
    }

    /// Write header+payload at slot `ordinal % capacity`, advance next_ordinal.
    ///
    /// Zero-alloc law (S12): header and payload go to the slot via two
    /// `write_all_at` calls — no assembled frame Vec.
    pub fn stamp_frame(&mut self, payload: &[u8], ordinal: u32) -> Result<(), WireError> {
        if payload.len() > MAX_PAYLOAD {
            return Err(WireError::FrameTooLarge {
                len: payload.len(),
                max: MAX_PAYLOAD,
            });
        }
        let next = self.next_ordinal()?;
        if (ordinal as u64) < next {
            return Err(WireError::StaleOrdinal {
                expected: next as u32,
                found: ordinal,
            });
        }
        let idx = ordinal % self.capacity;
        let off = Self::slot_offset(idx);
        let mut header = [0u8; HEADER_LEN];
        header[0..4].copy_from_slice(&MAGIC.to_be_bytes());
        header[4..8].copy_from_slice(&crc32_ieee(payload).to_be_bytes());
        header[8..12].copy_from_slice(&(payload.len() as u32).to_be_bytes());
        header[12..16].copy_from_slice(&ordinal.to_be_bytes());
        self.file.write_all_at(&header, off)?;
        self.file.write_all_at(payload, off + HEADER_LEN as u64)?;
        // Persist highest_ordinal+1 AFTER the frame so recovery sees the frame.
        self.write_book(NEXT_ORD_OFF, &((ordinal as u64) + 1).to_be_bytes())?;
        Ok(())
    }

    /// Read+validate the frame at `ordinal % capacity`: magic, ordinal match,
    /// payload_len, crc32.
    pub fn read_frame(&self, ordinal: u32) -> Result<Vec<u8>, WireError> {
        let mut buf = [0u8; SLOT_LEN];
        let n = self.read_frame_into(ordinal, &mut buf)?;
        Ok(buf[..n].to_vec())
    }

    /// Read and validate; returns (payload, crc) for tamper courts.
    pub fn read_frame_with_crc(&self, ordinal: u32) -> Result<(Vec<u8>, u32), WireError> {
        let off = Self::slot_offset(ordinal % self.capacity);
        let header = self.read_validated_header(ordinal)?;
        let plen = Self::header_payload_len(&header)?;
        let mut payload = [0u8; MAX_PAYLOAD];
        self.file
            .read_exact_at(&mut payload[..plen], off + HEADER_LEN as u64)?;
        let expect_crc = u32::from_be_bytes(header[4..8].try_into().unwrap());
        let found_crc = crc32_ieee(&payload[..plen]);
        if found_crc != expect_crc {
            return Err(WireError::CrcMismatch {
                expected: expect_crc,
                found: found_crc,
            });
        }
        Ok((payload[..plen].to_vec(), expect_crc))
    }

    /// Read+validate the frame at `ordinal % capacity` into the caller's
    /// buffer; returns the payload length.
    ///
    /// Zero-alloc law (S12): the only wire-sized memory is `out`, supplied
    /// by the caller. `plen` is wire input, so the budget check is
    /// `HEADER_LEN.checked_add(plen)` — overflow and undersized buffers are
    /// typed refusals (`FrameTooLarge` / `BudgetTooSmall`), never panics.
    pub fn read_frame_into(
        &self,
        ordinal: u32,
        out: &mut [u8],
    ) -> Result<usize, WireError> {
        let off = Self::slot_offset(ordinal % self.capacity);
        let header = self.read_validated_header(ordinal)?;
        let plen = Self::header_payload_len(&header)?;
        let total = HEADER_LEN
            .checked_add(plen)
            .ok_or(WireError::FrameTooLarge {
                len: plen,
                max: MAX_PAYLOAD,
            })?;
        if total > out.len() {
            return Err(WireError::BudgetTooSmall {
                need: total,
                have: out.len(),
            });
        }
        self.file
            .read_exact_at(&mut out[..plen], off + HEADER_LEN as u64)?;
        let expect_crc = u32::from_be_bytes(header[4..8].try_into().unwrap());
        let found_crc = crc32_ieee(&out[..plen]);
        if found_crc != expect_crc {
            return Err(WireError::CrcMismatch {
                expected: expect_crc,
                found: found_crc,
            });
        }
        Ok(plen)
    }

    /// Read the 16-byte header at `ordinal % capacity`, validating magic and
    /// the stored ordinal. Stack-only; payload length is NOT yet checked.
    fn read_validated_header(&self, ordinal: u32) -> Result<[u8; HEADER_LEN], WireError> {
        let off = Self::slot_offset(ordinal % self.capacity);
        let mut header = [0u8; HEADER_LEN];
        self.file.read_exact_at(&mut header, off)?;
        let magic = u32::from_be_bytes(header[0..4].try_into().unwrap());
        if magic != MAGIC {
            return Err(WireError::BadMagic { found: magic });
        }
        let stored_ord = u32::from_be_bytes(header[12..16].try_into().unwrap());
        if stored_ord != ordinal {
            return Err(WireError::StaleOrdinal {
                expected: ordinal,
                found: stored_ord,
            });
        }
        Ok(header)
    }

    /// Extract and bounds-check the payload length from a validated header.
    /// `plen` is wire input: above `MAX_PAYLOAD` it is a typed refusal.
    fn header_payload_len(header: &[u8; HEADER_LEN]) -> Result<usize, WireError> {
        let plen = u32::from_be_bytes(header[8..12].try_into().unwrap()) as usize;
        if plen > MAX_PAYLOAD {
            return Err(WireError::FrameTooLarge {
                len: plen,
                max: MAX_PAYLOAD,
            });
        }
        Ok(plen)
    }

    /// Verify magic + crc32 (header ordinal match is enforced too) at
    /// `ordinal % capacity`.
    ///
    /// Thin wrapper over `read_frame_into` with a stack scratch buffer —
    /// zero-alloc law (S12) keeps the read path Vec-free.
    pub fn verify_frame(&self, ordinal: u32) -> Result<(), WireError> {
        let mut buf = [0u8; SLOT_LEN];
        self.read_frame_into(ordinal, &mut buf)?;
        Ok(())
    }
}
