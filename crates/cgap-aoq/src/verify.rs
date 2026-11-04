//! AOQ1 window verification — verify-then-decrypt seam (AB-8, PG-A2).
//!
//! Wire law (PG-A2 §2 per-magic, landed by this file + `writer.rs`):
//! an AOQ1 window reuses the IPC prefix framing (continuation word,
//! `meta_span` word) with the 64-byte AOQ1 header in the metadata slot.
//! The header field table is NEVER re-declared here: header law is
//! composed through [`crate::codec`] / [`crate::header`] (AB-6, R-1).
//! The metadata padding carries, per-magic:
//! - `body_len` u32 LE at window offset 72 (`BODY_LEN_AT`)
//! - auth tag, 16 bytes at window offset 76 (`TAG_AT`)
//! - remaining pad zero on write, nonzero-pad refused on read
//!   (`AoqError::ReservedNotZero`, pad-class)
//! Verified AOQ1 windows are alignment-64 only: a 64-byte metadata span
//! (alignment 8) has no padding, so no tag/body_len can be carried; such
//! windows refuse at the span gate.
//!
//! TAG LAW: the tag is a TAMPER-tier truncation
//! `blake3(header[0x00..0x3C) || body)[..16]` — the same tier as
//! cgap-arrow's body-digest path (`ipc::emit_verified`). This crate ships
//! no crypto (R-K1): the tag is SUPPLIED by the host (caller parameter)
//! and compared, never computed here. `decrypt_in_place` likewise applies
//! a caller-derived keystream (host KDF output), so the cipher body stays
//! cgap-crypto/cgap-blake3 territory end to end.
//!
//! VERIFY-THEN-DECRYPT BY TYPE: every gate in [`verify_window`] takes
//! `&[u8]` only; [`decrypt_in_place`] takes `&mut [u8]` and is lawful to
//! call only downstream of a `verify_window` `Ok`. Every refusal leaves
//! the window bytes untouched (no-plaintext-on-error law, court A12).

use crate::codec;
use crate::error::AoqError;
use crate::header::AoqHeader;

/// Auth tag width in bytes.
pub const TAG_BYTES: usize = 16;
/// Window offset of the `body_len` word (u32 LE) in the metadata padding.
pub const BODY_LEN_AT: usize = 72;
/// Window offset of the auth tag in the metadata padding.
pub const TAG_AT: usize = 76;
/// IPC continuation marker (shared prefix law with cgap-arrow `ipc`).
pub const CONTINUATION: u32 = 0xFFFF_FFFF;
/// Metadata span for alignment-64 AOQ1 windows (prefix 8 + 128 rounded).
pub const META_SPAN_W64: u32 = 120;
/// Metadata span for alignment-8 framing (no padding: no tag carrier).
pub const META_SPAN_W8: u32 = 64;
/// Header slot offset in a window (prefix size).
pub const HEADER_AT: usize = 8;

/// A verified AOQ1 window: the decoded header plus everything the body
/// phase needs. Returned only after every gate has passed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifiedWindow {
    /// Decoded, digest-verified AOQ1 header.
    pub header: AoqHeader,
    /// Offset of the body in the window buffer.
    pub body_at: usize,
    /// Body length in bytes (from the padding word, u32 LE).
    pub body_len: usize,
    /// The 16-byte tag carried in the padding.
    pub tag: [u8; TAG_BYTES],
}

impl VerifiedWindow {
    /// Window length implied by `body_len` at alignment 64.
    #[must_use]
    pub const fn window_len(&self) -> usize {
        8 + META_SPAN_W64 as usize
            + self.body_len
            + pad64(self.body_len)
            + 8
    }
}

/// `n` padded to 64 (alignment-64 body pad).
const fn pad64(n: usize) -> usize {
    (n + 63) & !63
}

