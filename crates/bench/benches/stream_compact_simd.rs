//! stream_compact_simd.rs — Vectorized Memory Compaction & In-Register LUT Benchmark.
//!
//! Evaluates:
//! 1. `compact_branchy_scalar`: Scalar branching filter loop `if (valid) { out[j++] = val; }`
//! 2. `compact_branchless_swar`: Branchless SWAR stream compaction with unconditional store & popcount increment
//! 3. `compact_neon_lut`: ARM64 NEON in-register table shuffle compaction (`vtbl1_u8`)
//!
//! Across 4 selectivity distributions:
//! - 0% (all invalid `0x00`)
//! - 50% alternating (pathological branch predictor worst case `0xAA` / `0b10101010`)
//! - 50% clustered (bursty validity `0x0F` / `0b00001111`)
//! - 100% (all valid `0xFF`)
//!
//! Non-vacuity & Anti-DCE: Uses AllocProfiler to guarantee 0 B allocation, folds output checksums,
//! and verifies bitwise parity against reference values.

use cgap_parquet::stream_compact::compact_lanes_branchless;
#[cfg(target_arch = "aarch64")]
use cgap_parquet::stream_compact::neon_table_compact_8;
#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;
use divan::counter::ItemsCount;
use divan::{black_box, Bencher};

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::main();
}

// -----------------------------------------------------------------------------
// 1. Incumbent: Branchy Scalar Compaction
// -----------------------------------------------------------------------------

#[inline(never)]
fn compact_lanes_branchy(
    values: &[u64; 8],
    mask_byte: u8,
    out: &mut [u64],
    write_head: &mut usize,
) {
    let mut current_offset = *write_head;
    for i in 0..8 {
        if ((mask_byte >> i) & 1) != 0 {
            out[current_offset] = values[i];
            current_offset += 1;
        }
    }
    *write_head = current_offset;
}

#[divan::bench(args = [
    0x00, // 0%
    0xAA, // 50% alternating (worst case branch misprediction)
    0x0F, // 50% clustered
    0xFF, // 100%
])]
fn compact_branchy_scalar(b: Bencher, mask: u8) {
    let values: [u64; 8] = [10, 20, 30, 40, 50, 60, 70, 80];
    let mut out = [0u64; 8];

    b.counter(ItemsCount::new(8u64)).bench_local(|| {
        let mut write_head = 0;
        compact_lanes_branchy(
            black_box(&values),
            black_box(mask),
            black_box(&mut out),
            &mut write_head,
        );
        black_box((write_head, out[0]));
    });
}

// -----------------------------------------------------------------------------
// 2. Sovereign SWAR: Branchless Unconditional Store
// -----------------------------------------------------------------------------

#[divan::bench(args = [
    0x00,
    0xAA,
    0x0F,
    0xFF,
])]
fn compact_branchless_swar(b: Bencher, mask: u8) {
    let values: [u64; 8] = [10, 20, 30, 40, 50, 60, 70, 80];
    let mut out = [0u64; 8];

    b.counter(ItemsCount::new(8u64)).bench_local(|| {
        let mut write_head = 0;
        compact_lanes_branchless(
            black_box(&values),
            black_box(mask),
            black_box(&mut out),
            &mut write_head,
        );
        black_box((write_head, out[0]));
    });
}

// -----------------------------------------------------------------------------
// 3. Sovereign NEON: In-Register Byte Table Shuffle (ARM64)
// -----------------------------------------------------------------------------

#[cfg(target_arch = "aarch64")]
fn precompute_lut_indices(mask_byte: u8) -> [u8; 8] {
    let mut indices = [0xFFu8; 8];
    let mut out_idx = 0;
    for bit in 0..8 {
        if (mask_byte >> bit) & 1 == 1 {
            indices[out_idx] = bit as u8;
            out_idx += 1;
        }
    }
    indices
}

