//! AoqWriter — zero-alloc 5-segment scatter writer (AB-8, PG-A2).
//!
//! SEGMENT LAW: one frame = five contiguous segments, scatterable straight
//! into a `writev` sink with zero copies of the caller body:
//! 0. prefix: continuation word + `meta_span` word (8 B, w64 span 120)
//! 1. header: the 64-byte sealed AOQ1 header (encoded via
//!    [`crate::codec::encode`] — the landed codec seals the FNV-1a32
//!    digest over `[0x00,0x3C)` at `0x3C`; never re-declared here)
//! 2. metadata padding (56 B): `body_len` u32 LE + the 16-byte tag
//!    + zero tail
//! 3. body: the caller's bytes BY REFERENCE — the writer never copies or
//!    owns them
//! 4. tail: body pad (to 64) + the 8-byte EOS (continuation + zero)
//!
//! PROGRESS LAW (ZeroProgress carry): the writer carries `(ordinal, seq)`
//! and advances ONLY on [`AoqWriter::commit`] after the sink accepted all
//! five segments — a refused or partial write leaves the cursor untouched.
//!
//! LOCKSTEP LAW: ordinal advances by exactly 1; seq advances by exactly 1
//! wrapping at the u32 boundary (same pair law as cgap-arrow `ipc::seq_ok`).
//!
//! TAG LAW: caller-supplied (R-K1 — no crypto in this crate), TAMPER-tier
//! `blake3(header[0x00..0x3C) || body)[..16]` by convention with
//! `verify.rs`.

use crate::codec;
use crate::error::AoqError;
use crate::header::{AoqHeader, SIZE_BYTES};

/// IPC continuation marker.
pub const CONTINUATION: u32 = 0xFFFF_FFFF;
/// Metadata span word for w64 AOQ1 windows.
pub const META_SPAN_W64: u32 = 120;
/// EOS tail word (the metadata_size slot carries zero).
pub const EOS: u32 = 0x0000_0000;
/// Metadata padding width at w64 (span 120 - header 64).
pub const PAD_BYTES: usize = 56;
/// EOS width in the tail.
pub const EOS_BYTES: usize = 8;
/// Total writer scratch: prefix 8 + header 64 + meta pad 56 + EOS 8 = 136.
/// The BODY pad (`pad64(body_len)`, per the landed IPC window law) is NOT
/// scratch-resident: `write_window` zero-fills it inline into the output
/// window, and writev callers scatter their own zero buffer sized by
/// [`Segments::body_pad`].
pub const SCRATCH_BYTES: usize = 8 + SIZE_BYTES + PAD_BYTES + EOS_BYTES;
/// Scratch offsets, pinned.
pub const PREFIX_AT: usize = 0;
/// Scratch offset of the header slot.
pub const HEADER_AT: usize = 8;
/// Scratch offset of the padding slot.
pub const PAD_AT: usize = HEADER_AT + SIZE_BYTES;
/// Scratch offset of the EOS slot.
pub const EOS_AT: usize = PAD_AT + PAD_BYTES;
/// Tag width (mirrors `verify::TAG_BYTES`; pinned by court A13).
pub const TAG_BYTES: usize = 16;
/// Body ceiling: the `body_len` padding word is u32.
pub const MAX_BODY_LEN: usize = u32::MAX as usize;

/// `n` padded to 64.
const fn pad64(n: usize) -> usize {
    (n + 63) & !63
}

/// The five scatter segments of one frame (writev-ready, body by
/// reference). Stack-shaped segments borrow the writer scratch; the body
/// segment borrows the caller's bytes.
#[derive(Debug)]
pub struct Segments<'s, 'a> {
    /// Segment 0: continuation + span (8 B).
    pub prefix: &'s [u8],
    /// Segment 1: sealed 64-byte header.
    pub header: &'s [u8],
    /// Segment 2: metadata padding (56 B).
    pub pad: &'s [u8],
    /// Segment 3: caller body, by reference (never copied by the writer).
    pub body: &'a [u8],
    /// Segment 4: the 8-byte EOS.
    pub tail: &'s [u8],
}

impl<'s, 'a> Segments<'s, 'a> {
    /// Body-pad width (window law: `pad64(body_len)`); writev callers
    /// scatter a zero buffer of this length between `body` and `tail`.
    #[must_use]
    pub fn body_pad(&self) -> usize {
        pad64(self.body.len())
    }
}

impl<'s, 'a> Segments<'s, 'a> {
    /// Total wire bytes the five segments carry.
    #[must_use]
    pub fn total_len(&self) -> usize {
        self.prefix.len()
            + self.header.len()
            + self.pad.len()
            + self.body.len()
            + self.body_pad()
            + self.tail.len()
    }
}

/// Zero-alloc AOQ1 frame writer: 200 B inline scratch, carries the
/// seq/ordinal cursor, advances only on [`AoqWriter::commit`].
#[derive(Debug, Clone, Copy)]
pub struct AoqWriter {
    next_ordinal: u32,
    next_seq: u32,
    scratch: [u8; SCRATCH_BYTES],
}

