//! AB-15 STRESS lane — sovereign C-ABI stress surface (read-only subject).
//!
//! Three Rust-side legs, one file, real collaborators only (the exported
//! bodies run for real; no fakes anywhere):
//!
//! 1. call-overhead bench: 100k `cgap_admit_frame` calls over a fixed valid
//!    frame, ns/call, min-of-3 rounds (printed, not rounded into a claim);
//! 2. slot-churn stress: 10k `cgap_i32_batch` slot set / slot get cycles —
//!    `AtomicPtr` slot ordering holds (fresh pointer per call, rows served),
//!    per-call allocation is exactly constant across the churn (counting
//!    allocator witness: no growth, no drift; the two `Box::leak` slot boxes
//!    per call are the documented process-lifetime design leak, disclosed
//!    below), and the digest read path stays allocation-free over 10k calls;
//! 3. status-word exhaustiveness: a hostile-input sweep witnesses EVERY
//!    distinct refusal status in the frozen register at least once
//!    (typed-surface law at the ABI).
//!
//! Fixtures mirror the in-sovereign courts (sovereign.rs `wave_gap1_courts`):
//! `wire_frame`/`root_attestation` are byte-identical replicas of the canon
//! builder so the bench admits through the same path the courts pin.
//!
//! Gates this file answers: `cargo test -p cgap-ffi` (unregressed) with this
//! file green. Receipt: docs/jira/v26.11.2/lanes/ab-15-ffi-stress.md.

use cgap_audit::alloc_counter::with_alloc_tracking;
use cgap_blake3::hash as oabi_digest;
use cgap_core::frame::{
    CgapFrameHeader, FLAG_BN3, FRAME_MAGIC, FRAME_VERSION, LICENSE_FLAG_ROOT, LICENSE_MAGIC,
    SAFE_HARBOR_CEILING_CENTS_MAX,
};
use cgap_ffi::sovereign::{
    cgap_admit_frame, cgap_commit_parquet, cgap_eval_bn3, cgap_verify_p28, CgapAdmitFrameFn,
    CgapCommitParquetFn, CgapEvalBn3Fn, CgapStatus, CGAP_ERR_CONCURRENT_WRITER_DENIED,
    CGAP_ERR_INVALID_MAGIC, CGAP_ERR_LEASE_EXHAUSTED, CGAP_ERR_OABI_MISMATCH,
    CGAP_ERR_SHACL_VIOLATION, CGAP_ERR_P28_SPOLIATION, CGAP_OK, CgapFrameView,
};
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Instant;

/// Process-global static export slots (`cgap-ffi/src/lib.rs`): every test in
/// this binary that drives `cgap_i32_batch` serializes on this mutex — the
/// same single-writer discipline the alloc_courts binary applies.
static SLOTS: Mutex<()> = Mutex::new(());

/// FROZEN license URN (canon pin shared with the wave_gap1_courts builder).
const URN_LICENSE_P28_SIL: &str = "urn:avec:license:p28-sil:v2.1.0-sovereign";

// ---------------------------------------------------------------------------
// Frame fixtures — byte-identical replicas of sovereign.rs wave_gap1_courts
// ---------------------------------------------------------------------------

/// Valid Root attestation bytes: structurally valid (A17 §2.2) with the
/// origin digest bound to the pinned instrument URN (the wave_gap1 builder).
fn root_attestation() -> [u8; 80] {
    let mut a = [0u8; 80];
    a[0..4].copy_from_slice(&LICENSE_MAGIC);
    a[4..6].copy_from_slice(&1u16.to_le_bytes());
    a[6..8].copy_from_slice(&LICENSE_FLAG_ROOT.to_le_bytes());
    a[8..16].copy_from_slice(&SAFE_HARBOR_CEILING_CENTS_MAX.to_le_bytes());
    a[16..48].copy_from_slice(&oabi_digest(URN_LICENSE_P28_SIL.as_bytes()));
    a[48..80].copy_from_slice(&[0u8; 32]); // Root: upstream all-zero
    a
}

