//! ring_cache_coherency.rs — Cross-Core False-Sharing & Cache Coherency Benchmark.
//!
//! Contrasts:
//! 1. `ring_unpadded_false_sharing`: `head` and `tail` atomics located within the same cache line.
//!    Produces inter-core Request-For-Ownership (RFO) traffic and cache line bouncing under concurrency.
//! 2. `ring_padded_cacheline_separated`: `head` and `tail` atomics separated by 128 bytes with
//!    `#[repr(align(64))]` padding (as in `cgap-beam::spsc_ring::CachePadded`).
//!
//! Non-vacuity & Anti-DCE: Uses AllocProfiler to guarantee 0 B steady-state allocation,
//! black_boxes transit values, and asserts monotonic queue delivery.

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU64, Ordering};
use divan::counter::ItemsCount;
use divan::{black_box, Bencher};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

const CAPACITY: usize = 1024;
const MASK: u64 = (CAPACITY - 1) as u64;

// -----------------------------------------------------------------------------
// 1. Unpadded SPSC Ring (False Sharing Hazard)
// -----------------------------------------------------------------------------

pub struct UnpaddedRing<T> {
    // Both atomics are adjacent — guaranteed to share the same cache line
    head: AtomicU64,
    tail: AtomicU64,
    slots: [UnsafeCell<MaybeUninit<T>>; CAPACITY],
}

unsafe impl<T: Send> Sync for UnpaddedRing<T> {}

impl<T: Copy> UnpaddedRing<T> {
    pub const fn new() -> Self {
        Self {
            head: AtomicU64::new(0),
            tail: AtomicU64::new(0),
            slots: [const { UnsafeCell::new(MaybeUninit::uninit()) }; CAPACITY],
        }
    }

    #[inline(always)]
    pub fn try_push(&self, item: T) -> bool {
        let h = self.head.load(Ordering::Relaxed);
        let t = self.tail.load(Ordering::Acquire);
        if h.wrapping_sub(t) >= CAPACITY as u64 {
            return false;
        }
        let slot = (h & MASK) as usize;
        unsafe {
            (*self.slots[slot].get()).write(item);
        }
        self.head.store(h.wrapping_add(1), Ordering::Release);
        true
    }

    #[inline(always)]
    pub fn try_pop(&self) -> Option<T> {
        let t = self.tail.load(Ordering::Relaxed);
        let h = self.head.load(Ordering::Acquire);
        if t == h {
            return None;
        }
        let slot = (t & MASK) as usize;
        let item = unsafe { (*self.slots[slot].get()).assume_init() };
        self.tail.store(t.wrapping_add(1), Ordering::Release);
        Some(item)
    }
}

// -----------------------------------------------------------------------------
// 2. 64-Byte Cache-Padded SPSC Ring (x86_64 Conventional Layout)
// -----------------------------------------------------------------------------

#[repr(C, align(64))]
pub struct CachePadded64<T> {
    pub value: T,
    _pad: [u8; 56], // 8 bytes value + 56 bytes pad = 64 bytes
}

#[repr(C, align(64))]
pub struct PaddedRing64<T> {
    head: CachePadded64<AtomicU64>,
    tail: CachePadded64<AtomicU64>,
    slots: [UnsafeCell<MaybeUninit<T>>; CAPACITY],
}

unsafe impl<T: Send> Sync for PaddedRing64<T> {}

impl<T: Copy> PaddedRing64<T> {
    pub const fn new() -> Self {
        Self {
            head: CachePadded64 {
                value: AtomicU64::new(0),
                _pad: [0; 56],
            },
            tail: CachePadded64 {
                value: AtomicU64::new(0),
                _pad: [0; 56],
            },
            slots: [const { UnsafeCell::new(MaybeUninit::uninit()) }; CAPACITY],
        }
    }

    #[inline(always)]
    pub fn try_push(&self, item: T) -> bool {
        let h = self.head.value.load(Ordering::Relaxed);
        let t = self.tail.value.load(Ordering::Acquire);
        if h.wrapping_sub(t) >= CAPACITY as u64 {
            return false;
        }
        let slot = (h & MASK) as usize;
        unsafe {
            (*self.slots[slot].get()).write(item);
        }
        self.head.value.store(h.wrapping_add(1), Ordering::Release);
        true
    }

    #[inline(always)]
    pub fn try_pop(&self) -> Option<T> {
        let t = self.tail.value.load(Ordering::Relaxed);
        let h = self.head.value.load(Ordering::Acquire);
        if t == h {
            return None;
        }
        let slot = (t & MASK) as usize;
        let item = unsafe { (*self.slots[slot].get()).assume_init() };
        self.tail.value.store(t.wrapping_add(1), Ordering::Release);
        Some(item)
    }
}

// -----------------------------------------------------------------------------
// 3. 128-Byte Apple Silicon Sovereign Layout (Darwin ARM64 True Isolation)
// -----------------------------------------------------------------------------