/// Verify one AOQ1 window: every framing and header-law gate, on `&[u8]`
/// only. Gate order mirrors cgap-arrow `ipc::parse_meta` (PG-6 order):
/// truncation -> prefix word -> span word -> truncation-to-header ->
/// header law (magic/version/epoch/digest via [`codec::decode`]) ->
/// seq/ordinal cursor lockstep -> pad cleanliness + body_len/tag read.
///
/// Framing-class mapping (R-H2: IpcError reuse at the cgap-arrow seam;
/// this crate speaks header law only — see `tests/ab8_sweep.rs` for the
/// classification-identity court pairing these classes):
/// - window shorter than the header slot: [`AoqError::HeaderTooShort`]
/// - continuation word wrong: [`AoqError::BadMagic`] (leading-word class)
/// - span word not the w64 span: [`AoqError::HeaderTooShort`] (span class:
///   a w64-span window is the shortest lawful verified window)
/// - pad bytes nonzero outside the body_len/tag law region:
///   [`AoqError::ReservedNotZero`]
/// - tag absent (all zero): [`AoqError::DigestMismatch { expected: 0, got: 0 }`]
///   (absent-vs-mismatch collapse; cgap-arrow's `IpcError::TagMissing`
///   carries the finer class on that surface)
///
/// # Errors
/// Typed [`AoqError`] refusals, integer fields only; `buf` never mutated.
pub fn verify_window(
    buf: &[u8],
    expected_ordinal: u32,
    expected_seq: u32,
    expected_tag: &[u8; TAG_BYTES],
) -> Result<VerifiedWindow, AoqError> {
    // 1-4: prefix/span shape (w64 only).
    if buf.len() < HEADER_AT + crate::header::SIZE_BYTES {
        return Err(AoqError::HeaderTooShort { got: buf.len() as u16 });
    }
    let cont = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
    if cont != CONTINUATION {
        return Err(AoqError::BadMagic { got: cont });
    }
    let span = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
    if span != META_SPAN_W64 {
        return Err(AoqError::HeaderTooShort { got: span as u16 });
    }
    let body_at = HEADER_AT + META_SPAN_W64 as usize;
    if buf.len() < body_at + crate::header::SIZE_BYTES {
        return Err(AoqError::HeaderTooShort { got: buf.len() as u16 });
    }
    // 5: header law — composed through the landed codec, never re-declared.
    let header = codec::decode(&buf[HEADER_AT..body_at])?;
    // 6: seq/ordinal cursor lockstep (caller-owned cursor, PG-A2 law).
    if header.seq != expected_seq {
        return Err(AoqError::SeqGap { expected: expected_seq, got: header.seq });
    }
    if header.frame_ordinal != expected_ordinal {
        return Err(AoqError::FrameOrdinalRegression {
            expected: expected_ordinal,
            got: header.frame_ordinal,
        });
    }
    // 7: pad cleanliness + body_len/tag read.
    let mut body_len_bytes = [0u8; 4];
    body_len_bytes.copy_from_slice(&buf[BODY_LEN_AT..BODY_LEN_AT + 4]);
    let body_len = u32::from_le_bytes(body_len_bytes) as usize;
    let mut tag = [0u8; TAG_BYTES];
    tag.copy_from_slice(&buf[TAG_AT..TAG_AT + TAG_BYTES]);
    for (i, &b) in buf[HEADER_AT + crate::header::SIZE_BYTES..body_at]
        .iter()
        .enumerate()
    {
        let in_law_region = HEADER_AT + crate::header::SIZE_BYTES + i >= BODY_LEN_AT
            && HEADER_AT + crate::header::SIZE_BYTES + i < TAG_AT + TAG_BYTES;
        if b != 0 && !in_law_region {
            return Err(AoqError::ReservedNotZero);
        }
    }
    // 8: tag presence + match (host supplies the expected tag; R-K1).
    if tag == [0u8; TAG_BYTES] {
        return Err(AoqError::DigestMismatch { expected: 0, got: 0 });
    }
    if &tag != expected_tag {
        return Err(AoqError::DigestMismatch {
            expected: u32::from_le_bytes([expected_tag[0], expected_tag[1], expected_tag[2], expected_tag[3]]),
            got: u32::from_le_bytes([tag[0], tag[1], tag[2], tag[3]]),
        });
    }
    // 9: full window law — body pad zero, EOS present (classification
    // identity with the cgap-arrow framing surface, court ab8_sweep).
    let body_pad = pad64(body_len);
    if buf.len() < body_at + body_len + body_pad + 8 {
        return Err(AoqError::HeaderTooShort { got: buf.len() as u16 });
    }
    // [AB-1 SLA MINIMAL UNBLOCK 2026-11-03] enumerate-destructure pattern
    // fix; owning lane may restyle. Disclosed in ab-1.md.
    for (_i, &b) in buf[body_at + body_len..body_at + body_len + body_pad]
        .iter()
        .enumerate()
    {
        if b != 0 {
            return Err(AoqError::ReservedNotZero);
        }
    }
    let eos_at = body_at + body_len + body_pad;
    let eos_cont = u32::from_le_bytes([
        buf[eos_at],
        buf[eos_at + 1],
        buf[eos_at + 2],
        buf[eos_at + 3],
    ]);
    let eos_tail = u32::from_le_bytes([
        buf[eos_at + 4],
        buf[eos_at + 5],
        buf[eos_at + 6],
        buf[eos_at + 7],
    ]);
    if eos_cont != CONTINUATION || eos_tail != 0 {
        return Err(AoqError::HeaderTooShort { got: buf.len() as u16 });
    }
    Ok(VerifiedWindow { header, body_at, body_len, tag })
}

/// Apply the caller-derived keystream to the body IN PLACE. Lawful only
/// downstream of [`verify_window`] `Ok` (verify-then-decrypt by type:
/// the gates above run on `&[u8]`; this is the `&mut [u8]` phase).
///
/// # Errors
/// [`AoqError::HeaderTooShort`] when `keystream` is shorter than the body
/// — refused BEFORE any mutation, so the ciphertext is never half-keystreamed
/// (no-plaintext-on-error law, court A12).
pub fn decrypt_in_place(
    window: &VerifiedWindow,
    keystream: &[u8],
    buf: &mut [u8],
) -> Result<(), AoqError> {
    let body_at = window.body_at;
    let body_len = window.body_len;
    if buf.len() < body_at + body_len {
        return Err(AoqError::HeaderTooShort { got: buf.len() as u16 });
    }
    if keystream.len() < body_len {
        return Err(AoqError::HeaderTooShort { got: keystream.len() as u16 });
    }
    for i in 0..body_len {
        buf[body_at + i] ^= keystream[i];
    }
    Ok(())
}
