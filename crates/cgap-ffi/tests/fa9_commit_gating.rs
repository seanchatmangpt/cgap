//! FA9 commit-gating court (w6-fa9-commit-gating).
//!
//! Real threads, real files, final-state asserts; no mocks, no sleeps.
//! Anti-vacuity: a mutant that removes the spill-gate lets the second
//! thread GRANT (File::create O_TRUNC) on an owned path — the court's
//! DENIED observation and prefix-preservation leg flip. Witnessed in a
//! /tmp sandbox build; the repo tree is never mutated.
//!
//! Court legs:
//!   1. register pin: the denial code is exactly 0x4347_0006.
//!   2. race grid: owner thread commits continuously on path P; a racing
//!      thread attempts the SAME path — the racer observes only
//!      {OK, DENIED}, DENIED == 0x4347_0006 exactly, the owner is all-OK,
//!      and the owner's drained spill bytes are prefix-preserved
//!      (append-only) across the whole grid.
//!   3. release leg: after the owner thread EXITS (lease lifetime ==
//!      thread lifetime), the path is claimable again — the gate is not a
//!      permanently-stuck deny.
//!
//! Divergence from dispatch (recorded): the gate is a per-(path, thread)
//! ownership registry with lease-lifetime release, not a per-call global
//! AtomicBool — a per-call flag cannot deliver leg 2's preservation
//! invariant (the racer can win the inter-call gap and legally truncate
//! at grant). Reasoning in the receipt.

use std::ffi::CString;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use cgap_ffi::sovereign::{cgap_commit_parquet, CGAP_ERR_CONCURRENT_WRITER_DENIED, CGAP_OK};

const DENIED: u32 = CGAP_ERR_CONCURRENT_WRITER_DENIED;

