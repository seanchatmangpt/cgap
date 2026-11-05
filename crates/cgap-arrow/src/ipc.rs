//! Arrow IPC-style encapsulated-message envelope (lane p1-ipc, v26.11.2).
//!
//! Wire layout per the Apache Arrow IPC streaming format ("Encapsulated
//! message" section of format/Message.fbs + format/IPCStreaming.rst):
//!
//! ```text
//! <continuation: 0xFFFFFFFF>
//! <metadata_size: u32 LE>
//! <metadata, zero-padded so the body starts `alignment`-aligned>
//! <body, zero-padded to `alignment`>
//! <end-of-stream: 0xFFFFFFFF 0x00000000>
//! ```
//!
//! SCOPE DIVERGENCE (recorded, not hidden): the real IPC `metadata` is an
//! Arrow flatbuffer `Message` (Schema / RecordBatch / ...). Full flatbuffer
//! read/write is DEFERRED. This module frames a 64-byte cgap metadata
//! header whose schema identity is the cgap O-ABI schema digest (32 bytes,
//! donor `crate::blake3`) instead of a flatbuffer Schema. An external
//! Arrow IPC reader is expected to:
//!   - accept the continuation / metadata_size / alignment framing (spec
//!     sections "Encapsulated message" / "Stream format"), and
//!   - REFUSE the metadata payload (an unknown flatbuffer identifier is a
//!     typed parse failure there, never a misread).
//!     Implemented: byte-exact framing, alignment 8 or 64, typed refusal of
//!     every corrupt framing class (continuation, metadata size, magic, header
//!     digest, schema digest, body length, alignment field, nonzero padding,
//!     truncation, missing EOS). A future lane may swap the flatbuffer in
//!     behind the same emit/read signatures.
//!
//! WITNESS TIERS (G3, CG-1 v26.11.3; PG-6 decision: NO shared digest
//! seam). The fleet's wire surfaces use deliberately different digests and
//! they must not be confused:
//! - `wire-shm` CRC-32 = TRANSPORT tier: accidental bit-flip detection on
//!   a shared-memory ring; a checksum, forgeable by an adversary —
//!   correct for its threat model.
//! - this module's blake3 = TAMPER tier: adversarial body-integrity on the
//!   verified path ([`read_verified`]).
//! - plain [`read`] is NOT a verifying tier: it never re-hashes the body
//!   (see BODY INTEGRITY CONTRACT above); body integrity is the
//!   consumer's duty on that path.
//!   Anti-confusion is courted, not narrated: `tests/ipc_courts.rs` C13
//!   asserts zero checksum-tier contamination in this file's source, and
//!   wire-shm's court asserts zero "blake3" in its sources, each with a
//!   sandbox-add-token anti-vacuity kill.
//!
//! Zero external deps, `no_std`, no `alloc`: emit/read operate on caller
//! buffers, so the module is allocation-free and panic-free on inputs.
//!
//! BODY INTEGRITY CONTRACT (v26.11.3, CL-L finding #1 / witness W4): the
//! framing layer's [`read`] never re-hashes the body — a body-corrupted
//! window passes a digest-bearing read, and body integrity is the
//! consumer's duty on that path. [`emit_verified`]/[`read_verified`] raise
//! body integrity into the framing layer (blake3 digest carried in the
//! alignment-64 metadata padding, which `read` never inspects); windows
//! from plain [`emit`] refuse on `read_verified` with
//! [`IpcError::BodyDigestMissing`]. Compat window: both paths remain
//! published, legacy `read`/`emit` unchanged, existing courts green.

use core::fmt;

/// ARRO fb-window magic (the Message flatbuffer's 4-byte file identifier,
/// seen LE as a magic word for the parse fork). Synthesized MetaHeaders on
/// the fb branch carry it so verified paths dispatch per magic.
pub const FB_MAGIC: u32 = u32::from_le_bytes(*b"ARRO");

/// The fb-branch parse result.
#[derive(Clone, Copy)]
pub struct FbParsed<'a> {
    /// Parsed Message envelope (borrows the window).
    pub message: crate::flatbuf::MessageView<'a>,
    /// blake3 over the flatbuffer region (the `expected_schema` seam
    /// value on the fb read paths).
    pub schema_digest: [u8; 32],
    /// cgap:schema_blake3 carried value (None when absent).
    pub schema_blake3: Option<[u8; 32]>,
    /// cgap:body_blake3 carried value (None when absent).
    pub body_blake3: Option<[u8; 32]>,
    /// cgap:rows carried value (None when absent or non-decimal — never
    /// guessed).
    pub rows: Option<u64>,
}

/// Continuation marker (0xFFFFFFFF), Message.fbs `Continuation`.
pub const CONTINUATION: u32 = 0xFFFF_FFFF;
/// End-of-stream marker (metadata_size slot carries 0x00000000).
pub const EOS: u32 = 0x0000_0000;
/// Metadata header magic `"IPC1"`.
pub const MAGIC: u32 = 0x4950_4331;
/// AOQ1 header magic, `"AOQ1"` (PG-A2 §2 per-magic fork; landed ontology
/// `packs/aoq-pack/ontology.ttl`). The parse fork below carries the
/// per-magic law: big-endian header, version 2, FNV-1a32 digest over
/// `[0x00,0x3C)` stored at `0x3C`, closed epoch enum in the flags-word
/// position, `body_len` u32 LE + 16-byte tag in the metadata padding.
pub const AOQ1_MAGIC: u32 = 0x414F_5131;
/// AOQ1 tag offset in the window (`PREFIX_SIZE + 64 + 4`): the padding
/// carries `body_len` u32 LE at +0 and the tag at +4.
pub const AOQ1_TAG_AT: usize = PREFIX_SIZE + 64 + 4;
/// AOQ1 tag width.
pub const AOQ1_TAG_BYTES: usize = 16;
/// Envelope layout version.
pub const LAYOUT_VERSION: u16 = 1;
/// Metadata header size: one cache line.
pub const META_HEADER_SIZE: usize = 64;
/// Prefix bytes: continuation + metadata_size.
pub const PREFIX_SIZE: usize = 8;
/// End-of-stream size: continuation + zero.
pub const EOS_SIZE: usize = 8;

