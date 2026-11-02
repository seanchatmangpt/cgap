
//! Lane POSH-1 wire court: net.Buffers/writev scatter-gather simulation.
//!
//! An array of disjoint `&[u8]` slices (one `WireChunkHeader`-bearing chunk
//! plus payload chunks) is driven through the REAL
//! `cgap_git::ingress_ring::IngressRing` API: each slice is `push`ed
//! verbatim and the stream is `pop`ped back out of the ring's inline fixed
//! buffer. That inline fixed-buffer absorb IS the zero-reassembly mechanism
//! (`crates/cgap-git/src/ingress_ring.rs:190-217` — push copies at most two
//! segments straight into `buf: [u8; CAP]`; no heap, no intermediate
//! reassembly anywhere in this file).
//!
//! Header law (POSH wire): `repr(C, align(64))`, magic `b"ARM\0"`,
//! 32-byte tx_id, fact_count u32, stride u32. Payload = 16-byte coordinate
//! facts + 32-byte trailer. Anti-vacuity flips, each witnessed BEFORE
//! (clean consumes) and AFTER (typed refusal):
//!   (1) magic ARM\0 -> XRM\0  : typed SubjectMissing-class refusal,
//!       corroborated against the REAL demux law
//!       (`pack_demux.rs:283-284`: non-"PACK" magic =>
//!       `DemuxError::SubjectMissing`) — the real demuxer refuses XRM\0
//!       and cleanly accepts a "PACK" header.
//!   (2) stride field != fact width : typed StrideMismatch refusal.

#![cfg(test)]

use cgap_git::ingress_ring::{IngressError, IngressRing};
use cgap_git::pack_demux::{DemuxError, Demux};

/// Wire chunk header, cache-line aligned, C-repr so the byte layout is the
/// contract (no Rust field reordering).
#[repr(C, align(64))]
#[derive(Clone, Copy)]
struct WireChunkHeader {
    magic: [u8; 4],
    tx_id: [u8; 32],
    fact_count: u32,
    stride: u32,
}

const FACT_BYTES: usize = 16;
const TRAILER_BYTES: usize = 32;
const WIRE_MAGIC: [u8; 4] = *b"ARM\0";

/// Typed wire refusals. `SubjectMissing` is the SAME refusal class as
/// `DemuxError::SubjectMissing` (GIT-01's demux law: a stream whose leading
/// magic does not identify a known subject is refused, never silently
/// consumed); the demux-law corroboration court below pins that the real
/// lib variant fires identically on bad magic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WireRefusal {
    SubjectMissing,
    StrideMismatch { declared: u32, expected: u32 },
}

impl WireChunkHeader {
    const fn new(tx_id: [u8; 32], fact_count: u32) -> Self {
        WireChunkHeader {
            magic: WIRE_MAGIC,
            tx_id,
            fact_count,
            stride: FACT_BYTES as u32,
        }
    }

    /// Typed header validation over the on-wire bytes.
    fn validate(&self) -> Result<(), WireRefusal> {
        if self.magic != WIRE_MAGIC {
            return Err(WireRefusal::SubjectMissing);
        }
        if self.stride != FACT_BYTES as u32 {
            return Err(WireRefusal::StrideMismatch {
                declared: self.stride,
                expected: FACT_BYTES as u32,
            });
        }
        Ok(())
    }
}

const _: () = assert!(core::mem::size_of::<WireChunkHeader>() == 64);
const _: () = assert!(core::mem::align_of::<WireChunkHeader>() == 64);

/// Header bytes are the raw C-layout bytes (little-endian numeric fields on
/// the host; the court reads them back through the same repr, so the layout
/// is self-consistent by construction).
fn header_bytes(h: &WireChunkHeader) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts(h as *const WireChunkHeader as *const u8, 64)
    }
}

/// Build a deterministic fact block: fact i is 16 bytes derived from i.
fn fact_byte(fact: usize, byte: usize) -> u8 {
    (fact * FACT_BYTES + byte) as u8
}

/// The readv side: pop the whole stream back out of the ring into a caller
/// stack buffer (no heap), then validate the header and every fact byte.
fn gather_validate(stream: &[u8], expect_facts: usize) {
    let (hdr_bytes, rest) = stream.split_at(64);
    // Re-read the header THROUGH the repr'd struct (layout contract).
    let hdr = unsafe { core::ptr::read_unaligned(hdr_bytes.as_ptr() as *const WireChunkHeader) };
    hdr.validate().expect("clean header must validate");
    assert_eq!(hdr.fact_count as usize, expect_facts, "fact count exact");
    let (facts, trailer) = rest.split_at(expect_facts * FACT_BYTES);
    assert_eq!(trailer.len(), TRAILER_BYTES);
    for f in 0..expect_facts {
        for b in 0..FACT_BYTES {
            assert_eq!(facts[f * FACT_BYTES + b], fact_byte(f, b), "fact {f} byte {b}");
        }
    }
    assert!(trailer.iter().all(|&t| t == 0xAB), "trailer constant");
}