fn sandbox(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("fa9-court-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("sandbox dir");
    dir
}

fn cpath(p: &PathBuf) -> CString {
    CString::new(p.as_os_str().as_encoded_bytes()).expect("NUL-free path")
}

/// One REAL commit through the sovereign surface (small in-lease frame).
fn commit(path: &CString, n: usize) -> u32 {
    let lanes: Vec<i32> = (0..n).map(|i| (i % 251) as i32).collect();
    // SAFETY: valid lane pointer + NUL-terminated path, live for the call.
    unsafe { cgap_commit_parquet(lanes.as_ptr(), lanes.len(), path.as_ptr()) }
}

/// Bounded spin on a real observable (no sleep): deadline by iterations.
fn spin_until(flag: &AtomicBool) {
    for _ in 0..50_000_000 {
        if flag.load(Ordering::SeqCst) {
            return;
        }
    }
    panic!("bounded spin deadline expired — owner never signalled ready");
}

#[test]
fn fa9_gate_register_pin() {
    assert_eq!(DENIED, 0x4347_0006, "exact register pin ('CG' + 06)");
    assert_eq!(DENIED & 0xFFFF_0000, 0x4347_0000, "'CG' class");
    assert_ne!(DENIED & 0x0000_FFFF, 0, "'CG' sequence nonzero");
    assert_ne!(DENIED, CGAP_OK, "denial is not OK");
}

#[test]
fn fa9_race_grid_one_writer_no_truncation() {
    let dir = sandbox("race");
    let spill = dir.join("race.par2");
    let _cp = cpath(&spill);

    // The OWNER thread performs the grant and rotates at least once
    // BEFORE the racer starts (ownership is per (path, thread), so the
    // grant must happen on the thread that will hold the lease).
    let ready = Arc::new(AtomicBool::new(false));
    let (owner_ok, owner_bad) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
    let (racer_ok, racer_deny, racer_bad) = (
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
    );
    let stop = Arc::new(AtomicBool::new(false));

    let owner = {
        let (ready, owner_ok, owner_bad, stop) = (
            Arc::clone(&ready),
            Arc::clone(&owner_ok),
            Arc::clone(&owner_bad),
            Arc::clone(&stop),
        );
        let spill_for_owner = spill.clone();
        let cp2 = cpath(&spill_for_owner);
        std::thread::spawn(move || {
            // Phase 1 (owner-local): grant + rotate at least once.
            let mut rotated = false;
            for _ in 0..4096 {
                match commit(&cp2, 64) {
                    CGAP_OK => {
                        owner_ok.fetch_add(1, Ordering::Relaxed);
                    }
                    _ => {
                        owner_bad.fetch_add(1, Ordering::Relaxed);
                        return;
                    }
                }
                if !rotated && !std::fs::read(&spill_for_owner).unwrap_or_default().is_empty() {
                    rotated = true;
                    ready.store(true, Ordering::SeqCst);
                }
            }
            if !rotated {
                owner_bad.fetch_add(1, Ordering::Relaxed);
                return;
            }
            // Phase 2: back-to-back commits until the stop flag.
            while !stop.load(Ordering::Relaxed) {
                match commit(&cp2, 64) {
                    CGAP_OK => {
                        owner_ok.fetch_add(1, Ordering::Relaxed);
                    }
                    _ => {
                        owner_bad.fetch_add(1, Ordering::Relaxed);
                        break;
                    }
                }
            }
        })
    };

    // Racer waits for the observable ready signal (bounded spin, no sleep),
    // then tight-loops REAL commit attempts at the SAME path.
    spin_until(&ready);
    let before = fs::read(&spill).expect("spill readable at race start");
    let racer = {
        let (racer_ok, racer_deny, racer_bad) = (
            Arc::clone(&racer_ok),
            Arc::clone(&racer_deny),
            Arc::clone(&racer_bad),
        );
        let cp3 = cpath(&spill);
        std::thread::spawn(move || {
            for _ in 0..20_000 {
                match commit(&cp3, 64) {
                    CGAP_OK => {
                        racer_ok.fetch_add(1, Ordering::Relaxed);
                    }
                    DENIED => {
                        racer_deny.fetch_add(1, Ordering::Relaxed);
                    }
                    _ => {
                        racer_bad.fetch_add(1, Ordering::Relaxed);
                        break;
                    }
                }
            }
        })
    };

    racer.join().expect("racer thread");
    stop.store(true, Ordering::Relaxed);
    owner.join().expect("owner thread");

    // Final-state asserts (Chicago).
    assert_eq!(owner_bad.load(Ordering::Relaxed), 0, "owner never refused");
    assert_eq!(racer_bad.load(Ordering::Relaxed), 0, "no third status");
    assert!(
        racer_deny.load(Ordering::Relaxed) >= 1,
        "the gate must fire under same-path contention (grid vacuous otherwise)"
    );
    let after = fs::read(&spill).expect("spill readable after grid");
    assert!(
        after.len() >= before.len(),
        "spill must never shrink (truncation: {} -> {})",
        before.len(),
        after.len()
    );
    assert!(
        after[..before.len()] == before[..],
        "drained prefix clobbered by a second grant — truncation the gate must prevent"
    );
    fs::remove_dir_all(&dir).expect("sandbox cleanup");
}

#[test]
fn fa9_release_after_owner_thread_exit() {
    let dir = sandbox("release");
    let spill = dir.join("release.par2");
    let cp = cpath(&spill);

    // A separate owner thread grants at P and signals readiness; the test
    // thread's attempt at the SAME path while that owner is ALIVE must be
    // denied exactly.
    let ready = Arc::new(AtomicBool::new(false));
    let owner = {
        let (ready, cp2) = (Arc::clone(&ready), cpath(&spill));
        std::thread::spawn(move || {
            let st = commit(&cp2, 64);
            assert_eq!(st, CGAP_OK, "release-leg owner must grant (st {st:#x})");
            ready.store(true, Ordering::SeqCst);
        })
    };
    spin_until(&ready);

    // While the owner thread is alive, the same path from HERE is denied.
    let st = commit(&cp, 64);
    assert_eq!(
        st, DENIED,
        "same-path attempt while owner alive must be 0x4347_0006, got {st:#x}"
    );

    // The owner exits (lease lifetime == thread lifetime); the path is
    // free again. Grant from this thread must now admit.
    owner.join().expect("owner thread");
    let st2 = commit(&cp, 64);
    assert_eq!(
        st2, CGAP_OK,
        "path must be claimable after the owner thread exits, got {st2:#x}"
    );
    fs::remove_dir_all(&dir).expect("sandbox cleanup");
}
