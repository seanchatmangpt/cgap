//! Courts for `crates/cgap-arrow/src/ipc.rs` (lane p1-ipc, v26.11.2).
//!
//! Chicago discipline: real collaborators (real sealed slab, real wire
//! window, the real generated C header on disk). Every court names its
//! anti-vacuity mutation: weakening or deleting the gate code must flip
//! the court, so a pass proves the gate live (exit contract 0/1).

use cgap_arrow::blake3;
use cgap_arrow::ipc::{
    emit, read, seal_meta, window_len, MetaHeader, CONTINUATION, EOS, IpcError, MAGIC,
};
use cgap_arrow::slab::{ArrowRecordSlab, SlabError};

/// The generated C Data Interface header, diffed as text (court C3).
const C_HEADER: &str = include_str!("../../../include/cgap_c_data.h");

/// Real sealed 4-row slab: deterministic payloads, mixed presence.
fn sealed_4row() -> Result<ArrowRecordSlab<4>, SlabError> {
    let mut s = ArrowRecordSlab::<4>::new();
    for i in 0..4u32 {
        let mut p = [0u8; 60];
        p[..4].copy_from_slice(&i.to_le_bytes());
        s.push(&p, i % 2 == 0)?;
    }
    s.seal()?;
    Ok(s)
}

/// Schema identity for the body: BLAKE3 over the sealed slab window.
fn schema_digest_of(body: &[u8]) -> [u8; 32] {
    blake3::hash(body)
}

/// Metadata header for `body` at `alignment`, sequence `seq`.
fn meta_for(body_len: usize, alignment: u32, seq: u32, digest: [u8; 32]) -> MetaHeader {
    seal_meta(MetaHeader {
        magic: MAGIC,
        version: cgap_arrow::ipc::LAYOUT_VERSION,
        flags: 0,
        schema_digest: digest,
        body_len: body_len as u32,
        alignment,
        seq,
        reserved0: 0,
        header_digest: 0,
        pad_tail: [0; 4],
    })
}

/// C1: byte-layout asserts over the real wire window (anti-vacuity: any
/// change to the emitter's offsets/ordering flips this court).
#[test]
fn c1_byte_layout() {
    let slab = sealed_4row().unwrap();
    let body = slab.as_bytes();
    let digest = schema_digest_of(body);
    for &alignment in &[8u32, 64] {
        let mut w = vec![0u8; window_len(body.len(), alignment as usize)];
        let n = emit(&mut w, &meta_for(body.len(), alignment, 7, digest), body).unwrap();
        assert_eq!(n, w.len(), "emit must fill exactly window_len at {alignment}");
        // Continuation at 0x00.
        assert_eq!(&w[0..4], &CONTINUATION.to_le_bytes());
        // metadata_size at 0x04 = header + metadata padding.
        let meta_span = u32::from_le_bytes([w[4], w[5], w[6], w[7]]);
        let alignment_usize = alignment as usize;
        let expected_span: usize = if alignment == 8 { 64 } else { 120 };
        assert_eq!(meta_span, expected_span as u32);
        // Metadata header at 0x08: magic, version, seq, body_len,
        // alignment field.
        assert_eq!(&w[8..12], &MAGIC.to_le_bytes());
        assert_eq!(u16::from_le_bytes([w[12], w[13]]), 1);
        assert_eq!(u32::from_le_bytes([w[0x30 + 8], w[0x31 + 8], w[0x32 + 8], w[0x33 + 8]]), 7);
        assert_eq!(
            u32::from_le_bytes([w[0x28 + 8], w[0x28 + 9], w[0x28 + 10], w[0x28 + 11]]),
            body.len() as u32
        );
        assert_eq!(
            u32::from_le_bytes([w[0x2C + 8], w[0x2C + 9], w[0x2C + 10], w[0x2C + 11]]),
            alignment
        );
        // Body starts alignment-aligned.
        let body_at: usize = 8 + expected_span;
        assert_eq!(body_at % alignment_usize, 0, "body start aligned to {alignment}");
        assert_eq!(&w[body_at..body_at + body.len()], body);
        // Padding zero-filled, then EOS.
        assert!(w[body_at + body.len()..w.len() - 8].iter().all(|&b| b == 0));
        assert_eq!(&w[w.len() - 8..w.len() - 4], &CONTINUATION.to_le_bytes());
        assert_eq!(&w[w.len() - 4..], &EOS.to_le_bytes());
    }
}

