//! Ingress layout court — lane A04 finisher (§6.4 + §8.1 memory tier).
//!
//! Freezes the wire-to-memory ingress: `CgapFrameHeader::project` is the
//! `slice::from_raw_parts`-style safe projection of a 64-aligned 128-byte
//! region onto `&CgapFrameHeader` with zero copies. Acceptance rows:
//!
//!   compile-time + runtime align_of==64 / size_of==128  → rows 01-02 (here)
//!   zero-copy addressability (field addr == base + §6.1 offset) → row 03
//!   alignment invariance (layout independent of address)        → row 04
//!   misaligned ingress refused, typed `FrameError::Misaligned`  → row 05
//!   mutation: region byte flip is VISIBLE through the view (a copy would
//!     go stale) — proves the reference aliases the region      → row 06
//!   wire→memory identity: decode(project(bytes)) == the source  → row 07
//!
//! Real collaborators only: a real 64-byte-aligned heap region (the same
//! allocator the DMA ring consumes), real §6.1 offsets, no mocks.

mod common;

use common::{leak_aligned_region, valid_header};
use std::convert::TryInto;

use cgap_core::frame::{
    CgapFrameHeader, FrameError, P28LicenseAttestation, ATTESTATION_LEN, FLAG_BN3,
    FRAME_HEADER_LEN, FRAME_MAGIC, FRAME_VERSION, LICENSE_FLAG_ROOT,
};

/// Row 01 (compile-time half): the court itself carries the §6.4 pins, so a
/// regression in `frame.rs` cannot compile this court away.
const _: () = assert!(core::mem::size_of::<CgapFrameHeader>() == 128);
const _: () = assert!(core::mem::align_of::<CgapFrameHeader>() == 64);
const _: () = assert!(core::mem::size_of::<P28LicenseAttestation>() == 80);
const _: () = assert!(ATTESTATION_LEN == 80 && FRAME_HEADER_LEN == 128);

/// Row 02 (runtime half of the §6.4 pins): the same facts asserted at
/// runtime, so a `const`-eval bypass or target-dependent layout change is
/// witnessed by an executed assert, not only by a compile gate.
#[test]
fn row_02_runtime_pins_size_128_align_64() {
    assert_eq!(core::mem::size_of::<CgapFrameHeader>(), 128);
    assert_eq!(core::mem::align_of::<CgapFrameHeader>(), 64);
    assert_eq!(
        core::mem::size_of::<cgap_core::frame::P28LicenseAttestation>(),
        80
    );
    assert_eq!(
        core::mem::align_of::<cgap_core::frame::P28LicenseAttestation>(),
        8
    );
    assert_eq!(FRAME_HEADER_LEN, 128);
    assert_eq!(ATTESTATION_LEN, 80);
}

/// Row 03 — zero-copy addressability: the projection returns a reference
/// INTO the region (same base address), and every field lives at its §6.1
/// byte offset from that base. A single copy anywhere would move the base
/// and break the address equations; the offsets double as a runtime
/// re-check of the §6.1 doc table against real addresses.
#[test]
fn row_03_projection_is_zero_copy_addressable() {
    let region = leak_aligned_region(4 * FRAME_HEADER_LEN);
    let src = valid_header(0x0102_0304_0506_0708, [0xAA; 32], FLAG_BN3);
    src.encode_into(&mut region[..FRAME_HEADER_LEN])
        .expect("encode");
    let slot: &[u8; FRAME_HEADER_LEN] = region[..FRAME_HEADER_LEN].try_into().expect("width");
    let view = CgapFrameHeader::project(slot).expect("64-aligned region");
    let base = region.as_ptr() as usize;
    assert_eq!(
        view as *const CgapFrameHeader as usize, base,
        "projection must alias the region base, not a copy"
    );
    let field_addr = |p: *const u8| p as usize - base;
    assert_eq!(field_addr(view.magic.as_ptr()), 0);
    assert_eq!(field_addr(&view.version), 4);
    assert_eq!(field_addr(&view.flags), 5);
    assert_eq!(field_addr(view.reserved.as_ptr()), 6);
    assert_eq!(
        field_addr(&view.payload_byte_len as *const u64 as *const u8),
        8
    );
    assert_eq!(field_addr(view.oabi_digest.as_ptr()), 16);
    assert_eq!(
        field_addr(&view.attestation as *const P28LicenseAttestation as *const u8),
        48
    );
    assert_eq!(
        field_addr(view.attestation.upstream_receipt.as_ptr()),
        48 + 48
    );
    // and the values read through the view are the values encoded on the wire
    assert_eq!(view.magic, FRAME_MAGIC);
    assert_eq!(view.version, FRAME_VERSION);
    assert_eq!(view.payload_byte_len, 0x0102_0304_0506_0708);
    assert_eq!(view.attestation.attestation_flags, LICENSE_FLAG_ROOT);
}

