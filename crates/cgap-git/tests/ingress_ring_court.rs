//! Lane GIT-03 ingress-ring courts.
//!
//! Self-contained `#[path]` wire-in (lib.rs is off-limits this lane): the
//! module is compiled directly into this test binary. The module uses no
//! `crate::` paths, so this is byte-faithful to how lib.rs will wire it.
//!
//! Chicago courts, real ring real bytes, assert final state. Anti-vacuity
//! flip twins: the overflow court names its mutation (DELETE the
//! `would_overflow` guard in `IngressRing::push`) and the twin computes
//! the mutant's observable outcome (head advanced past tail capacity) and
//! asserts it DIFFERS from the guarded outcome - a guard deletion flips
//! the court from pass to fail. Loud `[SKIP]` only, never a fake pass.

#![cfg(test)]

#[path = "../src/ingress_ring.rs"]
mod ingress_ring;

use ingress_ring::{IngressError, IngressRing, StdIngressRing};

/// Wrap-around correctness court: a frame pushed across the ring boundary
/// reads back byte-exact in FIFO order (dispatch-mandated court). The
/// geometry is pinned: after draining 100 bytes the head sits at pos
/// 32_468, so the 400-byte second push copies 300 bytes to the physical
/// edge (pos 32_767) and 100 bytes wrapped to pos 0 — a genuine
/// two-segment boundary crossing, asserted byte-exact on drain.
#[test]
fn wrap_around_byte_exact_court() {
    let mut ring = StdIngressRing::new();
    assert_eq!(ring.capacity(), 32_768);
    assert_eq!(ring.pending(), 0);

    // First frame: CAP - 300 bytes, so head lands at pos CAP - 300.
    let big: [u8; 32_468] = core::array::from_fn(|i| (i * 7 + 3) as u8);
    ring.push(&big).expect("push first frame");
    assert_eq!(ring.pending(), 32_468);

    // Drain 100 bytes; now a 400-byte frame fits AND its write starts at
    // pos CAP-300: 300 bytes to the physical edge, 100 bytes wrapped.
    let mut sink = [0u8; 100];
    assert_eq!(ring.pop(&mut sink).unwrap(), 100);
    assert_eq!(&sink[..], &big[..100]);
    assert_eq!(ring.pending(), 32_368);

    let crossing: [u8; 400] = core::array::from_fn(|i| (i as u32 * 13 + 91) as u8);
    ring.push(&crossing).expect("push across boundary");
    assert_eq!(ring.pending(), 32_768);

    // Drain everything; FIFO order must hold and bytes must match the
    // concatenated source exactly (boundary span included).
    let mut out = [0u8; 32_768];
    let n = ring.pop(&mut out).expect("drain all");
    assert_eq!(n, 32_768);
    assert_eq!(&out[..32_368], &big[100..]);
    assert_eq!(&out[32_368..], &crossing[..]);
    assert!(matches!(
        ring.pop(&mut out[..100]),
        Err(IngressError::Empty)
    ));

    // Full-capacity push right after a full drain (free-running counters:
    // all CAP bytes usable; distance==CAP is full, 0 is empty).
    let full: [u8; 32_768] = core::array::from_fn(|i| (i * 11 + 5) as u8);
    ring.push(&full).expect("full-capacity push");
    assert_eq!(ring.pending(), 32_768);
    let mut out2 = [0u8; 32_768];
    assert_eq!(ring.pop(&mut out2).unwrap(), 32_768);
    assert_eq!(&out2[..], &full[..]);
    assert!(matches!(ring.pop(&mut out2[..1]), Err(IngressError::Empty)));
}

