//! RC-14 quota alloc-purity court (red-09 + red-14 hardening).
//!
//! The posh_alloc_sentry_court proves steady-state ingress purity, but
//! nothing courts the FIX-04 `RefusalQuota` escalation state machine:
//! a quota escalation that allocated would be an invisible purity
//! regression. This court runs a red-14-shaped refusal-flood battery
//! (oversized frames + overflow pushes) through BOTH ring seams under an
//! open `CountingAlloc` window and asserts allocs==0 AND bytes==0
//! across the ENTIRE flood, INCLUDING the quota-escalation transition
//! itself and the post-flood recovery admission. The quota is pure
//! field math (`note_refusal`/`note_admission` touch only `u32`/`u64`
//! fields — ingress_ring.rs:134-148), so zero alloc across escalation
//! is the law pinned here; if a future quota path allocates by design
//! (e.g. error-string formatting), this court FAILS loudly rather than
//! suppressing the assert — the measured finding then lives in the
//! receipt, not a silenced gate.
//!
//! Anti-vacuity: the positive-control leg allocates once inside an
//! open window and asserts the sentry FLIPS (allocs>=1, bytes>=512).
//! A window that cannot flip is VACUOUS and the court refuses.
//!
//! Self-contained `#[path]` wire-in (lib.rs off-limits this lane); the
//! modules use no `crate::` paths, so this is byte-faithful to lib.rs.

#![cfg(test)]

extern crate alloc;

#[path = "../src/egress_ring.rs"]
mod egress_ring;
#[path = "../src/ingress_ring.rs"]
mod ingress_ring;

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering as AOrd};

use egress_ring::EgressRing;
use ingress_ring::IngressRing;

// ---------------------------------------------------------------------------
// CountingAlloc: window-gated alloc/byte ledger over System. Shape mirrors
// posh_alloc_sentry_court.rs:49-84 exactly (hand-written real allocator,
// not a mock).
// ---------------------------------------------------------------------------

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static ALLOC_BYTES: AtomicUsize = AtomicUsize::new(0);
static WINDOW_OPEN: AtomicBool = AtomicBool::new(false);

