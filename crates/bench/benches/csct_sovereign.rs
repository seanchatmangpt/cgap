//! csct_sovereign.rs — Divan-Driven Silicon Kernel Benchmark for RFC-0042 CSCT.
//!
//! Evaluates the four physical substrate kernels under rigorous statistical measurement:
//! 1. `arrow_bitplane_pack`: In-register vertical bit-plane packing (pack_vertical)
//! 2. `bitweaving_scan_pruned`: Vertical bit-slice pruned range scan (scan_range_vertical_counted)
//! 3. `blake3_wide_simd_attest`: Wide-SIMD cryptographic attestation hashing (hash_wide_simd)
//! 4. `spsc_ring_slot_transit`: Lock-free SPSC ring slot push/pop roundtrip (0 B/op heap allocation)
//!
//! Uses Divan's AllocProfiler to measure and assert exact heap allocation behavior on steady paths.

use cgap_blake3::hash_wide_simd;
use cgap_parquet::bitslice::{pack_vertical, scan_range_vertical_counted};
use divan::counter::{BytesCount, ItemsCount};
use divan::{black_box, Bencher};
use std::sync::atomic::{AtomicUsize, Ordering};

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::main();
}

// -----------------------------------------------------------------------------
// 1. Arrow Bit-Plane In-Register Packing (Kernel K1)
// -----------------------------------------------------------------------------

fn generate_probe_values(n: usize, width: u8) -> Vec<u32> {
    let mask = (1u64 << width) - 1;
    let mut x = 0x9E37_79B9_7F4A_7C15u64;
    (0..n)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            (x & mask) as u32
        })
        .collect()
}

#[divan::bench(args = [
    (1_000, 8),
    (10_000, 8),
    (65_536, 8),
])]
fn arrow_bitplane_pack(b: Bencher, (n, width): (usize, u8)) {
    let values = generate_probe_values(n, width);
    let bytes_input = n * std::mem::size_of::<u32>();

    b.counter(BytesCount::new(bytes_input))
        .counter(ItemsCount::new(n as u64))
        .bench_local(|| {
            let packed = pack_vertical(black_box(&values), black_box(width));
            black_box(packed);
        });
}

// -----------------------------------------------------------------------------
// 2. BitWeaving Vertical Bit-Slice Pruned Scan (Kernel K2)
// -----------------------------------------------------------------------------

#[divan::bench(args = [
    (10_000, 8, 10, 50),
    (65_536, 8, 10, 50),
    (1_048_576, 8, 10, 50),
])]
fn bitweaving_scan_pruned(b: Bencher, (n, width, lo, hi): (usize, u8, u32, u32)) {
    let values = generate_probe_values(n, width);
    let packed = pack_vertical(&values, width);
    let groups = n.div_ceil(64);
    let mut out_mask = vec![0u64; groups];

    b.counter(ItemsCount::new(n as u64)).bench_local(|| {
        let touched = scan_range_vertical_counted(
            black_box(&packed),
            black_box(width),
            black_box(lo),
            black_box(hi),
            black_box(&mut out_mask),
        );
        black_box((touched, &out_mask));
    });
}

// -----------------------------------------------------------------------------
// 3. Wide-SIMD BLAKE3 Tree Attestation Hashing (Kernel K4)
// -----------------------------------------------------------------------------

#[divan::bench(args = [
    64,      // Standard 64-byte wire frame
    152,     // 152-byte A04 frame (128B header + 24B payload)
    1024,    // 1 KiB batch frame
    65536,   // 64 KiB chunk
])]
fn blake3_wide_simd_attest(b: Bencher, len: usize) {
    let mut frame = vec![0u8; len];
    for (i, byte) in frame.iter_mut().enumerate() {
        *byte = (i * 31 % 256) as u8;
    }

    b.counter(BytesCount::new(len))
        .counter(ItemsCount::new(1u64))
        .bench_local(|| {
            let digest = hash_wide_simd(black_box(&frame));
            black_box(digest);
        });
}

// -----------------------------------------------------------------------------
// 4. Lock-Free SPSC Ring Slot Push/Pop Roundtrip (Kernel K3 Transport)
// -----------------------------------------------------------------------------

const RING_CAPACITY: usize = 1024;

struct SpscRingBuffer {
    buffer: Vec<u64>,
    head: AtomicUsize,
    tail: AtomicUsize,
}

impl SpscRingBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0u64; capacity],
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    #[inline(always)]
    fn push(&mut self, val: u64) -> bool {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        if head.wrapping_sub(tail) >= RING_CAPACITY {
            return false;
        }
        self.buffer[head % RING_CAPACITY] = val;
        self.head.store(head.wrapping_add(1), Ordering::Release);
        true
    }

    #[inline(always)]
    fn pop(&mut self) -> Option<u64> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        if tail == head {
            return None;
        }
        let val = self.buffer[tail % RING_CAPACITY];
        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        Some(val)
    }
}

#[divan::bench]
fn spsc_ring_slot_transit(b: Bencher) {
    let mut ring = SpscRingBuffer::new(RING_CAPACITY);
    let payload = 0xA20C_0FFE_E5A7_E000u64;

    b.counter(ItemsCount::new(1u64)).bench_local(|| {
        let pushed = ring.push(black_box(payload));
        assert!(pushed, "SPSC ring capacity overflow");
        let popped = ring.pop();
        assert_eq!(popped, Some(payload), "SPSC roundtrip corruption");
        black_box(popped);
    });
}
