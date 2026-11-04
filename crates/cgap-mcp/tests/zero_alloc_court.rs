//! Zero-alloc window court: a CountingAlloc global allocator proves ZERO
//! heap allocations inside the cgap-mcp wire-encode window. Runs in its own
//! test binary (separate process) so the process-wide counter is uncontended.
//!
//! Anti-vacuity: the same court asserts the counter is NONZERO for an
//! allocating control (a String build) — a counter that never increments is
//! a vacuous gate.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

static ALLOCS: AtomicUsize = AtomicUsize::new(0);

struct CountingAlloc;

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::SeqCst);
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::SeqCst);
        unsafe { System.realloc(p, l, n) }
    }
}

#[global_allocator]
static A: CountingAlloc = CountingAlloc;

#[test]
fn zero_alloc_window_court() {
    // Warm any lazy init (crc table OnceLock) OUTSIDE the measured window.
    let warm = [7u8; 64];
    let _ = wire_shm::crc32_ieee(&warm);
    let mut buf = vec![0u8; 16 + 64];

    ALLOCS.store(0, Ordering::SeqCst);
    // --- measured window: cgap-mcp zero-alloc frame encode + crc verify ---
    let written = cgap_mcp::wire::encode_frame_into(&mut buf, &warm, 42)
        .expect("encode");
    let crc = u32::from_be_bytes(buf[4..8].try_into().unwrap());
    assert_eq!(crc, wire_shm::crc32_ieee(&warm));
    assert_eq!(written, 16 + 64);
    // ----------------------------------------------------------------------
    let in_window = ALLOCS.load(Ordering::SeqCst);
    assert_eq!(in_window, 0, "zero-alloc window: got {} allocs", in_window);

    // Non-vacuity flip: allocating control must move the counter.
    ALLOCS.store(0, Ordering::SeqCst);
    let control: Vec<u8> = (0..64u8).collect();
    let after = ALLOCS.load(Ordering::SeqCst);
    assert!(after > 0, "vacuous counter: allocating control read {}", after);
    assert_eq!(control.len(), 64);
}
