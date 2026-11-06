//! KD-1 strategy (b) — the `cgap_import_arrow` C-ABI symbol
//! (`#[no_mangle] pub extern "C"`, host surface like the slot exports).
//!
//! Close condition cite: arcus `KNOWN_DIVERGENCES.md` KD-1 — "cgap lands
//! `cgap_import_arrow` with that signature". The registered sketch was
//! `(array, schema, schema_digest[32], buffer_lens[], out_view MAY be NULL
//! = probe)`. The landed signature is that sketch plus two emitter
//! params the seal law requires (`alignment`, `seq`) — the disclosed
//! divergence, documented in
//! `docs/jira/v26.11.6/kd1-surface-receipt.md`.
//!
//! Semantics (seal/publish half): concatenates the array's already-decoded
//! buffers per `buffer_lens`, seals them into the O-ABI "IPC1"
//! `MetaHeader` window (`cgap-arrow/src/ipc.rs` wire law, consumed by
//! arcus `internal/arcus/adapter/bitmaps_ipc.go`), writes it to `out`.
//! `out == NULL` is the probe: returns 0 and writes the required window
//! length to `*out_len`.
//!
//! Return codes: 0 = sealed; negative = typed failure
//! (`CGAP_IMPORT_*` below); positive values are never returned.
//!
//! The Arrow IPC **read half** (foreign file/stream decode) is registered
//! as the follow-up in the KD-1 receipt — registered absence, not silence.

#![allow(dead_code)]

use cgap_arrow::c_abi::{ArrowArray, ArrowSchema};
use cgap_arrow::import_arrow::{import_arrow, import_arrow_window_len, ImportArrowError, ImportParams};

/// Return codes for [`cgap_import_arrow`]. Distinct per failure class.
pub mod status {
    /// Sealed successfully (`*out_len` = window bytes written).
    pub const OK: i32 = 0;
    /// `schema_digest` / `buffer_lens` / `out_len` pointer is null.
    pub const NULL_ARG: i32 = -1;
    /// Alignment not in {8, 64}.
    pub const BAD_ALIGNMENT: i32 = -2;
    /// `buffer_lens.len() != n_buffers` (or negative count).
    pub const BAD_BUFFER_COUNT: i32 = -3;
    /// A buffer pointer is null with a nonzero length (or the array's
    /// buffer pointer table is null).
    pub const NULL_BUFFER: i32 = -4;
    /// Non-null `out` smaller than the sealed window. (The probe is
    /// `out == NULL`, never a small buffer.)
    pub const TRUNCATED: i32 = -5;
}

fn code(e: ImportArrowError) -> i32 {
    match e {
        ImportArrowError::BadAlignment { .. } => status::BAD_ALIGNMENT,
        ImportArrowError::BadBufferCount { .. } => status::BAD_BUFFER_COUNT,
        ImportArrowError::NullBuffer { .. } => status::NULL_BUFFER,
        ImportArrowError::Truncated { .. } => status::TRUNCATED,
    }
}

