//! POSH alloc sentry court (lane POSH-4, v26.11.2).
//!
//! Zero-heap sentry over the ingress->seal pipeline stage pair, with an
//! honesty scoping for the bn3 closure fixpoint: the fixpoint's owned
//! delta-tail buffers (W6-BFX) allocate by construction, and C03 pinned
//! that the warmup call allocates
//! (crates/cgap-bn3/tests/edge_closure_courts.rs e5). So the sentry
//! window claims zero ONLY over ring absorb + BLAKE3 seal; the fixpoint
//! runs under a MEASURED window whose counts are recorded verbatim as
//! an honest finding, never folded into the zero claim.
//!
//! Pipeline: splitmix64 corpus (frozen seed 0xA20C_0FFE_E5A7_E000) ->
//! pack-v2 window serialization (1,365 facts, pack_demux::FACT_WINDOW)
//! -> StdIngressRing absorb (real push/pop API) -> ring byte fidelity ->
//! BLAKE3 receipt seal (real cgap-blake3) -> receipt non-zero.
//!
//! Courts, in order (allocator state is process-global, ONE test fn):
//! (1) positive control FIRST: a deliberate heap alloc inside an open
//!     zero-budget window MUST be observed (allocs>=1, bytes>=512) --
//!     witnessed before the clean run is trusted;
//! (2) sentry window: allocs==0 AND bytes==0 across the 1,365-fact
//!     window, receipt digest non-zero;
//! (3) honesty window: bn3 transitive-closure fixpoint over the
//!     window-derived edge set, converges, derived facts recorded, its
//!     measured alloc/byte counts printed as the honest finding.

#[path = "../src/ingress_ring.rs"]
mod ingress_ring;

use cgap_bn3::sigil::{encode_term, Sigil};
use cgap_bn3::unify::{unify_to_fixpoint, BodyAtom, Rule, StatementColumns};

use ingress_ring::StdIngressRing;
use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering as AOrd};

// ---------------------------------------------------------------------------
// Counting global allocator: scoped window counting BOTH call count and
// byte volume. Mirrors soc_full_chip_court.rs:57-104 exactly, with the
// bytes ledger added for the bytes==0 assertion.
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

// ---------------------------------------------------------------------------
// Corpus: splitmix64 over the frozen corpus seed, mirrors
// soc_full_chip_court.rs:109-130 (same law, same seed, never varied).
// ---------------------------------------------------------------------------

const CORPUS_SEED: u64 = 0xA20C_0FFE_E5A7_E000;
const FACT_CLASSES: usize = 128;
/// 1,365-fact window (pack_demux::FACT_WINDOW, coprime stride).
const WINDOW: usize = 1_365;
/// 12 (pack header) + 1365*(varint+32 body) + 20 (SHA-1 trailer).
const MAX_WINDOW_BYTES: usize = 12 + WINDOW * 34 + 20;
const FRAME_BYTES: usize = 512;

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn fact_class(i: usize) -> usize {
    let mut s = CORPUS_SEED ^ (i as u64);
    (splitmix64(&mut s) % FACT_CLASSES as u64) as usize
}

fn class_payload(class: usize, out: &mut [u8; 32]) {
    let mut s = CORPUS_SEED ^ (class as u64).wrapping_mul(0x1CE4_E5B9);
    for chunk in out.chunks_mut(8) {
        let w = splitmix64(&mut s).to_le_bytes();
        chunk.copy_from_slice(&w[..chunk.len()]);
    }
}

fn put_varint_header(out: &mut [u8], pos: &mut usize, ty: u8, size: u64) {
    let mut b = (ty << 4) | ((size & 0x0F) as u8);
    let mut rest = size >> 4;
    if rest > 0 {
        b |= 0x80;
    }
    out[*pos] = b;
    *pos += 1;
    while rest > 0 {
        let mut nb = (rest & 0x7F) as u8;
        rest >>= 7;
        if rest > 0 {
            nb |= 0x80;
        }
        out[*pos] = nb;
        *pos += 1;
    }
}

// ---------------------------------------------------------------------------
// Pack-v2 window serialization: byte law mirrors pack_write ("PACK",
// u32 BE v2, u32 BE object count, varint headers, SHA-1 trailer).
// ---------------------------------------------------------------------------

