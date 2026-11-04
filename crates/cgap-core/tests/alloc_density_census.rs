//! Alloc-density census for cgap-core's public entry points (audit lane,
//! violation class 3, 2026-11-04): frame encode/decode/validate, payload
//! projection, digest verify, dma_ring producer/consumer legs, and the
//! isomorphism identity gate — measured with the house counting
//! allocator (`cgap-audit`), 1 KiB payload, not asserted.
//!
//! Anti-vacuity: local positive control (a census that cannot count
//! `vec![0u8; 32]` as 1 alloc is vacuous); the harness itself is also
//! positively controlled by cgap-audit's own courts.
//!
//! Per-frame paths must be alloc-free (the 0-alloc law). Setup
//! allocations (leaked ring region, fixtures) live outside the census
//! scopes.

#![cfg(not(target_os = "none"))]

mod common;

use cgap_audit::alloc_counter::with_alloc_tracking;
use cgap_core::dma_ring::DmaRing;
use cgap_core::frame::{CgapFrameHeader, FRAME_HEADER_LEN};
use cgap_core::isomorphism::{identity_assert, Tier};
use common::{leak_aligned_region, payload, valid_header, wire_frame};

/// Deterministic FNV-style 32-byte digest for census fixtures.
fn digest32(data: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, &b) in data.iter().enumerate() {
        out[i % 32] = out[i % 32].wrapping_mul(31).wrapping_add(b).wrapping_add(1);
    }
    out
}

/// Local positive control: the census harness must count.
#[test]
fn census_harness_counts_positive_control() {
    let (_, report) = with_alloc_tracking(|| {
        let v = vec![0u8; 32];
        assert_eq!(v.len(), 32);
    });
    assert_eq!(
        report.count, 1,
        "vacuous census: harness cannot count ({report:?})"
    );
    assert_eq!(report.bytes, 32);
}

/// Frame header encode_into (fixed 128 B, per-frame producer leg).
#[test]
fn frame_encode_into_alloc_free() {
    let p = payload(1024);
    let h = valid_header(1024, digest32(&p), 0);
    let mut frame = vec![0u8; FRAME_HEADER_LEN + p.len()];
    let (_, r) = with_alloc_tracking(|| {
        h.encode_into(&mut frame).expect("encode_into");
    });
    assert_eq!(r.count, 0, "frame encode_into must be alloc-free ({r:?})");
}

/// Frame header decode (per-frame consumer leg, 128 B fixed read).
#[test]
fn frame_decode_alloc_free() {
    let p = payload(1024);
    let h = valid_header(1024, digest32(&p), 0);
    let frame = wire_frame(&h, &p);
    let (_, r) = with_alloc_tracking(|| {
        let decoded = CgapFrameHeader::decode(&frame).expect("decode");
        assert_eq!(decoded.payload_byte_len, 1024);
    });
    assert_eq!(r.count, 0, "frame decode must be alloc-free ({r:?})");
}

/// Frame header validate (per-frame gate).
#[test]
fn frame_validate_alloc_free() {
    let p = payload(1024);
    let h = valid_header(1024, digest32(&wired_digest(&p)), 0);
    let frame = wire_frame(&h, &p);
    let (_, r) = only_validate(&frame);
    assert_eq!(r.count, 0, "frame validate must be alloc-free ({r:?})");
}

/// One-frame header decode + validate, scoped together (consumer leg).
fn only_validate(frame: &[u8]) -> ((), cgap_audit::alloc_counter::AllocReport) {
    with_alloc_tracking(|| {
        let decoded = CgapFrameHeader::decode(&frame).expect("decode");
        decoded.validate().expect("validate");
    })
}

/// Digest the ON-WIRE payload bytes (what the header pins).
fn wired_digest(p: &[u8]) -> [u8; 32] {
    digest32(p)
}

/// Payload projection + digest verify over a 1 KiB frame (per-frame).
#[test]
fn frame_payload_and_verify_alloc_free() {
    let p = payload(1024);
    let d = digest32(&p);
    let h = valid_header(1024, d, 0);
    let frame = wire_frame(&h, &p);
    let (_, r) = with_alloc_tracking(|| {
        let decoded = CgapFrameHeader::decode(&frame).expect("decode");
        let view = decoded.payload(&frame).expect("payload");
        assert_eq!(view.len(), 1024);
        decoded
            .verify_payload(view, |b| digest32(b))
            .expect("verify_payload");
    });
    assert_eq!(
        r.count, 0,
        "frame payload+verify must be alloc-free ({r:?})"
    );
}

/// Ring producer leg: claim + publish (per-frame), 1 KiB payload slot.
#[test]
fn dma_ring_claim_publish_alloc_free() {
    let region = leak_aligned_region(common::slot_region_len(1024, 2));
    let mut ring = DmaRing::new(region, 1).expect("ring");
    let p = payload(1024);
    let h = valid_header(1024, digest32(&p), 0);
    let mut frame = vec![0u8; FRAME_HEADER_LEN + p.len()];
    h.encode_into(&mut frame).expect("encode");
    frame[FRAME_HEADER_LEN..].copy_from_slice(&p);
    let (_, r) = with_alloc_tracking(|| {
        let slot = ring.claim().expect("claim");
        let len = frame.len().min(slot.len());
        slot[..len].copy_from_slice(&frame[..len]);
        ring.publish(len).expect("publish");
    });
    assert_eq!(
        r.count, 0,
        "dma_ring claim+publish must be alloc-free ({r:?})"
    );
}

/// Ring consumer leg: read + release (per-frame).
#[test]
fn dma_ring_read_release_alloc_free() {
    let region = leak_aligned_region(common::slot_region_len(1024, 2));
    let mut ring = DmaRing::new(region, 1).expect("ring");
    let p = payload(1024);
    let h = valid_header(1024, digest32(&p), 0);
    let mut frame = vec![0u8; FRAME_HEADER_LEN + p.len()];
    h.encode_into(&mut frame).expect("encode");
    frame[FRAME_HEADER_LEN..].copy_from_slice(&p);
    let (_, _setup) = {
        let slot = ring.claim().expect("claim");
        let len = frame.len().min(slot.len());
        slot[..len].copy_from_slice(&frame[..len]);
        ring.publish(len).expect("publish");
        ((), cgap_audit::alloc_counter::AllocReport::default())
    };
    let (_, r) = with_alloc_tracking(|| {
        let (payload_view, header) = ring.read().expect("read");
        assert_eq!(header.payload_byte_len, 1024);
        assert_eq!(payload_view.len(), 1024);
        ring.release().expect("release");
    });
    assert_eq!(
        r.count, 0,
        "dma_ring read+release must be alloc-free ({r:?})"
    );
}

/// Isomorphism identity gate over 1 KiB equal regions (per-frame gate).
#[test]
fn isomorphism_identity_assert_alloc_free() {
    let a = payload(1024);
    let b = payload(1024);
    let (_, r) = with_alloc_tracking(|| {
        identity_assert(&a, Tier::Wire, &b).expect("identity_assert");
    });
    assert_eq!(
        r.count, 0,
        "identity_assert over equal regions must be alloc-free ({r:?})"
    );
}
