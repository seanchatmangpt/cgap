//! parquet_sink — column projection + parquet export over the EXISTING
//! cgap-parquet Tier H path (StaticPageWriter -> seal -> FileSink).
//! Reuse, never fork. Column projection lives HERE (lane-12 law): the
//! BEAM mount never projects columns.

use crate::{ExportError, ipc_walk, segment_source};
use cgap_arrow::ipc;
use cgap_parquet::footer::{self, SealedBlock};
use cgap_parquet::mmap_writer::{self, BlockSink, FileSink, SlabView, StaticPageWriter};
use std::path::Path;

/// The exported artifact identity: sealed-block bytes written through the
/// real FileSink, plus the digest needed to verify_seal the file later.
pub struct Exported {
    /// blake3-class block digest (cgap footer::block_digest) — pass it to
    /// footer::verify_seal when reading the file back.
    pub digest: [u8; 32],
    /// Rows exported (one row group, one lane per projected column).
    pub rows: u64,
    /// Bytes written to the file.
    pub bytes: usize,
}

/// Project the walked windows onto `columns` (currently the single u32
/// column "v") and export one sealed parquet block to `out_path` via the
/// REAL FileSink. Fail-closed: every window must walk verified, every
/// window becomes one row group (no partial export).
pub fn export_windows(
    windows: &[crate::Window],
    columns: &[&str],
    out_path: &Path,
) -> Result<Exported, ExportError> {
    if columns != [ipc_walk::COLUMN_NAME] {
        return Err(ExportError::Io);
    }
    let mut buf = vec![0u8; 4096 + windows.len() * 64 * 1024];
    let mut block = {
        let mut writer = StaticPageWriter::new(&mut buf);
        for w in windows {
            let mut values = ipc_walk::column_values(w);
            if values.len() != w.rows as usize {
                return Err(ExportError::BodyLayout {
                    body_len: w.body.len() as u32,
                    rows: w.rows,
                });
        }
            let view: [&[u32]; 1] = [values.as_slice()];
            let slab = SlabView { lanes: &view, rows: w.rows as u32, bits: 8 };
            writer.write_slab(&slab).map_err(ExportError::Parquet)?;
        }
        writer.seal(None).map_err(ExportError::Parquet)?
    };
    let digest = *block.digest();
    let sealed = block.bytes().to_vec();
    {
        let f = std::fs::File::create(out_path).map_err(|_| ExportError::Io)?;
        let sink = FileSink::new(f, sealed.clone().into_boxed_slice());
        block.flush_to(sink).map_err(ExportError::Parquet)?;
    }
    let bytes = std::fs::metadata(out_path).map_err(|_| ExportError::Io)?.len() as usize;
    Ok(Exported { digest, rows: windows.iter().map(|w| w.rows).sum(), bytes })
}
/// One lane, one row group, bits=8, sealed, flushed through the real
/// FileSink to `out_path`.
pub fn write_lane_block(
    lanes: &[Vec<u32>],
    rows: u32,
    out_path: &Path,
) -> Result<Exported, ExportError> {
    if rows == 0 || lanes.is_empty() {
        return Err(ExportError::Parquet(
            cgap_parquet::ParquetError::UnsupportedBitWidth(0),
        ));
    }
    let cap = 4096 + lanes.len() * rows as usize * 8;
    let mut buf = vec![0u8; cap];
    let block = {
        let mut writer = StaticPageWriter::new(&mut buf);
        let view: Vec<&[u32]> = lanes.iter().map(|l| l.as_slice()).collect();
        let slab = SlabView { lanes: &view, rows, bits: 8 };
        writer
            .write_slab(&slab)
            .map_err(ExportError::Parquet)?;
        writer.seal(None).map_err(ExportError::Parquet)?
    };
    let digest = *block.digest();
    let sealed = block.bytes().to_vec();
    {
        let f = std::fs::File::create(out_path).map_err(|_| ExportError::Io)?;
        let sink = FileSink::new(f, sealed.clone().into_boxed_slice());
        block.flush_to(sink).map_err(ExportError::Parquet)?;
    }
    let bytes = std::fs::metadata(out_path).map_err(|_| ExportError::Io)?.len() as usize;
    Ok(Exported { digest, rows: u64::from(rows), bytes })
}

/// Read back + verify + decode row group 0, lane 0 of a sealed block
/// file: the VALUE-equality arm of the fidelity court (RLE/page decode
/// via cgap's page decoder).
pub fn read_back_lane(
    path: &Path,
    digest: &[u8; 32],
    expect_rows: usize,
) -> Result<Vec<u32>, ExportError> {
    let bytes = std::fs::read(path).map_err(|_| ExportError::Io)?;
    let v = footer::verify_seal(&bytes, digest).map_err(|_| ExportError::Io)?;
    if v.rows != expect_rows as u64 {
        return Err(ExportError::BodyLayout {
            body_len: v.body_len as u32,
            rows: v.rows,
        });
    }
    // one page per row group (single lane); page i sits at the running
    // sum of the row-group spans from verify_seal
    let mut out: Vec<u32> = Vec::new();
    let mut at = 0usize;
    for i in 0..v.spans_len {
        let span = v.spans[i] as usize;
        let page = bytes
            .get(at..at + span)
            .ok_or(ExportError::Io)?;
        let mut slot = vec![0u32; expect_rows.max(16)];
        let n = cgap_parquet::page::decode_page_into(page, &mut slot)
            .map_err(ExportError::Parquet)?;
        out.extend_from_slice(&slot[..n]);
        at += span;
    }
    Ok(out)
}
/// Convenience: full sealed-dir -> parquet export (the `main` path).
pub fn export_segment_dir(dir: &Path, out_path: &Path) -> Result<Exported, ExportError> {
    let segs = segment_source::discover_sealed_segments(dir)?;
    let mut windows = Vec::new();
    for seg in segs {
        let segment = segment_source::read_sealed_segment(&seg)?;
        for f in &segment.frames {
            windows.push(ipc_walk::walk_verified(&f.payload)?);
        }
    }
    export_windows(&windows, &[ipc_walk::COLUMN_NAME], out_path)
}