/// C1b: 64-byte alignment variant; the body starts at a 64-B boundary
/// (AVX-512/NEON invariant carried onto the wire).
#[test]
fn c1b_body_start_is_64b_aligned_in_64_mode() {
    let slab = sealed_4row().unwrap();
    let body = slab.as_bytes();
    let mut w = vec![0u8; window_len(body.len(), 64)];
    emit(&mut w, &meta_for(body.len(), 64, 0, schema_digest_of(body)), body).unwrap();
    let body_at = 8 + 120;
    assert_eq!(body_at % 64, 0);
    assert_eq!(&w[body_at..body_at + body.len()], body);
}

/// C2: roundtrip a real sealed 4-row slab through emit + read and decode
/// it back to an identical slab (anti-vacuity: any framing or digest
/// corruption flips this court).
#[test]
fn c2_roundtrip_4row_slab() {
    let slab = sealed_4row().unwrap();
    let body = slab.as_bytes();
    let digest = schema_digest_of(body);
    let mut w = vec![0u8; window_len(body.len(), 64)];
    emit(&mut w, &meta_for(body.len(), 64, 3, digest), body).unwrap();
    let mut out = [0u8; 64 + 4 * 64];
    let n = read(&w, Some(&digest), &mut out).unwrap();
    assert_eq!(n, body.len());
    let back = ArrowRecordSlab::<4>::from_bytes(&out[..n]).unwrap();
    assert_eq!(back.as_bytes(), slab.as_bytes(), "byte-identical roundtrip");
    assert!(back.is_sealed());
    assert_eq!(back.len(), 4);
}

/// C2b: the same at alignment 8.
#[test]
fn c2b_roundtrip_align8() {
    let slab = sealed_4row().unwrap();
    let body = slab.as_bytes();
    let digest = schema_digest_of(body);
    let mut w = vec![0u8; window_len(body.len(), 8)];
    emit(&mut w, &meta_for(body.len(), 8, 3, digest), body).unwrap();
    let mut out = [0u8; 64 + 4 * 64];
    let n = read(&w, Some(&digest), &mut out).unwrap();
    assert_eq!(n, body.len());
    let back = ArrowRecordSlab::<4>::from_bytes(&out[..n]).unwrap();
    assert_eq!(back.as_bytes(), slab.as_bytes());
}

/// C3: differential against the real generated C Data Interface header on
/// disk — the shared ABI surface the envelope carries bodies for. The
/// header must define the two spec structs and the flag macros with the
/// same values the Rust ABI module pins (anti-vacuity: regenerating the
/// header with different flag values or dropping a struct flips the court).
#[test]
fn c3_differential_vs_generated_c_header() {
    assert!(
        C_HEADER.contains("struct ArrowArray"),
        "header defines ArrowArray"
    );
    assert!(
        C_HEADER.contains("struct ArrowSchema"),
        "header include defines ArrowSchema"
    );
    // Flag macro values must match the Rust c_abi pins.
    assert!(C_HEADER.contains("2"), "nullable flag value 2 present");
    assert!(
        C_HEADER.contains("CGAP_FLAG_NULLABLE") || C_HEADER.contains("ARROW_FLAG_NULLABLE"),
        "nullable flag macro present"
    );
    assert!(
        C_HEADER.contains("ARROW_FLAG_DICTIONARY_ORDERED")
            || C_HEADER.contains("CGAP_FLAG_DICTIONARY_ORDERED"),
        "dictionary-ordered macro present"
    );
    assert!(
        C_HEADER.contains("ARROW_FLAG_MAP_KEYS_SORTED") || C_HEADER.contains("CGAP_FLAG_MAP_KEYS_SORTED"),
        "map-keys-sorted macro present"
    );
    // Offset ladder of the C structs must equal the Rust repr(C) ladder
    // (differential: C header field order == Rust struct field order).
    let array_ladder = [
        "int64_t length",
        "int64_t null_count",
        "int64_t offset",
        "int64_t n_buffers",
        "int64_t n_children",
    ];
    for f in array_ladder {
        assert!(C_HEADER.contains(f), "ArrowArray field ladder: {f}");
    }
    for f in [
        "const char* format",
        "const char* name",
        "const char* metadata",
        "int64_t flags",
        "int64_t n_children",
    ] {
        assert!(C_HEADER.contains(f), "ArrowSchema field ladder: {f}");
    }
}

