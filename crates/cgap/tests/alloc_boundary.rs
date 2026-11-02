//! AE7 — permanent hot-path zero-alloc boundary courts (v26.11.2
//! countermeasure lane 07).
//!
//! Driver of the [`tracking_alloc`] interceptor (cgap-core, included here
//! via `#[path]` because cgap-core's lib.rs is outside this lane's
//! ownership — divergence recorded in the lane receipt). The interceptor
//! IS the binary's global allocator; each court brackets its hot loop
//! with `start_hot_path_budget(site, 0, 0)`; any allocation inside the
//! window breaches (record + disarm + null -> the standard allocation-
//! failure unwind; `last_breach()` names the site).
//!
//! Courts (budgets are laws, cites in the lane receipt):
//!   0. positive control: one deliberate Vec alloc under budget(0,0)
//!      MUST breach (a court that cannot flip is FAIL).
//!   1. penstock ingress admission, 16-frame class (SP2 pattern,
//!      penstock.rs zero-alloc witness): budget(0, 0).
//!   2. parquet seal steady-state (MB7 0/0 witness, lane P5 /
//!      self_play_datalog_parquet_seal.rs): budget(0, 0), real rotation
//!      drain to a real spill file inside the window.
//!   3. bn3 noalloc radix probe post-warmup (w5-l3-remeasure alloc-zero,
//!      lever_courts l3_noalloc_radix_* differential): budget(0, 0).
//!   4. HONESTY PER H8: the bn3 FIXPOINT is NOT zero-alloc (the `seen`
//!      BTreeSet is O(store) node allocs; w6-bn3-nostd-borrowed-driver
//!      documents the seen-set). This court does NOT pin budget(0,0) on
//!      it; it pins a MEASURED ceiling, asserted loudly.
//!
//! Sequencing law: the interceptor state is process-global atomics, so
//! all courts run inside ONE #[test] fn (the harness runs tests on
//! sibling threads whose allocations would pollute a global window).
//!
//! No mocks. No Process.sleep. Final state asserted.

use std::alloc::System;

use cgap_bn3::noalloc::{na_compile, NaEngine, NaMatches};
use cgap_bn3::sigil::{encode_term, Sigil};
use cgap_bn3::unify::{unify_to_fixpoint, BodyAtom, Rule, StatementColumns};
use cgap_gate::penstock::{
    ingress, FrameView, CGAP_OK, HEADER_LEN, HEADER_VERSION, MAGIC, OABI_DIGEST_LEN,
    OFF_ATTESTATION, OFF_FLAGS, OFF_MAGIC, OFF_OABI_DIGEST, OFF_PAYLOAD_BYTE_LEN, OFF_VERSION,
    P28_FLAG_ROOT, P28_LICENSE_MAGIC, P28_SAFE_HARBOR_CEILING_CENTS, P28_VERSION,
};
use cgap_parquet::mmap_writer::{RingSink, SlabView, StaticPageWriter};
use cgap_parquet::ParquetError;

// cgap-core's lib.rs is lane-frozen (off limits), so the interceptor
// module is compiled into this binary directly from its owned source
// file. If a later lane lands `pub mod tracking_alloc;` in cgap-core's
// lib.rs, this include becomes a plain `use cgap_core::tracking_alloc;`.
#[path = "../../cgap-core/src/tracking_alloc.rs"]
mod tracking_alloc;

use tracking_alloc::{last_breach, reset_breach, start_hot_path_budget, window_counts};

#[global_allocator]
static AE7_ALLOC: tracking_alloc::EnclaveTrackingAlloc<System> =
    tracking_alloc::EnclaveTrackingAlloc { inner: System };

/// Canonical splitmix64 (SEED LAW: 0xBF58_476D_1CE4_E5B9 multiplier,
/// corpus seed 0xA20C_0FFE_E5A7_E000 per repo CLAUDE.md; the bn3
/// lever_courts fixture uses the multiplier as its seed, mirrored here).
struct CourtRng {
    state: u64,
}

impl CourtRng {
    fn new(seed: u64) -> Self {
        CourtRng { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_490B_BADD_03);
        z ^ (z >> 31)
    }