struct CountingAlloc;
unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if WINDOW_OPEN.load(AOrd::Relaxed) {
            ALLOC_COUNT.fetch_add(1, AOrd::Relaxed);
            ALLOC_BYTES.fetch_add(layout.size(), AOrd::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new: usize) -> *mut u8 {
        if WINDOW_OPEN.load(AOrd::Relaxed) {
            ALLOC_COUNT.fetch_add(1, AOrd::Relaxed);
            ALLOC_BYTES.fetch_add(new, AOrd::Relaxed);
        }
        unsafe { System.realloc(ptr, layout, new) }
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

fn open_window() {
    ALLOC_COUNT.store(0, AOrd::Relaxed);
    ALLOC_BYTES.store(0, AOrd::Relaxed);
    WINDOW_OPEN.store(true, AOrd::Relaxed);
}
fn close_window() -> (usize, usize) {
    WINDOW_OPEN.store(false, AOrd::Relaxed);
    (
        ALLOC_COUNT.load(AOrd::Relaxed),
        ALLOC_BYTES.load(AOrd::Relaxed),
    )
}

/// One test fn: the counting allocator state is process-global, so the
/// legs run in one ordered body (posh_alloc_sentry_court pattern).
#[test]
fn quota_flood_alloc_purity() {
    // ---- Leg 1: POSITIVE CONTROL FIRST (anti-vacuity) -------------------
    // A deliberate heap alloc inside an open window MUST be observed.
    // A window that cannot flip is a vacuous sentry: refuse.
    open_window();
    let control: Vec<u8> = vec![0x5A; 512];
    assert_eq!(control[0], 0x5A);
    let (c, b) = close_window();
    assert!(
        c >= 1 && b >= 512,
        "VACUOUS sentry: positive control (allocs={c}, bytes={b}) did not flip"
    );

    // ---- Leg 2: INGRESS flood battery under one open window -------------
    // Ring pinned FULL (16 x 64 B = 1024 B = CAP), then:
    //   3_000 oversized pushes  -> FrameTooLarge refusals
    //   2_000 overflow pushes   -> Overflow refusals
    // 5_000 refusals > 4_096 default quota => escalation transition
    // happens MID-FLOOD. Window stays open across flood + recovery.
    let mut ring: IngressRing<1024> = IngressRing::new();
    let mut fill = [0u8; 64];
    for i in 0..64 {
        fill[i] = (i as u8) ^ 0xA5;
    }
    for _ in 0..16 {
        ring.push(&fill)
            .expect("pre-flood fill must admit all 16 frames");
    }
    assert_eq!(ring.pending(), 1024, "ring must be pinned exactly full");
    assert!(
        !ring.refusal_quota().tripped(),
        "pre-flood quota must be clear"
    );

    let oversized = [0xFFu8; 40_000];
    let mut ftl = 0usize;
    let mut ovf = 0usize;
    open_window();
    for _ in 0..3_000 {
        match ring.push(&oversized) {
            Err(e) => {
                assert_eq!(e.andon_exit_class(), 101);
                ftl += 1;
            }
            Ok(()) => panic!("oversized frame admitted mid-flood"),
        }
    }
    for _ in 0..2_000 {
        match ring.push(&fill) {
            Err(ingress_ring::IngressError::Overflow { .. }) => ovf += 1,
            Err(e) => panic!("unexpected refusal class: {e:?}"),
            Ok(()) => panic!("overflow push admitted mid-flood"),
        }
    }
    assert_eq!(ftl, 3_000);
    assert_eq!(ovf, 2_000);

    // Quota escalation transition (inside the window, mid-flood): the
    // quota MUST have tripped at the 4,096th refusal.
    let q = ring.refusal_quota();
    assert!(q.tripped(), "5_000 refusals must trip the 4_096 quota");
    assert_eq!(q.total_refusals, 5_000);
    assert_eq!(q.window_refusals, 5_000);
    assert_eq!(
        ring.pending(),
        1024,
        "flood must leave ring state untouched"
    );

    // ---- Leg 3: POST-FLOOD RECOVERY ADMIT (still inside the window) -----
    // Drain one frame (frees 64 B), admit a legit frame, quota resets.
    let mut out = [0u8; 64];
    let n = ring.pop(&mut out).expect("post-flood pop must succeed");
    assert_eq!(n, 64);
    for i in 0..64 {
        assert_eq!(out[i], (i as u8) ^ 0xA5, "drain must be byte-exact");
    }
    let legit = [0xABu8; 64];
    ring.push(&legit)
        .expect("post-flood recovery admit must succeed");
    // ADV-12 repair (lane R-ADV12FIX minimal unblock, disclosed in both
    // receipts): escalation is LEVEL-TRIGGERED now — one admitted frame
    // must NOT clear the sticky `tripped`; only the window resets.
    assert!(
        ring.refusal_quota().tripped(),
        "level-triggered: one admitted frame must NOT clear the escalation"
    );
    assert_eq!(ring.refusal_quota().window_refusals, 0);
    assert_eq!(
        ring.refusal_quota().total_refusals,
        5_000,
        "lifetime audit counter survives recovery"
    );
    assert_eq!(
        ring.pending(),
        1024,
        "recovery admit refills the freed slot"
    );

    // ---- Leg 4: EGRESS flood battery (same open window) -----------------
    // EgressRing<512, 64>: fill all 8 record slots, then refuse-flood with
    // wrong-stride pushes (RecordTooLarge class) + overflow pushes.
    let mut ering: EgressRing<512, 64> = EgressRing::new();
    let mut rec = [0u8; 64];
    for i in 0..64 {
        rec[i] = (i as u8) ^ 0x3C;
    }
    for _ in 0..8 {
        ering
            .push_record(&rec)
            .expect("pre-flood egress fill must admit all 8 records");
    }
    assert_eq!(
        ering.pending_bytes(),
        512,
        "egress ring pinned exactly full"
    );
    assert!(
        !ering.refusal_quota().tripped(),
        "pre-flood egress quota clear"
    );

    let big_rec = [0xEEu8; 128];
    let mut e_ftl = 0usize;
    let mut e_ovf = 0usize;
    for _ in 0..3_000 {
        match ering.push_record(&big_rec) {
            Err(_) => e_ftl += 1,
            Ok(()) => panic!("oversized egress record admitted mid-flood"),
        }
    }
    for _ in 0..2_000 {
        match ering.push_record(&rec) {
            Err(_) => e_ovf += 1,
            Ok(()) => panic!("overflow egress push admitted mid-flood"),
        }
    }
    assert_eq!(e_ftl, 3_000);
    assert_eq!(e_ovf, 2_000);
    let eq = ering.refusal_quota();
    assert!(eq.tripped(), "egress quota must trip under 5_000 refusals");
    assert_eq!(eq.total_refusals, 5_000);
    assert_eq!(
        ering.pending_bytes(),
        512,
        "egress flood must leave state untouched"
    );

    // Egress post-flood recovery: advance tail one record, admit again.
    ering
        .advance_tail(1)
        .expect("post-flood tail advance must succeed");
    ering
        .push_record(&rec)
        .expect("egress recovery admit must succeed");
    assert!(
        ering.refusal_quota().tripped(),
        "level-triggered: egress admission must NOT clear sticky escalation"
    );
    assert_eq!(
        ering.pending_bytes(),
        512,
        "egress recovery keeps ring full"
    );

    // ---- Leg 5: CLOSE WINDOW — the whole-battery purity assert ----------
    // allocs==0 AND bytes==0 across BOTH floods, BOTH quota-escalation
    // transitions, and BOTH recovery admissions.
    let (allocs, bytes) = close_window();
    assert_eq!(
        (allocs, bytes),
        (0, 0),
        "PURITY BREACH: flood+escalation+recovery allocated \
         allocs={allocs} bytes={bytes} — surface the measured finding, do not suppress"
    );

    // ---- Leg 6: FINAL RING STATE BYTE-EXACT -----------------------------
    // Drain the ingress ring FIFO and verify every surviving frame's
    // content byte-exactly (flood must not have corrupted payload).
    let mut total = 0usize;
    while ring.pending() > 0 {
        let mut buf = [0u8; 256];
        let n = ring
            .pop(&mut buf)
            .expect("post-flood drain pop must succeed");
        for k in 0..n {
            let a = total + k; // absolute byte offset in FIFO stream
            if a < 960 {
                assert_eq!(
                    buf[k],
                    ((a % 64) as u8) ^ 0xA5,
                    "fill frames must be byte-exact at offset {a}"
                );
            } else {
                assert_eq!(buf[k], 0xAB, "recovery frame must be intact");
            }
        }
        total += n;
    }
    assert_eq!(total, 1024);
}
