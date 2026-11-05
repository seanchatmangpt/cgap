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
//!   4. RED-13 alias race: the owner holds `dir/x`; a racer attempts
//!      `dir/./x` (textually different, SAME file). Post-fix the registry
//!      keys on canonical bytes, so every racer attempt is DENIED
//!      0x4347_0006 and the owner's drained spill is prefix-preserved.
//!      Anti-vacuity: reverting `canonical_key` to raw-byte FNV-1a makes
//!      the racer ADMIT and truncate (red-13 exit 7 shape) — this leg
//!      flips.
//!   5. ADV-10 rename race (R-ADV10FIX): owner holds `dir/x`; the test
//!      RENAMES `dir -> dir2` mid-lease; the racer attacks `dir2/x` —
//!      SAME inode, different text. Post-fix the lease re-binds to
//!      (dev, ino), so every racer attempt is DENIED and the spill is
//!      prefix-preserved. Anti-vacuity: reverting to text-canonical
//!      keying ADMITS the racer (truncate escape) — this leg flips.
//!   6. ADV-17 symlink key-flip: racer owns `dir/y`; attacker plants
//!      `dir/x -> dir/y`; the owner attempts `dir/x` — must be DENIED
//!      typed (final-component symlink refusal), and the racer's y bytes
//!      intact. Anti-vacuity: reverting to text-canonical keying admits
//!      the owner and `File::create(dir/x)` follows the link, O_TRUNC
//!      y — flips.
//!   7. rebind corollary: the text→(dev,ino) upgrade leaves no stale
//!      dual-entry shape (owner re-entry and post-exit re-claim both OK).
//!
//! Divergence from dispatch (recorded): the gate is a per-(path, thread)
//! ownership registry with lease-lifetime release, not a per-call global
//! AtomicBool — a per-call flag cannot deliver leg 2's preservation
//! invariant (the racer can win the inter-call gap and legally truncate
//! at grant). Reasoning in the receipt.

use std::ffi::CString;
use std::fs;
use std::path::{Path, PathBuf};
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

fn cpath(p: &Path) -> CString {
    CString::new(p.as_os_str().as_encoded_bytes()).expect("NUL-free path")
}

/// One REAL commit through the sovereign surface (small in-lease frame).
fn commit(path: &CString, n: usize) -> u32 {
    let lanes: Vec<i32> = (0..n).map(|i| (i % 251) as i32).collect();
    // SAFETY: valid lane pointer + NUL-terminated path, live for the call.
    unsafe { cgap_commit_parquet(lanes.as_ptr(), lanes.len(), path.as_ptr()) }
}

/// Bounded poll on a real observable (no sleep): deadline by WALL CLOCK,
/// not iterations. History (XA-2, v26.11.3): the original bound was
/// iteration-count (50M loads), which is machine- AND profile-dependent —
/// release compiles the spin ~1000x faster than debug, collapsing the
/// fence to ~15-20ms while the owner's real time-to-ready (grant +
/// ~309 commits + rotation drain incl. fsync) measures ~14.7ms in
/// release, so 3 grid legs expired deterministically. An iteration bound
/// fences the wrong variable: it shrinks exactly when the code speeds up,
/// while the owner's latency is IO-bound (fsync) and does not. Time
/// deadline, same observable, loud expiry.
fn spin_until(flag: &AtomicBool) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while std::time::Instant::now() < deadline {
        if flag.load(Ordering::SeqCst) {
            return;
        }
    }
    panic!("bounded poll deadline expired — owner never signalled ready");
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
                if !rotated
                    && !std::fs::read(&spill_for_owner)
                        .unwrap_or_default()
                        .is_empty()
                {
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
    // denied exactly. FIX-01 determinism repair: at HEAD this leg was
    // timing-lucky (the owner could exit its lease in the microsecond
    // window before the test thread's claim landed; the canonical re-key's
    // added latency exposed it 5/5 in a sandbox rerun). The owner now HOLDS
    // the lease on a bounded observable spin until the test thread signals
    // the denial was observed — structural sync, no sleep.
    let ready = Arc::new(AtomicBool::new(false));
    let observed = Arc::new(AtomicBool::new(false));
    let owner = {
        let (ready, observed, cp2) = (Arc::clone(&ready), Arc::clone(&observed), cpath(&spill));
        std::thread::spawn(move || {
            let st = commit(&cp2, 64);
            assert_eq!(st, CGAP_OK, "release-leg owner must grant (st {st:#x})");
            ready.store(true, Ordering::SeqCst);
            // Bounded spin on a real observable: hold the lease until the
            // test thread confirms its denial observation (deadline-bounded;
            // a bound expiry still ends the thread, loud not hung).
            for _ in 0..50_000_000 {
                if observed.load(Ordering::SeqCst) {
                    return;
                }
            }
        })
    };
    spin_until(&ready);

    // While the owner thread is PROVABLY alive (held on the observed spin),
    // the same path from HERE is denied.
    let st = commit(&cp, 64);
    assert_eq!(
        st, DENIED,
        "same-path attempt while owner alive must be 0x4347_0006, got {st:#x}"
    );
    observed.store(true, Ordering::SeqCst);

    // The owner exits (lease lifetime == thread lifetime); the path is
    // free again. Grant from this thread must now admit.
    owner.join().expect("owner thread");
    let st2 = commit(&cp, 64);
    assert_eq!(
        st2, CGAP_OK,
        "path must be claimable after the owner thread OWNER-THREAD-EXIT, got {st2:#x}"
    );
    fs::remove_dir_all(&dir).expect("sandbox cleanup");
}