fn serialize_window(payloads: &[[u8; 32]; FACT_CLASSES], buf: &mut [u8]) -> usize {
    assert!(WINDOW <= u32::MAX as usize);
    let mut pos = 12usize;
    buf[..4].copy_from_slice(b"PACK");
    buf[4..8].copy_from_slice(&2u32.to_be_bytes());
    buf[8..12].copy_from_slice(&(WINDOW as u32).to_be_bytes());
    for i in 0..WINDOW {
        let class = fact_class(i);
        let p = &payloads[class];
        put_varint_header(buf, &mut pos, 3, p.len() as u64);
        buf[pos..pos + 32].copy_from_slice(p);
        pos += 32;
    }
    let trailer = cgap_git::pack_write::sha1(&buf[..pos]);
    buf[pos..pos + 20].copy_from_slice(&trailer);
    pos + 20
}

// ---------------------------------------------------------------------------
// Sentry stage A (inside the zero window): ring absorb + BLAKE3 seal.
// Every buffer is stack-owned; no Vec, no String, no Box.
// ---------------------------------------------------------------------------

fn stage_ring_and_seal(
    window_bytes: &[u8],
    ring: &mut StdIngressRing,
    relay: &mut [u8; MAX_WINDOW_BYTES],
) -> (usize, [u8; 32]) {
    // Absorb: feed the window through the ring in fixed 512-B frames.
    let mut total_in = 0usize;
    let mut total_out = 0usize;
    let mut frame = [0u8; FRAME_BYTES];
    let mut rest = window_bytes;
    while !rest.is_empty() {
        let n = rest.len().min(FRAME_BYTES);
        frame[..n].copy_from_slice(&rest[..n]);
        ring.push(&frame[..n]).expect("ring push fits capacity");
        rest = &rest[n..];
        total_in += n;
        // Drain eagerly so the ring never saturates mid-window.
        while ring.pending() > 0 {
            let got = ring.pop(&mut frame).expect("ring pop");
            relay[total_out..total_out + got].copy_from_slice(&frame[..got]);
            total_out += got;
        }
    }
    assert_eq!(total_in, total_out, "ring byte fidelity");
    assert_eq!(total_out, window_bytes.len(), "window bytes preserved");
    // Seal: BLAKE3 over the relayed window (real cgap-blake3).
    let root = cgap_blake3::hash(&relay[..total_out]);
    (total_out, root)
}

// ---------------------------------------------------------------------------
// Stage B (honesty window): bn3 transitive-closure fixpoint over the
// edge set the window facts encode. Helpers mirror
// cgap-bn3/tests/edge_closure_courts.rs:60-98 (term words, closure rule,
// SoA columns leaked once, outside every window).
// ---------------------------------------------------------------------------

const E_PRED: u64 = 0x000A_0000_0000_0001;

fn cst(v: u64) -> u64 {
    encode_term(Sigil::SlotConst, v).expect("in-range payload")
}
fn vr(r: u64) -> u64 {
    encode_term(Sigil::VarUni, r).expect("in-range register")
}

fn closure_rule() -> Rule {
    Rule {
        rule_id: 42,
        body: vec![
            BodyAtom::new(vr(0), cst(E_PRED), vr(1)),
            BodyAtom::new(vr(1), cst(E_PRED), vr(2)),
        ],
        head: [vr(0), cst(E_PRED), vr(2)],
    }
}

/// Window facts -> deduped (s,o) edge pairs over the 128-class payload
/// space (chain edges class(i) -> class(i+1 mod WINDOW)).
fn window_edges(edges: &mut BTreeSet<(u64, u64)>) {
    edges.clear();
    for i in 0..WINDOW {
        let a = fact_class(i) as u64;
        let b = fact_class((i + 1) % WINDOW) as u64;
        if a != b {
            edges.insert((a, b));
        }
    }
}

fn cols_of(edges: &BTreeSet<(u64, u64)>) -> StatementColumns<'static> {
    let mut s: std::vec::Vec<u64> = std::vec::Vec::new();
    let mut p: std::vec::Vec<u64> = std::vec::Vec::new();
    let mut o: std::vec::Vec<u64> = std::vec::Vec::new();
    for (a, b) in edges {
        s.push(cst(*a));
        p.push(cst(E_PRED));
        o.push(cst(*b));
    }
    let bs: &'static mut [u64] = Box::leak(s.into_boxed_slice());
    let bp: &'static mut [u64] = Box::leak(p.into_boxed_slice());
    let bo: &'static mut [u64] = Box::leak(o.into_boxed_slice());
    StatementColumns::new(bs, bp, bo).expect("aligned columns")
}

// ---------------------------------------------------------------------------
// The court. ONE #[test] fn (allocator window state is process-global).
// ---------------------------------------------------------------------------

