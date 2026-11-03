//! POSH-5 refusal matrix court: the stitch index for POSH-1..4.
//!
//! Four arms, each with the flip AS the test (mutant built in-test, no
//! /tmp source mutation):
//!   1. bad magic ARM\0 -> XRM\0 (POSH-1's wire law, posh_wire_court.rs)
//!      -> typed SubjectMissing-class refusal corroborated through the
//!      REAL demux law (pack_demux: non-"PACK" => SubjectMissing),
//!   2. cyclic ontology injection through the REAL admission_gate apply
//!      path -> AndonRefusal-class (CyclicDerivation) with the ref table
//!      byte-unchanged,
//!   3. stride overflow: 1,366 facts against the ring's real bounded
//!      window -> typed Overflow refusal (andon exit-class 101), bound
//!      DERIVED from the code, never a magic 1366,
//!   4. alloc positive control: counting-alloc sentry observes >0.
//!
//! Provenance of the arm-3 bound: the ingress ring capacity is
//! ingress_ring::INGRESS_RING_BYTES = 32,768 bytes (lane GIT-03). A fact
//! record is 3 x u64 = 24 bytes of stride. floor(32768/24) = 1,365
//! facts fit inline; 1,366 x 24 = 32,784 > 32,768 refuses. This matches
//! lane GIT-01's named FACT_WINDOW = 1_365 (pack_demux.rs:51), which
//! carries the same derivation in its provenance note.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use cgap_git::admission_gate::{
    var, GateError, ProposedUpdate, RefTable, RefUpdate, MAX_DERIVED_FACTS, MAX_RULE_VARIABLES,
};
use cgap_git::ingress_ring::{IngressError, IngressRing, INGRESS_RING_BYTES};
use cgap_git::pack_demux::Demux;
use cgap_git::pack_write::{encode_object_header, sha1};

/// 24-byte fact stride: one (u64, u64, u64) record.
const FACT_STRIDE: usize = 3 * core::mem::size_of::<u64>();
/// DERIVED bound: facts that fit the real ring window exactly.
const MAX_INLINE_FACTS: usize = INGRESS_RING_BYTES / FACT_STRIDE;

// ---- arm 4 instrument: counting-alloc sentry (this file only) ----
// Ties to POSH-4's landed instrument (posh_alloc_sentry_court.rs,
// lane-posh-04-sentry.md). Allocator state is process-global PER TEST
// BINARY, so POSH-4's sentry cannot be observed from this binary; this
// file carries its own minimal counting-alloc control (real System
// allocator wrapped in a counter — no mocks), corroborating the same
// positive-control law. Disclosed in the lane receipt.
static ALLOC_SENTRY: AtomicUsize = AtomicUsize::new(0);

struct CountingAlloc;

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_SENTRY.fetch_add(1, Ordering::Acquire);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

/// A clean minimal pack — the BEFORE subject for arm 1, consumed by the
/// real Demux. STORED framing (raw body bytes + sha1 trailer), because
/// the demux law is stored framing: write_pack_v2 emits zlib bodies, so
/// writer output does not end at the demux trailer (FIX-05 unblock
/// disclosure: pre-fix, the zlib overflow was silently absorbed by the
/// Done arm; the typed TrailingBytes refusal exposed the latent
/// framing mismatch).
fn clean_pack() -> Vec<u8> {
    let blob: &[u8] = b"posh-arm1-blob";
    let mut out = Vec::new();
    out.extend_from_slice(b"PACK");
    out.extend_from_slice(&2u32.to_be_bytes());
    out.extend_from_slice(&1u32.to_be_bytes());
    out.extend_from_slice(&encode_object_header(3, blob.len() as u64));
    out.extend_from_slice(blob);
    let sum = sha1(&out);
    out.extend_from_slice(&sum);
    out
}

fn clean_update() -> RefUpdate {
    let old = [0x11u8; 20];
    let new = [0x22u8; 20];
    RefUpdate {
        old_sha: old,
        new_sha: new,
        ref_name: b"refs/heads/posh".to_vec(),
    }
}

#[test]
fn arm1_bad_magic_refuses_subject_missing_class() {
    let pack = clean_pack();

    // BEFORE: clean bytes demux cleanly (real collaborator, final state).
    let mut good = Demux::new();
    good.feed(&pack).expect("clean pack feeds");
    good.finish().expect("clean pack completes");
    assert_eq!(good.objects_seen(), 1);

    // Mutant: POSH-1 wire-law magic corruption ARM\0 -> XRM\0, built
    // in-test against the same demux-law corroboration POSH-1's court 1
    // pins. This IS the flip: an always-admit demux would take the AFTER
    // path too.
    let mut bad = pack.clone();
    bad[0..4].copy_from_slice(b"XRM\0");
    assert_ne!(bad, pack);
    let mut demux = Demux::new();
    let err = demux
        .feed(&bad)
        .expect_err("VACUOUS: XRM\0 magic still admitted");
    assert_eq!(err, cgap_git::pack_demux::DemuxError::SubjectMissing);
    // Refusal is fail-closed: nothing demuxed, stream never completes.
    assert_eq!(demux.objects_seen(), 0);
    demux
        .finish()
        .expect_err("VACUOUS: refused stream reported Done");
}