    fn pooled(&mut self, pool: u64) -> u64 {
        BASE + (self.next() % pool)
    }
}

const BASE: u64 = 0x000A_0000_0000_0000;
const SEED: u64 = 0xBF58_476D_1CE4_E5B9;
const S_POOL: u64 = 8;
const P_POOL: u64 = 4;
const O_POOL: u64 = 16;

fn cst(v: u64) -> u64 {
    encode_term(Sigil::SlotConst, v).expect("in-range payload")
}

fn vr(r: u64) -> u64 {
    encode_term(Sigil::VarUni, r).expect("in-range register")
}

/// The two-atom join rule (lever_courts fixture): (?x P ?y), (?y Q ?z).
fn court_rule() -> Rule {
    Rule {
        rule_id: 7,
        body: vec![
            BodyAtom::new(vr(0), cst(BASE + 1), vr(1)),
            BodyAtom::new(vr(1), cst(BASE + 2), vr(2)),
        ],
        head: [vr(0), cst(BASE + 3), vr(2)],
    }
}

fn corpus(n: usize) -> (Vec<u64>, Vec<u64>, Vec<u64>) {
    let mut rng = CourtRng::new(SEED);
    let (mut s, mut p, mut o) = (
        Vec::with_capacity(n),
        Vec::with_capacity(n),
        Vec::with_capacity(n),
    );
    for _ in 0..n {
        s.push(rng.pooled(S_POOL));
        p.push(rng.pooled(P_POOL));
        o.push(rng.pooled(O_POOL));
    }
    (s, p, o)
}

/// Zeroed-box helper (lever_courts `boxed_zeroed_na` pattern): the
/// noalloc engine/match buffers are too large for a stack frame.
fn boxed_zeroed<T>() -> Box<T> {
    let v = vec![0u8; core::mem::size_of::<T>()];
    unsafe { Box::from_raw(Box::into_raw(v.into_boxed_slice()) as *mut T) }
}

/// SP2-pattern admitted frame (128 B header + payload, ROOT lineage).
fn admitted_frame() -> Vec<u8> {
    let payload_len = 32usize;
    let mut buf = vec![0u8; HEADER_LEN + payload_len];
    buf[OFF_MAGIC..OFF_MAGIC + 4].copy_from_slice(&MAGIC);
    buf[OFF_VERSION] = HEADER_VERSION;
    buf[OFF_FLAGS] = 0;
    buf[OFF_PAYLOAD_BYTE_LEN..OFF_PAYLOAD_BYTE_LEN + 8]
        .copy_from_slice(&(payload_len as u64).to_le_bytes());
    for b in buf[OFF_OABI_DIGEST..OFF_OABI_DIGEST + OABI_DIGEST_LEN].iter_mut() {
        *b = 0xA5;
    }
    buf[OFF_ATTESTATION..OFF_ATTESTATION + 4].copy_from_slice(&P28_LICENSE_MAGIC);
    buf[OFF_ATTESTATION + 4..OFF_ATTESTATION + 6].copy_from_slice(&P28_VERSION.to_le_bytes());
    buf[OFF_ATTESTATION + 6..OFF_ATTESTATION + 8].copy_from_slice(&P28_FLAG_ROOT.to_le_bytes());
    buf[OFF_ATTESTATION + 8..OFF_ATTESTATION + 16]
        .copy_from_slice(&P28_SAFE_HARBOR_CEILING_CENTS.to_le_bytes());
    buf
}

/// One steady-state seal transaction (SP6 `seal_tx` pattern): writer
/// minted from the PRE-LEASED block, slab write, seal, ring flush.
fn seal_tx(buf: &mut [u8], lanes: &[u32], ring: &mut RingSink) -> Result<usize, ParquetError> {
    let slab = SlabView {
        lanes: &[lanes],
        rows: lanes.len() as u32,
        bits: 8,
    };
    let mut writer: StaticPageWriter<'_> = StaticPageWriter::new(buf);
    writer.write_slab(&slab)?;
    let sealed = writer.seal(None)?;
    let n = sealed.bytes().len();
    sealed.flush_to(ring)?;
    Ok(n)
}