/// RED-13 alias race (v26.11.2 FIX-01): owner holds `dir/x`; racer attacks
/// via `dir/./x` — syntactically different, byte-identical file. Final
/// state: racer DENIED every time, no racer OK, drained spill never
/// shrinks and its prefix is intact. This is the leg that flips when the
/// canonical re-key is reverted to raw-byte hashing.
#[test]
fn fa9_alias_race_denied() {
    let dir = sandbox("alias");
    let owner_path = dir.join("race.par2");
    let alias_path = dir.join(".").join("race.par2");
    assert_ne!(
        owner_path.as_os_str().as_encoded_bytes(),
        alias_path.as_os_str().as_encoded_bytes(),
        "test premise: the two path texts must differ (same file either way)"
    );

    let ready = Arc::new(AtomicBool::new(false));
    let (owner_ok, owner_bad) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
    let (racer_ok, racer_deny) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
    let stop = Arc::new(AtomicBool::new(false));

    let owner = {
        let (ready, owner_ok, owner_bad, stop) = (
            Arc::clone(&ready),
            Arc::clone(&owner_ok),
            Arc::clone(&owner_bad),
            Arc::clone(&stop),
        );
        let cp_owner = cpath(&owner_path);
        let spill_for_owner = owner_path.clone();
        std::thread::spawn(move || {
            // Grant + at least one rotation on the OWNER path so drained
            // bytes exist to protect (the truncation target of red-13).
            let mut rotated = false;
            for _ in 0..4096 {
                match commit(&cp_owner, 64) {
                    CGAP_OK => {
                        owner_ok.fetch_add(1, Ordering::Relaxed);
                    }
                    _ => {
                        owner_bad.fetch_add(1, Ordering::Relaxed);
                        return;
                    }
                }
                if !rotated
                    && !std::fs::read(&spill_for_owner)
                        .unwrap_or_default()
                        .is_empty()
                {
                    rotated = true;
                    ready.store(true, Ordering::SeqCst);
                }
            }
            if !rotated {
                owner_bad.fetch_add(1, Ordering::Relaxed);
                return;
            }
            while !stop.load(Ordering::Relaxed) {
                match commit(&cp_owner, 64) {
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

    spin_until(&ready);
    let before = fs::read(&owner_path).expect("spill readable at race start");
    let racer = {
        let (racer_ok, racer_deny) = (Arc::clone(&racer_ok), Arc::clone(&racer_deny));
        let cp_alias = cpath(&alias_path);
        std::thread::spawn(move || {
            for _ in 0..20_000 {
                match commit(&cp_alias, 64) {
                    CGAP_OK => {
                        racer_ok.fetch_add(1, Ordering::Relaxed);
                    }
                    DENIED => {
                        racer_deny.fetch_add(1, Ordering::Relaxed);
                    }
                    other => {
                        let _ = other;
                        break;
                    }
                }
            }
        })
    };

    racer.join().expect("racer thread");
    stop.store(true, Ordering::Relaxed);
    owner.join().expect("owner thread");

    assert_eq!(owner_bad.load(Ordering::Relaxed), 0, "owner never refused");
    assert_eq!(
        racer_ok.load(Ordering::Relaxed),
        0,
        "RED-13 alias must be DENIED (canonical re-key); an OK here is the breach"
    );
    assert!(
        racer_deny.load(Ordering::Relaxed) >= 1,
        "alias grid vacuous without >=1 racer attempt"
    );
    let after = fs::read(&owner_path).expect("spill readable after alias leg");
    assert!(
        after.len() >= before.len(),
        "spill must never shrink (alias truncation: {} -> {})",
        before.len(),
        after.len()
    );
    assert!(
        after[..before.len()] == before[..],
        "drained prefix clobbered by alias grant — red-13 truncation the re-key must prevent"
    );
    fs::remove_dir_all(&dir).expect("sandbox cleanup");
}

/// ADV-10 rename race (R-ADV10FIX): owner holds `dir/x`; the directory is
/// RENAMED `dir -> dir2` mid-lease; the racer attacks `dir2/x` — the SAME
/// inode through different text. Post-fix the lease re-binds to (dev, ino),
/// so the racer is DENIED every attempt and the drained spill (read via the
/// NEW path, since the rename moved the directory) is prefix-preserved.
/// Anti-vacuity: reverting the registry to text-canonical keying ADMITS the
/// racer — its grant File::create(dir2/x) O_TRUNCs the owner's spill (the
/// exact ADV-10 escape; adv-10-fmea-risk-laundering.md B verdict).
#[test]
fn fa9_rename_race_denied() {
    let dir = sandbox("adv10");
    let leasedir = dir.join("lease");
    fs::create_dir_all(&leasedir).expect("leasedir");
    let owner_path = leasedir.join("race.par2");
    let dir2 = dir.join("renamed");
    let racer_path = dir2.join("race.par2");

    let ready = Arc::new(AtomicBool::new(false));
    let (owner_ok, owner_bad) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
    let (racer_ok, racer_deny) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
    let stop = Arc::new(AtomicBool::new(false));

    let owner = {
        let (ready, owner_ok, owner_bad, stop) = (
            Arc::clone(&ready),
            Arc::clone(&owner_ok),
            Arc::clone(&owner_bad),
            Arc::clone(&stop),
        );
        let cp_owner = cpath(&owner_path);
        let spill_for_owner = owner_path.clone();
        std::thread::spawn(move || {
            let mut rotated = false;
            for _ in 0..4096 {
                match commit(&cp_owner, 64) {
                    CGAP_OK => {
                        owner_ok.fetch_add(1, Ordering::Relaxed);
                    }
                    _ => {
                        owner_bad.fetch_add(1, Ordering::Relaxed);
                        return;
                    }
                }
                if !rotated
                    && !std::fs::read(&spill_for_owner)
                        .unwrap_or_default()
                        .is_empty()
                {
                    rotated = true;
                    ready.store(true, Ordering::SeqCst);
                }
            }
            if !rotated {
                owner_bad.fetch_add(1, Ordering::Relaxed);
                return;
            }
            while !stop.load(Ordering::Relaxed) {
                match commit(&cp_owner, 64) {
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

    spin_until(&ready);

    // Rename the directory MID-LEASE (the ADV-10 attack step).
    fs::rename(&leasedir, &dir2).expect("rename leasedir -> dir2");
    let before = fs::read(&racer_path).expect("spill readable via NEW path after rename");

    let racer = {
        let (racer_ok, racer_deny) = (Arc::clone(&racer_ok), Arc::clone(&racer_deny));
        let cp_racer = cpath(&racer_path);
        std::thread::spawn(move || {
            for _ in 0..20_000 {
                match commit(&cp_racer, 64) {
                    CGAP_OK => {
                        racer_ok.fetch_add(1, Ordering::Relaxed);
                    }
                    DENIED => {
                        racer_deny.fetch_add(1, Ordering::Relaxed);
                    }
                    other => {
                        let _ = other;
                        break;
                    }
                }
            }
        })
    };

    racer.join().expect("racer thread");
    stop.store(true, Ordering::Relaxed);
    owner.join().expect("owner thread");

    assert_eq!(owner_bad.load(Ordering::Relaxed), 0, "owner never refused");
    assert_eq!(
        racer_ok.load(Ordering::Relaxed),
        0,
        "ADV-10 rename alias must be DENIED ((dev,ino) rebind); an OK here is the breach"
    );
    assert!(
        racer_deny.load(Ordering::Relaxed) >= 1,
        "rename grid vacuous without >=1 racer attempt"
    );
    let after = fs::read(&racer_path).expect("spill readable via NEW path after grid");
    assert!(
        after.len() >= before.len(),
        "spill must never shrink (rename truncation: {} -> {})",
        before.len(),
        after.len()
    );
    assert!(
        after[..before.len()] == before[..],
        "drained prefix clobbered via renamed-path grant — ADV-10 TOCTOU the identity key must prevent"
    );
    fs::remove_dir_all(&dir).expect("sandbox cleanup");
}

/// ADV-17 symlink key-flip (R-ADV10FIX): the attacker plants a DANGLING
/// symlink `dir/x -> dir/y` (y does not exist); the owner attempts to
/// claim `dir/x`. Post-fix a symlink at the spill path's FINAL component
/// is refused typed BEFORE any IO: the grant File::create would follow the
/// dangling link and O_TRUNC-create the target — under a live lease on the
/// target name this is the ADV-17 dual-owner/truncation shape. Final
/// state: owner DENIED exactly 0x4347_0006 and y is NEVER created through
/// the link. Anti-vacuity: disabling the symlink refusal ADMITS the owner,
/// whose grant File::create(dir/x) FOLLOWS the dangling link and CREATES
/// dir/y — this leg flips (witnessed: MUTANT-ADV17). The present-target
/// variant (y already exists) is independently denied by the (dev,ino)
/// identity key — see fa9_rename_race_denied's mechanism.
#[test]
fn fa9_symlink_keyflip_denied() {
    let dir = sandbox("adv17");
    let y_path = dir.join("y.par2");
    let x_path = dir.join("x.par2");

    // Attacker strikes FIRST: plant the symlink while it is DANGLING
    // (y does not exist — canonicalize cannot resolve it, so no path-text
    // or identity key can name the future target; refusal is the only
    // defense).
    std::os::unix::fs::symlink(&y_path, &x_path).expect("plant dangling symlink");
    assert!(!fs::metadata(&y_path).is_ok(), "premise: y absent");

    // The owner attempts the symlink path: post-fix DENIED typed, with
    // ZERO IO through the link (the test thread acts as the lease thread).
    let cp_x = cpath(&x_path);
    let st = commit(&cp_x, 64);
    assert_eq!(
        st, DENIED,
        "symlink at spill path must be refused typed, got {st:#x}"
    );
    assert!(
        !fs::symlink_metadata(&y_path).is_ok(),
        "y must NEVER be created through the dangling link (IO reached the target)"
    );
    fs::remove_dir_all(&dir).expect("sandbox cleanup");
}

/// Rebind corollary (coordinator scope extension): the text→(dev,ino) key
/// upgrade must leave no stale dual-ownership shape — the owner's own
/// re-entry after upgrade is OK, and after the owner thread EXITS the path
/// is claimable by a fresh thread (no multi-owner deadlock, no leaked
/// pre-upgrade text-key entry).
#[test]
fn fa9_rebind_no_stale_ownership() {
    let dir = sandbox("rebind");
    let spill = dir.join("rebind.par2");
    let cp = cpath(&spill);

    // Owner: grant (absent-claim → text key, then rebind → identity key),
    // then a SECOND commit on the same path (steady-state re-entry under
    // the upgraded key) — must still be OK, never self-denied.
    let ready = Arc::new(AtomicBool::new(false));
    let second_ok = Arc::new(AtomicBool::new(false));
    {
        let (ready, second_ok, cp2, spill2) = (
            Arc::clone(&ready),
            Arc::clone(&second_ok),
            cpath(&spill),
            spill.clone(),
        );
        std::thread::spawn(move || {
            let st = commit(&cp2, 64);
            assert_eq!(st, CGAP_OK, "first commit (grant+rebind) (st {st:#x})");
            // Bounded observable poll: file exists on disk.
            for _ in 0..50_000_000 {
                if std::fs::metadata(&spill2).is_ok() {
                    break;
                }
            }
            let st2 = commit(&cp2, 64);
            assert_eq!(st2, CGAP_OK, "re-entry under upgraded key (st {st2:#x})");
            second_ok.store(true, Ordering::SeqCst);
            ready.store(true, Ordering::SeqCst);
        })
        .join()
        .expect("owner thread");
    }
    assert!(second_ok.load(Ordering::Relaxed), "re-entry must succeed");

    // After the owner thread has EXITED (guard dropped both keys), a fresh
    // thread must claim the same path — no stale-entry deadlock.
    let st3 = commit(&cp, 64);
    assert_eq!(
        st3, CGAP_OK,
        "path claimable after owner exit (no stale dual-entry deadlock), got {st3:#x}"
    );
    fs::remove_dir_all(&dir).expect("sandbox cleanup");
}