// KD-195 gitpack transit pin (docs/jira/v26.11.6 receipt
// kd195-gitpack-transit.md): the AOQ1 window carries a frameFlag byte at
// window-absolute offset 32 — the AOQ1 metadata header's `reserved[0]`
// (header offset 24 = 8-byte prefix + 24). The FNV-1a32 header digest
// spans header [0x00,0x3C) (window [8,68)), so the flag IS digest-covered:
// a reader observes the flag only AFTER the digest gate, and an emitter
// must stamp the flag BEFORE sealing (the stamped byte is inside the
// digest span; stamping after sealing is a typed HeaderDigestMismatch).
/// Window-absolute offset of the KD-195 frameFlag byte (`reserved[0]`).
pub const FRAME_FLAG_OFFSET_IN_WINDOW: usize = PREFIX_SIZE + 24;
/// frameFlag value: Arrow transit (the legacy/all-zero-reserved default;
/// `0x00` reserved reads as this value for pre-KD-195 emitters).
pub const FRAME_FLAG_ARROW: u8 = 0x01;
/// frameFlag value: gitpack transit (raw Kafka RecordBatch bytes verbatim,
/// one payload per window body, no envelope).
pub const FRAME_FLAG_GITPACK: u8 = 0x08;

/// The frameFlag byte at [`FRAME_FLAG_OFFSET_IN_WINDOW`], post-digest-gate.
/// `None` only when the window is too short to carry a header (a shape the
/// digest gate refuses first); callers invoke this AFTER `read_verified`.
#[must_use]
pub fn window_frame_flag(window: &[u8]) -> Option<u8> {
    window.get(FRAME_FLAG_OFFSET_IN_WINDOW).copied()
}

/// Stamp `flag` into an ALREADY-SEALED 64-byte AOQ1 metadata header
/// (in place) and re-seal: `hdr[24] = flag`, FNV-1a32 recomputed over
/// `[0x00,0x3C)` and stored at `[0x3C,0x40)`. This is the emitter-side
/// mirror-law helper: the stamp is PRE-digest in the sealed output (the
/// digest covers the flag), which is the only seal order `read_verified`'s
/// digest gate accepts. `hdr` must be at least 64 bytes.
pub fn stamp_frame_flag(hdr: &mut [u8], flag: u8) {
    hdr[24] = flag;
    let digest = fnv1a32(&hdr[..0x3C]);
    hdr[0x3C..0x40].copy_from_slice(&digest.to_le_bytes());
}

/// Metadata header: exactly 64 bytes (`repr(C)`, offsets pinned by the
/// static assert below).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MetaHeader {
    /// 0x00: magic `"IPC1"`.
    pub magic: u32,
    /// 0x04: layout version.
    pub version: u16,
    /// 0x06: reserved-zero flag word.
    pub flags: u16,
    /// 0x08: cgap O-ABI schema digest (schema identity for the body).
    pub schema_digest: [u8; 32],
    /// 0x28: exact body length in bytes (before body padding).
    pub body_len: u32,
    /// 0x2C: alignment the emitter used (8 or 64).
    pub alignment: u32,
    /// 0x30: monotonic message sequence.
    pub seq: u32,
    /// 0x34: zero-filled; MUST be zero.
    pub reserved0: u32,
    /// 0x38: FNV-1a over bytes 0x00..0x38; checked on read.
    pub header_digest: u32,
    /// 0x3C: zero tail padding, MUST be zero.
    pub pad_tail: [u8; 4],
}

const _: () = {
    assert!(core::mem::size_of::<MetaHeader>() == 64);
    assert!(core::mem::align_of::<MetaHeader>() == 4);
    assert!(core::mem::offset_of!(MetaHeader, magic) == 0x00);
    assert!(core::mem::offset_of!(MetaHeader, schema_digest) == 0x08);
    assert!(core::mem::offset_of!(MetaHeader, body_len) == 0x28);
    assert!(core::mem::offset_of!(MetaHeader, header_digest) == 0x38);
};

/// Typed envelope error surface. Every corrupt framing class has exactly
/// one variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IpcError {
    /// First 4 bytes are not the continuation marker.
    BadContinuation {
        /// Value observed at offset 0.
        got: u32,
    },
    /// `metadata_size` disagrees with the pinned framing.
    BadMetaSize {
        /// Value observed.
        got: u32,
        /// Value required for the window's alignment.
        expected: u32,
    },
    /// Window too short for prefix / header / body / EOS.
    Truncated {
        /// Bytes the framing needed.
        needed: usize,
        /// Bytes the window had.
        got: usize,
    },
    /// Metadata magic is not `"IPC1"`.
    BadMagic {
        /// Magic observed.
        got: u32,
    },
    /// Layout version is not [`LAYOUT_VERSION`]. Checked after
    /// [`IpcError::BadMagic`], in PG-6's gate order.
    BadVersion {
        /// Version observed.
        got: u16,
    },
    /// Reserved flag word is not zero (0x0001 reserved: tamper-evident
    /// body-digest-present bit is the first candidate meaning). Checked
    /// after [`IpcError::BadVersion`].
    BadFlags {
        /// Flag word observed.
        got: u16,
    },
    /// FNV-1a header digest mismatch.
    HeaderDigestMismatch {
        /// Digest observed in the header.
        got: u32,
    },
    /// Schema digest differs from the expected O-ABI digest.
    SchemaMismatch {
        /// First divergent byte index, when one diverges.
        got: Option<u8>,
    },
    /// `body_len` disagrees with the body the window carries.
    BadBodyLen {
        /// Header-declared body length.
        got: u32,
        /// Body bytes implied by the window length.
        expected: u32,
    },
    /// Alignment field is not 8 or 64.
    BadAlignment {
        /// Alignment observed.
        got: u32,
    },
    /// Metadata/body padding bytes are nonzero.
    PadNonZero {
        /// Window offset of the first nonzero pad byte.
        at: usize,
    },
    /// Frame does not end in the 8-byte EOS.
    MissingEos {
        /// Final 4 bytes observed as u32 (LE).
        got: u32,
    },
    /// Verified read: the metadata padding carries no body digest (all
    /// zero) — the window was emitted by [`emit`], not [`emit_verified`].
    /// Consumers that verify downstream should use [`read`].
    BodyDigestMissing,
    /// Verified read: blake3(body) disagrees with the digest the metadata
    /// padding carried.
    BodyDigestMismatch {
        /// Digest observed in the metadata padding.
        got: [u8; 32],
    },
    /// Verified emit: a blake3 body digest needs 32 metadata-padding bytes,
    /// which only alignment-64 windows have (56 pad bytes; alignment 8 has
    /// none).
    BodyDigestNoRoom {
        /// Alignment requested.
        got: u32,
    },
    /// Verified read, AOQ1 magic (PG-A2 §2, AB-8): the metadata padding
    /// carries no tag (all zero) — the window was not sealed, or it is an
    /// alignment-8 AOQ1 framing (no padding to carry the tag).
    TagMissing,
    /// Verified read, AOQ1 tag mismatch variant.
    TagMismatch {
        /// Tag observed in the metadata padding.
        got: [u8; 16],
    },
    /// A window whose metadata is the flatbuffer Message (fb branch) read
    /// through a legacy path, or an IPC1/AOQ1 window read through the fb
    /// path. Typed refusal both directions (compat window law).
    LegacyReadForeignMeta,
    /// fb branch: the Message `bodyLength` disagrees with the framing body
    /// length.
    FbBodyLenInvalid {
        /// Value observed in the flatbuffer.
        got: i64,
        /// Body length implied by the framing.
        expected: u32,
    },
    /// fb branch: the flatbuffer metadata itself refused (typed
    /// [`crate::flatbuf::FbError`]).
    Fb(crate::flatbuf::FbError),
    /// fb branch: body larger than 2 GiB.
    BodyTooLarge {
        /// Observed bodyLength.
        got: i64,
    },
    /// metadata_size carries the 64-bit metadata continuation bit (high
    /// bit set) — typed refusal, never a silent reinterpretation.
    MetaSizeContinuation {
        /// metadata_size observed.
        got: u32,
    },
    /// fb branch, verified read: the schema custom_metadata carries no
    /// parseable `cgap:rows` value (absent, or not a decimal u64) — the
    /// rows carrier, fb-era form (R4-I: mirrors the legacy 104..112
    /// slot; zero-row batches are never emitted, so absence is a typed
    /// refusal, never a guess).
    RowsCarrierMissing,
}

