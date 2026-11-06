//! KD-1 strategy (b) import surface — the seal/publish half.
//!
//! Contract cites: arcus `KNOWN_DIVERGENCES.md` KD-1 row ("cgap ships
//! `cgap_import_arrow`; arcus imports via it instead of gap strategy (a)")
//! and the registered signature sketch `(array, schema,
//! schema_digest[32], buffer_lens[], out_view MAY be NULL = probe)`.
//!
//! Scope honesty (lane R-KD1, 2026-11-06): cgap-arrow carries **no Arrow
//! IPC reader dependency** (`[dependencies]` is empty — zero-dep, no_std
//! crate). A from-scratch IPC file/stream reader is out of lane budget.
//! What IS in budget — and what the consumer contract needs on the wire —
//! is the **seal/publish half**: take already-decoded Arrow buffers
//! (e.g. a C Data Interface `ArrowArray` produced by this crate's
//! `c_abi::export`), assemble the body, seal it into the O-ABI
//! `MetaHeader` form ("IPC1", the wire law published as
//! `arcus/docs/jira/v26.11.6/kd2-oabi-contract.md`), and emit the
//! encapsulated IPC window that
//! `internal/arcus/adapter/bitmaps_ipc.go` consumes.
//!
//! The **read half** (decoding a foreign Arrow IPC file/stream
//! flatbuffer into buffers) is registered as the follow-up in
//! `docs/jira/v26.11.6/kd1-surface-receipt.md`, not silently dropped.
//!
//! Host-gated: depends on [`crate::c_abi`] (alloc surface).

use crate::c_abi::{ArrowArray, ArrowSchema};
use crate::ipc::{self, window_len, IpcError};

/// Emitter parameters the registered signature sketch omitted: the O-ABI
/// seal law needs an alignment (8 or 64) and a monotonic `seq`. These are
/// the two disclosed signature divergences from the registered sketch
/// (see `docs/jira/v26.11.6/kd1-surface-receipt.md`, contract-fit).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImportParams {
    /// Alignment the emitter uses (8 or 64).
    pub alignment: u32,
    /// Monotonic message sequence (arcus binds this to the chainfile
    /// receipt `Index` per the KD-2 wire contract §3).
    pub seq: u32,
}

/// Typed error surface for the import seal half. Every failure class has
/// exactly one variant; the C-ABI wrapper (cgap-ffi) maps each to a
/// distinct code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportArrowError {
    /// Alignment not in {8, 64}.
    BadAlignment {
        /// Value observed.
        got: u32,
    },
    /// `buffer_lens.len() != array.n_buffers` (i64-cast), or `n_buffers`
    /// negative.
    BadBufferCount {
        /// Count observed on the array.
        got: i64,
        /// Count of lens supplied.
        supplied: usize,
    },
    /// A buffer pointer is null while its lens entry is nonzero.
    NullBuffer {
        /// Buffer index with the null pointer.
        index: usize,
    },
    /// `out` window too short for the sealed window
    /// ([`ipc::window_len`]); a too-small `out` with a null view is the
    /// probe contract, not an error (see [`import_arrow`]).
    Truncated {
        /// Bytes the sealed window needs.
        needed: usize,
        /// Bytes `out` had.
        got: usize,
    },
}

impl core::error::Error for ImportArrowError {}

impl core::fmt::Display for ImportArrowError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ImportArrowError::BadAlignment { got } => {
                write!(f, "bad alignment {got} (need 8 or 64)")
            }
            ImportArrowError::BadBufferCount { got, supplied } => {
                write!(f, "buffer count mismatch: array has {got}, {supplied} lens supplied")
            }
            ImportArrowError::NullBuffer { index } => {
                write!(f, "buffer {index} is null with a nonzero length")
            }
            ImportArrowError::Truncated { needed, got } => {
                write!(f, "out window truncated: need {needed}, got {got}")
            }
        }
    }
}

/// The sealed window's exact byte length for `buffer_lens` at
/// `params.alignment` — the probe answer (C-ABI: `out == NULL`).
#[must_use]
pub fn import_arrow_window_len(buffer_lens: &[u64], params: ImportParams) -> usize {
    let total: usize = buffer_lens.iter().map(|&l| l as usize).sum();
    window_len(total, params.alignment as usize)
}