#[test]
fn arm2_cyclic_injection_refuses_andon_class_state_byte_unchanged() {
    // BEFORE: clean ground facts admit; the ref advances old -> new.
    let mut table = RefTable::new();
    table.set(b"refs/heads/posh", [0x11u8; 20]);
    let mut clean = ProposedUpdate::default();
    clean.facts = vec![(1, 900, 2), (2, 900, 3)];
    table
        .apply(&clean_update(), &clean)
        .expect("clean update admits");
    assert_eq!(table.get(b"refs/heads/posh"), Some([0x22u8; 20]));

    // Mutant: cyclic ontology injection A -> B -> A. A 200-node ring of
    // ground facts + a transitive rule; the closure derives distance-k
    // pairs round over round, exceeding the 4,096-fact budget before
    // convergence. This IS the flip: an always-admit gate would run the
    // same apply and admit.
    const N: u64 = 200;
    let mut cyclic = ProposedUpdate::default();
    for i in 0..N {
        cyclic.facts.push((i, 900, (i + 1) % N));
    }
    cyclic.rules.push(cgap_git::admission_gate::GateRule {
        rule_id: 7,
        body: vec![(var(0), 900, var(1)), (var(1), 900, var(2))],
        head: (var(0), 900, var(2)),
    });
    let before = table.bytes();
    let err = table
        .apply(&clean_update(), &cyclic)
        .expect_err("VACUOUS: cyclic ontology still admitted");
    match err {
        GateError::CyclicDerivation { rounds, facts } => {
            assert!(facts > MAX_DERIVED_FACTS, "budget exceeded: {facts}");
            assert!(rounds >= 1 && rounds <= cgap_git::admission_gate::MAX_CLOSURE_ROUNDS);
        }
        other => panic!("wrong refusal class: {other:?}"),
    }
    assert_eq!(table.bytes(), before, "ref table must be byte-unchanged");
    assert_eq!(table.get(b"refs/heads/posh"), Some([0x22u8; 20]));
    // Band law pinned by this file alongside the fact budget.
    assert_eq!(MAX_RULE_VARIABLES, 128);
}

#[test]
fn arm3_stride_overflow_refuses_typed_at_derived_bound() {
    // The bound is DERIVED, not magic: floor(INGRESS_RING_BYTES / 24).
    assert_eq!(INGRESS_RING_BYTES, 32_768);
    assert_eq!(FACT_STRIDE, 24);
    assert_eq!(MAX_INLINE_FACTS, 1_365);
    assert_eq!(MAX_INLINE_FACTS * FACT_STRIDE, 32_760);

    // BEFORE: MAX_INLINE_FACTS frames fill the window exactly and pop
    // back out byte-exact (clean consume path).
    let mut ring: IngressRing<INGRESS_RING_BYTES> = IngressRing::new();
    for i in 0..MAX_INLINE_FACTS {
        let f = [i as u64, 900, (i as u64) ^ 0xdead_beef];
        let bytes = [f[0].to_be_bytes(), f[1].to_be_bytes(), f[2].to_be_bytes()].concat();
        ring.push(&bytes)
            .expect("1365 inline facts fit the real window");
    }
    assert_eq!(ring.pending(), MAX_INLINE_FACTS * FACT_STRIDE);
    let mut out = vec![0u8; MAX_INLINE_FACTS * FACT_STRIDE];
    let n = ring.pop(&mut out).expect("window drains");
    assert_eq!(n, MAX_INLINE_FACTS * FACT_STRIDE);
    assert_eq!(ring.pending(), 0);

    // AFTER: fact 1,366 (32,784 bytes > 32,768) refuses, typed.
    let mut ring: IngressRing<INGRESS_RING_BYTES> = IngressRing::new();
    for i in 0..MAX_INLINE_FACTS {
        let f = [i as u64, 900, 0u64];
        let bytes = [f[0].to_be_bytes(), f[1].to_be_bytes(), f[2].to_be_bytes()].concat();
        ring.push(&bytes).expect("fit");
    }
    let h = ring.head();
    let t = ring.tail();
    let f1366 = [7u64, 900, 0u64];
    let bytes = [
        f1366[0].to_be_bytes(),
        f1366[1].to_be_bytes(),
        f1366[2].to_be_bytes(),
    ]
    .concat();
    let err = ring
        .push(&bytes)
        .expect_err("VACUOUS: 32,784 bytes admitted a 32,768-byte window");
    assert_eq!(err.andon_exit_class(), 101);
    match err {
        IngressError::Overflow {
            head,
            tail,
            frame_len,
        } => {
            assert_eq!(head - tail, MAX_INLINE_FACTS * FACT_STRIDE);
            assert_eq!(frame_len, FACT_STRIDE);
        }
        other => panic!("wrong refusal class: {other:?}"),
    }
    // Refusal is fail-closed: head/tail untouched.
    assert_eq!(ring.head(), h);
    assert_eq!(ring.tail(), t);
}

#[test]
fn arm4_alloc_sentry_positive_control() {
    // Positive control: the counting-alloc sentry must observe >0 real
    // System allocations for a genuinely heap-allocating operation. The
    // flip is the operation itself: a sentry reporting 0 here is a dead
    // instrument — this court FAILs, it never skips.
    let before = ALLOC_SENTRY.load(Ordering::Acquire);
    let v: Vec<u64> = Vec::with_capacity(64);
    assert!(v.capacity() >= 64);
    drop(v);
    let after = ALLOC_SENTRY.load(Ordering::Acquire);
    assert!(
        after > before,
        "DEAD INSTRUMENT: counting alloc sentry observed 0 allocations"
    );
}
