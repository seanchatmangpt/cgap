//! ipc_walk — verified IPC1 window walk over journal frame payloads.
//! All framing gates are cgap's OWN (cgap_arrow::ipc) — this module adds
//! the lane-12 rows-carrier extraction and the window/body layout. The
//! tamper tier (blake3 body digest) is enforced HERE at export; smalt's
//! Erlang walker mirrors the same gate order with presence-only digest
//! checking (no blake3 on BEAM).
//!
//! DUAL-ERA FORK (R4-I): the walk forks on window era. A legacy
//! read_verified refusal of `LegacyReadForeignMeta` means the window's
//! metadata is the flatbuffer Message (fb era): retry through
//! read_verified_fb. Rows come from the era's carrier — legacy: the
//! u64 LE at window offset 104..112 ([`crate::rows_carrier`]); fb: the
//! `cgap:rows` decimal key ([`crate::rows_carrier_fb`] class, surfaced
//! directly as [`cgap_arrow::ipc::FbRead::rows`]). Both refuse =>
//! typed [`ExportError::RowsCarrierMissing`] — never a guess.

use crate::{ExportError, Window, rows_carrier};
use cgap_arrow::ipc::{self, IpcError};

/// Body layout exported by this crate (lane-12 law): N rows x one u32
/// column "v", little-endian. Column projection happens at the parquet
/// sink, not in the BEAM mount.
pub const COLUMN_NAME: &str = "v";

/// Walk one window: cgap's full framing + blake3 tamper tier, then the
/// era's rows carrier (absent/unparseable => typed refusal, never a
/// guess).
pub fn walk_verified(buf: &[u8]) -> Result<Window, ExportError> {
    let mut body = vec![0u8; buf.len()];
    match ipc::read_verified(buf, None, &mut body) {
        Ok(n) => {
            // Legacy era: framing + tamper tier passed through the IPC1
            // path; carrier at 104..112.
            body.truncate(n);
            let rows = rows_carrier(buf).ok_or(ExportError::RowsCarrierMissing)?;
            // Re-read the sealed header for identity fields (post-gate,
            // so the header is sealed and lawful).
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
        Err(IpcError::LegacyReadForeignMeta) => {
            // fb era: retry through the fb read (typed cross-era refusal
            // already excluded the legacy path).
            let r = ipc::read_verified_fb(buf, None, &mut body)
                .map_err(ExportError::Ipc)?;
            body.truncate(r.body_len);
            let rows = r.rows.ok_or(ExportError::RowsCarrierMissing)?;
            if body.len() != rows as usize * 4 {
                return Err(ExportError::BodyLayout {
                    body_len: r.body_len as u32,
                    rows,
                });
            }
            Ok(Window {
                body,
                rows,
                // fb windows carry no seq word; identity is the schema
                // digest (schema_blake3 seam, falling back to the region
                // digest).
                seq: 0,
                schema_digest: r.schema_blake3.unwrap_or(r.schema_digest),
            }
            )
        }
        Err(e) => Err(ExportError::Ipc(e)),
    }
}

/// Decode the u32 column values from a walked window body.
pub fn column_values(win: &Window) -> Vec<u32> {
    win.body
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
        .collect()
}
