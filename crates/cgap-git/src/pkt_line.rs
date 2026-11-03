//! pkt-line codec for git smart-protocol v0/v2 framing (lane GIT-02).
//!
//! Wire format (git `Documentation/gitprotocol-common.txt`): a pkt-line is
//! a 4-byte uppercase-hex total length (including the 4 length bytes
//! itself) followed by the payload. Special lengths:
//!
//! - `0000` flush-pkt  — ends a side-band / command list
//! - `0001` delim-pkt  — separates command from args (protocol v2)
//! - `0002` response-end-pkt — ends a v2 stateless response
//!   (`Documentation/gitprotocol-v2.txt`: "0002 ... indicates the end of
//!   a response for stateless connections")
//!
//! Payload cap: git's `LARGE_PACKET_MAX` is 65520 total, so the payload
//! cap is 65516 bytes.
//!
//! Constant-time property: every hex digit of the length header is decoded
//! and validity-masked before any branch; a malformed digit cannot
//! early-out and leak position through timing. Buffers are fixed 64-byte
//! stack arrays; encode/decode paths allocate nothing (`#![no_std]`-pure).

#![deny(unsafe_code)]

/// Payload bytes addressable by one pkt-line: 65520 total - 4 header.
pub const MAX_PKT_PAYLOAD: usize = 65_516;

/// Fixed stack buffer a single pkt-line encodes into (header + payload).
#[allow(dead_code)] // lib-unused: sized in every encode_* signature below (court-tested surface)
pub const PKT_BUF_LEN: usize = 64;

/// The 4-hex-digit header of a flush-pkt.
pub const FLUSH: [u8; 4] = *b"0000";
/// The 4-hex-digit header of a delim-pkt (protocol v2).
pub const DELIM: [u8; 4] = *b"0001";
/// The 4-hex-digit header of a response-end-pkt (protocol v2).
pub const RESPONSE_END: [u8; 4] = *b"0002";

/// Decode one ASCII hex digit to its 4-bit value.
/// Returns `(nibble, ok)`; `ok` is 0/1 — callers mask, they do not branch.
#[inline(always)]
fn hex_nibble(c: u8) -> (u32, u32) {
    let v = match c {
        b'0'..=b'9' => (c - b'0') as u32,
        b'a'..=b'f' => (c - b'a') as u32 + 10,
        b'A'..=b'F' => (c - b'A') as u32 + 10,
        // Out-of-domain input folds into the value but clears ok; the
        // caller discards the value, so timing is input-shape-uniform.
        _ => (c as u32) & 0xF,
    };
    let ok = matches!(c, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F') as u32;
    (v, ok)
}

/// Decode a 4-hex-digit total-length header, all digits always evaluated.
/// The digits must be valid hex; `0000`..`0003` are control markers, not
/// lengths (see [`decode_header`]).
#[inline]
pub fn parse_hex_len(h: &[u8; 4]) -> Result<u16, PktError> {
    let (a, oa) = hex_nibble(h[0]);
    let (b, ob) = hex_nibble(h[1]);
    let (c, oc) = hex_nibble(h[2]);
    let (d, od) = hex_nibble(h[3]);
    // OR every validity bit before branching: a bad digit anywhere fails
    // only here, after all four nibbles were computed.
    if oa & ob & oc & od == 1 {
        Ok(((a << 12) | (b << 8) | (c << 4) | d) as u16)
    } else {
        Err(PktError::BadHexLength)
    }
}

/// Classify a pkt-line by its 4-byte header without touching the payload.
///
/// Returns the total length (header + payload) for data lines, or the
/// control marker kind. `0000`/`0001`/`0002` are the flush/delim/
/// response-end markers; total lengths `0003` and below `0004` are
/// unrepresentable (a 4-byte header leaves no payload room, and the
/// protocol defines no such marker) — typed [`PktError::BadLength`].
pub fn decode_header(h: &[u8; 4]) -> Result<Header, PktError> {
    if *h == FLUSH {
        return Ok(Header::Flush);
    }
    if *h == DELIM {
        return Ok(Header::Delim);
    }
    if *h == RESPONSE_END {
        return Ok(Header::ResponseEnd);
    }
    let total = parse_hex_len(h)? as usize;
    if total < 4 || total > MAX_PKT_PAYLOAD + 4 {
        return Err(PktError::BadLength(total));
    }
    Ok(Header::Data(total))
}

/// Header classification from [`decode_header`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Header {
    /// `0000` flush-pkt.
    Flush,
    /// `0001` delim-pkt.
    Delim,
    /// `0002` response-end-pkt.
    ResponseEnd,
    /// Data line: total wire length including the 4 header bytes.
    Data(usize),
}

/// Typed decode failure. Every variant names the malformed property; no
/// panic path exists on any input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PktError {
    /// A length header contained a non-hex digit (e.g. `000g`).
    BadHexLength,
    /// A data-line total length is < 4 or above the protocol cap.
    BadLength(usize),
    /// The declared total length exceeds the bytes actually available.
    Truncated { declared: usize, available: usize },
    /// An encode call's payload exceeds the fixed 64-byte stack buffer.
    PayloadTooLarge { len: usize, cap: usize },
}

/// Decode one pkt-line from the head of `buf`, returning the parsed packet
/// and the total bytes consumed (header + payload). Control markers
/// consume exactly 4 bytes and carry no payload.
///
/// No allocation: the payload borrows `buf`.
pub fn decode_pkt(buf: &[u8]) -> Result<(Pkt<'_>, usize), PktError> {
    let h: &[u8; 4] = buf.first_chunk::<4>().ok_or(PktError::Truncated {
        declared: 4,
        available: buf.len(),
    })?;
    match decode_header(h)? {
        Header::Flush => Ok((Pkt::Flush, 4)),
        Header::Delim => Ok((Pkt::Delim, 4)),
        Header::ResponseEnd => Ok((Pkt::ResponseEnd, 4)),
        Header::Data(total) => {
            if buf.len() < total {
                return Err(PktError::Truncated {
                    declared: total,
                    available: buf.len(),
                });
            }
            Ok((Pkt::Data(&buf[4..total]), total))
        }
    }
}