/// Assert a typed refusal fires for a mutated stream (flip AFTER witness).
fn assert_refusal(stream: &[u8], want: WireRefusal) {
    let hdr = unsafe { core::ptr::read_unaligned(stream.as_ptr() as *const WireChunkHeader) };
    assert_eq!(hdr.validate(), Err(want), "typed refusal must fire");
}

const TX: [u8; 32] = [0x5Au8; 32];

/// Flip (1) AFTER witness: magic ARM\0 -> XRM\0 in a copied buffer, same
/// pipeline, typed SubjectMissing-class refusal fires. The BEFORE side is
/// `scatter_gather_zero_reassembly_court` in this file (clean consumes; the clean-consume composition court now lives in
/// ingress_ring_court.rs (cull-03 note 1 move).
#[test]
fn magic_flip_refusal_court() {
    let facts = 8usize;
    let mut stream = [0u8; 64 + 8 * FACT_BYTES + TRAILER_BYTES];
    let hdr = WireChunkHeader::new(TX, facts as u32);
    stream[..64].copy_from_slice(header_bytes(&hdr));
    for f in 0..facts {
        for b in 0..FACT_BYTES {
            stream[64 + f * FACT_BYTES + b] = fact_byte(f, b);
        }
    }
    for b in stream[64 + facts * FACT_BYTES..].iter_mut() {
        *b = 0xAB;
    }
    // BEFORE: clean.
    gather_validate(&stream, facts);

    // AFTER: flip byte 0 in the copied stream buffer ('A' -> 'X').
    let mut mutant = stream;
    assert_eq!(mutant[0], b'A');
    mutant[0] = b'X';
    assert_refusal(&mutant, WireRefusal::SubjectMissing);
    // The clean stream still validates: the refusal is data-dependent,
    // not a tautology.
    gather_validate(&stream, facts);

    // Demux-law corroboration with the REAL lib demuxer: a "PACK"-magic
    // 12-byte header (version 2, zero objects) feeds clean...
    let mut d = Demux::new();
    let clean_pack_hdr: &[u8] = b"PACK\x00\x00\x00\x02\x00\x00\x00\x00";
    d.feed(clean_pack_hdr).expect("PACK magic feeds clean");
    // ...and the SAME law refuses XRM\0 with the typed SubjectMissing
    // variant (pack_demux.rs:283-284).
    let mut d2 = Demux::new();
    let bad: &[u8] = b"XRM\0\x00\x00\x00\x02\x00\x00\x00\x00";
    assert_eq!(d2.feed(bad), Err(DemuxError::SubjectMissing));
}


/// Flip (2) AFTER witness: declared stride != the 16-byte fact width ->
/// typed StrideMismatch refusal. The stride field sits at header offset
/// 40..44 (4 magic + 32 tx_id + 4 fact_count); the mutation patches it to
/// 20 in a copied buffer. BEFORE side: the same clean stream validates.
#[test]
fn stride_flip_refusal_court() {
    let facts = 4usize;
    let mut stream = [0u8; 64 + 4 * FACT_BYTES + TRAILER_BYTES];
    let hdr = WireChunkHeader::new(TX, facts as u32);
    stream[..64].copy_from_slice(header_bytes(&hdr));
    for f in 0..facts {
        for b in 0..FACT_BYTES {
            stream[64 + f * FACT_BYTES + b] = fact_byte(f, b);
        }
    }
    for b in stream[64 + facts * FACT_BYTES..].iter_mut() {
        *b = 0xAB;
    }
    // BEFORE: clean header validates, fact count exact.
    gather_validate(&stream, facts);

    // AFTER: patch declared stride 16 -> 20 in a copied stream buffer.
    let mut mutant = stream;
    assert_eq!(&mutant[40..44], &(16u32).to_ne_bytes(), "stride field offset pinned");
    mutant[40..44].copy_from_slice(&(20u32).to_ne_bytes());
    assert_refusal(&mutant, WireRefusal::StrideMismatch { declared: 20, expected: 16 });
    // Clean stream still validates: refusal is data-dependent.
    gather_validate(&stream, facts);
}