/// Overflow refusal court: head advanced past tail capacity is refused
/// typed, ring state untouched, andon class pinned at 101.
#[test]
fn overflow_refusal_court() {
    let mut ring = StdIngressRing::new();
    let fill: [u8; 32_768] = core::array::from_fn(|i| i as u8);
    ring.push(&fill).expect("fill to capacity");
    let (h, t) = (ring.head(), ring.tail());
    assert_eq!(h.wrapping_sub(t), 32_768, "ring full");

    let extra = [0xABu8; 1];
    let err = ring.push(&extra).expect_err("overflow must refuse typed");
    assert_eq!(
        err,
        IngressError::Overflow {
            head: h,
            tail: t,
            frame_len: 1
        }
    );
    assert_eq!(err.andon_exit_class(), 101, "AndonRefusal exit-101 class");
    // Fail-closed: state untouched by the refusal.
    assert_eq!((ring.head(), ring.tail()), (h, t));
    assert_eq!(ring.pending(), 32_768);

    // FrameTooLarge: a frame bigger than the whole ring.
    let huge = vec![0u8; 32_769];
    assert!(matches!(
        ring.push(&huge),
        Err(IngressError::FrameTooLarge { frame_len: 32_769 })
    ));
    // Drain 1 byte; now exactly 1 byte of budget exists. A 2-byte push
    // overflows; a 1-byte push succeeds.
    let mut one = [0u8; 1];
    assert_eq!(ring.pop(&mut one).unwrap(), 1);
    assert!(matches!(
        ring.push(&[0u8; 2]),
        Err(IngressError::Overflow { .. })
    ));
    assert!(ring.push(&[0xEEu8; 1]).is_ok());
}

/// Anti-vacuity flip twin for the overflow court. Named mutation: DELETE
/// the `would_overflow` guard in `IngressRing::push` (mutant pushes
/// unconditionally). The twin computes the mutant's observable outcome -
/// the head value the mutant would store - and asserts it DIFFERS from
/// the guarded ring's head. If the guard is deleted, the guarded push
/// would produce the mutant's head, the assert below fails, and the
/// court flips (a court that cannot flip is FAIL).
#[test]
fn overflow_mutation_flip_twin() {
    // The array literal below needs a true const: use the module constant
    // backing the standard instantiation.
    const CAP: usize = ingress_ring::INGRESS_RING_BYTES;
    let mut ring = StdIngressRing::new();
    ring.push(&[0u8; CAP]).unwrap();
    let (head, tail) = (ring.head(), ring.tail());

    // Mutant outcome: unconditional advance past tail capacity.
    let mutant_head = head.wrapping_add(1);
    assert_eq!(
        mutant_head.wrapping_sub(tail),
        CAP + 1,
        "mutant really advances head past capacity"
    );

    // Guarded outcome: push is refused, head unchanged.
    assert!(matches!(
        ring.push(&[0u8; 1]),
        Err(IngressError::Overflow { .. })
    ));
    assert_ne!(
        ring.head(),
        mutant_head,
        "guard deletion would flip this court"
    );
    assert_eq!(ring.pending(), CAP);
}

/// Penstock sieve court: SWAR band filter admits only frames with a byte
/// in the tag band; dropped frames are not errors and leave the ring
/// untouched. Cross-checked against a naive scalar oracle.
#[test]
fn penstock_sieve_court() {
    let mut ring = StdIngressRing::new();
    let admit = [0u8, 0x41, 0x42]; // 0x41 in band
    let drop = [0x00u8, 0x01, 0x02]; // all below band
    let mut admitted = 0usize;
    let mut dropped = 0usize;
    for i in 0..64u32 {
        for f in [&admit[..], &drop[..]] {
            let mut frame = f.to_vec();
            frame[0] ^= i as u8; // vary first byte
            let got = ring.push_sieved(&frame, 0x41, 0x7E).unwrap();
            let want = naive_band_hit(&frame, 0x41, 0x7E);
            assert_eq!(got, want, "sieve disagrees with naive oracle");
            if got {
                admitted += 1;
            } else {
                dropped += 1;
            }
        }
    }
    // admitted frames are the only bytes in the ring; dropped frames are
    // not errors and leave the ring untouched.
    assert_eq!(admitted * admit.len(), ring.pending());
    assert!(admitted > 0, "sieve admitted nothing: vacuous");
    assert!(dropped > 0, "sieve dropped nothing: vacuous");
}

/// Naive oracle: any byte in [min, max]?
fn naive_band_hit(frame: &[u8], min: u8, max: u8) -> bool {
    frame.iter().any(|&b| b >= min && b <= max)
}