impl core::fmt::Debug for FbRead<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FbRead")
            .field("body_len", &self.body_len)
            .field("schema_digest", &self.schema_digest)
            .finish_non_exhaustive()
    }
}

impl fmt::Display for IpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(not(target_os = "none"))]
impl core::error::Error for IpcError {}

/// FNV-1a 32-bit over `bytes` (donor: slab `header_digest` convention).
#[must_use]
pub fn fnv1a32(bytes: &[u8]) -> u32 {
    let mut h: u32 = 0x811C_9DC5;
    let mut i = 0;
    while i < bytes.len() {
        h ^= u32::from(bytes[i]);
        h = h.wrapping_mul(0x0100_0193);
        i += 1;
    }
    h
}

/// `n` rounded up to a multiple of `a` (`a` a power of two).
#[must_use]
pub fn pad_to(n: usize, a: usize) -> usize {
    debug_assert!(a.is_power_of_two());
    (n + a - 1) & !(a - 1)
}

/// Metadata span (the `metadata_size` wire word): the 64-byte header plus
/// whatever zero padding puts the body start (prefix + span) on an
/// `alignment` boundary.
#[must_use]
pub fn meta_span_for(alignment: usize) -> usize {
    pad_to(PREFIX_SIZE + META_HEADER_SIZE, alignment) - PREFIX_SIZE
}

/// Window length in bytes for a `body_len` body at `alignment`.
#[must_use]
pub fn window_len(body_len: usize, alignment: usize) -> usize {
    PREFIX_SIZE + meta_span_for(alignment) + body_len + pad_to(body_len, alignment) + EOS_SIZE
}

/// Seal a metadata header: zero the reserved fields, FNV-1a bytes
/// 0x00..0x38 into `header_digest` at 0x38.
#[must_use]
pub fn seal_meta(mut meta: MetaHeader) -> MetaHeader {
    meta.reserved0 = 0;
    meta.pad_tail = [0; 4];
    let bytes: [u8; 64] = meta_bytes(&meta);
    meta.header_digest = fnv1a32(&bytes[..0x38]);
    meta
}

/// Byte view of a `MetaHeader` (repr(C), align 4, size 64).
fn meta_bytes(m: &MetaHeader) -> [u8; 64] {
    let mut out = [0u8; 64];
    out[0x00..0x04].copy_from_slice(&m.magic.to_le_bytes());
    out[0x04..0x06].copy_from_slice(&m.version.to_le_bytes());
    out[0x06..0x08].copy_from_slice(&m.flags.to_le_bytes());
    out[0x08..0x28].copy_from_slice(&m.schema_digest);
    out[0x28..0x2C].copy_from_slice(&m.body_len.to_le_bytes());
    out[0x2C..0x30].copy_from_slice(&m.alignment.to_le_bytes());
    out[0x30..0x34].copy_from_slice(&m.seq.to_le_bytes());
    out[0x34..0x38].copy_from_slice(&m.reserved0.to_le_bytes());
    out[0x38..0x3C].copy_from_slice(&m.header_digest.to_le_bytes());
    out
}

/// Parse a metadata header from its 64-byte window.
fn meta_from(b: &[u8]) -> MetaHeader {
    let rd32 = |o: usize| u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]);
    let rd16 = |o: usize| u16::from_le_bytes([b[o], b[o + 1]]);
    MetaHeader {
        magic: rd32(0x00),
        version: rd16(0x04),
        flags: rd16(0x06),
        schema_digest: {
            let mut d = [0u8; 32];
            d.copy_from_slice(&b[0x08..0x28]);
            d
        },
        body_len: rd32(0x28),
        alignment: rd32(0x2C),
        seq: rd32(0x30),
        reserved0: rd32(0x34),
        header_digest: rd32(0x38),
        pad_tail: [b[0x3C], b[0x3D], b[0x3E], b[0x3F]],
    }
}

/// Emit an encapsulated message into `out`: metadata `meta`, body `body`.
/// `meta.body_len` must equal `body.len()`; `meta.alignment` is 8 or 64.
/// Returns the window bytes written ([`window_len`]).
pub fn emit(out: &mut [u8], meta: &MetaHeader, body: &[u8]) -> Result<usize, IpcError> {
    let alignment = meta.alignment as usize;
    if !matches!(alignment, 8 | 64) {
        return Err(IpcError::BadAlignment {
            got: meta.alignment,
        });
    }
    if body.len() != meta.body_len as usize {
        return Err(IpcError::BadBodyLen {
            got: meta.body_len,
            expected: body.len() as u32,
        });
    }
    let total = window_len(body.len(), alignment);
    if out.len() < total {
        return Err(IpcError::Truncated {
            needed: total,
            got: out.len(),
        });
    }
    let sealed = seal_meta(*meta);
    let mut w = 0usize;
    out[0..4].copy_from_slice(&CONTINUATION.to_le_bytes());
    w += 4;
    let meta_span = meta_span_for(alignment);
    out[4..8].copy_from_slice(&(meta_span as u32).to_le_bytes());
    w += 4;
    out[8..72].copy_from_slice(&meta_bytes(&sealed));
    w += 64;
    for _ in 0..meta_span - META_HEADER_SIZE {
        out[w] = 0;
        w += 1;
    }
    out[w..w + body.len()].copy_from_slice(body);
    w += body.len();
    for _ in 0..pad_to(body.len(), alignment) {
        out[w] = 0;
        w += 1;
    }
    out[w..w + 4].copy_from_slice(&CONTINUATION.to_le_bytes());
    w += 4;
    out[w..w + 4].copy_from_slice(&EOS.to_le_bytes());
    w += 4;
    Ok(w)
}