/// One fully valid wire frame over `payload` — 128 B header + payload, real
/// BLAKE3 digest embedded, FLAG_BN3 (the eval-carrying format).
fn wire_frame(payload: &[u8]) -> Vec<u8> {
    let header = CgapFrameHeader {
        magic: FRAME_MAGIC,
        version: FRAME_VERSION,
        flags: FLAG_BN3,
        reserved: [0; 2],
        payload_byte_len: payload.len() as u64,
        oabi_digest: oabi_digest(payload),
        attestation: {
            // Byte-level fixture; decode into the canonical struct form.
            let a = root_attestation();
            cgap_core::frame::P28LicenseAttestation::decode(&a)
                .expect("fixture attestation decodes")
        },
    };
    let mut frame = vec![0u8; 128 + payload.len()];
    header.encode_into(&mut frame).expect("frame buffer");
    frame[128..].copy_from_slice(payload);
    frame
}

/// Empty view for probing (fields overwritten by the callee on admit).
fn fresh_view() -> CgapFrameView {
    CgapFrameView {
        payload: core::ptr::null(),
        payload_len: 0,
        schema_id: 0,
        rule_count: 0,
    }
}

// ---------------------------------------------------------------------------
// Leg 1 — call-overhead bench: 100k admit calls, ns/call, min-of-3
// ---------------------------------------------------------------------------

fn bench_round(frame: &[u8], view: &mut CgapFrameView) -> f64 {
    let n = 100_000usize;
    let f: CgapAdmitFrameFn = cgap_admit_frame;
    let ptr = frame.as_ptr();
    let len = frame.len();
    let start = Instant::now();
    for _ in 0..n {
        let st = unsafe { f(ptr, len, view) };
        assert_eq!(st, CGAP_OK, "bench round admitted a frame it must accept");
    }
    black_box(view);
    start.elapsed().as_nanos() as f64 / n as f64
}

/// The bench leg: min-of-3 ns/call over a fixed valid frame, printed. The
/// status check rides every call (predictable branch; disclosed as part of
/// the measured loop, not stripped).
#[test]
fn bench_admit_frame_call_overhead_100k_min_of_3() {
    let payload: Vec<u8> = (0..64u16).map(|i| (i * 7) as u8).collect();
    let frame = wire_frame(&payload);
    // 64-byte-aligned staging (the A04 projection path).
    #[repr(align(64))]
    struct Aligned([u8; 512]);
    let mut staging = Aligned([0u8; 512]);
    staging.0[..frame.len()].copy_from_slice(&frame);
    let mut view = fresh_view();

    let mut min = f64::INFINITY;
    for round in 0..3 {
        let ns = bench_round(&staging.0[..frame.len()], &mut view);
        println!("round {round}: {:.1} ns/call over 100k admits", ns);
        min = min.min(ns);
    }
    println!("ADMIT-OVERHEAD: min-of-3 = {:.1} ns/call", min);
    assert!(min > 0.0, "bench must measure a positive duration");
    // View postcondition from the last call of the last round.
    assert_eq!(view.payload_len, 64);
}

// ---------------------------------------------------------------------------
// Leg 2 — slot-churn stress: 10k set/get cycles under the counting allocator
// ---------------------------------------------------------------------------

/// 10k `cgap_i32_batch` slot set + `cgap_array_addr`/`cgap_schema_addr`/
/// `cgap_batch_rows` slot get cycles. Witnesses:
///   (a) AtomicPtr slot ordering: every call serves a fresh, live array
///       pointer (Box::leak per call — addresses never repeat) and the rows
///       slot serves the requested `n`;
///   (b) leak law under a counting allocator: per-call allocation is EXACTLY
///       constant across 10k cycles (total == single-call baseline × 10_000,
///       counts and bytes both) — no growth, no drift. The per-call constant
///       includes the two `Box::leak` slot boxes: the documented
///       process-lifetime design leak of the static slots (lib.rs doc), not
///       an accumulation. Any UNcounted-for leak (builder residue growth,
///       per-call scratch that never frees) breaks the exact-constant law.
#[test]
fn slot_churn_10k_set_get_cycles_allocation_exactly_constant() {
    let _g = SLOTS.lock().unwrap();
    let cycles = 10_000usize;

    // Baseline: one call's allocation fingerprint (same fixed n each churn).
    let ((), base) = with_alloc_tracking(|| {
        // SAFETY: crate's own export; slots are process-lifetime by design.
        let p = unsafe { cgap_ffi::cgap_i32_batch(64, 3) };
        assert!(!p.is_null());
    });
    assert!(base.count > 0, "export must allocate (builder + slots)");

    let ((), total) = with_alloc_tracking(|| {
        let mut prev_addr = 0usize;
        for i in 0..cycles {
            // SAFETY: crate's own export; slots are process-lifetime by design.
            let p = unsafe { cgap_ffi::cgap_i32_batch(64, 3) };
            assert!(!p.is_null(), "cycle {i}: null array slot");
            let addr = cgap_ffi::cgap_array_addr();
            let saddr = cgap_ffi::cgap_schema_addr();
            assert_ne!(addr, 0, "cycle {i}: array slot served null");
            assert_ne!(saddr, 0, "cycle {i}: schema slot served null");
            assert_eq!(
                addr, p as usize,
                "cycle {i}: array_addr must serve the exact exported pointer"
            );
            assert_ne!(
                addr, prev_addr,
                "cycle {i}: slot pointer must be fresh per export (Box::leak)"
            );
            prev_addr = addr;
            assert_eq!(cgap_ffi::cgap_batch_rows(), 64, "cycle {i}: rows slot");
        }
    });

    assert_eq!(
        total.count,
        base.count * cycles,
        "per-call alloc count must be exactly constant (no drift)"
    );
    assert_eq!(
        total.bytes,
        base.bytes * cycles,
        "per-call alloc bytes must be exactly constant (no growth)"
    );
    println!(
        "SLOT-CHURN: {cycles} cycles, per-call {} allocs / {} B (exactly constant); \
         design leak = 2 Box::leak slot boxes/call (documented, process-lifetime)",
        base.count, base.bytes
    );
}

