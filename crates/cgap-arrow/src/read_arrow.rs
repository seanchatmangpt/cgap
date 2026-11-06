//! KD-1 strategy (b) read half: decode an O-ABI ("IPC1") encapsulated Arrow window.
//!
//! Inverse of `import_arrow`: accepts an encapsulated memory window emitted by
//! `cgap_import_arrow` or `ipc::emit`, validates its framing (magic, digests, alignment,
//! EOS), and projects non-allocating, zero-copy buffer views into caller-provided
//! pointer/length slots.

use crate::ipc::{self, IpcError, MetaHeader};

/// Status/error codes for Arrow IPC reading.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadArrowError {
    /// Window framing or header is truncated / malformed.
    Truncated { needed: usize, got: usize },
    /// Magic is not "IPC1" (0x49504331).
    BadMagic { got: u32 },
    /// Schema digest mismatch against caller expectation.
    SchemaMismatch,
    /// Header checksum failure (tamper detection).
    HeaderDigestMismatch,
    /// Caller-provided buffer table is insufficient for requested buffers.
    BadBufferCount { needed: usize, supplied: usize },
    /// Caller destination buffer smaller than decoded body.
    BodyTruncated { needed: usize, got: usize },
}

impl From<IpcError> for ReadArrowError {
    fn from(e: IpcError) -> Self {
        match e {
            IpcError::Truncated { needed, got } => ReadArrowError::Truncated { needed, got },
            IpcError::BadMagic { got } => ReadArrowError::BadMagic { got },
            IpcError::SchemaMismatch { .. } => ReadArrowError::SchemaMismatch,
            IpcError::HeaderDigestMismatch { .. } => ReadArrowError::HeaderDigestMismatch,
            IpcError::BadBodyLen { expected, got } => ReadArrowError::Truncated {
                needed: expected as usize,
                got: got as usize,
            },
            _ => ReadArrowError::Truncated { needed: 0, got: 0 },
        }
    }
}

/// Decoded window summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodedArrowWindow {
    /// Emitter sequence counter.
    pub seq: u32,
    /// Decoded body byte length.
    pub body_len: usize,
    /// Total window bytes consumed.
    pub window_len: usize,
    /// Alignment declared in header (8 or 64).
    pub alignment: u32,
    /// 32-byte BLAKE3 schema digest.
    pub schema_digest: [u8; 32],
}

/// Decode an O-ABI IPC1 window and copy its payload into `body_out`.
///
/// # Panics
/// Does not panic. All validation errors are returned as typed [`ReadArrowError`].
pub fn read_arrow(
    window: &[u8],
    expected_schema: Option<&[u8; 32]>,
    body_out: &mut [u8],
) -> Result<DecodedArrowWindow, ReadArrowError> {
    let (written, meta): (usize, MetaHeader) = ipc::read_with_meta(window, expected_schema, body_out)?;
    let alignment = meta.alignment as usize;
    let total_window = ipc::window_len(meta.body_len as usize, alignment);

    Ok(DecodedArrowWindow {
        seq: meta.seq,
        body_len: written,
        window_len: total_window,
        alignment: meta.alignment,
        schema_digest: meta.schema_digest,
    })
}

/// Probe the required body length of an O-ABI IPC1 window without copying payload.
pub fn probe_arrow_body_len(window: &[u8]) -> Result<usize, ReadArrowError> {
    if window.len() < ipc::PREFIX_SIZE + ipc::META_HEADER_SIZE {
        return Err(ReadArrowError::Truncated {
            needed: ipc::PREFIX_SIZE + ipc::META_HEADER_SIZE,
            got: window.len(),
        });
    }
    let cont = u32::from_le_bytes([window[0], window[1], window[2], window[3]]);
    if cont != ipc::CONTINUATION {
        return Err(ReadArrowError::BadMagic { got: cont });
    }
    // MetaHeader starts at window[8]
    let magic = u32::from_le_bytes([window[8], window[9], window[10], window[11]]);
    if magic != ipc::MAGIC {
        return Err(ReadArrowError::BadMagic { got: magic });
    }
    let body_len = u32::from_le_bytes([window[48], window[49], window[50], window[51]]) as usize;
    Ok(body_len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use crate::import_arrow::{import_arrow, import_arrow_window_len, ImportParams};
    use crate::primitive::PrimitiveBuilder;
    use crate::c_abi::export;

    #[test]
    fn read_arrow_roundtrip_with_import_arrow() {
        let mut b = PrimitiveBuilder::<i32>::new();
        b.push(Some(10));
        b.push(None);
        b.push(Some(30));
        let prim = b.finish();
        let (array, schema) = export(prim, c"i".as_ptr());
        let lens = [1u64, 12u64]; // 1 byte bitmap, 12 bytes i32 values
        let digest = crate::blake3::hash(b"test:schema");
        let params = ImportParams { alignment: 64, seq: 42 };

        let win_size = import_arrow_window_len(&lens, params);
        let mut window = vec![0u8; win_size];
        let written = import_arrow(&array, &schema, &digest, &lens, params, &mut window).unwrap();
        assert_eq!(written, win_size);

        // Probe body length
        let probed = probe_arrow_body_len(&window).unwrap();
        assert_eq!(probed, 13); // 1 + 12

        // Read and decode
        let mut body_out = vec![0u8; probed];
        let decoded = read_arrow(&window, Some(&digest), &mut body_out).unwrap();
        assert_eq!(decoded.seq, 42);
        assert_eq!(decoded.body_len, 13);
        assert_eq!(decoded.alignment, 64);
        assert_eq!(decoded.schema_digest, digest);
    }
}
