//! Chicago courts for cgap-fleet-ops (A3/A6).
//!
//! Real collaborators (the register + a real in-process stats source),
//! assert final state, no mocks. Each court names its anti-vacuity
//! mutation; the sandbox mutation battery is run from the shell against
//! a /tmp copy (see receipt) so the shared tree is never mutated.

use cgap_fleet_ops::slo::SNAPSHOT_LEN;
use cgap_fleet_ops::*;

fn nid(b: u8) -> NodeId {
    let mut a = [0u8; 16];
    a[0] = b;
    NodeId(a)
}

fn degenerate() -> ReceiptDigest {
    ReceiptDigest::from_seed(0xA20C_0FFE_E5A7_E000)
}

/// COURT 1 (A3 legal lifecycle): commission -> promote -> degrade ->
/// RMA retire. Final state Retired asserted on the real register; the
/// hash chain advances per event; ingress eligibility drops 2 -> 1.
#[test]
fn court1_legal_lifecycle_rma_receipted() {
    let mut fleet = Fleet::new();
    fleet.commission(nid(1)).unwrap();
    fleet.commission(nid(2)).unwrap();
    assert_eq!(fleet.ingress_eligible(), 2);

    fleet
        .apply(
            &nid(1),
            Transition::Promote {
                identity_digest: degenerate(),
            },
        )
        .unwrap();
    fleet
        .apply(
            &nid(1),
            Transition::Degrade {
                fault_code: 0x4347_0004,
            },
        )
        .unwrap();
    let final_state = fleet
        .apply(
            &nid(1),
            Transition::RmaRetire {
                rma_receipt: degenerate(),
            },
        )
        .unwrap();

    assert_eq!(final_state, NodeState::Retired);
    assert_eq!(fleet.node(&nid(1)).unwrap().state(), NodeState::Retired);
    assert_eq!(
        fleet.ingress_eligible(),
        1,
        "retired node must leave ingress set"
    );
    assert_eq!(
        fleet.node(&nid(1)).unwrap().transition_log().len(),
        3,
        "3 receipted events"
    );
    // Chain tips strictly advance (hash-chained, not overwritten).
    let log = fleet.node(&nid(1)).unwrap().transition_log();
    assert!(log[0].2 != log[1].2 && log[1].2 != log[2].2);
}

/// COURT 2 (A3 illegal transitions): typed refusals, state unchanged.
/// ANTI-VACUITY MUTATION: removing the evidence check (the
/// `matches!(t, Transition::ForceRetire)` gate or the zero-digest
/// check) in a /tmp sandbox copy must flip this court to failure.
#[test]
fn court2_illegal_transitions_refused() {
    let mut fleet = Fleet::new();
    fleet.commission(nid(3)).unwrap();

    // Retire from Commissioned: wrong-state refusal, no state change.
    let r = fleet.apply(&nid(3), Transition::RmaRetire { rma_receipt: degenerate() });
    assert_eq!(r, Err(FleetRefusal::RetireFromWrongState(NodeState::Commissioned)));
    assert_eq!(fleet.node(&nid(3)).unwrap().state(), NodeState::Commissioned);

    // Promote past the legal path, then attempt bare-flag retire.
    fleet.apply(&nid(3), Transition::Promote { identity_digest: degenerate() }).unwrap();
    fleet.apply(&nid(3), Transition::Degrade { fault_code: 1 }).unwrap();

    // RMA with a zeroed digest = bare flag, not evidence. Refused.
    let r = fleet.apply(&nid(3), Transition::RmaRetire { rma_receipt: ReceiptDigest([0u8; 32]) });
    assert_eq!(r, Err(FleetRefusal::MissingRmaEvidence));
    assert_eq!(fleet.node(&nid(3)).unwrap().state(), NodeState::Degraded);

    // ForceRetire carries no evidence field by construction. Refused,
    // node stays Degraded.
    let r = fleet.apply(&nid(3), Transition::ForceRetire);
    assert_eq!(r, Err(FleetRefusal::MissingRmaEvidence));
    assert_eq!(fleet.node(&nid(3)).unwrap().state(), NodeState::Degraded);

    // Legal retire now; post-retirement transitions are refused.
    fleet.apply(&nid(3), Transition::RmaRetire { rma_receipt: degenerate() }).unwrap();
    let r = fleet.apply(&nid(3), Transition::Recover);
    assert_eq!(r, Err(FleetRefusal::AlreadyRetired));
    assert_eq!(fleet.node(&nid(3)).unwrap().state(), NodeState::Retired);

    // Unknown node refusal.
    let r = fleet.apply(&nid(9), Transition::Recover);
    assert_eq!(r, Err(FleetRefusal::UnknownNode));
}