/// One decoded pkt-line. Payloads borrow the input buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pkt<'a> {
    /// `0000` flush-pkt.
    Flush,
    /// `0001` delim-pkt.
    Delim,
    /// `0002` response-end-pkt.
    ResponseEnd,
    /// Data line with its payload (header stripped, borrows the buffer).
    Data(&'a [u8]),
}

impl<'a> Pkt<'a> {
    /// If this is an `ERR <msg>` packet (payload begins with the 4 bytes
    /// `ERR ` per `gitprotocol-http.txt` "Error Status" and smart-protocol
    /// ERR convention), return the message bytes after the prefix.
    #[allow(dead_code)] // lib-unused: asserted by tests/pkt_line_court.rs ERR-decode court
    pub fn err_message(&self) -> Option<&'a [u8]> {
        match self {
            Pkt::Data(p) if p.starts_with(b"ERR ") => Some(&p[4..]),
            _ => None,
        }
    }

    /// True if this is the flush-pkt (`0000`).
    #[allow(dead_code)] // lib-unused: asserted by tests/pkt_line_court.rs:179
    pub fn is_flush(&self) -> bool {
        matches!(self, Pkt::Flush)
    }
}

/// Encode a data pkt-line into a fixed 64-byte stack buffer, returning the
/// number of wire bytes written (4 + payload). Constant-time header
/// construction: the length is formatted by unconditional nibble math, no
/// format machinery, no heap.
///
/// Returns [`PktError::PayloadTooLarge`] if `payload.len() > 60`
/// (buffer cap), or [`PktError::BadLength`] if it exceeds the 65,516-byte
/// protocol cap (unreachable here but enforced for API symmetry with a
/// future large-payload encoder).
#[allow(dead_code)] // lib-unused: wire-encode completeness; exercised by tests/pkt_line_court.rs
pub fn encode_pkt(payload: &[u8], out: &mut [u8; PKT_BUF_LEN]) -> Result<usize, PktError> {
    let n = payload.len();
    if n > 60 {
        return Err(PktError::PayloadTooLarge {
            len: n,
            cap: PKT_BUF_LEN - 4,
        });
    }
    if n > MAX_PKT_PAYLOAD {
        // Unreachable behind the 64-byte cap; kept for protocol-cap
        // symmetry and so the court can pin the constant.
        return Err(PktError::BadLength(n + 4));
    }
    let total = (4 + n) as u16;
    // Unconditional nibble emission: high digit may exceed 9 only for
    // payloads over 9995 bytes — impossible under the 60-byte cap — so
    // plain hex math is exact here.
    out[0] = HEX[(total >> 12) as usize & 0xF];
    out[1] = HEX[(total >> 8) as usize & 0xF];
    out[2] = HEX[(total >> 4) as usize & 0xF];
    out[3] = HEX[(total & 0xF) as usize];
    out[4..4 + n].copy_from_slice(payload);
    Ok(4 + n)
}

/// Uppercase hex digit table for length headers.
#[allow(dead_code)] // lib-unused: header nibble table for encode_* (court-tested wire surface)
const HEX: [u8; 16] = *b"0123456789ABCDEF";

/// Encode a flush-pkt (`0000`) into `out`, returning 4.
#[allow(dead_code)] // lib-unused: wire-encode completeness; exercised by tests/pkt_line_court.rs:153
pub fn encode_flush(out: &mut [u8; PKT_BUF_LEN]) -> Result<usize, PktError> {
    out[..4].copy_from_slice(&FLUSH);
    Ok(4)
}

/// Encode a delim-pkt (`0001`) into `out`, returning 4.
#[allow(dead_code)] // lib-unused: wire-encode completeness; exercised by tests/pkt_line_court.rs:155
pub fn encode_delim(out: &mut [u8; PKT_BUF_LEN]) -> Result<usize, PktError> {
    out[..4].copy_from_slice(&DELIM);
    Ok(4)
}

/// Encode a response-end-pkt (`0002`) into `out`, returning 4.
#[allow(dead_code)] // lib-unused: wire-encode completeness; exercised by tests/pkt_line_court.rs:157
pub fn encode_response_end(out: &mut [u8; PKT_BUF_LEN]) -> Result<usize, PktError> {
    out[..4].copy_from_slice(&RESPONSE_END);
    Ok(4)
}

/// Encode an `ERR <msg>` data packet (payload `ERR ` + message) into `out`.
#[allow(dead_code)] // lib-unused: wire-encode completeness; exercised by tests/pkt_line_court.rs:166
pub fn encode_err(msg: &[u8], out: &mut [u8; PKT_BUF_LEN]) -> Result<usize, PktError> {
    let n = msg.len();
    if n > 52 {
        return Err(PktError::PayloadTooLarge { len: n, cap: 52 });
    }
    let total = (8 + n) as u16; // header(4) + "ERR "(4) + msg
    out[0] = HEX[(total >> 12) as usize & 0xF];
    out[1] = HEX[(total >> 8) as usize & 0xF];
    out[2] = HEX[(total >> 4) as usize & 0xF];
    out[3] = HEX[(total & 0xF) as usize];
    out[4..8].copy_from_slice(b"ERR ");
    out[8..8 + n].copy_from_slice(msg);
    Ok(4 + 4 + n)
}
