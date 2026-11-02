//! SP8 self-play concurrency spill court (w6-fa9 wiring through the cgap
//! root-crate surface).
//!
//! Re-drives the FA9 race at the SAME gate cgap-ffi's fa9_commit_gating.rs
//! proves, via `cgap_ffi::sovereign::cgap_commit_parquet`: two REAL threads
//! target the identical unrotated spill path. Owner thread streams commits;
//! racer thread attempts the same path and must observe EXACTLY
//! 0x4347_0006 (CGAP_ERR_CONCURRENT_WRITER_DENIED) every time — denial is
//! pre-IO, so the owner's drained spill bytes stay prefix-exact
//! (append-only, never truncated). After the owner thread EXITS (lease
//! lifetime == thread lifetime), the same path is claimable by a fresh
//! thread (gate is not a stuck deny).
//!
//! ANTI-VACUITY: the FA9FIX mutant (gate disabled) was witnessed flipping
//! this court class at exit 101 — docs/jira/v26.11.2/waves/w6-fa9-commit-
//! gating.md (sha-pinned receipt). The spill gate module is PRIVATE to
//! cgap-ffi (sovereign.rs `mod spill_gate`, no exported disable seam), so
//! this court cannot re-run the mutant from the exported surface; the
//! cited receipt is the flip witness. No mock, no Process.sleep: race
//! window and readiness are bounded iteration spins on real observables
//! (file non-empty; stop flag), never wall-clock sleeps.
//!
//! RECEIPT (SP8): files_written = [crates/cgap/tests/
//! self_play_concurrency_spill.rs] + minimal disclosed dev-dep unblock in
//! crates/cgap/Cargo.toml (cgap-ffi path dev-dep; manifest line was the
//! only route to the dispatch's required gate command — recorded as a
//! divergence in the lane report).

use std::ffi::CString;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use cgap_ffi::sovereign::{cgap_commit_parquet, CGAP_ERR_CONCURRENT_WRITER_DENIED, CGAP_OK};

const DENIED: u32 = CGAP_ERR_CONCURRENT_WRITER_DENIED;

fn sandbox(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sp8-spill-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("sandbox dir");
    dir
}

fn cpath(p: &PathBuf) -> CString {
    CString::new(p.as_os_str().as_encoded_bytes()).expect("NUL-free path")
}

/// One REAL commit through the sovereign surface.
fn commit(path: &CString, n: usize) -> u32 {
    let lanes: Vec<i32> = (0..n).map(|i| (i % 251) as i32).collect();
    // SAFETY: valid lane pointer + NUL-terminated path, live for the call.
    unsafe { cgap_commit_parquet(lanes.as_ptr(), lanes.len(), path.as_ptr()) }
}

/// Bounded spin on a real observable (no sleep): deadline by iterations.
fn spin_until(flag: &AtomicBool) {
    for _ in 0..100_000_000 {
        if flag.load(Ordering::SeqCst) {
            return;
        }
    }
    panic!("bounded spin deadline expired — owner never signalled ready");
}

#[test]
fn sp8_register_pin_exact_denial_code() {
    assert_eq!(DENIED, 0x4347_0006, "exact register pin ('CG' class + 06)");
    assert_eq!(DENIED & 0xFFFF_0000, 0x4347_0000, "'CG' class prefix");
    assert_ne!(DENIED & 0x0000_FFFF, 0, "'CG' sequence nonzero");
    assert_ne!(DENIED, CGAP_OK, "denial is not OK");
}