/// Court 0 body — ANTI-VACUITY positive control: one Vec alloc under
/// budget(0,0) MUST breach, record the site + counts (the interceptor
/// serves poison so the process survives; the court asserts the record).
fn positive_control_breaches() {
    reset_breach();
    let _g = start_hot_path_budget("ae7-positive-control", 0, 0);
    let v: Vec<u8> = Vec::with_capacity(8);
    assert!(
        v.capacity() >= 8,
        "poison-served alloc keeps the caller alive"
    );
    drop(v);
    drop(_g);
    let b = last_breach()
        .expect("budget(0,0) MUST breach on a Vec alloc — a court that cannot flip is FAIL");
    assert_eq!(b.site, "ae7-positive-control", "breach must name the site");
    assert_eq!(b.allocs, 1, "one tracked alloc at breach");
    assert!(b.bytes >= 8, "breach byte count must cover the 8 B request");
    assert!(
        b.offending_bytes >= 8,
        "offending size must cover the request"
    );
    reset_breach();
}

/// Courts 1-4 in one driver (process-global budget state; see file doc).
#[test]
fn ae7_alloc_boundary_courts() {
    // ---- Court 0: positive control (anti-vacuity; MUST flip). --------
    positive_control_breaches();

    // ---- Court 1: penstock ingress, 16-frame class (SP2). -------------
    let frames: Vec<Vec<u8>> = (0..16).map(|_| admitted_frame()).collect();
    let mut view = FrameView {
        payload: &[],
        prefilter: 0,
        escalate: false,
    };
    let g = start_hot_path_budget("ae7-penstock-ingress-16", 0, 0);
    for f in &frames {
        let v = ingress(f, f.len(), Some(&mut view));
        assert_eq!(v, CGAP_OK, "ingress must admit the lawful frame");
    }
    let counts = window_counts();
    drop(g);
    if let Some(b) = last_breach() {
        panic!("AE7 COURT FLIP: penstock ingress window breached: {b:?}");
    }
    assert_eq!(counts, (0, 0), "ingress window allocs/bytes (SP2 law)");
    reset_breach();

    // ---- Court 2: parquet seal steady-state (MB7 0/0). ---------------
    let domain: Vec<u32> = (0..=255u32).collect();
    let lanes: Vec<u32> = (0..16384).map(|i| domain[(i * 7 + 1) % 256]).collect();
    let mut block = vec![0u8; 65_536];
    let spill_path = std::env::temp_dir().join("ae7-alloc-boundary.spill");
    let _ = std::fs::remove_file(&spill_path);
    let spill = std::fs::File::create(&spill_path).expect("spill file");
    let mut ring = RingSink::new(Some(spill));
    seal_tx(&mut block, &lanes, &mut ring).expect("warm-up tx (lazy paths)");
    let g = start_hot_path_budget("ae7-parquet-seal-steady", 0, 0);
    let mut tx_bytes = 0usize;
    for _ in 0..16 {
        tx_bytes += seal_tx(&mut block, &lanes, &mut ring).expect("steady tx");
    }
    let counts = window_counts();
    drop(g);
    assert!(
        tx_bytes > RingSink::CAP,
        "window must include a real rotation drain"
    );
    if let Some(b) = last_breach() {
        panic!("AE7 COURT FLIP: parquet seal steady-state window breached: {b:?}");
    }
    assert_eq!(
        counts,
        (0, 0),
        "seal steady-state allocs/bytes (MB7 0/0 law)"
    );
    reset_breach();
    let _ = std::fs::remove_file(&spill_path);

    // ---- Court 3: bn3 noalloc radix probe post-warmup (w5-l3). -------
    let n = 4096usize;
    // Pools narrowed exactly like lever_courts::l3_noalloc_radix_
    // differential at n=4096 (pooled(1024)/4/1024) so the joined match
    // count stays inside the noalloc arm's NA_MAX_MATCHES.
    let mut rng = CourtRng::new(SEED);
    let (mut s, mut p, mut o) = (
        Vec::with_capacity(n),
        Vec::with_capacity(n),
        Vec::with_capacity(n),
    );
    for _ in 0..n {
        s.push(rng.pooled(1024));
        p.push(rng.pooled(4));
        o.push(rng.pooled(1024));
    }
    let (s, p, o) = (s, p, o);
    let rule = court_rule();
    let na_rule = na_compile(&rule).expect("na compile");
    // Engine/match buffers + corpus are ALL setup-phase (pre-window).
    let mut eng = boxed_zeroed::<NaEngine>();
    let mut out = boxed_zeroed::<NaMatches>();
    cgap_bn3::noalloc::na_unify_radix(&na_rule, &cols_for(&s, &p, &o), &mut eng, &mut out)
        .expect("warm probe");
    assert!(out.len > 0, "court must yield joins (anti-vacuity)");
    let cols = cols_for(&s, &p, &o);
    let g = start_hot_path_budget("ae7-bn3-radix-probe", 0, 0);
    for _ in 0..8 {
        cgap_bn3::noalloc::na_unify_radix(&na_rule, &cols, &mut eng, &mut out)
            .expect("steady probe");
        assert!(out.len > 0, "steady probe must yield joins");
    }
    let counts = window_counts();
    drop(g);
    if let Some(b) = last_breach() {
        panic!("AE7 COURT FLIP: bn3 radix probe window breached: {b:?}");
    }
    assert_eq!(
        counts,
        (0, 0),
        "bn3 probe window allocs/bytes (w5-l3 alloc-zero law)"
    );
    reset_breach();

    // ---- Court 4: H8 HONESTY — fixpoint is NOT zero-alloc. -----------
    // Documented budget, not 0/0: the `seen` BTreeSet is O(store) node
    // allocs (w6-bn3-nostd-borrowed-driver); H8 forbids wrapping the
    // fixpoint in a 0 budget. The ceiling below is the measured bound
    // for THIS exact 4096-row corpus (see lane receipt for the witness
    // run) with headroom; a regression beyond it flips the court.
    let (s, p, o) = corpus(4096);
    let cols = cols_for(&s, &p, &o);
    let rules = vec![court_rule()];
    let g = start_hot_path_budget(
        "ae7-bn3-fixpoint",
        AE7_FP_BUDGET_ALLOCS,
        AE7_FP_BUDGET_BYTES,
    );
    let batches = unify_to_fixpoint(&rules, &cols).expect("fixpoint ok");
    let counts = window_counts();
    drop(g);
    assert!(
        !batches.is_empty(),
        "fixpoint must emit batches (anti-vacuity)"
    );
    if let Some(b) = last_breach() {
        panic!("AE7 COURT FLIP: bn3 fixpoint exceeded the H8 ceiling: {b:?}");
    }
    println!("AE7-FP-MEASURE allocs={} bytes={}", counts.0, counts.1);
    assert!(
        counts.0 <= AE7_FP_BUDGET_ALLOCS && counts.1 <= AE7_FP_BUDGET_BYTES,
        "fixpoint counts {counts:?} exceed the H8 ceiling"
    );
    reset_breach();
}

/// AE7 fixpoint honesty ceiling (H8): measured this session on the exact
/// 4096-row lever_courts corpus (see lane receipt). NOT zero.
/// AE7 fixpoint honesty ceiling (H8): MEASURED this session on the exact
/// 4096-row lever_courts corpus — full-window actuals 278 allocs /
/// 14_216_246 bytes (AE7-FP-MEASURE witness, `cargo test -p cgap --test
/// alloc_boundary -- --nocapture`, exit 0). Ceiling = 2x measured
/// (allocs rounded up to 1024). NOT zero; a regression past 2x flips.
const AE7_FP_BUDGET_ALLOCS: usize = 1024;
const AE7_FP_BUDGET_BYTES: usize = 33_554_432;

fn cols_for<'a>(s: &'a [u64], p: &'a [u64], o: &'a [u64]) -> StatementColumns<'a> {
    StatementColumns::new(s, p, o).expect("columns")
}