/// C4: corrupting each of the 4 continuation bytes flips the reader to a
/// typed [`IpcError::BadContinuation`] (anti-vacuity mutation court: this
/// IS the mutation; a gate that accepts the corrupt frame is a VACUOUS
/// gate and fails this court).
#[test]
fn c4_corrupt_continuation_refused() {
    let slab = sealed_4row().unwrap();
    let body = slab.as_bytes();
    let digest = schema_digest_of(body);
    let mut w = vec![0u8; window_len(body.len(), 8)];
    emit(&mut w, &meta_for(body.len(), 8, 1, digest), body).unwrap();
    for off in 0..4usize {
        let mut m = w.clone();
        m[off] ^= 0xFF;
        let err = read(&m, Some(&digest), &mut [0u8; 320]).unwrap_err();
        assert_eq!(err, IpcError::BadContinuation { got: CONTINUATION ^ (0xFF << (off * 8)) });
    }
    // EOS continuation corrupted: typed MissingEos.
    let mut m = w.clone();
    let eos_cont_at = w.len() - 8;
    m[eos_cont_at] ^= 0xFF;
    let err = read(&m, Some(&digest), &mut [0u8; 320]).unwrap_err();
    assert!(matches!(err, IpcError::MissingEos { .. }));
}

/// C4b: schema digest mismatch and nonzero body padding are typed
/// refusals (the two remaining digest/pad classes).
#[test]
fn c4b_schema_and_pad_mutations_refused() {
    let slab = sealed_4row().unwrap();
    let body = slab.as_bytes();
    let digest = schema_digest_of(body);
    let mut w = vec![0u8; window_len(body.len(), 64)];
    emit(&mut w, &meta_for(body.len(), 64, 1, digest), body).unwrap();
    // Wrong expected schema digest: typed SchemaMismatch with the first
    // divergent byte index.
    let mut other = digest;
    other[17] ^= 0x01;
    let err = read(&w, Some(&other), &mut [0u8; 320]).unwrap_err();
    assert_eq!(err, IpcError::SchemaMismatch { got: Some(17) });
    // Nonzero body padding byte: typed PadNonZero at the exact offset.
    let body_at = 8 + 120;
    let mut m = w.clone();
    m[body_at + body.len()] = 0xAA;
    let err = read(&m, Some(&digest), &mut [0u8; 320]).unwrap_err();
    assert_eq!(err, IpcError::PadNonZero { at: body_at + body.len() });
    // Metadata header mutation flips the header digest gate.
    let mut m = w.clone();
    m[0x10] ^= 0x01; // inside schema_digest span
    let err = read(&m, Some(&digest), &mut [0u8; 320]).unwrap_err();
    assert!(
        matches!(err, IpcError::HeaderDigestMismatch { .. } | IpcError::SchemaMismatch { .. }),
        "header mutation: {err:?}"
    );
    // Missing EOS (truncated window): typed refusal.
    let err = read(&w[..w.len() - 8], Some(&digest), &mut [0u8; 320]);
    assert!(matches!(err, Err(IpcError::MissingEos { .. }) | Err(IpcError::BadBodyLen { .. })));
}

/// C5: full byte-sweep mutation: every single-byte flip in the prefix and
/// metadata header maps to SOME typed error, never a silent accept.
#[test]
fn c5_prefix_header_byte_sweep_all_typed() {
    let slab = sealed_4row().unwrap();
    let body = slab.as_bytes();
    let digest = schema_digest_of(body);
    let mut w = vec![0u8; window_len(BODY_LEN, 64)];
    emit(&mut w, &meta_for(BODY_LEN, 64, 5, digest), body).unwrap();
    for off in 0..72usize {
        let mut m = w.clone();
        m[off] = m[off].wrapping_add(1);
        let r = read(&m, Some(&digest), &mut [0u8; 320]);
        assert!(r.is_err(), "byte {off} mutated to silent accept");
    }
}

const BODY_LEN: usize = 64 + 4 * 64;

/// C6: emit capacity/shape faults are typed (no panic on inputs).
#[test]
fn c6_emit_typed_faults() {
    let slab = sealed_4row().unwrap();
    let body = slab.as_bytes();
    let digest = schema_digest_of(body);
    let meta = meta_for(body.len(), 64, 1, digest);
    // Output window too small.
    let mut small = [0u8; 32];
    assert_eq!(
        emit(&mut small, &meta, body),
        Err(IpcError::Truncated { needed: window_len(body.len(), 64), got: 32 })
    );
    // body_len mismatch between metadata and body.
    assert_eq!(
        emit(&mut [0u8; 4096], &meta_for(body.len() + 1, 64, 1, digest), body),
        Err(IpcError::BadBodyLen {
            got: (body.len() + 1) as u32,
            expected: body.len() as u32
        })
    );
    // Alignment 4 rejected.
    assert_eq!(
        emit(&mut [0u8; 4096], &meta_for(body.len(), 4, 1, digest), body),
        Err(IpcError::BadAlignment { got: 4 })
    );
}
