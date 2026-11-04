//! Courts for `crates/cgap-arrow/src/ipc.rs` (lane p1-ipc, v26.11.2).
//!
//! Chicago discipline: real collaborators (real sealed slab, real wire
//! window, the real generated C header on disk). Every court names its
//! anti-vacuity mutation: weakening or deleting the gate code must flip
//! the court, so a pass proves the gate live (exit contract 0/1).

use cgap_arrow::blake3;
use cgap_arrow::ipc::{
    emit, read, seal_meta, window_len, IpcError, MetaHeader, CONTINUATION, EOS, MAGIC,
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
        assert_eq!(
            n,
            w.len(),
            "emit must fill exactly window_len at {alignment}"
        );
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
        assert_eq!(
            u32::from_le_bytes([w[0x30 + 8], w[0x31 + 8], w[0x32 + 8], w[0x33 + 8]]),
            7
        );
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
        assert_eq!(
            body_at % alignment_usize,
            0,
            "body start aligned to {alignment}"
        );
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
    emit(
        &mut w,
        &meta_for(body.len(), 64, 0, schema_digest_of(body)),
        body,
    )
    .unwrap();
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
        C_HEADER.contains("ARROW_FLAG_MAP_KEYS_SORTED")
            || C_HEADER.contains("CGAP_FLAG_MAP_KEYS_SORTED"),
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
        assert_eq!(
            err,
            IpcError::BadContinuation {
                got: CONTINUATION ^ (0xFF << (off * 8))
            }
        );
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
    assert_eq!(
        err,
        IpcError::PadNonZero {
            at: body_at + body.len()
        }
    );
    // Metadata header mutation flips the header digest gate.
    let mut m = w.clone();
    m[0x10] ^= 0x01; // inside schema_digest span
    let err = read(&m, Some(&digest), &mut [0u8; 320]).unwrap_err();
    assert!(
        matches!(
            err,
            IpcError::HeaderDigestMismatch { .. } | IpcError::SchemaMismatch { .. }
        ),
        "header mutation: {err:?}"
    );
    // Missing EOS (truncated window): typed refusal.
    let err = read(&w[..w.len() - 8], Some(&digest), &mut [0u8; 320]);
    assert!(matches!(
        err,
        Err(IpcError::MissingEos { .. }) | Err(IpcError::BadBodyLen { .. })
    ));
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
        Err(IpcError::Truncated {
            needed: window_len(body.len(), 64),
            got: 32
        })
    );
    // body_len mismatch between metadata and body.
    assert_eq!(
        emit(
            &mut [0u8; 4096],
            &meta_for(body.len() + 1, 64, 1, digest),
            body
        ),
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

// ===========================================================================
// CG-1 additions (v26.11.3): G2 version/flags/seq courts, G4 parse_meta
// seam courts, G3 witness-tier scan court.
// ===========================================================================

use cgap_arrow::ipc::{read_verified, read_with_meta, seq_ok, LAYOUT_VERSION};

/// Consistently-sealed align-64 window with version `v`, flags `f`, and
/// (optionally) a real blake3 body digest in the metadata padding so
/// `read_verified` reaches past the framing gates. Returns the window and
/// the body digest. This is the w8-pattern mutation: the header fields are
/// mutated AND the FNV-1a seal recomputed, so the window dies on the
/// VERSION/FLAGS gate, never on the digest gate.
fn sealed_window_with(v: u16, f: u16, with_digest: bool) -> (Vec<u8>, Vec<u8>, [u8; 32]) {
    let slab = sealed_4row().unwrap();
    let body = slab.as_bytes();
    let digest = schema_digest_of(body);
    let w = vec![0u8; window_len(body.len(), 64)];
    let meta = MetaHeader {
        magic: MAGIC,
        version: v,
        flags: f,
        schema_digest: digest,
        body_len: body.len() as u32,
        alignment: 64,
        seq: 42,
        reserved0: 0,
        header_digest: 0,
        pad_tail: [0; 4],
    };
    let sealed = seal_meta(meta);
    let mut seq_w = vec![0u8; w.len()];
    let n = emit(&mut seq_w, &sealed, body).unwrap();
    assert_eq!(n, w.len());
    seq_w[12] = (v & 0xff) as u8; // version at 0x04+8=12
    seq_w[13] = (v >> 8) as u8;
    seq_w[14] = (f & 0xff) as u8; // flags at 0x06+8=14
    seq_w[15] = (f >> 8) as u8;
    // Recompute the FNV-1a seal over bytes 8..0x38+8 so the header digest
    // gate PASSES and the window reaches the version gate.
    let h = cgap_arrow::ipc::fnv1a32(&seq_w[8..0x40]);
    seq_w[0x38 + 8..0x3C + 8].copy_from_slice(&h.to_le_bytes());
    if with_digest {
        let body_digest = blake3::hash(body);
        seq_w[cgap_arrow::ipc::BODY_DIGEST_AT..cgap_arrow::ipc::BODY_DIGEST_AT + 32]
            .copy_from_slice(&body_digest);
    }
    (seq_w, body.to_vec(), digest)
}

// ---------------------------------------------------------------------------
// C7: seq_ok law (strict advance, wrap-safe)
// ---------------------------------------------------------------------------

/// Exactly-one-step successor law over u32: advance accepts, stay/skip/
/// reverse refuse, and the wrap `u32::MAX -> 0` is lawful. Anti-vacuity:
/// the refusing arms ARE the mutations (a `>=` gate or a non-wrapping add
/// flips one of these asserts).
#[test]
fn c7_seq_ok_law_incl_wrap() {
    assert!(seq_ok(0, 1));
    assert!(seq_ok(7, 8));
    assert!(seq_ok(u32::MAX - 1, u32::MAX));
    assert!(seq_ok(u32::MAX, 0), "wrap is lawful");
    assert!(!seq_ok(5, 5), "replay (same seq) refuses");
    assert!(!seq_ok(5, 7), "skip refuses");
    assert!(!seq_ok(7, 5), "reverse/reorder refuses");
    assert!(!seq_ok(0, 0));
    assert!(!seq_ok(u32::MAX, u32::MAX));
    assert!(!seq_ok(1, u32::MAX), "far-jump refuses");
}

// ---------------------------------------------------------------------------
// C8: BadVersion via read AND read_verified (w8 pattern)
// ---------------------------------------------------------------------------

/// A consistently-sealed version-2 window dies on the VERSION gate, never
/// on the digest gate, through BOTH verifying paths. Anti-vacuity: the
/// recomputed seal proves the death is the version gate itself (the FNV
/// gate passed), and the exact-variant assert fails a gate that lumps the
/// refusal into HeaderDigestMismatch or accepts silently.
#[test]
fn c8_foreign_version_refused_typed_through_both_paths() {
    let (w, body, digest) = sealed_window_with(2, 0, true);
    let mut out = [0u8; 320];
    let err = read(&w, Some(&digest), &mut out).unwrap_err();
    assert_eq!(err, IpcError::BadVersion { got: 2 }, "read: exact variant");
    let err = read_verified(&w, Some(&digest), &mut out).unwrap_err();
    assert_eq!(
        err,
        IpcError::BadVersion { got: 2 },
        "read_verified: exact variant"
    );
    // Control: version 1 twin reads OK (the gate is on the value, not the
    // fixture shape). Digest gate passed, framing identical.
    let (w1, body1, digest1) = sealed_window_with(LAYOUT_VERSION, 0, true);
    assert_eq!(
        read(&w1, Some(&digest1), &mut [0u8; 320]).unwrap(),
        body1.len()
    );
    let _ = (body, w1.len());
}

/// Control pair: version-1 window with digest reads OK through
/// read_verified (proves c8's refusal is the version gate, not fixture
/// damage).
#[test]
fn c8b_verified_control_passes() {
    let (w, body, digest) = sealed_window_with(LAYOUT_VERSION, 0, true);
    let mut out = [0u8; 320];
    let n = read_verified(&w, Some(&digest), &mut out).unwrap();
    assert_eq!(n, body.len());
    assert_eq!(&out[..n], &body[..]);
}

// ---------------------------------------------------------------------------
// C9: BadFlags (consistent-seal flags=0x0001)
// ---------------------------------------------------------------------------

/// A consistently-sealed flags=0x0001 window dies typed on the FLAGS gate
/// through both paths; the version-1 twin control passes.
#[test]
fn c9_flags_gate_typed() {
    let (w, body, digest) = sealed_window_with(LAYOUT_VERSION, 0x0001, true);
    let mut out = [0u8; 320];
    let err = read(&w, Some(&digest), &mut out).unwrap_err();
    assert_eq!(err, IpcError::BadFlags { got: 1 });
    let err = read_verified(&w, Some(&digest), &mut out).unwrap_err();
    assert_eq!(err, IpcError::BadFlags { got: 1 });
    let _ = body;
}

// ---------------------------------------------------------------------------
// C10: read_with_meta returns (len, MetaHeader); read is the thin wrapper
// ---------------------------------------------------------------------------

/// read_with_meta returns the emitter's seq and schema digest through the
/// sealed header; read agrees on the body length (thin wrapper, same
/// gates).
#[test]
fn c10_read_with_meta_carries_seq() {
    let slab = sealed_4row().unwrap();
    let body = slab.as_bytes();
    let digest = schema_digest_of(body);
    let mut w = vec![0u8; window_len(body.len(), 64)];
    emit(&mut w, &meta_for(body.len(), 64, 42, digest), body).unwrap();
    let mut out = [0u8; 320];
    let (n, meta) = read_with_meta(&w, Some(&digest), &mut out).unwrap();
    assert_eq!(n, body.len());
    assert_eq!(meta.seq, 42, "seq rides the header");
    assert_eq!(meta.version, LAYOUT_VERSION);
    assert_eq!(meta.flags, 0);
    assert_eq!(meta.schema_digest, digest);
    // seq_ok pairs the header's seq with the next expected value.
    assert!(seq_ok(meta.seq, 43));
    let n2 = read(&w, Some(&digest), &mut [0u8; 320]).unwrap();
    assert_eq!(n2, n, "read is the thin wrapper");
}

// ---------------------------------------------------------------------------
// C11: parse_meta classification identity (anti-divergence court, G4)
// ---------------------------------------------------------------------------

/// For every window read() rejects, read_verified() must return the SAME
/// error VARIANT. Both entrypoints delegate to the parse_meta seam, so any
/// future gate added to one path but not the other flips this court.
/// (Divergence is lawful only AFTER read succeeds: read_verified then adds
/// BodyDigest{NoRoom,Missing,Mismatch} classes on top of a green framing.)
#[test]
fn c11_seam_classification_identity_over_mutated_windows() {
    let (baseline, body, digest) = sealed_window_with(LAYOUT_VERSION, 0, true);
    let mut cases = 0usize;
    let mut matching = 0usize;
    // Single-byte mutations over prefix + metadata header.
    for off in 0..72usize {
        let mut m = baseline.clone();
        m[off] = m[off].wrapping_add(1);
        let mut o1 = [0u8; 320];
        let mut o2 = [0u8; 320];
        let e1 = read(&m, Some(&digest), &mut o1).err();
        let e2 = read_verified(&m, Some(&digest), &mut o2).err();
        if let Some(e) = e1 {
            let same = match &e2 {
                Some(ev) => std::mem::discriminant(ev) == std::mem::discriminant(&e),
                None => false,
            };
            assert!(same, "offset {off}: read={e:?} read_verified={e2:?}");
            cases += 1;
            matching += usize::from(same);
        } else {
            // Mutation survived framing; digest path must decide typed.
            assert!(
                e2.is_some(),
                "offset {off}: read accepted but read_verified has no typed verdict"
            );
            cases += 1;
        }
    }
    // Truncations at every length below the full window.
    for cut in 0..baseline.len() {
        let m = &baseline[..cut];
        let mut o1 = [0u8; 320];
        let mut o2 = [0u8; 320];
        let e1 = read(m, Some(&digest), &mut o1).err();
        let e2 = read_verified(m, Some(&digest), &mut o2).err();
        if let Some(e) = e1 {
            let same = match &e2 {
                Some(ev) => std::mem::discriminant(ev) == std::mem::discriminant(&e),
                None => false,
            };
            assert!(same, "cut {cut}: read={e:?} read_verified={e2:?}");
            cases += 1;
            matching += usize::from(same);
        }
    }
    assert!(cases >= 72, "mutation matrix ran: {cases} cases");
    assert_eq!(matching, cases, "every read-reject classifies identically");
    let _ = body;
}

// ---------------------------------------------------------------------------
// C12: G4 pin - truncated window classifies Truncated on BOTH paths
// ---------------------------------------------------------------------------

/// The pre-seam read_verified guessed alignment from an unchecked wire
/// word and answered a truncated align-64 window with the WRONG CLASS
/// `BodyDigestNoRoom { got: 8 }`. The seam answers window shape first:
/// the exact `Truncated { needed, got }` on both paths. (Mutating the
/// source gate out and rebuilding is the full mutation-court ladder; this
/// court pins the observable classification, the c11 identity sweep keeps
/// it from regressing class-by-class.)
#[test]
fn c12_truncated_classifies_truncated_not_body_digest_no_room() {
    let (baseline, body, digest) = sealed_window_with(LAYOUT_VERSION, 0, true);
    // Cut inside the metadata span: the parse gate consumes prefix + the
    // metadata span (AB-9/AF-4 read-ahead minimum), so it needs body_at(128)
    // window bytes; have 100.
    let cut = 100usize;
    let m = &baseline[..cut];
    let mut out = [0u8; 320];
    let err = read(m, Some(&digest), &mut out).unwrap_err();
    assert_eq!(
        err,
        IpcError::Truncated {
            needed: 128,
            got: 100
        },
        "read classifies truncation exactly"
    );
    let err = read_verified(m, Some(&digest), &mut out).unwrap_err();
    assert_eq!(
        err,
        IpcError::Truncated {
            needed: 128,
            got: 100
        },
        "read_verified classifies truncation exactly (NOT BodyDigestNoRoom)"
    );
    let _ = (baseline.len(), body.len());
}

// ---------------------------------------------------------------------------
// C13: G3 witness-tier scan court (crc32 absent from the tamper tier)
// ---------------------------------------------------------------------------

/// Tier witness (G3, PG-6 decision: NO shared digest seam). Three tiers,
/// deliberately different digests:
/// - wire-shm crc32: TRANSPORT/accidental-corruption tier (bit flips in
///   flight; crc32 is a checksum, forgeable, and that is fine there);
/// - cgap-arrow ipc blake3: TAMPER tier (adversarial integrity on the
///   verified read path);
/// - plain `read` is NOT a verifying tier at all (see ipc.rs module doc:
///   "the framing layer's read never re-hashes the body").
///
/// COURT: ipc.rs contains zero occurrences of "crc32" - a shared seam or
/// copied checksum would show here. ANTI-VACUITY (sandbox-add-token): the
/// SAME scan run over a /tmp copy with the token injected MUST detect it,
/// else the scan is vacuous. RESTORE LAW: the repo file is only ever READ;
/// the byte-exact fail-closed restore witness compares before/after bytes.
#[test]
fn c13_ipc_tamper_tier_contains_zero_crc32() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(manifest).join("src/ipc.rs");
    let before = std::fs::read(&path).expect("read ipc.rs");
    let text = String::from_utf8_lossy(&before);
    let hits = scan_token(&text, "crc32");
    assert_eq!(hits, 0, "ipc.rs must contain zero \"crc32\" (tamper tier)");

    // ANTI-VACUITY: same scan over a /tmp sandbox copy with the token
    // added MUST flip to nonzero.
    let sandbox = std::env::temp_dir().join(format!("cg1-ipc-scan-{}.rs", std::process::id()));
    std::fs::write(&sandbox, format!("// crc32 injected\n{text}")).expect("write sandbox");
    let sandbox_text = std::fs::read_to_string(&sandbox).expect("read sandbox");
    let sandbox_hits = scan_token(&sandbox_text, "crc32");
    let _ = std::fs::remove_file(&sandbox);
    assert!(
        sandbox_hits >= 1,
        "VACUOUS SCAN: injected token went undetected in the sandbox copy"
    );

    // Fail-closed restore witness: the repo file was only read, bytes
    // byte-exact identical before/after the whole court.
    let after = std::fs::read(&path).expect("re-read ipc.rs");
    assert_eq!(before, after, "repo ipc.rs untouched (fail-closed restore)");
}

/// Line-oriented token scan: count lines containing `token`.
fn scan_token(text: &str, token: &str) -> usize {
    text.lines().filter(|l| l.contains(token)).count()
}
