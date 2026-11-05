//! Alloc-density census over the uncensused public entry points (audit
//! lane, violation class 3, 2026-11-04): AL-1 measured only the session
//! emit/open surface (0 allocs) and the sidecar read_window (1 alloc);
//! this court extends the census to the remaining per-frame hot paths.
//!
//! House counting allocator (`cgap-audit` `CountingAlloc` /
//! `with_alloc_tracking`, exemplar `tests/session_alloc_census.rs`).
//! Anti-vacuity: the harness itself is positively controlled by
//! cgap-audit's own courts, plus a local positive control below — a
//! census that cannot count `vec![0u8; 32]` as 1 alloc is vacuous and
//! these rows carry no bits.
//!
//! Per-frame entries must be alloc-free (the 0-alloc law); one-time
//! setup entries are measured for the density table but not bound.
//! Measurements at a representative 1 KiB body / 64-link slab unless the
//! entry is fixed-width.

#![cfg(not(target_os = "none"))]

use cgap_aoq::aead::{open_detached, seal_detached, Nonce96, TAG_LEN};
use cgap_aoq::chain::{verify_records, AoqChain, AoqChainRecord, ChainDigest};
use cgap_aoq::codec;
use cgap_aoq::header::{AoqHeader, SIZE_BYTES};
use cgap_aoq::pin_table::{PinOutcome, PinTable};
use cgap_aoq::verify::{decrypt_in_place, verify_window};
use cgap_aoq::writer::AoqWriter;
use cgap_audit::alloc_counter::with_alloc_tracking;

/// Local positive control: the census harness must count. A scope
/// containing exactly one `vec![0u8; 32]` reads 1 alloc / 32 bytes.
#[test]
fn census_harness_counts_positive_control() {
    let (_, report) = with_alloc_tracking(|| {
        // Heap allocation IS the property under test here — the vec! is the
        // point (useless_vec would delete the allocation and vacuous the control).
        #[allow(clippy::useless_vec)]
        let v = vec![0u8; 32];
        assert_eq!(v.len(), 32);
    });
    assert_eq!(
        report.count, 1,
        "vacuous census: harness cannot count ({report:?})"
    );
    assert_eq!(report.bytes, 32);
}

/// Fixed-width header codec: encode into a caller buffer.
#[test]
fn codec_encode_alloc_free() {
    let h = AoqHeader::new(0, 7, 1, 1);
    let mut buf = [0u8; SIZE_BYTES];
    let (_, r) = with_alloc_tracking(|| codec::encode(&h, &mut buf).expect("encode"));
    assert_eq!(r.count, 0, "codec::encode must be alloc-free ({r:?})");
}

/// Fixed-width header codec: decode from a wire slice.
#[test]
fn codec_decode_alloc_free() {
    let mut buf = [0u8; SIZE_BYTES];
    codec::encode(&AoqHeader::new(0, 7, 1, 1), &mut buf).expect("encode");
    let (_, r) = with_alloc_tracking(|| {
        let h = codec::decode(&buf).expect("decode");
        assert_eq!(h.seq, 1);
    });
    assert_eq!(r.count, 0, "codec::decode must be alloc-free ({r:?})");
}

/// AEAD seal direction, 1 KiB body, detached tag into caller arrays.
#[test]
fn aead_seal_detached_1kib_alloc_free() {
    let key = [7u8; 32];
    let nonce = Nonce96::new(0, 42).expect("nonce");
    let payload = vec![0xA5u8; 1024];
    let mut ct = vec![0u8; payload.len()];
    let mut tag = [0u8; TAG_LEN];
    let (_, r) = with_alloc_tracking(|| {
        seal_detached(&mut ct, &mut tag, &payload, b"aad", &key, &nonce);
    });
    assert_eq!(r.count, 0, "aead seal at 1 KiB must be alloc-free ({r:?})");
    assert_ne!(
        ct,
        vec![0u8; 1024],
        "seal must produce ciphertext (final state)"
    );
}

/// AEAD open direction, 1 KiB body.
#[test]
fn aead_open_detached_1kib_alloc_free() {
    let key = [7u8; 32];
    let nonce = Nonce96::new(0, 42).expect("nonce");
    let payload = vec![0xA5u8; 1024];
    let mut ct = vec![0u8; payload.len()];
    let mut tag = [0u8; TAG_LEN];
    seal_detached(&mut ct, &mut tag, &payload, b"aad", &key, &nonce);
    let mut pt = vec![0u8; ct.len()];
    let (res, r) = with_alloc_tracking(|| open_detached(&mut pt, &ct, &tag, b"aad", &key, &nonce));
    res.expect("open must succeed");
    assert_eq!(r.count, 0, "aead open at 1 KiB must be alloc-free ({r:?})");
}

/// Deterministic FNV-style chain digest (no crypto; same house shape as
/// the chain unit courts).
struct XorDigest;
impl ChainDigest for XorDigest {
    fn digest(&self, parts: &[&[u8]]) -> [u8; 32] {
        let mut out = [0u8; 32];
        for part in parts {
            for (i, &b) in part.iter().enumerate() {
                out[i % 32] = out[i % 32].wrapping_mul(31).wrapping_add(b).wrapping_add(1);
            }
        }
        out
    }
}