/// Seal already-decoded Arrow buffers into an O-ABI ("IPC1") IPC window.
///
/// # Safety
/// Caller certifies: `array`/`schema` are live C Data Interface structs;
/// `schema_digest` points at 32 readable bytes; `buffer_lens` points at
/// `n_buffers` readable `u64`s, each the exact readable byte length of
/// the corresponding buffer pinned for the duration of the call;
/// `out_len` (if non-null) is writable. `array.buffers` must point at
/// `n_buffers` valid pointers.
#[no_mangle]
#[cfg(not(target_os = "none"))]
pub unsafe extern "C" fn cgap_import_arrow(
    array: *const ArrowArray,
    schema: *const ArrowSchema,
    schema_digest: *const u8,
    buffer_lens: *const u64,
    n_buffers: usize,
    alignment: u32,
    seq: u32,
    out: *mut u8, // MAY be NULL = probe
    out_cap: usize,
    out_len: *mut usize,
) -> i32 {
    if schema_digest.is_null() || buffer_lens.is_null() || out_len.is_null() || array.is_null() || schema.is_null() {
        return status::NULL_ARG;
    }
    let lens = core::slice::from_raw_parts(buffer_lens, n_buffers);
    let digest: &[u8; 32] = &*schema_digest.cast::<[u8; 32]>();
    let params = ImportParams { alignment, seq };
    // Alignment gates BOTH paths: a probe with an unlawful alignment is a
    // typed refusal, not a fabricated window length.
    if !matches!(alignment, 8 | 64) {
        return status::BAD_ALIGNMENT;
    }

    if out.is_null() {
        // Probe: required window length, no seal.
        *out_len = import_arrow_window_len(lens, params);
        return status::OK;
    }
    let sink = core::slice::from_raw_parts_mut(out, out_cap);
    match import_arrow(&*array, &*schema, digest, lens, params, sink) {
        Ok(written) => {
            *out_len = written;
            status::OK
        }
        Err(e) => code(e),
    }
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::status;
    use super::*;
    use cgap_arrow::c_abi::export;
    use cgap_arrow::primitive::PrimitiveBuilder;

    fn exported_pair(n: usize) -> (Box<ArrowArray>, Box<ArrowSchema>, Vec<u64>, [u8; 32]) {
        let mut b = PrimitiveBuilder::<i32>::new();
        for i in 0..n {
            if i % 3 == 2 {
                b.push(None);
            } else {
                b.push(Some(i as i32));
            }
        }
        let (array, schema) = export(b.finish(), c"i".as_ptr());
        let lens = vec![u64::try_from(n.div_ceil(8)).unwrap(), u64::try_from(n * 4).unwrap()];
        let digest = cgap_arrow::blake3::hash(b"cgap:import-arrow-ffi-test:i32");
        (Box::new(array), Box::new(schema), lens, digest)
    }

    #[test]
    fn probe_then_seal_then_ipc_read_back() {
        let (array, schema, lens, digest) = exported_pair(16);
        let mut out_len = 0usize;

        // Probe: NULL out view.
        let rc = unsafe {
            cgap_import_arrow(
                array.as_ref(), schema.as_ref(), digest.as_ptr(), lens.as_ptr(),
                lens.len(), 64, 7, core::ptr::null_mut(), 0, &mut out_len,
            )
        };
        assert_eq!(rc, status::OK);
        let need = out_len;
        assert_eq!(need, cgap_arrow::import_arrow::import_arrow_window_len(&lens, ImportParams { alignment: 64, seq: 7 }));

        let mut buf = vec![0u8; need];
        let rc = unsafe {
            cgap_import_arrow(
                array.as_ref(), schema.as_ref(), digest.as_ptr(), lens.as_ptr(),
                lens.len(), 64, 7, buf.as_mut_ptr(), buf.len(), &mut out_len,
            )
        };
        assert_eq!(rc, status::OK);
        assert_eq!(out_len, need);
        assert_eq!(u32::from_le_bytes(buf[8..12].try_into().unwrap()), cgap_arrow::ipc::MAGIC);
        assert_eq!(&buf[8 + 0x08..8 + 0x28], &digest[..]);

        // Real reader round trip through cgap-arrow's ipc law.
        let mut body = vec![0u8; lens.iter().sum::<u64>() as usize];
        let (n, meta) = cgap_arrow::ipc::read_with_meta(&buf, Some(&digest), &mut body)
            .expect("sealed window reads back clean");
        assert_eq!(n, body.len());
        assert_eq!(meta.seq, 7);
        assert_eq!(meta.alignment, 64);
    }

    #[test]
    fn typed_refusals_map_to_distinct_codes() {
        let (array, schema, lens, digest) = exported_pair(8);
        let mut out_len = 0usize;
        let rc = unsafe {
            cgap_import_arrow(
                array.as_ref(), schema.as_ref(), digest.as_ptr(), lens.as_ptr(),
                lens.len(), 13, 0, core::ptr::null_mut(), 0, &mut out_len,
            )
        };
        assert_eq!(rc, status::BAD_ALIGNMENT);

        let rc = unsafe {
            cgap_import_arrow(
                array.as_ref(), schema.as_ref(), core::ptr::null(), lens.as_ptr(),
                lens.len(), 64, 0, core::ptr::null_mut(), 0, &mut out_len,
            )
        };
        assert_eq!(rc, status::NULL_ARG);

        // Non-null out, too small: TRUNCATED (probe is NULL, not small).
        let mut small = [0u8; 16];
        let rc = unsafe {
            cgap_import_arrow(
                array.as_ref(), schema.as_ref(), digest.as_ptr(), lens.as_ptr(),
                lens.len(), 8, 0, small.as_mut_ptr(), small.len(), &mut out_len,
            )
        };
        assert_eq!(rc, status::TRUNCATED);
    }
}