/// Seal already-decoded Arrow buffers into an O-ABI ("IPC1") encapsulated
/// IPC window. Signature parity with the registered KD-1 sketch:
/// `(array, schema, schema_digest[32], buffer_lens[], out view)`.
///
/// - `array`: live C Data Interface array; buffers `0..n_buffers` are
///   concatenated in index order to form the body.
/// - `schema`: carried for signature parity; the O-ABI schema identity is
///   the caller-supplied `schema_digest` (KD-2 wire contract §3: cgap's
///   schema identity is the O-ABI BLAKE3 digest, not the flatbuffer
///   Schema form), so the struct itself is not re-digested here.
///   Disclosed, not silent — same receipt as above.
/// - `buffer_lens`: one exact byte length per buffer, `len == n_buffers`.
/// - `out`: sealed-window sink. Too small + non-null returns
///   [`ImportArrowError::Truncated`] (probe is expressed via the C-ABI
///   `NULL` view, mirroring the registered sketch's "MAY be NULL =
///   probe").
///
/// Returns the window bytes written ([`ipc::window_len`]).
///
/// # Panics
/// Panics only on an internal invariant break (`map_ipc`'s unreachable
/// seal-path arm) — no input combination reaches it.
pub fn import_arrow(
    array: &ArrowArray,
    _schema: &ArrowSchema,
    schema_digest: &[u8; 32],
    buffer_lens: &[u64],
    params: ImportParams,
    out: &mut [u8],
) -> Result<usize, ImportArrowError> {
    let alignment = params.alignment as usize;
    if !matches!(alignment, 8 | 64) {
        return Err(ImportArrowError::BadAlignment { got: params.alignment });
    }
    let n_buffers = array.n_buffers;
    if n_buffers < 0 || buffer_lens.len() != n_buffers as usize {
        return Err(ImportArrowError::BadBufferCount { got: n_buffers, supplied: buffer_lens.len() });
    }
    let total: usize = buffer_lens.iter().map(|&l| l as usize).sum();
    let needed = window_len(total, alignment);
    if out.len() < needed {
        return Err(ImportArrowError::Truncated { needed, got: out.len() });
    }

    // Body: concatenate buffers 0..n_buffers in index order, exact lens.
    // A null pointer table with a nonzero count is a typed refusal, never
    // a null-slice fabrication.
    if array.buffers.is_null() && n_buffers > 0 {
        return Err(ImportArrowError::NullBuffer { index: 0 });
    }
    let ptrs: &[*const u8] = if n_buffers == 0 {
        &[]
    } else {
        // SAFETY: `array.buffers` points to `n_buffers` valid pointers per
        // the C Data Interface contract; we read them for the array's
        // lifetime (borrowed `&ArrowArray`, no release call here).
        unsafe { core::slice::from_raw_parts(array.buffers as *const *const u8, n_buffers as usize) }
    };
    let mut body = alloc::vec::Vec::with_capacity(total);
    for i in 0..n_buffers as usize {
        let len = buffer_lens[i] as usize;
        if len == 0 {
            continue;
        }
        let p = ptrs[i];
        if p.is_null() {
            return Err(ImportArrowError::NullBuffer { index: i });
        }
        // SAFETY: caller pins each buffer for the call and certifies
        // `buffer_lens[i]` readable bytes (documented contract).
        let src = unsafe { core::slice::from_raw_parts(p, len) };
        body.extend_from_slice(src);
    }

    let meta = ipc::MetaHeader {
        magic: ipc::MAGIC,
        version: ipc::LAYOUT_VERSION,
        flags: 0,
        schema_digest: *schema_digest,
        body_len: total as u32,
        alignment: params.alignment,
        seq: params.seq,
        reserved0: 0,
        header_digest: 0,
        pad_tail: [0; 4],
    };
    // `emit` re-checks alignment/body_len/truncation and seals via
    // `seal_meta` (zeroes reserved fields, FNV-1a 0x00..0x38).
    let written = ipc::emit(out, &meta, &body).map_err(|e| map_ipc(e, needed))?;
    debug_assert_eq!(written, needed);
    Ok(written)
}