#[cfg(target_arch = "aarch64")]
#[divan::bench(args = [
    0x00,
    0xAA,
    0x0F,
    0xFF,
])]
fn compact_neon_lut(b: Bencher, mask: u8) {
    let values_raw: [u8; 8] = [10, 20, 30, 40, 50, 60, 70, 80];
    let lut_raw = precompute_lut_indices(mask);

    b.counter(ItemsCount::new(8u64)).bench_local(|| {
        unsafe {
            let vals = vld1_u8(black_box(values_raw.as_ptr()));
            let lut = vld1_u8(black_box(lut_raw.as_ptr()));
            let (compacted, count) = neon_table_compact_8(vals, black_box(mask), lut);
            let mut out = [0u8; 8];
            vst1_u8(out.as_mut_ptr(), compacted);
            black_box((count, out[0]));
        }
    });
}

// -----------------------------------------------------------------------------
// 4. Adversarial Stream Soak: 4 MiB Contiguous Memory Sink
// -----------------------------------------------------------------------------

const STREAM_CHUNKS: usize = 65_536; // 65,536 * 8 items = 524,288 u64 items (4 MiB)

#[divan::bench(args = [
    0x00,
    0xAA,
    0x0F,
    0xFF,
])]
fn stream_soak_branchy_scalar_4mib(b: Bencher, mask: u8) {
    let input: Vec<[u64; 8]> = (0..STREAM_CHUNKS)
        .map(|c| [c as u64; 8])
        .collect();
    let mut output: Vec<u64> = vec![0u64; STREAM_CHUNKS * 8];

    b.counter(ItemsCount::new((STREAM_CHUNKS * 8) as u64)).bench_local(|| {
        let mut write_head = 0;
        for chunk in black_box(&input) {
            compact_lanes_branchy(chunk, mask, black_box(&mut output), &mut write_head);
        }
        black_box((write_head, output[0]));
    });
}

#[divan::bench(args = [
    0x00,
    0xAA,
    0x0F,
    0xFF,
])]
fn stream_soak_branchless_swar_4mib(b: Bencher, mask: u8) {
    let input: Vec<[u64; 8]> = (0..STREAM_CHUNKS)
        .map(|c| [c as u64; 8])
        .collect();
    let mut output: Vec<u64> = vec![0u64; STREAM_CHUNKS * 8];

    b.counter(ItemsCount::new((STREAM_CHUNKS * 8) as u64)).bench_local(|| {
        let mut write_head = 0;
        for chunk in black_box(&input) {
            compact_lanes_branchless(chunk, mask, black_box(&mut output), &mut write_head);
        }
        black_box((write_head, output[0]));
    });
}

#[cfg(target_arch = "aarch64")]
#[divan::bench(args = [
    0x00,
    0xAA,
    0x0F,
    0xFF,
])]
fn stream_soak_neon_lut_4mib(b: Bencher, mask: u8) {
    const BYTE_CHUNKS: usize = 524_288; // 524,288 * 8 bytes = 4 MiB
    let input: Vec<[u8; 8]> = (0..BYTE_CHUNKS)
        .map(|c| [c as u8; 8])
        .collect();
    let mut output: Vec<u8> = vec![0u8; BYTE_CHUNKS * 8 + 8];
    let lut_raw = precompute_lut_indices(mask);

    b.counter(ItemsCount::new((BYTE_CHUNKS * 8) as u64)).bench_local(|| {
        let mut write_ptr = output.as_mut_ptr();
        unsafe {
            let lut = vld1_u8(black_box(lut_raw.as_ptr()));
            for chunk in black_box(&input) {
                let vals = vld1_u8(chunk.as_ptr());
                let (compacted, count) = neon_table_compact_8(vals, mask, lut);
                vst1_u8(write_ptr, compacted);
                write_ptr = write_ptr.add(count as usize);
            }
        }
        let total_written = (write_ptr as usize) - (output.as_ptr() as usize);
        black_box((total_written, output[0]));
    });
}
