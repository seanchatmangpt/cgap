//! String/Binary view vectors (Arrow 2.0 layout: "vu"/"vz").
//!
//! Descriptor (16 bytes): [len: u32][inline prefix: 12 bytes] when
//! len <= 12 the payload lives in the descriptor; otherwise the prefix holds
//! the first 4 bytes and [buffer_index: u32][offset: u32] address a variadic
//! data block. Inline-vs-foreign classification is branchless (arithmetic
//! mask, charter CC=1 law).

use crate::bitmap::Bitmap;
use crate::buffer::Buf;
use alloc::vec::Vec;

/// Inline payloads up to 12 bytes live inside the descriptor.
pub const MAX_INLINE: u32 = 12;

/// Typed refusal for a corrupt/hostile view vector: an attacker-controlled
/// descriptor (len/offset words) can never panic the reader — every
/// out-of-bounds access is a typed error naming the offending bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewError {
    /// Row index past the descriptor buffer (`i >= rows`).
    RowPastEnd {
        /// Requested row index.
        i: usize,
        /// Actual row count (`views.len() / 16`).
        rows: usize,
    },
    /// A row payload range exceeds its backing buffer.
    OutOfRange {
        /// Start of the payload range.
        off: usize,
        /// Payload length claimed by the descriptor.
        len: usize,
        /// Backing buffer length (`data` for foreign rows, `views` for
        /// inline rows).
        data_len: usize,
    },
}

impl core::fmt::Display for ViewError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ViewError::RowPastEnd { i, rows } => {
                write!(f, "row {i} past end: buffer holds {rows} rows")
            }
            ViewError::OutOfRange { off, len, data_len } => write!(
                f,
                "payload [{off}..{off}+{len}] exceeds buffer of {data_len} bytes"
            ),
        }
    }
}

const VIEW_SIZE: usize = 16;

/// Branchless inline test: 1 when len <= 12, else 0 (no conditional).
#[must_use]
pub fn is_inline(len: u32) -> u64 {
    // (len as i64 - 13) >> 63 is all-ones iff len < 13.
    (((i64::from(len) - 13) >> 63) as u64) & 1
}

/// A frozen view vector: descriptors buffer + one variadic data block.
pub struct ViewVec {
    validity: Option<Bitmap>,
    views: Buf,
    data: Buf,
    len: usize,
}

impl ViewVec {
    /// Row count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// True when zero rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Descriptor bytes (len * 16).
    #[must_use]
    pub fn views_bytes(&self) -> &[u8] {
        self.views.as_slice()
    }

    /// Variadic data block bytes.
    #[must_use]
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    /// Row i payload bytes. Typed refusal on a hostile descriptor (row past
    /// the buffer, or a len/offset pair whose payload range exceeds the
    /// backing buffer) — never a panic on external input.
    pub fn row(&self, i: usize) -> Result<&[u8], ViewError> {
        let d = self.views_bytes();
        let rows = d.len() / VIEW_SIZE;
        if i >= rows {
            return Err(ViewError::RowPastEnd { i, rows });
        }
        let base = i * VIEW_SIZE;
        let len = u32::from_le_bytes(
            d[base..base + 4]
                .try_into()
                .expect("row descriptor len word"),
        ) as usize;
        if is_inline(len as u32) == 1 {
            if base + 4 + len > d.len() {
                return Err(ViewError::OutOfRange {
                    off: base + 4,
                    len,
                    data_len: d.len(),
                });
            }
            Ok(&d[base + 4..base + 4 + len])
        } else {
            let off = u32::from_le_bytes(
                d[base + 12..base + 16]
                    .try_into()
                    .expect("row descriptor off word"),
            ) as usize;
            let data = self.data.as_slice();
            if off + len > data.len() {
                return Err(ViewError::OutOfRange {
                    off,
                    len,
                    data_len: data.len(),
                });
            }
            Ok(&data[off..off + len])
        }
    }

    /// Validity bitmap (None = all valid).
    #[must_use]
    pub fn validity(&self) -> Option<&Bitmap> {
        self.validity.as_ref()
    }
}

/// Builder producing a frozen [`ViewVec`].
pub struct ViewVecBuilder {
    views: Vec<u8>,
    data: Vec<u8>,
    validity: Vec<bool>,
}