/// Sieve flip twin: named mutation is SWAP the min/max band arguments
/// (mutant band `[max, min]` is empty under unsigned compare, so the
/// mutant drops every frame). The twin runs the SAME real sieve code path
/// with correct vs mutated band arguments, counts decision differences on
/// the real implementation, and fails if the mutant is indistinguishable —
/// a vacuous sieve (admit-nothing or admit-everything) yields differ == 0
/// and flips the court. Not tautological: `differ >= 128` depends on the
/// sieve actually admitting hit frames and dropping miss frames.
#[test]
fn sieve_mutation_flip_twin() {
    // Frames straddling the band: first byte sweeps a full period (some in
    // [0x41,0x7E], some out); byte 1 is a band-interior hit (0x50) for the
    // first half of the sweep only, so the mutant diverges on >=128 frames.
    let frames: Vec<Vec<u8>> = (0..256u32)
        .map(|i| {
            let hit_payload = if i < 128 { 0x50u8 } else { 0x20u8 };
            vec![(i & 0xFF) as u8, hit_payload]
        })
        .collect();
    let mut ring_ok = StdIngressRing::new();
    let mut ring_mutant = StdIngressRing::new();
    let mut differ = 0usize;
    for f in &frames {
        let a = ring_ok.push_sieved(f, 0x41, 0x7E).unwrap();
        let b = ring_mutant.push_sieved(f, 0x7E, 0x41).unwrap();
        // Correct-band decisions track the naive oracle on the real path.
        assert_eq!(a, naive_band_hit(f, 0x41, 0x7E), "sieve oracle drift");
        assert!(
            !b,
            "swapped-band mutant admitted a frame: mutation not exercised as specified"
        );
        if a != b {
            differ += 1;
        }
    }
    // Frames 0..128 carry a band-interior hit byte (0x50): the correct band
    // admits them, the mutant drops them.
    assert!(
        differ >= 128,
        "swapped-band mutant is indistinguishable from the sieve: vacuous twin"
    );
    assert!(ring_ok.pending() >= 128, "sieve admitted nothing: vacuous");
    assert_eq!(ring_mutant.pending(), 0, "mutant ring must stay empty");
}

/// Small-ring instantiation over CAP=16: exercises wrap with tiny frames.
#[test]
fn tiny_ring_wrap_court() {
    let mut ring = IngressRing::<16>::new();
    for round in 0..4u8 {
        ring.push(&[round; 5]).unwrap();
        let mut out = [0u8; 5];
        assert_eq!(ring.pop(&mut out).unwrap(), 5);
        assert_eq!(out, [round; 5]);
    }
    // Full then overflow at tiny scale.
    ring.push(&[1u8; 16]).unwrap();
    assert!(matches!(
        ring.push(&[0u8; 1]),
        Err(IngressError::Overflow { .. })
    ));
}

/// Release-profile measured report (NOT a debug gate): batch-amortized
/// ns/frame over 4096 sieved pushes. METHOD DISCLOSED (G01 CSCB lesson):
/// one Instant pair around the whole batch, divided by frame count -
/// literal per-frame Instant cannot meet 10 ns on macOS. This is a typed
/// REPORT line only; the <= 10 ns/frame number is a TARGET until measured
/// with this method, and this test only runs in release profile.
#[cfg(not(debug_assertions))]
#[test]
fn sieve_throughput_release_report() {
    use std::time::Instant;
    let mut ring = StdIngressRing::new();
    let frame = [0x41u8; 64];
    // Warm the ring path once.
    ring.push_sieved(&frame, 0x41, 0x7E).unwrap();
    ring.pop(&mut [0u8; 64]).unwrap();
    const FRAMES: usize = 4096;
    let t0 = Instant::now();
    let mut budget = ring.capacity();
    let mut drained: usize = 0;
    let mut out = [0u8; 32_768];
    for _ in 0..FRAMES {
        if budget < 64 {
            drained += ring.pop(&mut out).unwrap();
            budget = ring.capacity();
        }
        ring.push_sieved(&frame, 0x41, 0x7E).unwrap();
        budget -= 64;
    }
    let elapsed = t0.elapsed();
    let per_frame_ns = elapsed.as_nanos() / FRAMES as u128;
    println!(
        "MEASURED [sieve-throughput] method=batch-amortized(Instant over {} sieved 64B pushes, drain-at-capacity included) per_frame_ns={} target_ns_per_frame=10 status={}",
        FRAMES,
        per_frame_ns,
        if per_frame_ns <= 10 { "MEETS-TARGET" } else { "OVER-TARGET(report-only)" }
    );
    assert!(drained > 0, "drain path never exercised: vacuous");
}

// ---------------------------------------------------------------------------
// Scatter-gather composition court (moved from posh_wire_court.rs:143-197
// per cull-03 note 1, OVERLAP-PARTIAL verdict). Uses the REAL
// cgap_git::ingress_ring::IngressRing through the lib surface (fully
// qualified paths: this file already owns a `#[path]`-included
// `mod ingress_ring`), plus the POSH wire header law, self-contained here.
// ---------------------------------------------------------------------------

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