impl Default for AoqWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl AoqWriter {
    /// Writer at the genesis cursor (ordinal 0, seq 0).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            next_ordinal: 0,
            next_seq: 0,
            scratch: [0u8; SCRATCH_BYTES],
        }
    }

    /// The next (ordinal, seq) the writer will stamp.
    #[must_use]
    pub const fn cursor(&self) -> (u32, u32) {
        (self.next_ordinal, self.next_seq)
    }

    /// Stage the five segments for one frame. The cursor does NOT advance
    /// here — call [`AoqWriter::commit`] after the sink accepted every
    /// segment (ZeroProgress law).
    ///
    /// # Errors
    /// [`AoqError::BufferTooShort`] when `body` exceeds the u32 `body_len`
    /// padding word (documented overflow-class representative).
    #[allow(clippy::missing_panics_doc)] // slicing at pinned consts, never panics
    pub fn frame_segments<'s, 'a>(
        &mut self,
        epoch: u16,
        key_id: u64,
        tag: &[u8; TAG_BYTES],
        body: &'a [u8],
    ) -> Result<Segments<'_, 'a>, AoqError> {
        if body.len() > MAX_BODY_LEN {
            return Err(AoqError::BufferTooShort { got: u16::MAX });
        }
        let header = AoqHeader::new(epoch, key_id, self.next_ordinal, self.next_seq);
        // Segment 0: prefix (continuation + span).
        self.scratch[PREFIX_AT..PREFIX_AT + 4].copy_from_slice(&CONTINUATION.to_le_bytes());
        self.scratch[PREFIX_AT + 4..PREFIX_AT + 8].copy_from_slice(&META_SPAN_W64.to_le_bytes());
        // Segment 1: sealed header via the landed codec.
        codec::encode(
            &header,
            &mut self.scratch[HEADER_AT..HEADER_AT + SIZE_BYTES],
        )?;
        // Segment 2: pad = body_len u32 LE [0,4) + tag [4,20) + zero tail.
        // Pad segment byte j is window offset 72 + j (pin court A13).
        self.scratch[PAD_AT..PAD_AT + PAD_BYTES].fill(0);
        self.scratch[PAD_AT..PAD_AT + 4].copy_from_slice(&(body.len() as u32).to_le_bytes());
        self.scratch[PAD_AT + 4..PAD_AT + 4 + TAG_BYTES].copy_from_slice(tag);
        // Segment 4: EOS (the body pad is not scratch-resident).
        let eos_at = EOS_AT;
        self.scratch[eos_at..eos_at + 4].copy_from_slice(&CONTINUATION.to_le_bytes());
        self.scratch[eos_at + 4..eos_at + 8].copy_from_slice(&EOS.to_le_bytes());
        Ok(Segments {
            prefix: &self.scratch[PREFIX_AT..PREFIX_AT + 8],
            header: &self.scratch[HEADER_AT..HEADER_AT + SIZE_BYTES],
            pad: &self.scratch[PAD_AT..PAD_AT + PAD_BYTES],
            body,
            tail: &self.scratch[EOS_AT..EOS_AT + EOS_BYTES],
        })
    }

    /// Advance the cursor by one frame (ordinal +1, seq +1 wrapping).
    /// Lawful only after the sink accepted all five segments.
    pub fn commit(&mut self) {
        self.next_ordinal += 1;
        self.next_seq = self.next_seq.wrapping_add(1);
    }

    /// Convenience: scatter the five segments into `out` (one contiguous
    /// window) and commit. Returns the window length.
    ///
    /// # Errors
    /// [`AoqError::BufferTooShort`] when `out` is shorter than the window
    /// (refused before ANY byte is written — ZeroProgress law), overflow
    /// class as in [`AoqWriter::frame_segments`].
    pub fn write_window(
        &mut self,
        out: &mut [u8],
        epoch: u16,
        key_id: u64,
        tag: &[u8; TAG_BYTES],
        body: &[u8],
    ) -> Result<usize, AoqError> {
        let segs = self.frame_segments(epoch, key_id, tag, body)?;
        let total = segs.total_len();
        if out.len() < total {
            return Err(AoqError::BufferTooShort { got: u16::MAX });
        }
        let mut w = 0usize;
        let put = |dst: &mut [u8], src: &[u8], w: &mut usize| {
            dst[*w..*w + src.len()].copy_from_slice(src);
            *w += src.len();
        };
        put(out, segs.prefix, &mut w);
        put(out, segs.header, &mut w);
        put(out, segs.pad, &mut w);
        put(out, segs.body, &mut w);
        // Body pad: zero-filled inline (never scratch-resident).
        out[w..w + segs.body_pad()].fill(0);
        w += segs.body_pad();
        put(out, segs.tail, &mut w);
        self.commit();
        Ok(total)
    }
}

// Scratch-layout const court: the meta-pad segment covers window offsets
// [72, 128) and the EOS slot starts at scratch offset 128.
const _: () = {
    assert!(PREFIX_AT == 0);
    assert!(HEADER_AT == 8);
    assert!(PAD_AT == 72);
    assert!(EOS_AT == 128);
    assert!(SCRATCH_BYTES == 136);
};
