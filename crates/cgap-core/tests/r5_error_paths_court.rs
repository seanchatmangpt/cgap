//! R5 courts: (1) tracking allocator realloc with an unrepresentable
//! layout returns null instead of panicking inside the allocator;
//! (2) checkpoint restore decodes each field from its own offset.
//!
//! Mutations: M1 restore `.expect("realloc layout valid")` in
//! tracking_alloc.rs -> court 1 panics (flips). M2 swap the offsets of
//! `head`/`tail` in CheckpointBuf::restore -> court 2 flips.
//! ONE driver fn (allocator window state is process-global).

#[path = "../src/checkpoint.rs"]
mod checkpoint;
#[path = "../src/tracking_alloc.rs"]
mod tracking_alloc;

use std::alloc::{GlobalAlloc, Layout, System};

use checkpoint::{CheckpointBuf, CheckpointState};
use tracking_alloc::{last_breach, reset_breach, start_hot_path_budget};

#[global_allocator]
static R5_ALLOC: tracking_alloc::EnclaveTrackingAlloc<System> =
    tracking_alloc::EnclaveTrackingAlloc { inner: System };

#[test]
fn r5_driver() {
    // Court 1: zero-budget window, realloc to usize::MAX (layout invalid).
    reset_breach();
    let layout = Layout::from_size_align(8, 8).unwrap();
    let out = {
        let _g = start_hot_path_budget("r5-realloc", 0, 0);
        // SAFETY: the breach branch returns before touching `ptr`.
        unsafe { R5_ALLOC.realloc(core::ptr::NonNull::<u8>::dangling().as_ptr(), layout, usize::MAX) }
    };
    assert!(out.is_null(), "invalid realloc layout must yield null");
    let b = last_breach().expect("realloc breach must be recorded");
    assert_eq!(b.offending_bytes, usize::MAX);

    // Court 2: distinct head/tail/nonce/parent survive a restore.
    let st = CheckpointState {
        ring_head: 0x0102_0304_0506_0708,
        ring_tail: 0x1112_1314_1516_1718,
        receipt_nonce: 0x2122_2324_2526_2728,
        receipt_parent: [0x77; 32],
    };
    let dg = |i: &[u8]| cgap_blake3::hash(i);
    let mut buf = CheckpointBuf::new();
    buf.snapshot(&st, dg);
    let got = buf.restore(dg).expect("sealed region restores");
    assert_eq!(got.ring_head, st.ring_head);
    assert_eq!(got.ring_tail, st.ring_tail);
    assert_eq!(got.receipt_nonce, st.receipt_nonce);
    assert_eq!(got.receipt_parent, st.receipt_parent);
}
