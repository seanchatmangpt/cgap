//! FIX-04 andon liveness court (red-14 hardening).
//!
//! Sustained class-101 refusal floods escalate to a typed, reportable,
//! recoverable `RefusalQuota` state at the ring boundary — WITHOUT ever
//! gating admission (a garbage flood cannot wedge the ring), and with NO
//! process-exit behavior anywhere (grep-pinned in the receipt). Anti-
//! vacuity: the escalation courts name their mutation (DELETE the
//! `note_refusal()` escalation) and were executed in a /tmp sandbox with
//! sha256 fail-closed restore + mtime bump; removing the quota seam flips
//! each court to FAIL.
//!
//! Self-contained `#[path]` wire-in (lib.rs off-limits this lane); the
//! modules use no `crate::` paths, so this is byte-faithful to lib.rs.

#![cfg(test)]

extern crate alloc;

#[path = "../src/egress_ring.rs"]
mod egress_ring;
#[path = "../src/ingress_ring.rs"]
mod ingress_ring;

use egress_ring::EgressRing;
use ingress_ring::{IngressRing, RefusalQuota};

/// Flood 1,000,000 oversized frames at a small ring (red-14 harness shape,
/// scaled capacity): every push is a class-101 `FrameTooLarge` refusal,
/// ring state untouched, and the quota escalates. Then a legitimate frame
/// is ADMITTED (liveness), drained byte-exact (post-flood drain works),
/// and the escalation is LEVEL-TRIGGERED (ADV-12 repair): the window
/// resets but `tripped` STAYS SET through the single admitted frame.
#[test]
fn ingress_flood_liveness_and_quota_escalation() {
    let mut ring: IngressRing<1024> = IngressRing::new();
    let garbage = [0u8; 40_000];
    for _ in 0..1_000_000 {
        match ring.push(&garbage) {
            Err(e) => assert_eq!(e.andon_exit_class(), 101),
            Ok(()) => panic!("oversized frame admitted"),
        }
    }
    // Final state: zero bytes admitted by the flood; quota escalated.
    assert_eq!(ring.pending(), 0);
    let q = ring.refusal_quota();
    assert!(q.tripped(), "quota must escalate under 1M-refusal flood");
    assert_eq!(q.total_refusals, 1_000_000);
    assert_eq!(q.window_refusals, 1_000_000);

    // Post-flood liveness: a legitimate frame is still admitted.
    let legit = [0xABu8; 256];
    ring.push(&legit)
        .expect("admission after flood must succeed");
    assert_eq!(ring.pending(), 256);
    // ADV-12 repair, LEVEL-TRIGGERED leg: one admitted frame does NOT
    // erase the escalation — `tripped` stays observable after a single
    // admission; the WINDOW is what resets (recoverable seam).
    assert!(
        ring.refusal_quota().tripped(),
        "escalation is level-triggered: one admitted frame must NOT clear it"
    );
    assert_eq!(
        ring.refusal_quota().window_refusals,
        0,
        "admission resets the window"
    );
    // Host acknowledgement is the manual clear path.
    let mut quota = *ring.refusal_quota();
    quota.acknowledge();
    assert!(!quota.tripped(), "acknowledge clears the sticky escalation");

    // Post-flood drain: byte-exact FIFO drain of the admitted frame.
    let mut out = [0u8; 256];
    let n = ring.pop(&mut out).expect("drain after flood must succeed");
    assert_eq!(n, 256);
    assert!(out.iter().all(|&b| b == 0xAB));

    // Lifetime audit counter survives the recovery reset.
    assert_eq!(ring.refusal_quota().total_refusals, 1_000_000);
}

/// Reachability + recoverability witness for the ingress quota: refusals
/// accrue in the window, and a successful admission clears it (the
/// escalation path is observable, not dead code).
#[test]
fn ingress_quota_threshold_is_reachable_and_recoverable() {
    let mut ring: IngressRing<64> = IngressRing::new();
    // Fresh quota: not tripped, threshold is the default constant.
    assert_eq!(
        ring.refusal_quota().max_refusals,
        RefusalQuota::DEFAULT_MAX_REFUSALS
    );
    assert!(!ring.refusal_quota().tripped());

    // Two empty pops (the red-14 cheapest trip) escalate nothing at first,
    // then prove window/lifetime accounting.
    let _ = ring.pop(&mut []);
    let _ = ring.pop(&mut []);
    assert_eq!(ring.refusal_quota().window_refusals, 2);
    assert!(!ring.refusal_quota().tripped());

    // Push one frame sized between 64 and CAP to trip via FrameTooLarge
    // with a threshold small enough to observe — use the window count
    // directly (the 1M flood court above pins the trip itself).
    assert!(ring.push(&[0u8; 65]).is_err());
    assert_eq!(ring.refusal_quota().window_refusals, 3);
    assert_eq!(ring.refusal_quota().total_refusals, 3);

    // Successful admission clears the window (recoverable seam).
    ring.push(&[0x5Au8; 8]).expect("small frame admitted");
    let q = ring.refusal_quota();
    assert!(!q.tripped());
    assert_eq!(q.window_refusals, 0);
    assert_eq!(q.total_refusals, 3);
}

