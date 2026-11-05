//! ipc_walk — verified IPC1 window walk over journal frame payloads.
//! All framing gates are cgap's OWN (cgap_arrow::ipc::read_verified) —
//! this module adds the lane-12 rows-carrier extraction and the
//! window/body layout. The tamper tier (blake3 body digest) is enforced
//! HERE at export; smalt's Erlang walker mirrors the same gate order
//! with presence-only digest checking (no blake3 on BEAM).

use crate::{ExportError, Window, rows_carrier};
use cgap_arrow::ipc;

/// Body layout exported by this crate (lane-12 law): N rows x one u32
/// column "v", little-endian. Column projection happens at the parquet
/// sink, not in the BEAM mount.
pub const COLUMN_NAME: &str = "v";

/// Walk one window: cgap's full framing + blake3 tamper tier, then the
/// rows carrier (all-zero => typed refusal, never a guess).
pub fn walk_verified(buf: &[u8]) -> Result<Window, ExportError> {
    let mut body = vec![0u8; buf.len()];
    let n = ipc::read_verified(buf, None, &mut body)
        .map_err(ExportError::Ipc)?;
    body.truncate(n);
    let rows = rows_carrier(buf).ok_or(ExportError::RowsCarrierMissing)?;
    // Re-read the sealed header for identity fields (post-gate, so the
    // header is sealed and lawful).
    let mut meta_out = vec![0u8; buf.len()];
    let (_, meta) = ipc::read_with_meta(buf, None, &mut meta_out)
        .map_err(ExportError::Ipc)?;
    if body.len() != rows as usize * 4 {
        return Err(ExportError::BodyLayout {
            body_len: n as u32,
            rows,
        });
    }
    Ok(Window {
        body,
        rows,
        seq: meta.seq,
        schema_digest: meta.schema_digest,
    })
}

/// Decode the u32 column values from a walked window body.
pub fn column_values(win: &Window) -> Vec<u32> {
    win.body
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
        .collect()
}