/// Map the internal `emit` failure classes onto the import error surface
/// (defensive: `import_arrow` pre-gates alignment/body_len/truncation, so
/// these are unreachable on lawful inputs — typed, not silent).
fn map_ipc(e: IpcError, needed: usize) -> ImportArrowError {
    match e {
        IpcError::BadAlignment { got } => ImportArrowError::BadAlignment { got },
        IpcError::BadBodyLen { .. } | IpcError::Truncated { .. } => {
            ImportArrowError::Truncated { needed, got: 0 }
        }
        other => {
            // No other class is reachable from the seal path; keep the
            // surface closed without swallowing it silently.
            unreachable!("ipc::emit seal path cannot produce {other:?}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{vec, vec::Vec};
    use crate::c_abi::export;
    use crate::primitive::PrimitiveBuilder;

    /// Real collaborator: a live exported C-Data-Interface pair, not a
    /// hand-built struct (Chicago discipline).
    fn exported_pair(n: usize) -> (ArrowArray, ArrowSchema, Vec<u64>, [u8; 32]) {
        let mut b = PrimitiveBuilder::<i32>::new();
        let mut i = 0;
        while i < n {
            if i % 3 == 2 {
                b.push(None); // keep the validity buffer real
            } else {
                b.push(Some(i as i32));
            }
            i += 1;
        }
        let prim = b.finish();
        let (array, schema) = export(prim, c"i".as_ptr());
        let lens = vec![
            u64::try_from(n.div_ceil(8)).unwrap(), // validity bitmap
            u64::try_from(n * 4).unwrap(),         // values
        ];
        let digest = crate::blake3::hash(b"cgap:import-arrow-test-schema:i32");
        (array, schema, lens, digest)
    }

    fn buffer_ptrs(array: &ArrowArray) -> Vec<*const u8> {
        let n = array.n_buffers as usize;
        // SAFETY: test-side read of the exported pair's pointer table.
        unsafe { core::slice::from_raw_parts(array.buffers as *const *const u8, n) }.to_vec()
    }

    #[test]
    fn seals_and_round_trips_through_ipc_read() {
        let (array, schema, lens, digest) = exported_pair(16);
        let params = ImportParams { alignment: 64, seq: 7 };
        let needed = import_arrow_window_len(&lens, params);
        let mut out = vec![0xA5u8; needed + 16]; // poison tail proves exact write
        let written = import_arrow(&array, &schema, &digest, &lens, params, &mut out)
            .expect("seal succeeds on a lawful exported pair");
        assert_eq!(written, needed);

        // Header law (KD-2 contract §2): magic IPC1 at 0x00 of the meta
        // header (window offset 8), stored LE as the u32 0x4950_4331.
        assert_eq!(u32::from_le_bytes(out[8 + 0x00..8 + 0x04].try_into().unwrap()), ipc::MAGIC);
        assert_eq!(u16::from_le_bytes([out[8 + 0x04], out[8 + 0x05]]), ipc::LAYOUT_VERSION);
        assert_eq!(&out[8 + 0x08..8 + 0x28], &digest[..]);
        assert_eq!(
            u32::from_le_bytes(out[8 + 0x30..8 + 0x34].try_into().unwrap()),
            7
        );
        assert_eq!(u32::from_le_bytes(out[8 + 0x34..8 + 0x38].try_into().unwrap()), 0);
        assert_eq!(&out[8 + 0x3C..8 + 0x40], &[0; 4][..]);
        // Prefix: continuation + meta span on a 64-byte boundary.
        assert_eq!(u32::from_le_bytes(out[0..4].try_into().unwrap()), ipc::CONTINUATION);
        let meta_span = u32::from_le_bytes(out[4..8].try_into().unwrap()) as usize;
        assert_eq!((8 + meta_span) % 64, 0);

        // Round trip through the real reader.
        let mut body = vec![0u8; lens.iter().sum::<u64>() as usize];
        let (n, meta) = ipc::read_with_meta(&out[..written], Some(&digest), &mut body)
            .expect("sealed window reads back clean");
        assert_eq!(n, body.len());
        assert_eq!(meta.schema_digest, digest);
        assert_eq!(meta.seq, 7);
        assert_eq!(meta.alignment, 64);
        // Body bytes equal the concatenated buffers, byte for byte.
        for (i, &p) in buffer_ptrs(&array).iter().enumerate() {
            let expect = unsafe { core::slice::from_raw_parts(p, lens[i] as usize) };
            let at = lens[..i].iter().sum::<u64>() as usize;
            assert_eq!(&body[at..at + lens[i] as usize], expect, "buffer {i} bytes");
        }
    }

    #[test]
    fn probe_len_matches_sealed_window() {
        let (array, _schema, lens, digest) = exported_pair(5);
        let params = ImportParams { alignment: 8, seq: 0 };
        let mut out = vec![0u8; import_arrow_window_len(&lens, params)];
        let written = import_arrow(&array, &_schema, &digest, &lens, params, &mut out).unwrap();
        assert_eq!(written, out.len());
        assert_eq!(u32::from_le_bytes(out[8..12].try_into().unwrap()), ipc::MAGIC);
    }

    #[test]
    fn rejects_bad_alignment_buffer_count_null_buffer_and_truncation() {
        let (array, schema, lens, digest) = exported_pair(8);
        let mut out = vec![0u8; import_arrow_window_len(&lens, ImportParams { alignment: 64, seq: 0 })];
        assert_eq!(
            import_arrow(&array, &schema, &digest, &lens, ImportParams { alignment: 13, seq: 0 }, &mut out),
            Err(ImportArrowError::BadAlignment { got: 13 })
        );
        let mut short_lens = lens.clone();
        short_lens.pop();
        assert_eq!(
            import_arrow(&array, &schema, &digest, &short_lens, ImportParams { alignment: 8, seq: 0 }, &mut out),
            Err(ImportArrowError::BadBufferCount { got: 2, supplied: 1 })
        );
        let lens_null = lens.clone();
        // Null pointer table: typed refusal, not silent UB.
        let null_array = ArrowArray {
            length: 8,
            null_count: 0,
            offset: 0,
            n_buffers: 2,
            n_children: 0,
            buffers: core::ptr::null_mut(),
            children: core::ptr::null_mut(),
            dictionary: core::ptr::null_mut(),
            release: None,
            private_data: core::ptr::null_mut(),
        };
        assert_eq!(
            import_arrow(&null_array, &schema, &digest, &lens_null, ImportParams { alignment: 8, seq: 0 }, &mut out),
            Err(ImportArrowError::NullBuffer { index: 0 })
        );
        let mut half = vec![0u8; import_arrow_window_len(&lens, ImportParams { alignment: 8, seq: 0 }) / 2];
        assert!(matches!(
            import_arrow(&array, &schema, &digest, &lens, ImportParams { alignment: 8, seq: 0 }, &mut half),
            Err(ImportArrowError::Truncated { .. })
        ));
    }
}