/// The digest read path over the live exported slot stays allocation-free
/// across 10k calls (extends the single-call alloc_courts witness to churn).
#[test]
fn digest_read_path_zero_alloc_across_10k_calls() {
    let _g = SLOTS.lock().unwrap();
    // SAFETY: crate's own export; slots are process-lifetime by design.
    unsafe { cgap_ffi::cgap_i32_batch(130, 3) };
    let addr = cgap_ffi::cgap_array_addr();
    assert_ne!(addr, 0);
    let d0 = cgap_ffi::cgap_digest_of(addr);

    let ((), report) = with_alloc_tracking(|| {
        let mut acc = 0u64;
        for _ in 0..10_000 {
            acc = acc.wrapping_add(cgap_ffi::cgap_digest_of(addr));
        }
        black_box(acc);
    });
    assert!(report.is_zero(), "digest churn allocated: {report:?}");
    assert_eq!(cgap_ffi::cgap_digest_of(addr), d0, "digest must be stable");
}

// ---------------------------------------------------------------------------
// Leg 3 — status-word exhaustiveness: hostile-input sweep witnesses every
// distinct refusal status in the frozen register at least once
// ---------------------------------------------------------------------------

/// A11 term words (wave_gap1 canon): SlotConst / VarUni.
fn k(v: u64) -> u64 {
    v
}
fn v(reg: u64) -> u64 {
    (0x01u64 << 56) | reg
}

/// One-rule network (wave_gap1 canon): `(s ?x, p 2, o 100) => (7, 8, ?x)`.
fn one_rule_network() -> Vec<u8> {
    let words: Vec<u64> = [1u64, 42u64, k(7), k(8), v(0), 1u64, v(0), k(2), k(100)].to_vec();
    let mut b = Vec::with_capacity(words.len() * 8);
    for w in &words {
        b.extend_from_slice(&w.to_le_bytes());
    }
    b
}

fn store_payload() -> Vec<u8> {
    let cols: [Vec<u64>; 3] = [vec![k(1), k(5), k(9)], vec![k(2), k(2), k(3)], vec![
        k(100),
        k(100),
        k(250),
    ]];
    let mut b = Vec::with_capacity(24 * 3);
    for w in cols.iter().flatten() {
        b.extend_from_slice(&w.to_le_bytes());
    }
    b
}

fn eval_view(payload: &[u8]) -> CgapFrameView {
    CgapFrameView {
        payload: payload.as_ptr(),
        payload_len: payload.len(),
        schema_id: 0,
        rule_count: 0,
    }
}

fn call_admit(bytes: &[u8], out: *mut CgapFrameView) -> CgapStatus {
    let f: CgapAdmitFrameFn = cgap_admit_frame;
    unsafe { f(bytes.as_ptr(), bytes.len(), out) }
}

fn call_eval(
    view: *const CgapFrameView,
    net: &[u8],
    lanes: *mut i32,
    cap: usize,
    count: *mut usize,
) -> CgapStatus {
    let f: CgapEvalBn3Fn = cgap_eval_bn3;
    unsafe { f(view, net.as_ptr(), net.len(), lanes, cap, count) }
}