#[test]
fn posh_alloc_sentry_driver() {
    // ---- Court 1: positive control FIRST (the anti-vacuity flip) ----
    // A deliberate heap allocation inside an open zero window MUST be
    // observed. If this cannot flip, the sentry is VACUOUS and the
    // clean run below proves nothing.
    open_window();
    let controlled: std::vec::Vec<u8> = std::vec![7u8; 512];
    let (pc_allocs, pc_bytes) = close_window();
    std::mem::drop(controlled);
    assert!(
        pc_allocs >= 1 && pc_bytes >= 512,
        "POSITIVE CONTROL: zero window must observe a deliberate alloc \
         (got allocs={pc_allocs}, bytes={pc_bytes}) -- sentry would be vacuous"
    );

    // ---- Court 2: the zero-alloc sentry over ring absorb + seal ----
    let mut payloads = [[0u8; 32]; FACT_CLASSES];
    for (c, slot) in payloads.iter_mut().enumerate() {
        class_payload(c, slot);
    }
    let mut window_buf = [0u8; MAX_WINDOW_BYTES];
    let wlen = serialize_window(&payloads, &mut window_buf);
    assert_eq!(
        wlen,
        12 + WINDOW * 34 + 20,
        "window byte law: 1,365 facts, 32-B bodies + 2-B varint header"
    );
    let mut ring = StdIngressRing::new();
    let mut relay = [0u8; MAX_WINDOW_BYTES];
    open_window();
    let (relayed, root) = stage_ring_and_seal(&window_buf[..wlen], &mut ring, &mut relay);
    let (sentry_allocs, sentry_bytes) = close_window();
    assert_eq!(relayed, wlen, "1,365-fact window relayed end to end");
    assert_ne!(root, [0u8; 32], "BLAKE3 receipt seal is non-zero");
    assert_eq!(
        (sentry_allocs, sentry_bytes),
        (0, 0),
        "SENTRY: ring absorb + seal over the 1,365-fact window must be \
         heap-zero (observed allocs={sentry_allocs}, bytes={sentry_bytes})"
    );

    // ---- Court 3: honesty window over the bn3 closure fixpoint ----
    // The fixpoint's owned delta-tail buffers allocate by construction
    // (W6-BFX); C03 pinned warmup allocation. We DO NOT claim zero here:
    // we MEASURE and record, asserting only convergence and that the
    // derived fact set is real (edges beyond the base chains exist and
    // the fixpoint's batches carry the E predicate).
    let mut edges: BTreeSet<(u64, u64)> = BTreeSet::new();
    window_edges(&mut edges);
    let base_len = edges.len();
    assert!(
        base_len >= FACT_CLASSES / 2,
        "window edge corpus is non-trivial (got {base_len} edges)"
    );
    let cols = cols_of(&edges);
    open_window();
    let batches = unify_to_fixpoint(&[closure_rule()], &cols).expect("fixpoint converges");
    let mut derived_total = 0usize;
    for b in &batches {
        for &p_val in b.rows.p.iter().take(b.rows.len()) {
            assert_eq!(p_val, cst(E_PRED), "derived predicate is E");
            derived_total += 1;
        }
    }
    let (fp_allocs, fp_bytes) = close_window();
    assert!(
        fp_allocs >= 1,
        "C03 PIN: the bn3 fixpoint warmup call allocates (got \
         {fp_allocs}) -- if this ever reads 0, tighten the sentry \
         scoping to cover the fixpoint too"
    );
    println!(
        "POSH-4 HONEST FINDING: bn3 unify_to_fixpoint over {base_len} \
         window-derived edges allocated {fp_allocs} times / {fp_bytes} \
         bytes (warmup, measured; not folded into the zero claim; C03 \
         pinned warmup allocation in cgap-bn3/tests/edge_closure_courts.rs e5)"
    );
    assert!(
        !batches.is_empty(),
        "fixpoint produced derivation batches (non-vacuous closure)"
    );
    assert!(derived_total > 0, "fixpoint derived real facts");
    // Honest standing: the scoping decision is itself asserted --
    // the sentry window NEVER covered the fixpoint (it closed before).
    println!(
        "POSH-4 TRANSCRIPT: positive control allocs={pc_allocs} \
         bytes={pc_bytes}; sentry window allocs={sentry_allocs} \
         bytes={sentry_bytes} over {relayed} relayed bytes; receipt \
         root[0..4]={:?}; fixpoint measured allocs={fp_allocs} \
         bytes={fp_bytes}, batches={} derived_rows={derived_total}",
        &root[0..4],
        batches.len(),
    );
}