impl WireChunkHeader {
    const fn new(tx_id: [u8; 32], fact_count: u32) -> Self {
        WireChunkHeader {
            magic: WIRE_MAGIC,
            tx_id,
            fact_count,
            stride: FACT_BYTES as u32,
        }
    }
}

const _: () = assert!(core::mem::size_of::<WireChunkHeader>() == 64);
const _: () = assert!(core::mem::align_of::<WireChunkHeader>() == 64);

fn header_bytes(h: &WireChunkHeader) -> &[u8] {
    unsafe { core::slice::from_raw_parts(h as *const WireChunkHeader as *const u8, 64) }
}

/// fact i is 16 bytes derived from i.
fn fact_byte(fact: usize, byte: usize) -> u8 {
    (fact * FACT_BYTES + byte) as u8
}

/// Validate the popped stream: header law exact, every fact byte exact,
/// trailer constant. Asserts final state on real bytes.
fn gather_validate(stream: &[u8], expect_facts: usize) {
    let (hdr_bytes, rest) = stream.split_at(64);
    let hdr = unsafe { core::ptr::read_unaligned(hdr_bytes.as_ptr() as *const WireChunkHeader) };
    assert_eq!(hdr.magic, WIRE_MAGIC, "clean header magic");
    assert_eq!(hdr.stride, FACT_BYTES as u32, "clean header stride");
    assert_eq!(hdr.fact_count as usize, expect_facts, "fact count exact");
    let (facts, trailer) = rest.split_at(expect_facts * FACT_BYTES);
    assert_eq!(trailer.len(), TRAILER_BYTES);
    for f in 0..expect_facts {
        for b in 0..FACT_BYTES {
            assert_eq!(
                facts[f * FACT_BYTES + b],
                fact_byte(f, b),
                "fact {f} byte {b}"
            );
        }
    }
    assert!(trailer.iter().all(|&t| t == 0xAB), "trailer constant");
}

const TX: [u8; 32] = [0x5Au8; 32];

/// Clean BEFORE witness, moved verbatim in law from posh_wire_court.rs.
/// 4 scatter-gather slices with disjoint arbitrary byte offsets, one
/// splitting a 16-byte fact in half; the whole stream consumes through the
/// REAL ring; fact count and every fact byte are exact.
#[test]
fn scatter_gather_zero_reassembly_court() {
    let facts = 40usize;
    let stream_len = 64 + facts * FACT_BYTES + TRAILER_BYTES;
    let mut stream = [0u8; 64 + 40 * FACT_BYTES + TRAILER_BYTES];
    let hdr = WireChunkHeader::new(TX, facts as u32);
    stream[..64].copy_from_slice(header_bytes(&hdr));
    for f in 0..facts {
        for b in 0..FACT_BYTES {
            stream[64 + f * FACT_BYTES + b] = fact_byte(f, b);
        }
    }
    let t0 = 64 + facts * FACT_BYTES;
    for b in stream[t0..].iter_mut() {
        *b = 0xAB;
    }
    // Disjoint iov boundaries: 30 / 97 / 150 / rest. 97 splits fact 2
    // ((97-64)=33 into the facts: fact 2, byte 1); 150 splits fact 5.
    let iov = [
        &stream[..30],
        &stream[30..97],
        &stream[97..150],
        &stream[150..],
    ];
    let starts = [0usize, 30, 97, 150];
    for w in starts.windows(2) {
        assert!(w[0] < w[1], "iov slices must be disjoint");
    }
    let mut ring = cgap_git::ingress_ring::IngressRing::<4096>::new();
    assert_eq!(ring.capacity(), 4096);
    let mut total = 0usize;
    for s in &iov {
        ring.push(s).expect("clean scatter push");
        total += s.len();
    }
    assert_eq!(total, stream_len);
    assert_eq!(ring.pending(), stream_len);
    // Zero intermediate heap reassembly: pop lands in a stack array.
    let mut out = [0u8; 64 + 40 * FACT_BYTES + TRAILER_BYTES];
    let mut got = 0usize;
    while got < stream_len {
        let n = ring.pop(&mut out[got..]).expect("clean gather pop");
        assert!(n > 0);
        got += n;
    }
    assert_eq!(ring.pending(), 0);
    assert_eq!(out[..stream_len], stream[..stream_len]);
    gather_validate(&out, facts);
}