/// COURT 3 (A6 SLO wire round-trip): snapshot encodes/decodes
/// byte-identical; any corrupted payload byte is a typed rejection
/// (checksum), never a silent zero.
#[test]
fn court3_slo_snapshot_round_trip() {
    let snap = SloSnapshot {
        node: nid(7),
        commit_latency_class_max: 3,
        lease_deny_count: 42,
        lease_exhausted: 7,
        concurrent_writer_denied: 9,
        p28_spoliation: 1,
    };
    let bytes = snap.to_bytes();
    assert_eq!(SloSnapshot::from_bytes(&bytes), Some(snap));

    // Flip one payload byte: checksum must reject.
    let mut bad = bytes;
    bad[20] ^= 0xff;
    assert_eq!(SloSnapshot::from_bytes(&bad), None, "corrupt wire payload must not decode");

    // Flip one checksum byte: typed rejection too.
    let mut badc = bytes;
    badc[SNAPSHOT_LEN - 1] ^= 0x01;
    assert_eq!(SloSnapshot::from_bytes(&badc), None);
}

/// COURT 4 (A6 alert predicates): strictly-greater crossing, off-by-one
/// at ceiling stays quiet, stalled source degrades to typed UNHEALTHY.
/// ANTI-VACUITY MUTATION: inverting or off-by-one-ing a threshold in a
/// /tmp sandbox copy must flip this court.
#[test]
fn court4_alert_predicate_and_bounded_poll() {
    let pred = AlertPredicate { lease_deny_ceiling: 10, status_code_ceiling: 10 };

    let quiet = SloSnapshot {
        node: nid(5),
        commit_latency_class_max: 1,
        lease_deny_count: 10, // at ceiling: quiet
        lease_exhausted: 10,
        concurrent_writer_denied: 0,
        p28_spoliation: 0,
    };
    assert_eq!(pred.evaluate(&quiet), None);

    let stormy = SloSnapshot {
        node: nid(5),
        commit_latency_class_max: 1,
        lease_deny_count: 11, // strictly over
        lease_exhausted: 0,
        concurrent_writer_denied: 0,
        p28_spoliation: 0,
    };
    assert_eq!(pred.evaluate(&stormy), Some(Alert::LeaseDenyStorm(nid(5))));

    // Status-code crossing names the exact pinned code.
    let spol = SloSnapshot {
        node: nid(6),
        commit_latency_class_max: 1,
        lease_deny_count: 0,
        lease_exhausted: 0,
        concurrent_writer_denied: 0,
        p28_spoliation: 11,
    };
    assert_eq!(
        pred.evaluate(&spol),
        Some(Alert::StatusStorm(nid(6), slo::status_codes::P28_SPOLIATION))
    );

    // Bounded poll against a real in-process source: no alert, no
    // stall, loop terminates within budget.
    struct LiveSource(SloSnapshot);
    impl SloSource for LiveSource {
        fn snapshot(&self) -> SloSnapshot { self.0 }
    }
    let (snap, alert) = poll_until(&LiveSource(quiet), &pred, 16);
    assert_eq!(snap, quiet);
    assert_eq!(alert, None);

    // Stalled source => typed SourceUnhealthy, not a wait.
    struct StalledSource(SloSnapshot, bool);
    impl SloSource for StalledSource {
        fn snapshot(&self) -> SloSnapshot { self.0 }
        fn is_stalled(&self) -> bool { self.1 }
    }
    let (_, alert) = poll_until(&StalledSource(quiet, true), &pred, 16);
    assert_eq!(alert, Some(Alert::SourceUnhealthy(nid(5))));
}