/// Chain link append (per-record hot path) over a 64-link slab; every
/// append is individually scoped.
#[test]
fn chain_append_64_links_alloc_free() {
    let mut slab = vec![AoqChainRecord::zeroed(); 64];
    let mut chain = AoqChain::new(XorDigest, &mut slab);
    let payload = vec![0xA5u8; 1024];
    for i in 0..64u64 {
        let (_, r) = with_alloc_tracking(|| {
            chain.append(i as u16, &payload).expect("append");
        });
        assert_eq!(r.count, 0, "chain append {i} must be alloc-free ({r:?})");
    }
}

/// Chain verify over 64 links (batch path, 1 KiB bodies).
#[test]
fn chain_verify_all_64_links_alloc_free() {
    let mut slab = vec![AoqChainRecord::zeroed(); 64];
    let mut chain = AoqChain::new(XorDigest, &mut slab);
    let payload = vec![0xA5u8; 1024];
    for i in 0..64u64 {
        chain.append(i as u16, &payload).expect("append");
    }
    let mut records = Vec::with_capacity(64);
    records.extend_from_slice(chain.records());
    let (_, r) = with_alloc_tracking(|| {
        verify_records(&XorDigest, &records).expect("verify_records");
    });
    assert_eq!(
        r.count, 0,
        "verify_records over 64 links must be alloc-free ({r:?})"
    );
}

/// TOFU pin-table hit path (linear scan over a populated table).
#[test]
fn pin_table_verify_hit_alloc_free() {
    let mut table = PinTable::new();
    for peer in 0..8u64 {
        table.verify(peer, [peer as u8; 32]).expect("TOFU insert");
    }
    let (_, r) = with_alloc_tracking(|| {
        let outcome = table.verify(3, [3u8; 32]).expect("verify");
        assert!(matches!(outcome, PinOutcome::Pinned));
    });
    assert_eq!(r.count, 0, "pin_table verify must be alloc-free ({r:?})");
}

/// Build a lawful sealed window (setup, outside the census scopes): the
/// ab8 canonical shape — tag = blake3(ct)[..16].
fn sealed_window() -> (Vec<u8>, [u8; 16]) {
    let plain = vec![0xA5u8; 1024];
    let mut ct = plain;
    for (i, byte) in ct.iter_mut().enumerate() {
        *byte ^= (i as u8) | 1;
    }
    let tag_full = cgap_blake3::hash(&ct);
    let mut tag = [0u8; 16];
    tag.copy_from_slice(&tag_full[..16]);
    let mut w = AoqWriter::new();
    let mut out = vec![0u8; 8 + 120 + ct.len() + 4096];
    let n = w
        .write_window(&mut out, 0, 7, &tag, &ct)
        .expect("write_window");
    out.truncate(n);
    (out, tag)
}

/// AOQ window verify (per-frame inbound gate), 1 KiB body.
#[test]
fn verify_window_1kib_alloc_free() {
    let (win, tag) = sealed_window();
    let (_, r) = with_alloc_tracking(|| {
        let vw = verify_window(&win, 0, 0, &tag).expect("verify_window");
        assert_eq!(vw.body_len, 1024);
    });
    assert_eq!(
        r.count, 0,
        "verify_window at 1 KiB must be alloc-free ({r:?})"
    );
}

/// In-place keystream decrypt over the verified window (per-frame).
#[test]
fn decrypt_in_place_1kib_alloc_free() {
    let (win, tag) = sealed_window();
    let vw = verify_window(&win, 0, 0, &tag).expect("verify_window");
    let mut keystream = Vec::with_capacity(vw.body_len);
    for i in 0..vw.body_len {
        keystream.push((i as u8) | 1);
    }
    let mut buf = win;
    let (_, r) = with_alloc_tracking(|| {
        decrypt_in_place(&vw, &keystream, &mut buf).expect("decrypt");
    });
    assert_eq!(
        r.count, 0,
        "decrypt_in_place at 1 KiB must be alloc-free ({r:?})"
    );
}

// ---------------------------------------------------------------------------
// cgap-arrow IPC surface (dev-dep; same binary, same counting allocator).
// ---------------------------------------------------------------------------

use cgap_arrow::ipc::{self, MetaHeader};

/// Arrow IPC emit + read_verified round trip, 1 KiB body, alignment 64.
/// Scoped separately so emit and read densities are distinct rows.
#[test]
fn arrow_ipc_emit_read_verified_1kib_alloc_free() {
    let mut meta = MetaHeader {
        magic: ipc::MAGIC,
        version: 1,
        flags: 0,
        schema_digest: [9u8; 32],
        body_len: 1024,
        alignment: 64,
        seq: 1,
        reserved0: 0,
        header_digest: 0,
        pad_tail: [0; 4],
    };
    meta.header_digest = ipc::seal_meta(meta).header_digest;
    let body = vec![0x5Au8; 1024];
    let total = ipc::window_len(1024, 64);
    let mut out = vec![0u8; total];
    let (n, r_emit) =
        with_alloc_tracking(|| ipc::emit_verified(&mut out, &meta, &body).expect("emit_verified"));
    assert_eq!(n, total);
    assert_eq!(
        r_emit.count, 0,
        "ipc emit_verified must be alloc-free ({r_emit:?})"
    );

    let mut body_out = vec![0u8; 1024];
    let (got, r_read) = with_alloc_tracking(|| {
        ipc::read_verified(&out, Some(&meta.schema_digest), &mut body_out).expect("read_verified")
    });
    assert_eq!(got, 1024);
    assert_eq!(body_out, body, "ipc round trip (final state)");
    assert_eq!(
        r_read.count, 0,
        "ipc read_verified must be alloc-free ({r_read:?})"
    );
}
