//! Zero-alloc court: counting global allocator proves 0 allocations across
//! keygen / encaps / decaps INCLUDING corrupted-ct decaps (FO rejection
//! path). Non-vacuity: the court also demonstrates the counter counts (a
//! local Vec allocates), so a counter that never fires fails the court.
use cgap_crypto::kem;
use std::alloc::{GlobalAlloc, Layout, System};

use std::sync::atomic::{AtomicUsize, Ordering};

static ALLOCS: AtomicUsize = AtomicUsize::new(0);

struct Counting;
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::SeqCst);
        System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        System.dealloc(p, l)
    }
}

#[global_allocator]
static A: Counting = Counting;

#[test]
fn zero_alloc_kem_ops() {
    let d = [7u8; 32];
    let z = [9u8; 32];
    // non-vacuity witness: an actual allocation must move the counter
    let probe = Vec::<u8>::with_capacity(64);
    assert!(probe.capacity() >= 64);
    let before_probe = ALLOCS.load(Ordering::SeqCst);
    assert!(before_probe > 0, "counter never fired (vacuous court)");
    drop(probe);

    ALLOCS.store(0, Ordering::SeqCst);
    let mut ek = [0u8; 1184];
    let mut dk = [0u8; 2400];
    kem::keygen(&d, &z, &mut ek, &mut dk);
    let m = [3u8; 32];
    let mut ct = [0u8; 1088];
    let mut ss = [0u8; 32];
    kem::encaps(&ek, &m, &mut ct, &mut ss);
    let _ss2 = kem::decaps(&dk, &ct);
    // corrupted-ct decaps (rejection path) must also be alloc-free
    let mut bad = ct;
    bad[13] ^= 0x40;
    let _rej = kem::decaps(&dk, &bad);
    let after = ALLOCS.load(Ordering::SeqCst);
    assert_eq!(after, 0, "KEM ops allocated {} times", after);
}