/// The parse_meta seam (G4, CG-1 v26.11.3): every window-shape gate shared
/// by `read` and `read_verified` lives HERE, once. Both entrypoints
/// delegate to it, so their classifications of the same window can never
/// diverge (the N-window classification-identity court pins this).
///
/// Gates, in PG-6's order: truncation-to-prefix -> continuation ->
/// metadata span -> truncation-to-header -> magic -> reserved fields ->
/// header digest -> version -> flags -> alignment field.
struct ParsedMeta<'a> {
    alignment: usize,
    body_at: usize,
    meta: MetaHeader,
    /// fb-branch payload (`None` on the legacy/AOQ1 branches).
    fb: Option<FbParsed<'a>>,
}


fn parse_meta(buf: &[u8]) -> Result<ParsedMeta<'_>, IpcError> {
    parse_meta_with(buf, false)
}

/// The parse seam with the fb window (`allow_fb`). Legacy callers
/// (`read`/`read_verified`) pass `false` and refuse fb windows with
/// [`IpcError::LegacyReadForeignMeta`]; fb callers pass `true` and refuse
/// legacy windows the same way. The magic fork: `IPC1` legacy,
/// `AOQ1` per the AOQ1 branch, anything else goes to the flatbuffer
/// Message branch (typed [`IpcError::Fb`] on any flatbuffer refusal).
fn parse_meta_with(buf: &[u8], allow_fb: bool) -> Result<ParsedMeta<'_>, IpcError> {
    if buf.len() < PREFIX_SIZE {
        return Err(IpcError::Truncated {
            needed: PREFIX_SIZE,
            got: buf.len(),
        });
    }
    let cont = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
    if cont != CONTINUATION {
        return Err(IpcError::BadContinuation { got: cont });
    }
    let meta_span = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize;
    // The 64-bit metadata continuation bit (high bit) is a typed refusal.
    if meta_span & 0x8000_0000 != 0 {
        return Err(IpcError::MetaSizeContinuation {
            got: meta_span as u32,
        });
    }
    // Fork shape: legacy windows carry fixed metadata spans {64, 120};
    // fb windows carry the variable flatbuffer length (multiple of 8, so
    // the body starts 8-aligned; 64-alignment is derived when the span
    // puts the body on a 64-B boundary).
    let raw_magic = if buf.len() >= 12 {
        u32::from_le_bytes([buf[8], buf[8 + 1], buf[8 + 2], buf[8 + 3]])
    } else {
        0
    };
    let legacy_span = meta_span == meta_span_for(8) || meta_span == meta_span_for(64);
    let is_fb = !legacy_span || (raw_magic != MAGIC && raw_magic != AOQ1_MAGIC && raw_magic != 0);
    if !is_fb && !legacy_span {
        // Legacy magic with a non-legacy span: BadMetaSize, unchanged law.
        return Err(IpcError::BadMetaSize {
            got: meta_span as u32,
            expected: meta_span as u32,
        });
    }
    let body_at = PREFIX_SIZE + meta_span;
    if buf.len() < body_at {
        return Err(IpcError::Truncated {
            needed: body_at,
            got: buf.len(),
        });
    }
    if !is_fb {
        // Legacy gate order unchanged: header fields + seal + version/flags/alignment.
        let meta = meta_from(&buf[8..72]);
        if raw_magic == AOQ1_MAGIC {
            return parse_meta_aoq1(buf, meta_span);
        }
        if meta.magic != MAGIC {
            return Err(IpcError::BadMagic { got: meta.magic });
        }
        if meta.reserved0 != 0 || meta.pad_tail != [0; 4] {
            return Err(IpcError::HeaderDigestMismatch { got: meta.header_digest });
        }
        let sealed = seal_meta(meta);
        if sealed.header_digest != meta.header_digest {
            return Err(IpcError::HeaderDigestMismatch { got: meta.header_digest });
        }
        if meta.version != LAYOUT_VERSION {
            return Err(IpcError::BadVersion { got: meta.version });
        }
        if meta.flags != 0 {
            return Err(IpcError::BadFlags { got: meta.flags });
        }
        let alignment = if meta_span == meta_span_for(8) { 8 } else { 64 };
        if meta.alignment as usize != alignment {
            return Err(IpcError::BadAlignment { got: meta.alignment });
        }
        return Ok(ParsedMeta {
            alignment,
            body_at,
            meta,
            fb: None,
        });
    }
    // fb branch: variable metadata span, Message flatbuffer validated.
    let alignment = if (PREFIX_SIZE + meta_span).is_multiple_of(64) { 64 } else { 8 };
    let region = &buf[8..body_at];
    let message = crate::flatbuf::read_message(region)
        .map_err(|e| IpcError::Fb(e))?;
    let body_len64 = message.body_length;
    if body_len64 < 0 || body_len64 > FB_BODY_MAX as i64 {
        return Err(IpcError::BodyTooLarge { got: body_len64 });
    }
    let body_len = body_len64 as u32;
    // Ancillary key values ride the schema-level custom_metadata.
    let (sd_key, bd_key, rows_key) = ancillary_keys(&message);
    if !allow_fb {
        return Err(IpcError::LegacyReadForeignMeta);
    }
    Ok(ParsedMeta {
        alignment,
        body_at,
        fb: Some(FbParsed { message, schema_digest: fb_schema_digest(region), schema_blake3: sd_key, body_blake3: bd_key, rows: rows_key }),
        meta: MetaHeader {
            magic: FB_MAGIC,
            version: message.version as u16,
            flags: 0,
            schema_digest: fb_schema_digest(region),
            body_len,
            alignment: alignment as u32,
            seq: 0,
            reserved0: 0,
            header_digest: 0,
            pad_tail: [0; 4],
        },
    })
}

/// fb window body ceiling (2 GiB).
const FB_BODY_MAX: usize = 2 * 1024 * 1024 * 1024;