impl Default for ViewVecBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewVecBuilder {
    #[must_use]
    pub fn new() -> Self {
        ViewVecBuilder {
            views: Vec::new(),
            data: Vec::new(),
            validity: Vec::new(),
        }
    }

    /// Rows appended so far.
    #[must_use]
    pub fn len(&self) -> usize {
        self.validity.len()
    }

    /// True when no rows appended yet.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.validity.is_empty()
    }

    /// Append one row.
    pub fn push(&mut self, row: Option<&[u8]>) {
        match row {
            Some(bytes) => {
                let len = bytes.len() as u32;
                self.views.extend_from_slice(&len.to_le_bytes());
                if len <= MAX_INLINE {
                    let mut inline = [0u8; 12];
                    inline[..bytes.len()].copy_from_slice(bytes);
                    self.views.extend_from_slice(&inline);
                } else {
                    let off = self.data.len() as u32;
                    self.views.extend_from_slice(&bytes[..4]);
                    // [buffer_index: u32][offset: u32] — index slot first, then offset.
                    self.views.extend_from_slice(&0u32.to_le_bytes());
                    self.views.extend_from_slice(&off.to_le_bytes());
                    self.data.extend_from_slice(bytes);
                }
                self.validity.push(true);
            }
            None => {
                self.views.extend_from_slice(&0u32.to_le_bytes());
                self.views.extend_from_slice(&[0u8; 12]);
                self.validity.push(false);
            }
        }
    }

    /// Freeze: copy to 64-byte aligned buffers.
    #[must_use]
    pub fn finish(&self) -> ViewVec {
        let n = self.validity.len();
        let mut views = Buf::new_zeroed(self.views.len());
        views.as_mut_slice().copy_from_slice(&self.views);
        let mut data = Buf::new_zeroed(self.data.len());
        data.as_mut_slice().copy_from_slice(&self.data);
        let validity = Bitmap::from_bools(&self.validity);
        ViewVec {
            validity,
            views,
            data,
            len: n,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branchless_inline_classification() {
        assert_eq!(is_inline(0), 1);
        assert_eq!(is_inline(12), 1);
        assert_eq!(is_inline(13), 0);
        assert_eq!(is_inline(4096), 0);
    }

    #[test]
    fn inline_and_foreign_rows() {
        let mut b = ViewVecBuilder::new();
        b.push(Some(b"short"));
        b.push(Some(b"this-one-is-longer-than-twelve"));
        b.push(None);
        let v = b.finish();
        assert_eq!(v.len(), 3);
        assert_eq!(v.row(0).expect("row 0"), b"short");
        assert_eq!(v.row(1).expect("row 1"), b"this-one-is-longer-than-twelve");
        assert!(!v.validity().unwrap().get(2));
        // descriptor count
        assert_eq!(v.views_bytes().len(), 3 * 16);
        // foreign payload landed in the data block
        assert_eq!(v.data().len(), 30);
    }

    /// Corrupt-input court (negative leg): a hostile descriptor claiming a
    /// payload range past the data buffer refuses typed, never panics.
    #[test]
    fn hostile_foreign_descriptor_refuses_typed() {
        let mut b = ViewVecBuilder::new();
        b.push(Some(b"a-payload-longer-than-twelve-bytes"));
        let mut v = b.finish();
        // Tamper descriptor 0's offset word (bytes 12..16) to point past
        // the data block.
        let data_len = v.data().len();
        v.views.as_mut_slice()[12..16].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(
            v.row(0),
            Err(ViewError::OutOfRange {
                off: u32::MAX as usize,
                len: "a-payload-longer-than-twelve-bytes".len(),
                data_len,
            }),
            "mutant kill: zero the range guard and this panics instead"
        );
    }

    /// Corrupt-input court (negative leg): the reachable hostile edge on an
    /// all-inline vector is the row index (inline payloads are structurally
    /// bounded — len <= 12 always fits the 16-byte descriptor).
    #[test]
    fn hostile_row_index_refuses_typed() {
        let mut b = ViewVecBuilder::new();
        b.push(Some(b"tiny"));
        let v = b.finish();
        assert_eq!(
            v.row(5),
            Err(ViewError::RowPastEnd { i: 5, rows: 1 }),
            "mutant kill: delete the i >= rows guard and this panics"
        );
    }
}
