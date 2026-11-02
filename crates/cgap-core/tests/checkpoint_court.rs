//! PRD-6 checkpoint courts - zero-alloc stage snapshot/restore.
//!
//! Chicago: real collaborators (real cgap-blake3, real counting
//! allocator, real region bytes), assert final state. ONE driver test
//! fn (allocator window state is process-global; lane-posh-04-sentry
//! sequencing law).
//!
//! Anti-vacuity mutations named per court; a court that cannot flip is
//! FAIL. Positive control runs FIRST (court 0).

// Landed #[path]-include convention (E0152 divergence note,
// tracking_alloc.rs:14-18; pattern: crates/cgap/tests/alloc_boundary.rs).
#[path = "../src/checkpoint.rs"]
mod checkpoint;

#[path = "../src/tracking_alloc.rs"]
mod tracking_alloc;

use std::alloc::System;

use checkpoint::{CheckpointBuf, CheckpointMismatch, CheckpointState, CHECKPOINT_REGION_LEN};
use tracking_alloc::{last_breach, reset_breach, start_hot_path_budget, window_counts};

#[global_allocator]
static AE7_ALLOC: tracking_alloc::EnclaveTrackingAlloc<System> =
    tracking_alloc::EnclaveTrackingAlloc { inner: System };

/// Real BLAKE3 (dev-dep section 3 named exception, never in lib target).
/// cgap_blake3::hash(&[u8]) -> [u8; 32] (cgap-blake3/src/lib.rs:197).
fn real_blake3(input: &[u8]) -> [u8; 32] {
    cgap_blake3::hash(input)
}

/// Fresh non-trivial state (frozen seeds per repo seed law).
fn court_state() -> CheckpointState {
    CheckpointState {
        ring_head: 0x1CE4_E5B9_DEAD_C0DE,
        ring_tail: 0x476D_1CE4_0000_00FF,
        receipt_nonce: 0xA20C_0FFE_E5A7_E001,
        receipt_parent: {
            let mut p = [0u8; 32];
            for (i, b) in p.iter_mut().enumerate() {
                *b = (i as u8) * 7 ^ 0x5A;
            }
            p
        },
    }
}

fn main_driver() {
    // ---- COURT 0: positive control FIRST (anti-vacuity of the window).
    // A deliberate 512-B alloc inside a zero-budget window MUST be
    // observed as a sticky breach; a window that cannot flip is VACUOUS.
    reset_breach();
    {
        let _g = start_hot_path_budget("prd6-positive-control", 0, 0);
        let v: Vec<u8> = Vec::with_capacity(512);
        assert!(!v.is_empty() || v.capacity() == 512);
    } // guard drop = disarm; counts sticky until next arm.
    let breach = last_breach().expect("COURT0 VACUOUS: zero-budget window caught no allocation");
    assert_eq!(breach.site, "prd6-positive-control");
    assert!(
        breach.offending_bytes >= 512,
        "offending={}",
        breach.offending_bytes
    );
    reset_breach();

    // ---- COURT 4: zero-alloc across snapshot+restore (budget usize::MAX
    // so the window observes counts without breaching; counts sticky).
    let state = court_state();
    let mut region = CheckpointBuf::new();
    let _g = start_hot_path_budget("prd6-snapshot-restore", usize::MAX, usize::MAX);
    // window_counts() read AFTER arming: arm resets counters to 0.
    let (a0, b0) = window_counts();
    region.snapshot(&state, real_blake3);
    let restored = region
        .restore(real_blake3)
        .expect("roundtrip restore must succeed");
    let (a1, b1) = window_counts();
    drop(_g);
    let (da, db) = (a1 - a0, b1 - b0);
    assert_eq!(da, 0, "COURT4: snapshot+restore allocated {} times", da);
    assert_eq!(db, 0, "COURT4: snapshot+restore allocated {} bytes", db);
    assert_eq!(
        restored, state,
        "COURT4 roundtrip state must be field-equal"
    );

    // ---- COURT 1: roundtrip byte-equal serialization.
    let mut r1 = CheckpointBuf::new();
    r1.snapshot(&state, real_blake3);
    let mut r2 = CheckpointBuf::new();
    r2.snapshot(&state, real_blake3);
    assert_eq!(
        r1.bytes(),
        r2.bytes(),
        "COURT1: snapshot serialization not byte-deterministic"
    );
    let rt = r1.restore(real_blake3).expect("COURT1 restore");
    assert_eq!(rt, state, "COURT1 roundtrip field-equal");

    // ---- COURT 2 (ANTI-VACUITY FLIP): corrupt one region byte ->
    // typed CheckpointMismatch. Flip at each of two zones: state zone
    // (digest no longer matches) and digest trailer zone.
    let mut rc = r1;
    rc.bytes_mut()[7] ^= 0x01;
    assert_eq!(
        rc.restore(real_blake3),
        Err(CheckpointMismatch),
        "COURT2: state-zone corruption did not flip restore"
    );
    let mut rd = r1;
    rd.bytes_mut()[87] ^= 0x80;
    assert_eq!(
        rd.restore(real_blake3),
        Err(CheckpointMismatch),
        "COURT2: digest-trailer corruption did not flip restore"
    );
    // Mutation on the SOURCE state also changes the region.
    let mut s2 = state;
    s2.receipt_nonce = s2.receipt_nonce.wrapping_add(1);
    let mut r3 = CheckpointBuf::new();
    r3.snapshot(&s2, real_blake3);
    assert_ne!(
        r3.bytes(),
        r1.bytes(),
        "COURT2: state mutation must change region"
    );

    // ---- COURT 3: digest determinism / sensitivity.
    // Identical state -> identical region (covered byte-equal above);
    // state change -> different digest trailer.
    let d1 = &r1.bytes()[56..88];
    let d3 = &r3.bytes()[56..88];
    assert_ne!(d1, d3, "COURT3: digest must differ on state change");
    let mut r4 = CheckpointBuf::new();
    r4.snapshot(&state, real_blake3);
    assert_eq!(
        r4.bytes()[56..88],
        r1.bytes()[56..88],
        "COURT3: digest not deterministic across snapshots of identical state"
    );

    // ---- Layout law: region is exactly 88 bytes.
    assert_eq!(CHECKPOINT_REGION_LEN, 88);
}

#[test]
fn prd6_checkpoint_court_driver() {
    main_driver();
}