/// blake3 over the metadata flatbuffer region.
fn fb_schema_digest(region: &[u8]) -> [u8; 32] {
    crate::blake3::hash(region)
}
/// The AOQ1 branch of the [`parse_meta`] magic fork (PG-A2 §2, AB-8):
/// same gate order as the IPC1 path, per-magic domain values. The AOQ1
/// header is LITTLE-endian (landed codec law), the digest span is
/// `[0x00,0x3C)` stored LE at `0x3C`, the flags-word position carries the
/// closed epoch enum, and the metadata padding carries `body_len` u32 LE
/// plus the 16-byte tag. Alignment-8 AOQ1 framing has no padding to carry
/// either — typed [`IpcError::TagMissing`], never a misread.
fn parse_meta_aoq1(buf: &[u8], meta_span: usize) -> Result<ParsedMeta<'_>, IpcError> {
    let body_at = PREFIX_SIZE + meta_span;
    if meta_span != meta_span_for(64) {
        return Err(IpcError::TagMissing);
    }
    let hb = &buf[8..72];
    // Digest gate: FNV-1a32 over header bytes [0x00,0x3C), stored LE.
    let stored = u32::from_le_bytes([hb[60], hb[61], hb[62], hb[63]]);
    if fnv1a32(&hb[..60]) != stored {
        return Err(IpcError::HeaderDigestMismatch { got: stored });
    }
    // Version gate (LE domain value; v3 wire law, 2026-11-04: AOQ1
    // header version promoted 2 -> 3 by the generated layout projection).
    let version = u16::from_le_bytes([hb[4], hb[5]]);
    if version != 3 {
        return Err(IpcError::BadVersion { got: version });
    }
    // Flags-position gate: the closed epoch enum {0,1,2,65535}.
    let epoch = u16::from_le_bytes([hb[6], hb[7]]);
    if !matches!(epoch, 0 | 1 | 2 | 65535) {
        return Err(IpcError::BadFlags { got: epoch });
    }
    // Pad cleanliness: zero outside the body_len/tag law region.
    for (i, &b) in buf[72..body_at].iter().enumerate() {
        if b != 0 && i >= 20 {
            return Err(IpcError::PadNonZero { at: 72 + i });
        }
    }
    let body_len = u32::from_le_bytes([buf[72], buf[73], buf[74], buf[75]]);
    let seq = u32::from_le_bytes([hb[20], hb[21], hb[22], hb[23]]);
    Ok(ParsedMeta {
        alignment: 64,
        body_at,
        fb: None,
        meta: MetaHeader {
            magic: AOQ1_MAGIC,
            version: 2,
            flags: epoch,
            schema_digest: [0; 32],
            body_len,
            alignment: 64,
            seq,
            reserved0: 0,
            header_digest: stored,
            pad_tail: [0; 4],
        },
    })
}

/// Sequence-pair law (G2): a strictly-advancing, wrap-safe successor test
/// over the wire `seq` words. `(prev, next)` is a lawful ordered pair iff
/// `next == prev + 1 (mod 2^32)` — exactly one step, wrapping silently at
/// the u32 boundary. Reordered, replayed, or skipped messages refuse.
#[must_use]
pub fn seq_ok(prev: u32, next: u32) -> bool {
    next == prev.wrapping_add(1)
}

/// Read an encapsulated message: validate every framing class and copy the
/// exact body into `body_out`. Returns the body length. `expected_schema`
/// is the O-ABI schema digest the caller requires; `None` skips the check.
pub fn read(
    buf: &[u8],
    expected_schema: Option<&[u8; 32]>,
    body_out: &mut [u8],
) -> Result<usize, IpcError> {
    read_with_meta(buf, expected_schema, body_out).map(|(n, _)| n)
}

/// [`read`] plus the sealed metadata header: returns `(body_len, meta)`.
/// The header carries the emitter's `seq` (pair it with [`seq_ok`] to
/// detect reorder/replay/skip) and `schema_digest` for streaming readers.
pub fn read_with_meta(
    buf: &[u8],
    expected_schema: Option<&[u8; 32]>,
    body_out: &mut [u8],
) -> Result<(usize, MetaHeader), IpcError> {
    let p = parse_meta(buf)?;
    let meta = p.meta;
    let alignment = p.alignment;
    let body_at = p.body_at;
    if let Some(exp) = expected_schema {
        for i in 0..32 {
            if exp[i] != meta.schema_digest[i] {
                return Err(IpcError::SchemaMismatch { got: Some(i as u8) });
            }
        }
    }
    let body_len = meta.body_len as usize;
    let body_pad = pad_to(body_len, alignment);
    if buf.len() < body_at + body_len + body_pad {
        return Err(IpcError::BadBodyLen {
            got: meta.body_len,
            expected: expected_body(buf.len(), body_at, alignment),
        });
    }
    if body_out.len() < body_len {
        return Err(IpcError::Truncated {
            needed: body_len,
            got: body_out.len(),
        });
    }
    body_out[..body_len].copy_from_slice(&buf[body_at..body_at + body_len]);
    for i in 0..body_pad {
        if buf[body_at + body_len + i] != 0 {
            return Err(IpcError::PadNonZero {
                at: body_at + body_len + i,
            });
        }
    }
    let eos_at = body_at + body_len + body_pad;
    if buf.len() < eos_at + EOS_SIZE {
        return Err(IpcError::MissingEos { got: 0 });
    }
    let cont2 = u32::from_le_bytes([
        buf[eos_at],
        buf[eos_at + 1],
        buf[eos_at + 2],
        buf[eos_at + 3],
    ]);
    let tail = u32::from_le_bytes([
        buf[eos_at + 4],
        buf[eos_at + 5],
        buf[eos_at + 6],
        buf[eos_at + 7],
    ]);
    if cont2 != CONTINUATION || tail != EOS {
        return Err(IpcError::MissingEos { got: tail });
    }
    Ok((body_len, meta))
}
/// Body length implied by a window of `buf_len` bytes at metadata end
/// `body_at` and `alignment`.
fn expected_body(buf_len: usize, body_at: usize, alignment: usize) -> u32 {
    if buf_len <= body_at {
        return 0;
    }
    ((buf_len - body_at) / (alignment + 1)) as u32
}

/// Offset (from window start) of the 32-byte blake3 body digest that
/// `emit_verified` writes into the metadata padding. Alignment-64 windows
/// carry 56 zero pad bytes there (`meta_span_for(64) == 120`), and `read`
/// never inspects metadata padding, so a verified window stays readable
/// through the legacy path.
pub const BODY_DIGEST_AT: usize = PREFIX_SIZE + META_HEADER_SIZE;

/// The all-zero digest marks "no body digest carried".
const BODY_DIGEST_ABSENT: [u8; 32] = [0; 32];

