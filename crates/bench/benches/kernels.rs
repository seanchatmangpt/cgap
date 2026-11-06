//! v26.11.1 Phase 2 (Measure) membrane benchmark target.
//!
//! Evaluates the core hot-path silicon kernels:
//! 1. AoS->SoA Quad transpose (cgap-arrow)
//! 2. Branchless quad predicate filter (cgap-arrow)
//! 3. Wide-SIMD BLAKE3 tree hash (cgap-blake3)
//! 4. Parquet level-free page roundtrip (cgap-parquet)
//!
//! Measured under Divan with AllocProfiler installed for exact alloc tracking.

use cgap_arrow::predicates::filter_quads_eq;
use cgap_arrow::transpose::{transpose, Quad64};
use cgap_blake3::hash_wide_simd;
use cgap_parquet::level_free::{read_page_payload, write_page, LevelFreeType};
use divan::counter::BytesCount;
use divan::{black_box, Bencher};

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

const QUADS: usize = 1 << 14; // 16,384 quads = 1 MiB of 64B records
const HASH_BUF: usize = 16 * 1024 * 1024; // 16 MiB
const PARQUET_ROWS: usize = 1 << 14; // 16,384 x u64 rows

fn xorshift(seed: u64) -> impl FnMut() -> u64 {
    let mut x = seed;
    move || {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x
    }
}

fn quad_corpus(len: usize) -> Vec<Quad64> {
    let mut next = xorshift(0xBF58_476D_1CE4_E5B9);
    (0..len)
        .map(|_| Quad64 {
            s_id: next() as u32,
            p_id: (next() >> 32) as u32,
            o_id: next() as u32,
            g_id: (next() >> 32) as u32,
            t_ns: next(),
            vector_clock: next() as u32,
            policy_mask: (next() >> 32) as u32 & 0x0FFF_FFFF,
            inline_digest: core::array::from_fn(|b| (next() >> (b % 8)) as u8),
        })
        .collect()
}

#[divan::bench]
fn quad_transpose_16k(b: Bencher) {
    let aos = quad_corpus(QUADS);
    b.counter(BytesCount::new(QUADS * std::mem::size_of::<Quad64>()))
        .bench(|| {
            black_box(transpose(black_box(&aos)));
        });
}

#[divan::bench]
fn predicate_filter_16k(b: Bencher) {
    let mut next = xorshift(0x243F_6A88_85A3_08D3);
    let mut data = Vec::with_capacity(QUADS * 4);
    for q in 0..QUADS {
        if q % 64 == 0 {
            data.extend_from_slice(&[0x0; 4]);
        } else {
            data.push(next() as u32 | 1);
        }
        for _ in 1..4 {
            data.push(next() as u32 | 1);
        }
    }
    let out = std::sync::Mutex::new(vec![0u32; QUADS + 1]);
    let needle = 0u32;
    b.counter(BytesCount::new(QUADS * 16)).bench(|| {
        let mut out = out.lock().unwrap();
        let n = filter_quads_eq(black_box(&data), QUADS, needle, &mut out);
        black_box(n);
    });
}

#[divan::bench]
fn blake3_hash_16mib(b: Bencher) {
    let mut next = xorshift(0xDEAD_BEEF);
    let buf: Vec<u8> = (0..HASH_BUF).map(|_| next() as u8).collect();
    b.counter(BytesCount::new(HASH_BUF)).bench(|| {
        black_box(hash_wide_simd(black_box(&buf)));
    });
}

#[divan::bench]
fn parquet_level_free_roundtrip_16k(b: Bencher) {
    let mut next = xorshift(0x5EED_5EED_5EED_5EED);
    let payload: Vec<u8> = (0..PARQUET_ROWS * 8).map(|_| next() as u8).collect();
    b.counter(BytesCount::new(PARQUET_ROWS * 8)).bench(|| {
        let page =
            write_page(LevelFreeType::U64, black_box(&payload), PARQUET_ROWS).expect("write_page");
        let (got, rows) = read_page_payload(&page, LevelFreeType::U64).expect("read_page_payload");
        black_box((got.len(), rows));
    });
}

fn main() {
    divan::main();
}
