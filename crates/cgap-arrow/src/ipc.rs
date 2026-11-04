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
//! Implemented: byte-exact framing, alignment 8 or 64, typed refusal of
//! every corrupt framing class (continuation, metadata size, magic, header
//! digest, schema digest, body length, alignment field, nonzero padding,
//! truncation, missing EOS). A future lane may swap the flatbuffer in
//! behind the same emit/read signatures.
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
//! Anti-confusion is courted, not narrated: `tests/ipc_courts.rs` C13
//! asserts zero checksum-tier contamination in this file's source, and
//! wire-shm's court asserts zero "blake3" in its sources, each with a
//! sandbox-add-token anti-vacuity kill.
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
    /// Verified read, AOQ1 magic: the stored 16-byte tag disagrees with
    /// `blake3(body)[..16]` recomputed over the carried body.
    TagMismatch {
        /// Tag observed in the metadata padding.
        got: [u8; 16],
    },
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
struct ParsedMeta {
    alignment: usize,
    body_at: usize,
    meta: MetaHeader,
}

fn parse_meta(buf: &[u8]) -> Result<ParsedMeta, IpcError> {
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
    if meta_span != meta_span_for(8) && meta_span != meta_span_for(64) {
        return Err(IpcError::BadMetaSize {
            got: meta_span as u32,
            expected: meta_span as u32,
        });
    }
    let alignment = if meta_span == meta_span_for(8) { 8 } else { 64 };
    let body_at = PREFIX_SIZE + meta_span;
    // Read-ahead minimum (AB-9, AF-4 v26.11.3): demand only the bytes the
    // gate actually consumes — prefix + the full metadata span (the header
    // is `buf[8..72]`, and the AOQ1 branch additionally inspects the span's
    // padding `buf[72..body_at]`), all strictly inside `body_at`. The old
    // fixed `+ META_HEADER_SIZE` past body start demanded bytes no emitted
    // window carries for tiny bodies (body+pad < 56): an emit-accepted
    // window classified `Truncated` on read. Every typed refusal class
    // stays intact — genuinely truncated windows still classify `Truncated`
    // here, and shorter-body/missing-EOS windows classify downstream as
    // `BadBodyLen`/`MissingEos` in `read_with_meta`.
    if buf.len() < body_at {
        return Err(IpcError::Truncated {
            needed: body_at,
            got: buf.len(),
        });
    }
    let raw = meta_from(&buf[8..72]);
    // The magic fork is a single LE read: both magics are little-endian
    // on the wire (AOQ1's landed codec law is LE, reconciled 22:03).
    if raw.magic == AOQ1_MAGIC {
        return parse_meta_aoq1(buf, alignment, body_at);
    }
    let meta = raw;
    if meta.magic != MAGIC {
        return Err(IpcError::BadMagic { got: meta.magic });
    }
    if meta.reserved0 != 0 || meta.pad_tail != [0; 4] {
        return Err(IpcError::HeaderDigestMismatch {
            got: meta.header_digest,
        });
    }
    let sealed = seal_meta(meta);
    if sealed.header_digest != meta.header_digest {
        return Err(IpcError::HeaderDigestMismatch {
            got: meta.header_digest,
        });
    }
    if meta.version != LAYOUT_VERSION {
        return Err(IpcError::BadVersion { got: meta.version });
    }
    if meta.flags != 0 {
        return Err(IpcError::BadFlags { got: meta.flags });
    }
    if meta.alignment as usize != alignment {
        return Err(IpcError::BadAlignment {
            got: meta.alignment,
        });
    }
    Ok(ParsedMeta { alignment, body_at, meta })
}

/// The AOQ1 branch of the [`parse_meta`] magic fork (PG-A2 §2, AB-8):
/// same gate order as the IPC1 path, per-magic domain values. The AOQ1
/// header is LITTLE-endian (landed codec law), the digest span is
/// `[0x00,0x3C)` stored LE at `0x3C`, the flags-word position carries the
/// closed epoch enum, and the metadata padding carries `body_len` u32 LE
/// plus the 16-byte tag. Alignment-8 AOQ1 framing has no padding to carry
/// either — typed [`IpcError::TagMissing`], never a misread.
fn parse_meta_aoq1(
    buf: &[u8],
    alignment: usize,
    body_at: usize,
) -> Result<ParsedMeta, IpcError> {
    if alignment != 64 {
        return Err(IpcError::TagMissing);
    }
    let hb = &buf[8..72];
    // Digest gate: FNV-1a32 over header bytes [0x00,0x3C), stored LE.
    let stored = u32::from_le_bytes([hb[60], hb[61], hb[62], hb[63]]);
    if fnv1a32(&hb[..60]) != stored {
        return Err(IpcError::HeaderDigestMismatch { got: stored });
    }
    // Version gate (LE domain value 2).
    let version = u16::from_le_bytes([hb[4], hb[5]]);
    if version != 2 {
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
pub fn emit_verified(
    out: &mut [u8],
    meta: &MetaHeader,
    body: &[u8],
) -> Result<usize, IpcError> {
    if meta.alignment != 64 {
        return Err(IpcError::BodyDigestNoRoom { got: meta.alignment });
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
        return Err(IpcError::BodyDigestNoRoom { got: p.meta.alignment });
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