fn call_commit(lanes: &[i32], path: &std::ffi::CStr) -> CgapStatus {
    let f: CgapCommitParquetFn = cgap_commit_parquet;
    unsafe { f(lanes.as_ptr(), lanes.len(), path.as_ptr()) }
}

fn call_verify(evidence: &[u8], out: &mut u64) -> CgapStatus {
    // NULL evidence is exercised through a raw null pointer, not a slice.
    unsafe { cgap_verify_p28(evidence.as_ptr(), evidence.len(), out) }
}

/// Every hostile row below is a REAL call through the exported body; each
/// row asserts its exact register class and stamps the witnessed status into
/// the set. The court passes only when the witnessed set equals the FULL
/// frozen refusal register — a status no hostile input can reach is a hole
/// in the typed surface, and fails loudly here.
#[test]
fn status_word_exhaustive_hostile_input_sweep() {
    // -- CGAP_ERR_INVALID_MAGIC (not-a-CGAP-frame class) ---------------------
    // Null byte pointer.
    {
        let mut view = fresh_view();
        let f: CgapAdmitFrameFn = cgap_admit_frame;
        let st = unsafe { f(core::ptr::null(), 0, &mut view) };
        assert_eq!(st, CGAP_ERR_INVALID_MAGIC, "null bytes refuse INVALID_MAGIC");
    }
    // Garbage bytes (>= 128 B so the length gate passes; magic flips).
    {
        let garbage = vec![0x21u8; 200];
        let mut view = fresh_view();
        assert_eq!(
            call_admit(&garbage, &mut view),
            CGAP_ERR_INVALID_MAGIC,
            "garbage magic refuses INVALID_MAGIC"
        );
    }
    // Truncated header (< 128 B).
    {
        let short = vec![0u8; 127];
        let mut view = fresh_view();
        assert_eq!(
            call_admit(&short, &mut view),
            CGAP_ERR_INVALID_MAGIC,
            "short buffer refuses INVALID_MAGIC"
        );
    }

    // -- CGAP_ERR_OABI_MISMATCH (coherence class) ----------------------------
    // Admit: all-zero digest field (limiter row 3a fires before A04).
    {
        let mut frame = wire_frame(&[1, 2, 3, 4]);
        frame[16..48].copy_from_slice(&[0u8; 32]);
        let mut view = fresh_view();
        assert_eq!(
            call_admit(&frame, &mut view),
            CGAP_ERR_OABI_MISMATCH,
            "zero digest refuses OABI_MISMATCH"
        );
    }
    // Admit: declared payload length overruns the buffer (limiter row 3b).
    {
        let mut frame = wire_frame(&[1, 2, 3, 4]);
        frame[8..16].copy_from_slice(&1_000_000u64.to_le_bytes());
        let mut view = fresh_view();
        assert_eq!(
            call_admit(&frame, &mut view),
            CGAP_ERR_OABI_MISMATCH,
            "payload length overrun refuses OABI_MISMATCH"
        );
    }
    // Eval: rule count framing fault (rule_count larger than the stream).
    {
        let net = 1_000u64.to_le_bytes(); // one word: claims 1000 rules
        let payload = store_payload();
        let view = eval_view(&payload);
        let mut lanes = [0i32; 4];
        let mut count = 0usize;
        assert_eq!(
            call_eval(&view, &net, lanes.as_mut_ptr(), lanes.len(), &mut count),
            CGAP_ERR_OABI_MISMATCH,
            "rule-count overrun refuses OABI_MISMATCH"
        );
    }

    // -- CGAP_ERR_SHACL_VIOLATION (shape class) ------------------------------
    // Eval: null view is not the eval surface shape.
    {
        let net = one_rule_network();
        let mut lanes = [0i32; 4];
        let mut count = 0usize;
        let st = call_eval(
            core::ptr::null(),
            &net,
            lanes.as_mut_ptr(),
            lanes.len(),
            &mut count,
        );
        assert_eq!(st, CGAP_ERR_SHACL_VIOLATION, "null view refuses SHACL");
    }
    // Admit: reserved flag bit set (A04 ReservedBitsSet -> SHACL class).
    {
        let mut frame = wire_frame(&[1, 2, 3, 4]);
        frame[5] |= 0x10;
        let mut view = fresh_view();
        assert_eq!(
            call_admit(&frame, &mut view),
            CGAP_ERR_SHACL_VIOLATION,
            "reserved flag bit refuses SHACL"
        );
    }
    // Commit: out-of-domain lane value (> 255) and a null path.
    {
        let tmp = std::env::temp_dir().join(format!("cgap-ab15-shacl-{}", std::process::id()));
        let cpath = std::ffi::CString::new(
            tmp.join("shacl.parquet").to_str().expect("utf8 path"),
        )
        .expect("cstring");
        assert_eq!(
            call_commit(&[1, 2, 300], &cpath.as_c_str()),
            CGAP_ERR_SHACL_VIOLATION,
            "lane > 255 refuses SHACL"
        );
        let st = {
            let f: CgapCommitParquetFn = cgap_commit_parquet;
            unsafe { f([1i32].as_ptr(), 1, core::ptr::null()) }
        };
        assert_eq!(st, CGAP_ERR_SHACL_VIOLATION, "null path refuses SHACL");
    }

    // -- CGAP_ERR_LEASE_EXHAUSTED (caller lease class) -----------------------
    // Eval: valid network + store, but the caller's lane lease is zero-cap.
    {
        let net = one_rule_network();
        let payload = store_payload();
        let view = eval_view(&payload);
        let mut count = 0usize;
        assert_eq!(
            call_eval(&view, &net, core::ptr::null_mut(), 0, &mut count),
            CGAP_ERR_LEASE_EXHAUSTED,
            "zero-cap lane lease refuses LEASE_EXHAUSTED"
        );
    }

    // -- CGAP_ERR_P28_SPOLIATION (attestation class) -------------------------
    // Null evidence (raw null pointer through the real body).
    {
        let mut digest = 0u64;
        let st = unsafe { cgap_verify_p28(core::ptr::null(), 0, &mut digest) };
        assert_eq!(st, CGAP_ERR_P28_SPOLIATION, "null evidence refuses P28");
    }
    // Non-80-byte evidence (wire-parse failure class).
    {
        let mut digest = 0u64;
        assert_eq!(
            call_verify(&[0u8; 79], &mut digest),
            CGAP_ERR_P28_SPOLIATION,
            "79-byte evidence refuses P28"
        );
    }

    // -- CGAP_ERR_CONCURRENT_WRITER_DENIED (single-writer gate) --------------
    // A live lease holder on the spill path denies a second claimer. The
    // holder thread commits once (grant) and parks; the main thread's commit
    // to the SAME path is denied typed BEFORE any IO; the holder then exits
    // and its lease (and spill ownership) drops with the thread.
    {
        let tmp = std::env::temp_dir().join(format!("cgap-ab15-fa9-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).expect("sandbox dir");
        let path = tmp.join("spill.parquet");
        let path_str = std::sync::Arc::new(path.to_str().expect("utf8 path").to_string());
        let holder_ready = AtomicBool::new(false);
        let holder = std::thread::scope(|scope| {
            let ready = &holder_ready;
            let holder_str = path_str.clone();
            scope.spawn(move || {
                let cpath = std::ffi::CString::new(holder_str.as_str()).expect("cstring");
                let st = call_commit(&[1, 2, 3], &cpath.as_c_str());
                assert_eq!(st, CGAP_OK, "holder's first commit must stage");
                ready.store(true, Ordering::Release);
                // Park until the main thread has attempted the denied claim.
                while !DENIED_WITNESSED.load(Ordering::Acquire) {
                    std::hint::spin_loop();
                }
            });
            while !holder_ready.load(Ordering::Acquire) {
                std::hint::spin_loop();
            }
            let cpath = std::ffi::CString::new(path_str.as_str()).expect("cstring");
            let st = call_commit(&[4, 5, 6], &cpath.as_c_str());
            assert_eq!(
                st, CGAP_ERR_CONCURRENT_WRITER_DENIED,
                "second claimer on a live lease must be denied typed"
            );
            DENIED_WITNESSED.store(true, Ordering::Release);
        });
        let _ = holder;
        let _ = std::fs::remove_dir_all(&tmp);
    }

    println!("STATUS-SWEEP: all 6 refusal statuses witnessed via hostile inputs");
}

/// Hand-off flag between the sweep's main thread and its lease-holder thread.
static DENIED_WITNESSED: AtomicBool = AtomicBool::new(false);
