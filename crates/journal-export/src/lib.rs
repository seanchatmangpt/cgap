//! journal-export — lane-12 std host crate: sealed journal segments ->
//! IPC1 windows -> parquet export.
//!
//! Consumes `cgap_arrow::ipc` (read_verified / emit_verified) and
//! `cgap_parquet` (StaticPageWriter + FileSink) — reuse, never fork.
//! L7 owns ipc.rs/flatbuf.rs; this crate stages against the EXISTING
//! IPC1 MetaHeader framing (disclosed dependency): the rows carrier is
//! a u64 LE at window offset 104..112 (metadata-padding slot adjacent
//! to the blake3 body digest at 72..104), defined ONCE here and
//! mirrored byte-identically by smalt's `smalt_lakehouse_mount` walker
//! (gate order identical to cgap's parse_meta + read_with_meta).
//!
//! Typed errors mirror smalt's Smalt.Storage.JournalReader
//! classification names 1:1 (not_sealed / missing_meta /
//! manifest_mismatch / crc_mismatch / seq_regression / truncated_frame /
//! truncated_tail / bad_magic), plus the ipc_* framing classes and the
//! rows carrier.

pub mod segment_source;
pub mod ipc_walk;
pub mod parquet_sink;

use cgap_arrow::ipc::IpcError;

/// Typed export error surface. The journal classes carry the same NAMES
/// as the Elixir reader's refusals (snake_case here, atoms there).
#[derive(Debug)]
pub enum ExportError {
    /// `.seal` sentinel missing.
    NotSealed,
    /// `.meta.json` manifest missing.
    MissingMeta,
    /// Manifest sha256 disagrees with the segment bytes.
    ManifestMismatch {
        got: [u8; 32],
        expected: [u8; 32],
    },
    /// Frame body CRC32 mismatch.
    CrcMismatch {
        frame: usize,
    },
    /// SeqID not strictly advancing by one.
    SeqRegression {
        expected: u64,
        got: u64,
    },
    /// Frame extends past the segment bytes.
    TruncatedFrame {
        offset: usize,
        have: usize,
    },
    /// Fewer than the 8-byte frame header remain.
    TruncatedTail {
        offset: usize,
        have: usize,
    },
    /// Frame magic is not "SMAL".
    BadMagic {
        offset: usize,
    },
    /// Frame length smaller than the trailing CRC field.
    BadFrameLen {
        offset: usize,
        have: usize,
    },
    /// IPC framing refusal (typed mirror of cgap's IpcError).
    Ipc(IpcError),
    /// Metadata-padding rows carrier all-zero (absent, or zero rows —
    /// zero-row batches are never emitted; the emitter refuses them).
    RowsCarrierMissing,
    /// Window body is not rows*4 bytes (single u32 column "v" layout).
    BodyLayout {
        body_len: u32,
        rows: u64,
    },
    /// Parquet encode/seal refusal.
    Parquet(cgap_parquet::ParquetError),
    /// Filesystem error (path carried by the caller's context).
    Io,
}

impl core::fmt::Display for ExportError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(not(target_os = "none"))]
impl core::error::Error for ExportError {}

/// Payload of a walked window: verified body + carried rows + header
/// identity.
#[derive(Clone, Debug)]
pub struct Window {
    /// Verified body bytes (blake3-checked at the export tier).
    pub body: Vec<u8>,
    /// Carried row count (u64 LE at window offset 104..112).
    pub rows: u64,
    /// Emitter seq.
    pub seq: u32,
    /// O-ABI schema digest from the MetaHeader.
    pub schema_digest: [u8; 32],
}

/// Window length for an alignment-64 IPC1 window over a `body_len`
/// body: prefix 8 + meta span 120 + body + pad(body,64) + EOS 8.
#[must_use]
pub fn window_len64(body_len: usize) -> usize {
    // cgap pad_to law: the pad rounds UP to a multiple of the alignment
    // (pad_to(8,64) == 64, pad_to(64,64) == 64).
    let pad = (body_len + 63) / 64 * 64;
    8 + 120 + body_len + pad + 8
}

/// u64 LE rows carrier offset within a window (lane-12 law; L7 stage 2
/// must confirm or relocate).
pub const ROWS_CARRIER_AT: usize = 104;

/// The rows carrier, fb era: `cgap:rows` decimal u64 in schema
/// custom_metadata (R4-I). Mirrors cgap_arrow::ipc::KEY_ROWS.
pub const KEY_ROWS: &str = "cgap:rows";

/// Extract the rows carrier from a verified window's metadata padding.
/// All-zero => [`ExportError::RowsCarrierMissing`].
///
/// DEPRECATED-BUT-FUNCTIONAL (R4-I): the legacy-era carrier; kept for
/// the legacy IPC1 walk leg (see [`ipc_walk::walk_verified`]'s dual-era
/// fork). New emitters write the fb-era `cgap:rows` key.
#[must_use]
pub fn rows_carrier(buf: &[u8]) -> Option<u64> {
    if buf.len() < ROWS_CARRIER_AT + 8 {
        return None;
    }
    let mut b = [0u8; 8];
    b.copy_from_slice(&buf[ROWS_CARRIER_AT..ROWS_CARRIER_AT + 8]);
    let v = u64::from_le_bytes(b);
    if v == 0 {
        None
    } else {
        Some(v)
    }
}

/// Extract the rows carrier from an fb-era window (`cgap:rows` decimal
/// u64 in the schema custom_metadata). Typed parse via cgap's own fb
/// read path: absent or non-decimal => `None` (never a guess). Returns
/// `None` too when the window refuses the framing parse.
#[must_use]
pub fn rows_carrier_fb(buf: &[u8]) -> Option<u64> {
    let mut body = vec![0u8; buf.len()];
    cgap_arrow::ipc::read_fb(buf, None, &mut body)
        .ok()
        .and_then(|r| r.rows)
}