/// Emit a verified encapsulated message: identical framing to [`emit`]
/// plus a blake3 body digest in the metadata padding. Alignment-64 only
/// (see [`IpcError::BodyDigestNoRoom`]).
///
/// COMPAT WINDOW: [`read`] does NOT check the body digest — a
/// body-corrupted verified window still passes `read` (documented
/// consumer duty; witness W4, `tests/stream_writev_witness.rs`). Use
/// [`read_verified`] to refuse corruption at the framing layer.
pub fn emit_verified(out: &mut [u8], meta: &MetaHeader, body: &[u8]) -> Result<usize, IpcError> {
    if meta.alignment != 64 {
        return Err(IpcError::BodyDigestNoRoom {
            got: meta.alignment,
        });
    }
    let n = emit(out, meta, body)?;
    let digest = crate::blake3::hash(body);
    out[BODY_DIGEST_AT..BODY_DIGEST_AT + 32].copy_from_slice(&digest);
    Ok(n)
}

/// Read with body integrity: every framing class [`read`] checks, plus the
/// metadata padding must carry a blake3 body digest matching `body`.
/// Alignment-64 windows only ([`IpcError::BodyDigestNoRoom`] for align 8;
/// [`IpcError::BodyDigestMissing`] for windows emitted by plain [`emit`]).
pub fn read_verified(
    buf: &[u8],
    expected_schema: Option<&[u8; 32]>,
    body_out: &mut [u8],
) -> Result<usize, IpcError> {
    // The seam answers window shape FIRST, so a truncated window classifies
    // as `Truncated` (typed, exact `needed`/`got`), never the wrong-class
    // `BodyDigestNoRoom { got: 8 }` the pre-seam alignment guess produced.
    let p = parse_meta(buf)?;
    if p.alignment != 64 {
        return Err(IpcError::BodyDigestNoRoom {
            got: p.meta.alignment,
        });
    }
    let (n, meta) = read_with_meta(buf, expected_schema, body_out)?;
    if meta.magic == AOQ1_MAGIC {
        // AOQ1 tag gate: stored tag must match blake3(body)[..16] over the
        // carried body (tamper tier; the tag supersedes the IPC1
        // 32-byte body digest on this magic).
        let mut stored = [0u8; AOQ1_TAG_BYTES];
        stored.copy_from_slice(&buf[AOQ1_TAG_AT..AOQ1_TAG_AT + AOQ1_TAG_BYTES]);
        if stored == [0u8; AOQ1_TAG_BYTES] {
            return Err(IpcError::TagMissing);
        }
        let digest = crate::blake3::hash(&body_out[..n]);
        let mut expected = [0u8; AOQ1_TAG_BYTES];
        expected.copy_from_slice(&digest[..AOQ1_TAG_BYTES]);
        if stored != expected {
            return Err(IpcError::TagMismatch { got: stored });
        }
        return Ok(n);
    }
    let stored_slice = &buf[BODY_DIGEST_AT..BODY_DIGEST_AT + 32];
    let mut stored = [0u8; 32];
    stored.copy_from_slice(stored_slice);
    if stored == BODY_DIGEST_ABSENT {
        return Err(IpcError::BodyDigestMissing);
    }
    let digest = crate::blake3::hash(&body_out[..n]);
    if digest != stored {
        return Err(IpcError::BodyDigestMismatch { got: stored });
    }
    let _ = meta;
    Ok(n)
}

// ===========================================================================
// fb branch: flatbuffer Message windows (lane 7, v26.11.6)
// ===========================================================================

/// Window length for an fb window: prefix + metadata span + body + body
/// pad + EOS. `meta_span` must be `pad_to(fb_len + 8, alignment) - 8`.
#[must_use]
pub fn window_len_fb(fb_len: usize, body_len: usize, alignment: usize) -> usize {
    PREFIX_SIZE + meta_span_from_fb(fb_len, alignment) + body_len + pad_to(body_len, alignment) + EOS_SIZE
}

/// Metadata span for an fb window: the flatbuffer length zero-padded so
/// the body starts `alignment`-aligned (variable metadata_size law).
#[must_use]
pub fn meta_span_from_fb(fb_len: usize, alignment: usize) -> usize {
    pad_to(fb_len + PREFIX_SIZE, alignment) - PREFIX_SIZE
}

/// Ancillary keys (custom_metadata KeyValue law). `cgap:body_blake3`
/// carries the body tamper tier on the fb branch (replaces the legacy
/// padding-slot digest); `cgap:schema_blake3` keeps the expected_schema
/// seam.
pub const KEY_BODY_DIGEST: &str = "cgap:body_blake3";
pub const KEY_SCHEMA_DIGEST: &str = "cgap:schema_blake3";
/// The rows carrier, fb era (R4-I): decimal u64 in schema
/// custom_metadata, mirroring the legacy alignment-64 padding slot at
/// window offset 104..112.
pub const KEY_ROWS: &str = "cgap:rows";

/// Hex-encode 32 bytes (lowercase, 64 chars) into `out`; returns the hex
/// length.
fn hex32(d: &[u8; 32], out: &mut [u8]) -> usize {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for (i, b) in d.iter().enumerate() {
        out[2 * i] = HEX[usize::from(b >> 4)];
        out[2 * i + 1] = HEX[usize::from(b & 0xf)];
    }
    64
}


/// Parse a 64-char lowercase hex string back to 32 bytes.
fn unhex32(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 {
        return None;
    }
    let b = s.as_bytes();
    let hv = |c: u8| -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            _ => None,
        }
    };
    let mut out = [0u8; 32];
    for i in 0..32 {
        let hi = hv(b[2 * i])?;
        let lo = hv(b[2 * i + 1])?;
        out[i] = (hi << 4) | lo;
    }
    Some(out)
}

/// Ancillary keys ride the schema-level custom_metadata (lane 7 law):
/// `cgap:body_blake3` is the body tamper tier (replaces the legacy
/// padding-slot digest); `cgap:schema_blake3` keeps the expected_schema
/// seam (blake3 over the pre-ancillary schema region, pass 1).

/// Emit an fb window: Message flatbuffer + body, framed per the variable
/// metadata_size law. `schema_meta` is the caller's schema custom_metadata
/// (<= 62 pairs); the ancillary keys are appended by the emitter.
pub fn emit_fb(
    out: &mut [u8], schema: &crate::flatbuf::SchemaEnc<'_>, body: &[u8], alignment: usize,
) -> Result<usize, IpcError> {
    emit_fb_inner(out, schema, body, alignment, false, None)
}

/// Emit a verified fb window: [`emit_fb`] plus the `cgap:body_blake3`
/// key. Plain fb windows refuse on [`read_verified_fb`] with
/// [`IpcError::BodyDigestMissing`]. The window carries NO `cgap:rows`
/// key, so [`read_verified_fb`] refuses it with
/// [`IpcError::RowsCarrierMissing`] — verified emitters should use
/// [`emit_verified_fb_rows`].
pub fn emit_verified_fb(
    out: &mut [u8], schema: &crate::flatbuf::SchemaEnc<'_>, body: &[u8], alignment: usize,
) -> Result<usize, IpcError> {
    emit_fb_inner(out, schema, body, alignment, true, None)
}

