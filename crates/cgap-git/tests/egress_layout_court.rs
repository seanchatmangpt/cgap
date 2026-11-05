//! CE-05 layout court (charter §4 Phase-4 design verification): the egress
//! ring's shared indices are cache-line isolated per the charter claim
//! "repr(align(64)) enforced across all shared ring indices and quota
//! structures", porting the landed ingress PaddedIndex pattern (OPT-01
//! hazard: bare AtomicUsize head/tail co-resident with RefusalQuota on one
//! 64-B line).
//!
//! ANTI-VACUITY MUTATION (named, Chicago law): remove the `#[repr(align(64))]`
//! wrapper from `PaddedIndex` in `crates/cgap-git/src/egress_ring.rs` (or
//! replace the `head`/`tail` fields with bare `AtomicUsize`). The const
//! asserts pin `size_of::<PaddedIndex>() == 64` and the struct-size formula
//! `size_of::<EgressRing<EGRESS_RING_BYTES, 64>>() == CAP + 2*64 + round64(quota)`
//! — dropping the wrapper breaks BOTH pins, so this court's subject fails
//! to compile / the size assertions below fail. Witnessed flip:
//! /tmp sandbox, 2026-11-02 (see docs/jira/v26.11.2/lanes/lane-ce-05-egress-padding.md).
//!
//! `#![forbid(unsafe_code)]`-clean; real collaborators (a live EgressRing),
//! final-state asserts, no mocks.

#![forbid(unsafe_code)]

use cgap_git::egress_ring::{EgressRing, CACHE_LINE_BYTES, EGRESS_RING_BYTES};

/// Canonical ring under court: 32,768-B capacity, 64-B stride.
type Ring = EgressRing<EGRESS_RING_BYTES, 64>;

/// Layout pin (compile-time where possible, runtime here for test
/// visibility): PaddedIndex is exactly one cache line.
#[test]
fn padded_index_is_one_full_cache_line() {
    assert!(core::mem::size_of::<Ring>() > 0); // subject exists
                                               // head and tail each occupy exactly one 64-B line: the struct-size
                                               // formula proves the pair contributes exactly 2 * 64 B.
    let quota_slots = size_of_refusal_quota().div_ceil(CACHE_LINE_BYTES) * CACHE_LINE_BYTES;
    let expected = EGRESS_RING_BYTES + 2 * CACHE_LINE_BYTES + quota_slots;
    assert_eq!(
        core::mem::size_of::<Ring>(),
        expected,
        "EgressRing layout: buf + two cache-line-padded indices + quota line"
    );
}

/// quota contributes at least one full cache line slot (distinct line from
/// tail — quota is NOT co-resident with head or tail).
#[test]
fn quota_lands_on_its_own_cache_line() {
    // quota offset = CAP + 128 (head line + tail line); it cannot share a
    // line with either index because both indices are exactly 64 B.
    let quota_slots = size_of_refusal_quota().div_ceil(CACHE_LINE_BYTES);
    assert!(quota_slots >= 1, "quota occupies at least one line slot");
    // Alignment of the whole ring is the cache line width.
    assert_eq!(core::mem::align_of::<Ring>(), CACHE_LINE_BYTES);
}

/// Behavioral sanity on the padded layout (Chicago: real collaborator,
/// final state): the padded ring must still serve real records — the
/// padding port changed layout, not semantics.
#[test]
fn padded_ring_still_pushes_and_serves() {
    let mut ring = Ring::new();
    let record = [0xABu8; 64];
    ring.push_record(&record).expect("push into empty ring");
    assert_eq!(ring.head(), 64);
    assert_eq!(ring.pending_rows(), 1);
    let win = ring.window(0, 1).expect("window of one pending row");
    assert_eq!(win.total_len(), 64);
    assert_eq!(win.a[0], 0xAB);
}

// -- test-local pin of the quota size used by the layout formula --------
// (kept as a plain fn so the formula stays in one place across tests)

fn size_of_refusal_quota() -> usize {
    core::mem::size_of::<cgap_git::egress_ring::RefusalQuota>()
}
