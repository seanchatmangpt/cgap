//! R5 court: typed refusals from `BitPlaneSeenSet::try_new_boxed`.
//! Anti-vacuity mutation M1: drop the null check
//! (`Ok(Box::from_raw(null))`) -> allocator_exhaustion test flips. Positive control: test 3.

use cgap_storage::bitplane_seen::{BitPlaneSeenSet, SeenAllocError};

#[test]
fn zero_slots_is_not_zst_and_refuses_inserts_typed() {
    // Finding: the old `assert!(size_of > 0)` was dead (`len` field).
    let mut s = BitPlaneSeenSet::<0>::try_new_boxed().expect("SLOTS=0 allocates the len field");
    assert!(s.is_full());
    assert_eq!(s.insert(1, 2), Err(cgap_storage::bitplane_seen::SeenFull));
}

#[test]
fn allocator_exhaustion_is_typed_refusal() {
    // 2^40 slots * 128 planes * 8 B = 2^50 B (1 PiB) > any address space;
    // size < isize::MAX so the Layout is valid and calloc itself fails.
    let r = BitPlaneSeenSet::<{ 1usize << 40 }>::try_new_boxed();
    assert_eq!(r.err(), Some(SeenAllocError::OutOfMemory));
}

#[test]
fn positive_control_small_set_constructs_and_works() {
    let mut s = BitPlaneSeenSet::<4>::try_new_boxed().expect("4 slots must allocate");
    assert_eq!(s.insert(1, 2), Ok(true));
    assert!(s.contains(1, 2));
    assert_eq!(s.len(), 1);
}