/// Emit a verified fb window WITH the rows carrier: [`emit_verified_fb`]
/// plus the `cgap:rows` custom-metadata key (decimal u64, mirroring the
/// legacy 104..112 padding slot on the fb era).
pub fn emit_verified_fb_rows(
    out: &mut [u8], schema: &crate::flatbuf::SchemaEnc<'_>, body: &[u8], alignment: usize,
    rows: u64,
) -> Result<usize, IpcError> {
    emit_fb_inner(out, schema, body, alignment, true, Some(rows))
}

fn emit_fb_inner(
    out: &mut [u8],
    schema: &crate::flatbuf::SchemaEnc<'_>,
    body: &[u8],
    alignment: usize,
    verified: bool,
    rows: Option<u64>,
) -> Result<usize, IpcError> {
    if !matches!(alignment, 8 | 64) {
        return Err(IpcError::BadAlignment { got: alignment as u32 });
    }
    if body.len() > FB_BODY_MAX {
        return Err(IpcError::BodyTooLarge { got: body.len() as i64 });
    }
    let buf_len = out.len();
    // Pass 1: pre-ancillary region + its digest (non-circular: the digest
    // is over the schema message WITHOUT the ancillary keys).
    let n1 = match crate::flatbuf::emit_message(out, schema, body.len() as i64) {
        Ok(v) => v,
        Err(e) => return Err(IpcError::Fb(e)),
    };
    let schema_digest = crate::blake3::hash(&out[buf_len - n1..buf_len]);
    // Pass 2: schema meta = caller meta ++ ancillary keys.
    let mut hex1 = [0u8; 64];
    hex32(&schema_digest, &mut hex1);
    let sd = core::str::from_utf8(&hex1).unwrap_or("");
    let body_digest = crate::blake3::hash(body);
    let mut hex2 = [0u8; 64];
    hex32(&body_digest, &mut hex2);
    let bd = core::str::from_utf8(&hex2).unwrap_or("");
    // cgap:rows decimal rendering buffer (lives to end of fn so the
    // borrowed slice in `combined` stays valid).
    let mut rd = [0u8; 20];
    let mut rl = 0usize;
    let mut combined: [(&str, &str); crate::flatbuf::MAX_SLOTS] =
        [("", ""); crate::flatbuf::MAX_SLOTS];
    let mut k = 0usize;
    // MAX_SLOTS-3 headroom when the rows carrier rides along (R4-I): two
    // ancillary slots (schema + body digests) plus the rows slot.
    let headroom = crate::flatbuf::MAX_SLOTS - 2 - usize::from(rows.is_some());
    for pair in schema.meta {
        if k >= headroom {
            return Err(IpcError::Fb(crate::flatbuf::FbError::FieldCap));
        }
        combined[k] = *pair;
        k += 1;
    }
    combined[k] = (KEY_SCHEMA_DIGEST, sd);
    k += 1;
    if verified {
        combined[k] = (KEY_BODY_DIGEST, bd);
        k += 1;
    }
    // cgap:rows rides custom_metadata as a decimal u64 string; emitted
    // only on the verified path (Some), mirroring the legacy 104..112
    // padding-slot carrier.
    if let Some(r) = rows {
        let mut v = r;
        if v == 0 {
            rd[0] = b'0';
            rl = 1;
        }
        while v > 0 {
            rd[rl] = b'0' + (v % 10) as u8;
            rl += 1;
            v /= 10;
        }
        for i in 0..rl / 2 {
            rd.swap(i, rl - 1 - i);
        }
        combined[k] = (KEY_ROWS, core::str::from_utf8(&rd[..rl]).unwrap_or(""));
        k += 1;
    }
    let enc = crate::flatbuf::SchemaEnc {
        little_endian: schema.little_endian,
        fields: schema.fields,
        meta: &combined[..k],
    };
    let n2 = match crate::flatbuf::emit_message(out, &enc, body.len() as i64) {
        Ok(v) => v,
        Err(e) => return Err(IpcError::Fb(e)),
    };
    // Relocate fb to the window position out[8..], pad, frame.
    let span = meta_span_from_fb(n2, alignment);
    let total = window_len_fb(n2, body.len(), alignment);
    if out.len() < total {
        return Err(IpcError::Truncated { needed: total, got: out.len() });
    }
    out.copy_within(buf_len - n2..buf_len, 8);
    for b in out[8 + n2..8 + span].iter_mut() {
        *b = 0;
    }
    let body_at = PREFIX_SIZE + span;
    out[body_at..body_at + body.len()].copy_from_slice(body);
    for b in out[body_at + body.len()..body_at + span + body.len()].iter_mut() {
        *b = 0;
    }
    let eos_at = body_at + body.len() + pad_to(body.len(), alignment);
    out[eos_at..eos_at + 4].copy_from_slice(&CONTINUATION.to_le_bytes());
    out[eos_at + 4..eos_at + 8].copy_from_slice(&EOS.to_le_bytes());
    // Prefix words.
    out[0..4].copy_from_slice(&CONTINUATION.to_le_bytes());
    out[4..8].copy_from_slice(&((span) as u32).to_le_bytes());
    Ok(total)
}

/// Read result for the fb branch: body length, schema view, digests.
pub struct FbRead<'a> {
    /// Body length (== Message.bodyLength, equality is a gate).
    pub body_len: usize,
    /// The parsed schema message.
    pub schema: crate::flatbuf::SchemaView<'a>,
    /// blake3 over the pre-ancillary schema region? NO: blake3 over the
    /// carried flatbuffer region (as carried).
    pub schema_digest: [u8; 32],
    /// cgap:schema_blake3 value carried in custom_metadata.
    pub schema_blake3: Option<[u8; 32]>,
    /// cgap:body_blake3 value carried in custom_metadata.
    pub body_blake3: Option<[u8; 32]>,
    /// cgap:rows value carried in custom_metadata (decimal u64; None
    /// when absent or unparseable — never guessed).
    pub rows: Option<u64>,
}