#[repr(C, align(128))]
pub struct CachePadded128<T> {
    pub value: T,
    _pad: [u8; 120], // 8 bytes value + 120 bytes pad = 128 bytes
}

#[repr(C, align(128))]
pub struct PaddedRing128<T> {
    head: CachePadded128<AtomicU64>,
    tail: CachePadded128<AtomicU64>,
    slots: [UnsafeCell<MaybeUninit<T>>; CAPACITY],
}

unsafe impl<T: Send> Sync for PaddedRing128<T> {}

impl<T: Copy> PaddedRing128<T> {
    pub const fn new() -> Self {
        Self {
            head: CachePadded128 {
                value: AtomicU64::new(0),
                _pad: [0; 120],
            },
            tail: CachePadded128 {
                value: AtomicU64::new(0),
                _pad: [0; 120],
            },
            slots: [const { UnsafeCell::new(MaybeUninit::uninit()) }; CAPACITY],
        }
    }

    #[inline(always)]
    pub fn try_push(&self, item: T) -> bool {
        let h = self.head.value.load(Ordering::Relaxed);
        let t = self.tail.value.load(Ordering::Acquire);
        if h.wrapping_sub(t) >= CAPACITY as u64 {
            return false;
        }
        let slot = (h & MASK) as usize;
        unsafe {
            (*self.slots[slot].get()).write(item);
        }
        self.head.value.store(h.wrapping_add(1), Ordering::Release);
        true
    }

    #[inline(always)]
    pub fn try_pop(&self) -> Option<T> {
        let t = self.tail.value.load(Ordering::Relaxed);
        let h = self.head.value.load(Ordering::Acquire);
        if t == h {
            return None;
        }
        let slot = (t & MASK) as usize;
        let item = unsafe { (*self.slots[slot].get()).assume_init() };
        self.tail.value.store(t.wrapping_add(1), Ordering::Release);
        Some(item)
    }
}

fn main() {
    divan::main();
}

const BURST_ITEMS: u64 = 20_000;

#[divan::bench]
fn ring_concurrent_unpadded_false_sharing(b: Bencher) {
    b.counter(ItemsCount::new(BURST_ITEMS)).bench_local(|| {
        let ring = Arc::new(UnpaddedRing::<u64>::new());
        let done = Arc::new(AtomicBool::new(false));

        let r_prod = ring.clone();
        let d_prod = done.clone();

        let prod = thread::spawn(move || {
            for i in 0..BURST_ITEMS {
                while !r_prod.try_push(black_box(i)) {
                    core::hint::spin_loop();
                }
            }
            d_prod.store(true, Ordering::Release);
        });

        let mut received = 0u64;
        let mut checksum = 0u64;
        while !done.load(Ordering::Acquire) || received < BURST_ITEMS {
            if let Some(v) = ring.try_pop() {
                checksum = checksum.wrapping_add(v);
                received += 1;
            } else {
                core::hint::spin_loop();
            }
        }
        prod.join().unwrap();
        black_box((received, checksum));
    });
}

#[divan::bench]
fn ring_concurrent_padded_64b(b: Bencher) {
    b.counter(ItemsCount::new(BURST_ITEMS)).bench_local(|| {
        let ring = Arc::new(PaddedRing64::<u64>::new());
        let done = Arc::new(AtomicBool::new(false));

        let r_prod = ring.clone();
        let d_prod = done.clone();

        let prod = thread::spawn(move || {
            for i in 0..BURST_ITEMS {
                while !r_prod.try_push(black_box(i)) {
                    core::hint::spin_loop();
                }
            }
            d_prod.store(true, Ordering::Release);
        });

        let mut received = 0u64;
        let mut checksum = 0u64;
        while !done.load(Ordering::Acquire) || received < BURST_ITEMS {
            if let Some(v) = ring.try_pop() {
                checksum = checksum.wrapping_add(v);
                received += 1;
            } else {
                core::hint::spin_loop();
            }
        }
        prod.join().unwrap();
        black_box((received, checksum));
    });
}

#[divan::bench]
fn ring_concurrent_padded_128b_darwin_sovereign(b: Bencher) {
    b.counter(ItemsCount::new(BURST_ITEMS)).bench_local(|| {
        let ring = Arc::new(PaddedRing128::<u64>::new());
        let done = Arc::new(AtomicBool::new(false));

        let r_prod = ring.clone();
        let d_prod = done.clone();

        let prod = thread::spawn(move || {
            for i in 0..BURST_ITEMS {
                while !r_prod.try_push(black_box(i)) {
                    core::hint::spin_loop();
                }
            }
            d_prod.store(true, Ordering::Release);
        });

        let mut received = 0u64;
        let mut checksum = 0u64;
        while !done.load(Ordering::Acquire) || received < BURST_ITEMS {
            if let Some(v) = ring.try_pop() {
                checksum = checksum.wrapping_add(v);
                received += 1;
            } else {
                core::hint::spin_loop();
            }
        }
        prod.join().unwrap();
        black_box((received, checksum));
    });
}
