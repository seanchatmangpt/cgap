//! Challenger Sales Benchmark: Parquet Stream Compaction & Vertical BitSlice
//!
//! Compares CGAP's branchless stream compaction and vertical bitslice scan
//! against conventional branchy and horizontal scan approaches.

use cgap_parquet::bitslice::{pack_vertical, scan_range_vertical};
use cgap_parquet::stream_compact::compact_lanes_branchless;
use divan::{black_box, Bencher};

fn main() {
    divan::main();
}

// -----------------------------------------------------------------------------
// 1. Stream Compaction: Branchy vs CGAP Branchless (at 50% selectivity)
// -----------------------------------------------------------------------------

/// Conventional branchy compaction: `if (valid) { out[j++] = val; }`
/// Suffers severe pipeline branch misprediction on alternating valid bits (0b10101010).
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
    0x00, // 0% selectivity
    0xAA, // 50% selectivity alternating (worst branch predictor case)
    0xFF, // 100% selectivity
])]
fn stream_compact_branchy_incumbent(b: Bencher, mask: u8) {
    let values: [u64; 8] = [10, 20, 30, 40, 50, 60, 70, 80];
    let mut out = [0u64; 8];

    b.bench_local(|| {
        let mut write_head = 0;
        compact_lanes_branchy(
            black_box(&values),
            black_box(mask),
            black_box(&mut out),
            &mut write_head,
        );
        black_box(write_head);
    });
}

#[divan::bench(args = [
    0x00, // 0% selectivity
    0xAA, // 50% selectivity alternating
    0xFF, // 100% selectivity
])]
fn stream_compact_cgap_challenger(b: Bencher, mask: u8) {
    let values: [u64; 8] = [10, 20, 30, 40, 50, 60, 70, 80];
    let mut out = [0u64; 8];

    b.bench_local(|| {
        let mut write_head = 0;
        compact_lanes_branchless(
            black_box(&values),
            black_box(mask),
            black_box(&mut out),
            &mut write_head,
        );
        black_box(write_head);
    });
}

// -----------------------------------------------------------------------------
// 2. Range Scan: Horizontal Scalar vs Vertical Bit-Slice Pruned Scan
// -----------------------------------------------------------------------------

const ROWS: usize = 4096;
const BIT_WIDTH: u8 = 8;

fn make_sample_data(len: usize) -> Vec<u32> {
    let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
    (0..len)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            (x & 0xFF) as u32 // 8-bit values
        })
        .collect()
}

#[divan::bench]
fn range_scan_horizontal_incumbent(b: Bencher) {
    let data = make_sample_data(ROWS);
    let lo = 50u32;
    let hi = 200u32;

    b.bench_local(|| {
        let mut match_count = 0usize;
        for &val in black_box(&data) {
            if val >= lo && val <= hi {
                match_count += 1;
            }
        }
        black_box(match_count);
    });
}

#[divan::bench]
fn range_scan_cgap_vertical_bitslice(b: Bencher) {
    let data = make_sample_data(ROWS);
    let packed = pack_vertical(&data, BIT_WIDTH);
    let lo = 50u32;
    let hi = 200u32;
    let groups = (ROWS + 63) / 64;
    let mut out_valid = vec![0u64; groups];

    b.bench_local(|| {
        scan_range_vertical(
            black_box(&packed),
            black_box(BIT_WIDTH),
            black_box(lo),
            black_box(hi),
            black_box(&mut out_valid),
        );
        black_box(out_valid[0]);
    });
}