/// Row 04 — alignment invariance: the layout is a property of the TYPE, not
/// of the address. The same header encoded at five different 64-aligned
/// addresses in one real region projects to identical fields everywhere,
/// and `decode(project(bytes))` at every address equals
/// `decode(encode(header))` at the first — the memory tier is the wire
/// tier, address-independent (§8.1, §2 thesis).
#[test]
fn row_04_alignment_invariance_across_addresses() {
    let slots = 5usize;
    let region = leak_aligned_region(slots * FRAME_HEADER_LEN);
    let src = valid_header(4096, [0x33; 32], FLAG_BN3);
    for s in 0..slots {
        let off = s * FRAME_HEADER_LEN;
        src.encode_into(&mut region[off..off + FRAME_HEADER_LEN])
            .expect("encode");
    }
    for s in 0..slots {
        let off = s * FRAME_HEADER_LEN;
        let slice: &[u8; FRAME_HEADER_LEN] = region[off..off + FRAME_HEADER_LEN]
            .try_into()
            .expect("128-byte slot slice");
        let view = CgapFrameHeader::project(slice)
            .unwrap_or_else(|e| panic!("slot {s} at base+{off} refused: {e:?}"));
        assert_eq!(view.magic, FRAME_MAGIC);
        assert_eq!(view.version, FRAME_VERSION);
        assert_eq!(view.payload_byte_len, 4096);
        assert_eq!(view.oabi_digest, [0x33; 32]);
        assert_eq!(view.attestation, src.attestation);
    }
    // decode∘project is the identity on the source, at every address
    let first: &[u8; FRAME_HEADER_LEN] = region[..FRAME_HEADER_LEN].try_into().unwrap();
    let back0 = CgapFrameHeader::decode(first).unwrap();
    for s in 0..slots {
        let off = s * FRAME_HEADER_LEN;
        let slice: &[u8; 128] = region[off..off + 128].try_into().unwrap();
        assert_eq!(CgapFrameHeader::decode(slice).unwrap(), back0);
    }
}

/// Row 05 — misaligned ingress is refused, typed, before any reference
/// forms: every non-64-aligned residue (base+1 .. base+63) yields
/// `Err(FrameError::Misaligned)`, never a projection (§7.2 fail-closed).
#[test]
fn row_05_misaligned_ingress_refused_typed() {
    let region = leak_aligned_region(FRAME_HEADER_LEN + 128);
    let src = valid_header(8, [0x44; 32], FLAG_BN3);
    src.encode_into(&mut region[..FRAME_HEADER_LEN])
        .expect("encode");
    for off in 1usize..64 {
        let slice: &[u8; FRAME_HEADER_LEN] = region[off..off + FRAME_HEADER_LEN]
            .try_into()
            .expect("slice width");
        let got = CgapFrameHeader::project(slice);
        assert_eq!(
            got,
            Err(FrameError::Misaligned),
            "ingress at base+{off} (mod 64 = {}) must refuse",
            off % 64
        );
    }
}

/// Row 06 — the court's named anti-vacuity mutation: flip one byte of the
/// region under the view and the mutation is VISIBLE through a fresh
/// projection. A projection that copies would return stale bytes and the
/// equality would fail — this row cannot pass on a copying implementation.
#[test]
fn row_06_region_mutation_is_visible_through_view() {
    let region = leak_aligned_region(FRAME_HEADER_LEN);
    let src = valid_header(0xFF, [0x55; 32], FLAG_BN3);
    src.encode_into(&mut region[..FRAME_HEADER_LEN])
        .expect("encode");
    let slice: &[u8; FRAME_HEADER_LEN] = region[..FRAME_HEADER_LEN].try_into().unwrap();
    let before = CgapFrameHeader::project(slice).unwrap().payload_byte_len;
    assert_eq!(before, 0xFF);
    region[8] ^= 0x01; // byte 8 = LSB of payload_byte_len (§6.1)
    let slice: &[u8; FRAME_HEADER_LEN] = region[..FRAME_HEADER_LEN].try_into().unwrap();
    let after = CgapFrameHeader::project(slice).unwrap().payload_byte_len;
    assert_eq!(after, 0xFE, "view must observe region bytes, not a copy");
    // ...and flipping a wire byte flips exactly one header field
    let slice: &[u8; FRAME_HEADER_LEN] = region[..FRAME_HEADER_LEN].try_into().unwrap();
    let v = CgapFrameHeader::project(slice).unwrap();
    assert_eq!(v.magic, FRAME_MAGIC);
    assert_eq!(v.oabi_digest, [0x55; 32]);
}

/// Row 07 — wire→memory identity at header granularity (§8.1): bytes on the
/// wire (encode), projected in memory (project), decoded back — the very
/// same fields, zero copies. Runs over the doc-pinned payload-size sweep
/// (§8.2 step 1), each frame in its own 64-aligned slot.
#[test]
fn row_07_wire_to_memory_ingress_identity() {
    for len in common::SIZES {
        let pl = common::payload(len);
        let hdr = valid_header(pl.len() as u64, cgap_blake3::hash(&pl), FLAG_BN3);
        // region is rounded up to the 64-byte slot stride (§7.2; the fixture
        // refuses non-multiples fail-closed)
        let region_len = (FRAME_HEADER_LEN + pl.len()).div_ceil(64) * 64;
        let region = leak_aligned_region(region_len);
        let frame_len = FRAME_HEADER_LEN + pl.len();
        region[..frame_len].copy_from_slice(&common::wire_frame(&hdr, &pl)[..frame_len]);
        let slot: &[u8; FRAME_HEADER_LEN] = region[..FRAME_HEADER_LEN].try_into().unwrap();
        let view = CgapFrameHeader::project(slot).expect("ingress");
        assert_eq!(
            view, &hdr,
            "size {len}: projected view must equal the source"
        );
        let back = CgapFrameHeader::decode(slot).unwrap();
        assert_eq!(back, hdr);
    }
}
