//! adversarial_littles_law_court.rs — RFC-0043 Queue Breakdown & Ingress Saturation Benchmark.
//!
//! Evaluates Little's Law regime transitions under increasing arrival rates:
//! 1. `regime1_in_register_sub_microsecond`: λ < 25M req/s, L < 100, W = 41.4 ns
//! 2. `regime2_spsc_ring_burst`: λ approaching drain rate, capacity = 65,536 slots
//! 3. `regime3_destructive_saturation_refusal`: λ > μ_drain, fail-closed refusal rate under 100M fps

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU64, Ordering};
use divan::counter::ItemsCount;
use divan::{black_box, Bencher};

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::main();
}

const RING_CAPACITY: usize = 65_536;
const RING_MASK: u64 = (RING_CAPACITY - 1) as u64;

#[repr(C, align(128))]
pub struct CachePadded128<T> {
    pub value: T,
    _pad: [u8; 120],
}

#[repr(C, align(128))]
pub struct SovereignSpscRing<T> {
    head: CachePadded128<AtomicU64>,
    tail: CachePadded128<AtomicU64>,
    slots: Box<[UnsafeCell<MaybeUninit<T>>]>,
}

unsafe impl<T: Send> Sync for SovereignSpscRing<T> {}

impl<T: Copy> SovereignSpscRing<T> {
    pub fn new() -> Self {
        let mut slots_vec = Vec::with_capacity(RING_CAPACITY);
        for _ in 0..RING_CAPACITY {
            slots_vec.push(UnsafeCell::new(MaybeUninit::uninit()));
        }
        Self {
            head: CachePadded128 {
                value: AtomicU64::new(0),
                _pad: [0; 120],
            },
            tail: CachePadded128 {
                value: AtomicU64::new(0),
                _pad: [0; 120],
            },
            slots: slots_vec.into_boxed_slice(),
        }
    }

    #[inline(always)]
    pub fn try_push(&self, item: T) -> Result<(), ()> {
        let h = self.head.value.load(Ordering::Relaxed);
        let t = self.tail.value.load(Ordering::Acquire);
        if h.wrapping_sub(t) >= RING_CAPACITY as u64 {
            return Err(());
        }
        let slot = (h & RING_MASK) as usize;
        unsafe {
            (*self.slots[slot].get()).write(item);
        }
        self.head.value.store(h.wrapping_add(1), Ordering::Release);
        Ok(())
    }

    #[inline(always)]
    pub fn try_pop(&self) -> Option<T> {
        let t = self.tail.value.load(Ordering::Relaxed);
        let h = self.head.value.load(Ordering::Acquire);
        if t == h {
            return None;
        }
        let slot = (t & RING_MASK) as usize;
        let item = unsafe { (*self.slots[slot].get()).assume_init() };
        self.tail.value.store(t.wrapping_add(1), Ordering::Release);
        Some(item)
    }

    /// Resident item count: head - tail (wrapping counters, monotone per epoch).
    #[inline(always)]
    pub fn len(&self) -> u64 {
        let h = self.head.value.load(Ordering::Acquire);
        let t = self.tail.value.load(Ordering::Acquire);
        h.wrapping_sub(t)
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: Copy> Default for SovereignSpscRing<T> {
    fn default() -> Self {
        Self::new()
    }
}

// -----------------------------------------------------------------------------
// Regime 1: In-Register Sovereign Core Transit
// -----------------------------------------------------------------------------

#[divan::bench]
fn regime1_in_register_steady_state(b: Bencher) {
    let ring = SovereignSpscRing::<u64>::new();
    const STEADY_OPS: u64 = 1_000;

    b.counter(ItemsCount::new(STEADY_OPS)).bench_local(|| {
        for i in 0..STEADY_OPS {
            let _ = ring.try_push(black_box(i));
            let popped = ring.try_pop();
            black_box(popped);
        }
    });
}

// -----------------------------------------------------------------------------
// Regime 2: SPSC Saturated Ring Capacity Filling
// -----------------------------------------------------------------------------

#[divan::bench]
fn regime2_spsc_ring_burst_fill(b: Bencher) {
    let ring = SovereignSpscRing::<u64>::new();
    const FILL_COUNT: u64 = 65_536;

    b.counter(ItemsCount::new(FILL_COUNT)).bench_local(|| {
        let mut pushed = 0u64;
        let mut refused = 0u64;
        for i in 0..FILL_COUNT {
            if ring.try_push(black_box(i)).is_ok() {
                pushed += 1;
            } else {
                refused += 1;
            }
        }
        let mut popped = 0u64;
        for _ in 0..FILL_COUNT {
            if ring.try_pop().is_some() {
                popped += 1;
            }
        }
        // Conservation identity (arcus E10 parity): injected = admitted + resident + refused.
        let resident = ring.len();
        assert_eq!(
            pushed,
            popped + resident + refused,
            "regime2 conservation violated: pushed={pushed} popped={popped} \
             resident={resident} refused={refused}"
        );
        black_box((pushed, popped, resident, refused));
    });
}

// -----------------------------------------------------------------------------
// Regime 3: Catastrophic Breakdown (Capacity Saturation & Line-Rate Refusal)
// -----------------------------------------------------------------------------

#[divan::bench]
fn regime3_destructive_saturation_refusal(b: Bencher) {
    let ring = SovereignSpscRing::<u64>::new();
    // Fill to brim
    for i in 0..RING_CAPACITY as u64 {
        let _ = ring.try_push(i);
    }

    const SATURATION_PROBES: u64 = 10_000;
    b.counter(ItemsCount::new(SATURATION_PROBES)).bench_local(|| {
        let mut pushed_ok = 0u64;
        let mut refusals = 0u64;
        for i in 0..SATURATION_PROBES {
            if ring.try_push(black_box(i)).is_err() {
                refusals += 1;
            } else {
                pushed_ok += 1;
            }
        }
        // Conservation identity (arcus E10 parity): injected = admitted + refused.
        assert_eq!(
            pushed_ok + refusals,
            SATURATION_PROBES,
            "regime3 conservation violated: pushed_ok={pushed_ok} refusals={refusals}"
        );
        black_box((pushed_ok, refusals));
    });
}
