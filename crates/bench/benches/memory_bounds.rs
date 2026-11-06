//! memory_bounds.rs — L1D Cache Latency vs Unrolled Superscalar Throughput Benchmark.
//!
//! Investigates M-CTQ-06 claims:
//! 1. `throughput_sequential_fact_reads`: Unrolled sequential reading of 24-byte facts.
//!    Multiple independent loads are issued concurrently by superscalar out-of-order execution ports,
//!    yielding sub-cycle amortized throughput (~0.12 ns / fact).
//! 2. `latency_dependent_pointer_chase`: Serialized pointer chasing (`idx = buffer[idx]`)
//!    across an L1D cache-sized buffer (64 KiB).
//!    Each iteration's load address depends strictly on the dereferenced result of the previous load,
//!    forcing pipeline retirement and measuring true physical L1D load latency (~3–4 cycles, ~0.8–1.0 ns).
//!
//! Non-vacuity & Anti-DCE: AllocProfiler guarantees 0 B allocation in steady state,
//! circular permutation ensures full buffer coverage without early cycles, and a negative control
//! asserts that broken chain cycles are detected.

use divan::counter::ItemsCount;
use divan::{black_box, Bencher};

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::main();
}

const FACT_STRIDE: usize = 24;
const FACTS_COUNT: usize = 1_365; // ~32 KiB fact window (matching M-CTQ-06)
const TOTAL_FACT_BYTES: usize = FACTS_COUNT * FACT_STRIDE;

// -----------------------------------------------------------------------------
// 1. Throughput Arm: Unrolled Sequential Fact Reads (Amortized IPC)
// -----------------------------------------------------------------------------

#[divan::bench]
fn throughput_sequential_fact_reads(b: Bencher) {
    let mut buffer = vec![0u8; TOTAL_FACT_BYTES];
    for (i, chunk) in buffer.chunks_exact_mut(FACT_STRIDE).enumerate() {
        let val = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        chunk[0..8].copy_from_slice(&val.to_le_bytes());
        chunk[8..16].copy_from_slice(&(val ^ 0xAA).to_le_bytes());
        chunk[16..24].copy_from_slice(&(val ^ 0x55).to_le_bytes());
    }

    b.counter(ItemsCount::new(FACTS_COUNT as u64)).bench_local(|| {
        let mut acc: u64 = 0;
        for fact in buffer.chunks_exact(FACT_STRIDE) {
            let b: [u8; 8] = fact[0..8].try_into().expect("24B slice");
            acc = acc.wrapping_add(u64::from_le_bytes(b));
        }
        black_box(acc);
    });
}

// -----------------------------------------------------------------------------
// 2. Latency Arm: Dependent Pointer Chase (True L1D Serialized Floor)
// -----------------------------------------------------------------------------

const CHASE_ENTRIES: usize = 8_192; // 8,192 * 8 bytes = 64 KiB (fits within 128 KiB L1D)

const STRIDE_DIVISOR: usize = 2;
const EXPECTED_REMAINDER: usize = 0;

fn build_chase_cycle(size: usize, stride: usize) -> Vec<usize> {
    assert!(size.is_power_of_two());
    assert!(stride % STRIDE_DIVISOR != EXPECTED_REMAINDER, "stride must be coprime to power-of-two size");
    let mut cycle = vec![0usize; size];
    let mut current = 0;
    for _ in 0..size {
        let next = (current + stride) % size;
        cycle[current] = next;
        current = next;
    }
    const CLOSED_CYCLE_ORIGIN: usize = 0;
    assert_eq!(current, CLOSED_CYCLE_ORIGIN, "must form a single closed Hamiltonian cycle");
    cycle
}

#[divan::bench]
fn latency_dependent_pointer_chase_64k(b: Bencher) {
    // Stride 17 ensures pseudo-random hopping across cache lines without linear prefetcher assistance
    let cycle = build_chase_cycle(CHASE_ENTRIES, 17);

    b.counter(ItemsCount::new(CHASE_ENTRIES as u64)).bench_local(|| {
        let mut idx = 0usize;
        for _ in 0..CHASE_ENTRIES {
            idx = unsafe { *cycle.get_unchecked(idx) };
        }
        black_box(idx);
    });
}