/// The fb read core: framing + flatbuffer parse + bodyLength equality
/// gate + body copy. `verified` adds the cgap:body_blake3 gate.
fn read_fb_core<'a>(
    buf: &'a [u8],
    expected_schema: Option<&[u8; 32]>,
    body_out: &mut [u8],
    verified: bool,
) -> Result<FbRead<'a>, IpcError> {
    let p = parse_meta_with(buf, true)?;
    let fbi = match p.fb {
        Some(f) => f,
        None => return Err(IpcError::LegacyReadForeignMeta),
    };
    let body_len = p.meta.body_len as usize;
    let body_at = p.body_at;
    let alignment = p.alignment;
    // bodyLength equality gate.
    if i64::from(p.meta.body_len) != fbi.message.body_length {
        return Err(IpcError::FbBodyLenInvalid {
            got: fbi.message.body_length,
            expected: p.meta.body_len,
        });
    }
    if body_out.len() < body_len {
        return Err(IpcError::Truncated {
            needed: body_len,
            got: body_out.len(),
        });
    }
    // Extent gate BEFORE the copy (DoS fence: a mutated bodyLength must
    // classify typed, never panic or over-read).
    let body_pad = pad_to(body_len, alignment);
    if buf.len() < body_at + body_len + body_pad + EOS_SIZE {
        return Err(IpcError::MissingEos { got: 0 });
    }
    body_out[..body_len].copy_from_slice(&buf[body_at..body_at + body_len]);
    // Body pad + EOS gates (same classes as legacy).
    for i in 0..body_pad {
        if buf[body_at + body_len + i] != 0 {
            return Err(IpcError::PadNonZero { at: body_at + body_len + i });
        }
    }
    let eos_at = body_at + body_len + body_pad;
    let cont2 = u32::from_le_bytes([buf[eos_at], buf[eos_at+1], buf[eos_at+2], buf[eos_at+3]]);
    let tail = u32::from_le_bytes([buf[eos_at+4], buf[eos_at+5], buf[eos_at+6], buf[eos_at+7]]);
    if cont2 != CONTINUATION || tail != EOS {
        return Err(IpcError::MissingEos { got: tail });
    }
    // Tamper tier (fb): cgap:body_blake3 gate on the verified path.
    if verified {
        match fbi.body_blake3 {
            None => return Err(IpcError::BodyDigestMissing),
            Some(d) => {
                if crate::blake3::hash(&body_out[..body_len]) != d {
                    return Err(IpcError::BodyDigestMismatch { got: d });
                }
            }
        }
        // Rows carrier, fb era (R4-I): the verified tier requires a
        // parseable cgap:rows value, mirroring the legacy walk's
        // rows-carrier refusal.
        if fbi.rows.is_none() {
            return Err(IpcError::RowsCarrierMissing);
        }
    }
    // expected_schema seam: compared against cgap:schema_blake3.
    if let Some(exp) = expected_schema {
        match fbi.schema_blake3 {
            None => return Err(IpcError::SchemaMismatch { got: None }),
            Some(sd) => {
                if &sd != exp {
                    return Err(IpcError::SchemaMismatch { got: None });
                }
            }
        }
    }
    Ok(FbRead {
        body_len,
        schema: fbi.message.schema,
        schema_digest: fbi.schema_digest,
        schema_blake3: fbi.schema_blake3,
        body_blake3: fbi.body_blake3,
        rows: fbi.rows,
    })
}

/// Extract the ancillary key values from the schema custom_metadata.
/// `cgap:rows` is parsed TYPED: absent or non-decimal => None, never a
/// guessed value.
fn ancillary_keys(
    m: &crate::flatbuf::MessageView<'_>,
) -> (Option<[u8; 32]>, Option<[u8; 32]>, Option<u64>) {
    let mut sd: Option<[u8; 32]> = None;
    let mut bd: Option<[u8; 32]> = None;
    let mut rows: Option<u64> = None;
    if let Ok(meta) = m.schema.meta() {
        for i in 0..meta.len() {
            if let Ok((k, v)) = meta.get(i) {
                if k == KEY_SCHEMA_DIGEST {
                    sd = unhex32(v);
                } else if k == KEY_BODY_DIGEST {
                    bd = unhex32(v);
                } else if k == KEY_ROWS {
                    rows = parse_decimal_u64(v);
                }
            }
        }
    }
    (sd, bd, rows)
}

/// Strict decimal u64 parse (ASCII digits only, no sign, no whitespace,
/// no leading `+`); overflow or any non-digit => None.
fn parse_decimal_u64(s: &str) -> Option<u64> {
    if s.is_empty() {
        return None;
    }
    let mut v: u64 = 0;
    for b in s.bytes() {
        if !b.is_ascii_digit() {
            return None;
        }
        v = v.checked_mul(10)?.checked_add(u64::from(b - b'0'))?;
    }
    Some(v)
}

/// Read an fb window (framing + flatbuffer parse + body copy). The
/// expected_schema seam is compared against the carried cgap:schema_blake3
/// (None expected => skip; expected but window lacks the key =>
/// SchemaMismatch { got: None }).
pub fn read_fb<'a>(
    buf: &'a [u8],
    expected_schema: Option<&[u8; 32]>,
    body_out: &mut [u8],
) -> Result<FbRead<'a>, IpcError> {
    read_fb_core(buf, expected_schema, body_out, false)
}

/// Read with body integrity on the fb branch: every framing class plus the
/// cgap:body_blake3 gate (tamper tier): missing key => BodyDigestMissing,
/// mismatch => BodyDigestMismatch { got }.
pub fn read_verified_fb<'a>(
    buf: &'a [u8],
    expected_schema: Option<&[u8; 32]>,
    body_out: &mut [u8],
) -> Result<FbRead<'a>, IpcError> {
    read_fb_core(buf, expected_schema, body_out, true)
}


// ===========================================================================
// gitpack view (KD-195 transit, v26.11.6): zero-copy payload projection
// ===========================================================================

/// The gitpack view of one AOQ1 window body (KD-195 carrier decision):
/// the body bytes ARE the batch — raw Kafka RecordBatch bytes verbatim,
/// exactly one payload per window body, NO envelope, NO length prefix, NO
/// copy. The offsets span `[0, len)` by construction (the whole body is
/// the batch; a consumer with a parser slices it itself).
///
/// `no_std` pure fn: a borrowed slice view, no allocation, no panic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BinaryBatchView<'a> {
    /// `[0, len)` — the batch's offsets within the payload (verbatim law:
    /// the batch starts at 0 and ends at the body length).
    pub offsets: (usize, usize),
    /// The batch bytes, borrowed from the payload (zero-copy: the pointer
    /// and length alias the source; no re-allocation ever happens).
    pub data: &'a [u8],
}

/// Project the gitpack view of `payload` (see [`BinaryBatchView`]).
#[must_use]
pub fn binary_batch_view(payload: &[u8]) -> BinaryBatchView<'_> {
    BinaryBatchView {
        offsets: (0, payload.len()),
        data: payload,
    }
}