/// Egress twin (red-14 harness shape): 1,000,000 RecordTooLarge refusals
/// trip the egress quota; ring state untouched; a legitimate record is
/// then ADMITTED (liveness), the window clears (recoverable), and the
/// pending page still serves byte-exact.
#[test]
fn egress_flood_liveness_and_quota_escalation() {
    let mut ring: EgressRing<1024, 32> = EgressRing::new();
    let garbage = [0u8; 64];
    for _ in 0..1_000_000 {
        match ring.push_record(&garbage) {
            Err(e) => assert_eq!(e.andon_exit_class(), 101),
            Ok(()) => panic!("oversized record admitted"),
        }
    }
    // Final state: nothing admitted by the flood; quota escalated.
    assert_eq!(ring.pending_bytes(), 0);
    let q = ring.refusal_quota();
    assert!(q.tripped(), "egress quota must escalate under flood");
    assert_eq!(q.total_refusals, 1_000_000);

    // Post-flood liveness: legitimate records admitted, window cleared.
    let legit = [0x21u8; 32];
    for _ in 0..4 {
        ring.push_record(&legit).expect("admission after flood");
    }
    assert_eq!(ring.pending_rows(), 4);
    // ADV-12 level-triggered: four admitted frames (<< 1024) must NOT
    // clear the sticky escalation; only the window resets.
    assert!(
        ring.refusal_quota().tripped(),
        "egress escalation is level-triggered: a few admissions must NOT clear it"
    );
    assert_eq!(ring.refusal_quota().window_refusals, 0);
    assert_eq!(ring.refusal_quota().total_refusals, 1_000_000);

    // Post-flood serve: page renders byte-exact (0x21 x 128 payload
    // after the fixed 72-byte header).
    let mut page = [0u8; 256];
    let n = ring
        .serve_parquet_page(&mut page)
        .expect("serve after flood");
    assert_eq!(n, 72 + 128);
    assert_eq!(&page[0..4], b"PAR2");
    assert!(page[72..200].iter().all(|&b| b == 0x21));
}

/// ADV-12 repair, sustained-quiet-window leg (both rings' quota twins):
/// after a trip, `tripped` clears ONLY after exactly
/// `CLEAN_ADMISSIONS_TO_CLEAR` consecutive clean admissions — never on a
/// single frame, and a refusal in the middle restarts the streak.
#[test]
fn quota_escalation_clears_only_after_sustained_quiet_window() {
    // Ingress twin.
    let mut q = RefusalQuota::new();
    for _ in 0..q.max_refusals {
        q.note_refusal();
    }
    assert!(q.tripped(), "quota must trip before the quiet window");
    let n = RefusalQuota::CLEAN_ADMISSIONS_TO_CLEAR;
    for i in 0..n - 1 {
        q.note_admission();
        assert!(
            q.tripped(),
            "admission {} of {} must NOT clear the sticky escalation",
            i + 1,
            n - 1
        );
    }
    q.note_admission();
    assert!(
        !q.tripped(),
        "exactly {} clean admissions clear the escalation",
        n
    );

    // A refusal mid-streak restarts the hysteresis.
    for _ in 0..q.max_refusals {
        q.note_refusal();
    }
    assert!(q.tripped());
    for _ in 0..n / 2 {
        q.note_admission();
    }
    q.note_refusal();
    for _ in 0..n - 1 {
        q.note_admission();
    }
    assert!(
        q.tripped(),
        "a refusal mid-streak must restart the clean-admission window"
    );
    q.note_admission();
    assert!(!q.tripped());

    // Egress twin: same contract on the mirrored struct.
    let mut eq = egress_ring::RefusalQuota::new();
    for _ in 0..eq.max_refusals {
        eq.note_refusal();
    }
    assert!(eq.tripped());
    for _ in 0..egress_ring::RefusalQuota::CLEAN_ADMISSIONS_TO_CLEAR - 1 {
        eq.note_admission();
        assert!(eq.tripped());
    }
    eq.note_admission();
    assert!(!eq.tripped());
}

/// ADV-12 repair, saturation leg: `window_refusals` is `u32` and the
/// pre-repair `+=` panicked in debug builds at `u32::MAX`. The seam is
/// the public field: pin the counter one below saturation, refuse past
/// it — typed saturation, no panic, `tripped` latches, and the window
/// still resets on admission (window semantics preserved).
#[test]
fn window_refusals_saturate_without_overflow() {
    let mut q = RefusalQuota::new();
    q.window_refusals = u32::MAX - 1;
    q.note_refusal();
    assert_eq!(q.window_refusals, u32::MAX, "counter saturates at u32::MAX");
    // The mutation under test: one MORE refusal at saturation — the old
    // `+=` panicked here in debug builds.
    q.note_refusal();
    assert_eq!(q.window_refusals, u32::MAX, "saturation holds, no wrap");
    assert!(q.tripped());
    assert_eq!(q.total_refusals, 2, "lifetime counter still counts");
    // Window semantics preserved: admission resets the saturated window.
    q.note_admission();
    assert_eq!(q.window_refusals, 0);

    // Egress twin.
    let mut eq = egress_ring::RefusalQuota::new();
    eq.window_refusals = u32::MAX - 1;
    eq.note_refusal();
    eq.note_refusal();
    assert_eq!(eq.window_refusals, u32::MAX);
    assert!(eq.tripped());
}