#[test]
fn sp8_two_threads_one_spill_path_racer_denied_owner_prefix_exact() {
    let dir = sandbox("race");
    let spill = dir.join("race.par2");
    let cp = cpath(&spill);

    let ready = Arc::new(AtomicBool::new(false));
    let (owner_ok, owner_bad) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
    let racer = Arc::new(AtomicUsize::new(0));
    let stop = Arc::new(AtomicBool::new(false));

    // OWNER: grants the lease and streams commits on path P until stopped.
    let owner = {
        let (ready, owner_ok, owner_bad, stop) = (
            Arc::clone(&ready),
            Arc::clone(&owner_ok),
            Arc::clone(&owner_bad),
            Arc::clone(&stop),
        );
        let spill_for_owner = spill.clone();
        let cp_owner = cpath(&spill_for_owner);
        std::thread::spawn(move || {
            // Phase 1: grant + at least one rotation so the spill file is
            // non-empty (readiness = REAL observable, not a clock).
            let mut ready_set = false;
            for _ in 0..4096 {
                match commit(&cp_owner, 64) {
                    CGAP_OK => { owner_ok.fetch_add(1, Ordering::Relaxed); }
                    _ => {
                        owner_bad.fetch_add(1, Ordering::Relaxed);
                        return;
                    }
                }
                if !ready_set
                    && !std::fs::read(&spill_for_owner)
                        .unwrap_or_default()
                        .is_empty()
                {
                    ready_set = true;
                    ready.store(true, Ordering::SeqCst);
                }
            }
            assert!(ready_set, "owner never drained bytes to the spill file");
            // Phase 2: stream until the stop flag (race window).
            while !stop.load(Ordering::Relaxed) {
                match commit(&cp_owner, 64) {
                    CGAP_OK => { owner_ok.fetch_add(1, Ordering::Relaxed); }
                    _ => {
                        owner_bad.fetch_add(1, Ordering::Relaxed);
                        break;
                    }
                }
            }
        })
    };

    // RACER: bounded-wait for readiness, then tight-loop REAL attempts at
    // the SAME path. Every attempt must be EXACTLY 0x4347_0006.
    spin_until(&ready);
    let prefix = fs::read(&spill).expect("spill readable at race start");
    assert!(!prefix.is_empty(), "race starts on a drained prefix");
    let racer_handle = {
        let (racer, stop) = (Arc::clone(&racer), Arc::clone(&stop));
        let cp_racer = cp.clone();
        std::thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                match commit(&cp_racer, 64) {
                    DENIED => { racer.fetch_add(1, Ordering::Relaxed); }
                    CGAP_OK => panic!("racer GRANTed an owned spill path (truncation channel)"),
                    other => panic!("racer observed non-DENIED typed code 0x{other:08X}"),
                }
            }
        })
    };
    // Bounded race window: fixed attempt budget on the racer (no clock).
    let mut seen = 0usize;
    while seen < 2_000 && !racer_handle.is_finished() {
        let cur = racer.load(Ordering::Relaxed);
        if cur > seen {
            seen = cur;
        }
    }
    stop.store(true, Ordering::SeqCst);
    owner.join().expect("owner thread");
    racer_handle.join().expect("racer thread");

    assert!(racer.load(Ordering::Relaxed) > 0, "racer never attempted");
    assert_eq!(
        owner_bad.load(Ordering::Relaxed),
        0,
        "owner thread observed a non-OK commit"
    );
    assert!(owner_ok.load(Ordering::Relaxed) > 0);

    // PREFIX-EXACT: everything drained before the race is still there,
    // byte-for-byte, and the file only ever grew (append-only).
    let final_bytes = fs::read(&spill).expect("spill readable at race end");
    assert!(
        final_bytes.len() > prefix.len(),
        "spill file did not grow during the race"
    );
    assert_eq!(
        &final_bytes[..prefix.len()],
        &prefix[..],
        "owner's drained prefix bytes were truncated/mutated by the racer"
    );
}

#[test]
fn sp8_release_after_owner_exit_reclaims_path() {
    let dir = sandbox("release");
    let spill = dir.join("release.par2");
    let cp = cpath(&spill);

    // Owner thread: acquire the path, commit, then EXIT (drop = release).
    let owner = {
        let cp_owner = cp.clone();
        std::thread::spawn(move || {
            let first = commit(&cp_owner, 64);
            assert_eq!(first, CGAP_OK, "owner initial commit refused");
        })
    };
    owner.join().expect("owner thread exits");

    // Fresh thread reclaims the SAME path — gate released at owner exit.
    let reclaimer_ok = {
        let cp_new = cp.clone();
        std::thread::spawn(move || {
            let mut oks = 0usize;
            for _ in 0..64 {
                match commit(&cp_new, 64) {
                    CGAP_OK => oks += 1,
                    other => panic!("reclaim denied after owner exit: 0x{other:08X}"),
                }
            }
            oks
        })
    };
    let oks = reclaimer_ok.join().expect("reclaimer thread");
    assert!(oks > 0, "path not reclaimable after owner exit");
    // Real drained-byte observable: commit until at least one rotation
    // drains to the spill file (bounded budget; no clock).
    let cp_drain = cp.clone();
    let drained = std::thread::spawn(move || {
        for _ in 0..8192 {
            if commit(&cp_drain, 64) != CGAP_OK {
                panic!("drain commit refused");
            }
            if !std::fs::read(&spill).unwrap_or_default().is_empty() {
                return true;
            }
        }
        false
    })
    .join()
    .expect("drain thread");
    assert!(drained, "reclaimer never drained bytes to the spill file");
}
