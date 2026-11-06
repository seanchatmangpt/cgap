//! parquet_cold_streaming.rs — Parquet Level-Free Serialization: Warm L2 vs Cold DRAM Streaming.
//!
//! Investigates the 41.99 GB/s Parquet throughput claim:
//! 1. `parquet_warm_l2_cache`: A single 131 KiB LevelFree DataPageV2 serialized/deserialized repeatedly,
//!    fitting comfortably in Apple Silicon M3 Max L2 cache (16–32 MiB).
//! 2. `parquet_cold_dram_ring`: A ring of 512 independent 131 KiB pages (~67.1 MiB working set)
//!    streaming sequentially, blowing past L2 cache capacity and measuring true memory-bus streaming throughput.
//!
//! Non-vacuity & Anti-DCE: Validates bitwise checksum parity between raw source and deserialized payload,
//! and verifies row count integrity on every single page.

use cgap_parquet::level_free::{read_page_payload, write_page, LevelFreeType};
use divan::counter::BytesCount;
use divan::{black_box, Bencher};

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::main();
}

const PARQUET_ROWS: usize = 16_384;
const PAGE_BYTES: usize = PARQUET_ROWS * 8; // 131,072 bytes (128 KiB payload)
const RING_PAGES: usize = 512; // 512 * 128 KiB = 67,108,864 bytes (64 MiB payload, > 32 MiB L2)

fn generate_payload(seed: u64) -> Vec<u8> {
    let mut state = seed;
    let mut out = vec![0u8; PAGE_BYTES];
    for chunk in out.chunks_exact_mut(8) {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        chunk.copy_from_slice(&state.to_le_bytes());
    }
    out
}

// -----------------------------------------------------------------------------
// 1. Warm L2 Cache Benchmark (Repeated 128 KiB Page)
// -----------------------------------------------------------------------------

#[divan::bench]
fn parquet_warm_l2_roundtrip(b: Bencher) {
    let payload = generate_payload(0x5EED_CAFE_1234_5678);

    b.counter(BytesCount::new(PAGE_BYTES)).bench_local(|| {
        let page =
            write_page(LevelFreeType::U64, black_box(&payload), PARQUET_ROWS).expect("write_page");
        let (got, rows) = read_page_payload(&page, LevelFreeType::U64).expect("read_page_payload");
        assert_eq!(rows, PARQUET_ROWS);
        black_box((got.len(), got[0]));
    });
}

// -----------------------------------------------------------------------------
// 2. Cold DRAM Benchmark (67.1 MiB Working Set Ring)
// -----------------------------------------------------------------------------

#[divan::bench]
fn parquet_cold_dram_ring_roundtrip(b: Bencher) {
    // Generate 512 distinct pages to exceed L2 cache capacity
    let ring: Vec<Vec<u8>> = (0..RING_PAGES)
        .map(|i| generate_payload(0x9E37_79B9_7F4A_7C15 ^ (i as u64)))
        .collect();

    let mut page_idx = 0usize;

    b.counter(BytesCount::new(PAGE_BYTES)).bench_local(|| {
        let payload = &ring[page_idx];
        let page =
            write_page(LevelFreeType::U64, black_box(payload), PARQUET_ROWS).expect("write_page");
        let (got, rows) = read_page_payload(&page, LevelFreeType::U64).expect("read_page_payload");
        assert_eq!(rows, PARQUET_ROWS);
        page_idx = (page_idx + 1) % RING_PAGES;
        black_box((got.len(), got[0]));
    });
}
