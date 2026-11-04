//! AF-30 Cage Match — Contender A runner (cgap-aoq landed substrate).
//!
//! Composes the landed courts per frame over a fixed frame count
//! (N = 1_000_000, both contenders same N):
//!   1. header decode   — `codec::decode` (AB-6 landed kernel)
//!   2. AEAD seal       — `cgap_crypto::aead::sealdetached` (AB-4 real
//!      kernel, dev-dep; `cgap_aoq::aead::seal` is a pre-admission seam
//!      that refuses, so the real kernel is bound here as AF-1's corridor
//!      court does)
//!   3. chain receipt   — `AoqChain::append` with a real digest seam
//!      (AB-7; same Mix32 seam family as the landed chain_bench.rs)
//!
//! SCOPE LIMIT (honest): this measures the per-frame compute layer only.
//! TLS/network, GC-pause, JVM/containers legs are TARGET-UNMEASURED —
//! see docs/jira/v26.11.3/receipts/af-30-cage-match.md in /Users/sac/arcus
//! for the verdict table.
//!
//! Method parity notes (disclosed, not hidden):
//! - Contender B (Go `encoding/json` unmarshal+remarshal of an equivalent
//!   64-byte struct) covers the serialization layer only; the ratio
//!   reported is serialization-vs-frame-pipeline, an apples-to-oranges
//!   caveat recorded in the receipt, not swept under.
//! - The chain digest seam is Mix32 (the AB-7 landed bench's seam), NOT
//!   BLAKE3 — disclosed so no one cites the per-link figure as a
//!   BLAKE3-chain figure.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

use cgap_aoq::chain::{AoqChain, AoqChainRecord, ChainDigest};
use cgap_aoq::codec::{decode, encode};
use cgap_aoq::header::{AoqHeader, Epoch};
use cgap_crypto::aead::{self, Nonce96};

// ---------------------------------------------------------------- alloc
// census (mirrors the AF-14 runner's G1 method: counting global allocator)

static ALLOCS: AtomicU64 = AtomicU64::new(0);
static ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);

struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(l.size() as u64, Ordering::Relaxed);
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
}

#[global_allocator]
static A: Counting = Counting;

// ---------------------------------------------------------------- seam

#[derive(Clone, Copy)]
struct Mix32;
impl ChainDigest for Mix32 {
    fn digest(&self, parts: &[&[u8]]) -> [u8; 32] {
        mix32(parts)
    }
}

/// Same seam family as the landed `chain_bench.rs` (AB-7 deliverable).
fn mix32(parts: &[&[u8]]) -> [u8; 32] {
    let mut a: u64 = 0xcbf2_9ce4_8422_2325;
    let mut b: u64 = 0x9e37_79b9_7f4a_7c15;
    for part in parts {
        for &byte in *part {
            a ^= byte as u64;
            a = a.wrapping_mul(0x100_0000_01b3);
            b = (b ^ ((byte as u64) | 1)).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        }
    }
    let mut out = [0u8; 32];
    for (i, chunk) in out.chunks_exact_mut(8).enumerate() {
        let v = if i % 2 == 0 {
            a.wrapping_add(i as u64)
        } else {
            b.wrapping_add(i as u64)
        };
        chunk.copy_from_slice(&v.to_le_bytes());
    }
    out
}

// ---------------------------------------------------------------- harness

const N: usize = 1_000_000; // fixed frame count, both contenders
const ROUNDS: usize = 3; // min-of-N rounds

fn make_header(ordinal: u32) -> (AoqHeader, [u8; 64]) {
    let h = AoqHeader::new(Epoch::Genesis as u16, 0xDEAD_BEEF, ordinal, 7);
    let mut wire = [0u8; 64];
    encode(&h, &mut wire).expect("encode");
    (h, wire)
}

#[test]
fn af30_contender_a_frame_pipeline() {
    // Build a valid sealed wire header once; per-frame decode re-runs the
    // full codec law (magic/version/epoch/digest checks) on a fresh copy.
    let (h0, wire0) = make_header(0);
    assert_eq!(decode(&wire0).map(|r| r.magic), Ok(h0.magic));

    // Chain slab: 1M links, allocated once outside the timed region.
    let slab: &'static mut [AoqChainRecord] =
        Box::leak(vec![AoqChainRecord::zeroed(); N].into_boxed_slice());
    let key = [0x42u8; 32];
    let mut aad = [0u8; 36];
    let payload = [0xa5u8; 64];
    let mut ct = [0u8; 64];
    let mut tag = [0u8; 16];

    let mut best_us = f64::INFINITY;
    let mut sink = 0u64;
    for _round in 0..ROUNDS {
        let mut chain = AoqChain::new(Mix32, slab);
        let t0 = std::time::Instant::now();
        sink = 0;
        for i in 0u64..N as u64 {
            // fresh wire copy so decode runs the real law each frame
            let wire = wire0;
            let hdr = match decode(&wire) {
                Ok(h) => h,
                Err(e) => panic!("frame {i}: decode refused: {e:?}"),
            };
            // AAD = first 36 bytes of the frame header (per-frame, real)
            aad[..36].copy_from_slice(&wire[..36]);
            let nonce = Nonce96::new(0, i).expect("nonce");
            aead::seal_detached(&mut ct, &mut tag, &payload, &aad, &key, &nonce);
            sink = sink.wrapping_add(hdr.seq as u64).wrapping_add(ct[0] as u64).wrapping_add(tag[0] as u64);
            chain.append(hdr.epoch, &payload).expect("append");
        }
        let us = t0.elapsed().as_secs_f64() * 1e6 / N as f64;
        if us < best_us {
            best_us = us;
        }
        assert_eq!(chain.len(), N);
    }
    std::hint::black_box(sink);

    // Alloc census window (untimed): 8192 frames through the same path.
    ALLOCS.store(0, Ordering::Relaxed);
    ALLOC_BYTES.store(0, Ordering::Relaxed);
    let mut chain = AoqChain::new(Mix32, slab);
    for i in 0u64..8192u64 {
        let wire = wire0;
        let hdr = decode(&wire).expect("decode");
        aad[..36].copy_from_slice(&wire[..36]);
        let nonce = Nonce96::new(0, i).expect("nonce");
        aead::seal_detached(&mut ct, &mut tag, &payload, &aad, &key, &nonce);
        let _ = hdr.seq;
        let _ = ct[0];
        chain.append(hdr.epoch, &payload).expect("append");
    }
    let census_allocs = ALLOCS.load(Ordering::Relaxed);
    let census_bytes = ALLOC_BYTES.load(Ordering::Relaxed);

    println!(
        "AF30_CONTENDER_A: frames={N} rounds={ROUNDS} best_us_per_frame={best_us:.4} \
         census_frames=8192 census_allocs={census_allocs} census_bytes={census_bytes}"
    );
    assert_eq!(census_allocs, 0, "per-frame pipeline must be alloc-free");
    assert_eq!(census_bytes, 0);
}
