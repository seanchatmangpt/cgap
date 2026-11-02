//! Repair-11 (lane F9) courts: allocation-elimination falsifier + the
//! flat SeenSet semantics court. Chicago: real collaborators, final
//! state, no mocks, no Process.sleep. The falsifier measures REAL
//! allocations via a counting global allocator — a court that cannot
//! flip is FAIL.
//!
//! H8's falsifier: allocs(N=1M) < 1000; the pre-repair measurement was
//! 1,771 allocs / ~1.68 GB churn (MB4's independent measurement: 1,117
//! / ~1.20 GB; same defect class, corpus shape differs). Both sit far
//! above the 1000 ceiling, so either number falsifies a regression to
//! per-round fresh deltas + std HashSet.
//!
//! Counters are THREAD-LOCAL: the harness runs tests in parallel, and a
//! process-global ON gate counts concurrent tests' allocations into
//! each other (witnessed this session: 7.9M allocs / 3.4 GB when two
//! N=1M courts cross-counted). Thread-local gating makes each court
//! measure only its own thread.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use cgap_storage::closure::{
    naive_closure, pair_key, semi_naive_closure, SeenSet, CORPUS_SEED,
};
use cgap_storage::dred::splitmix64;

thread_local! {
    static ON: Cell<bool> = const { Cell::new(false) };
    static ALLOCS: Cell<usize> = const { Cell::new(0) };
    static BYTES: Cell<usize> = const { Cell::new(0) };
}

struct Counting;
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        if ON.with(Cell::get) {
            ALLOCS.with(|c| c.set(c.get() + 1));
            BYTES.with(|c| c.set(c.get() + l.size()));
        }
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        if ON.with(Cell::get) {
            ALLOCS.with(|c| c.set(c.get() + 1));
            BYTES.with(|c| c.set(c.get() + n));
        }
        unsafe { System.realloc(p, l, n) }
    }
}

#[global_allocator]
static A: Counting = Counting;

/// Begin counting allocations on THIS thread only.
fn counting_on() {
    ALLOCS.with(|c| c.set(0));
    BYTES.with(|c| c.set(0));
    ON.with(|c| c.set(true));
}

fn counting_off() -> (usize, usize) {
    ON.with(|c| c.set(false));
    (ALLOCS.with(Cell::get), BYTES.with(Cell::get))
}

fn chains_corpus(n_nodes: usize, chain_len: usize) -> (Vec<u32>, Vec<u32>) {
    let mut st = CORPUS_SEED;
    let n_chains = n_nodes / chain_len;
    let mut xs: Vec<u32> = Vec::new();
    let mut zs: Vec<u32> = Vec::new();
    for c in 0..n_chains {
        for i in 0..chain_len - 1 {
            let a = (c * chain_len + i) as u32;
            xs.push(a);
            zs.push(a + 1);
        }
    }
    let mut order: Vec<usize> = (0..xs.len()).collect();
    for i in (1..xs.len()).rev() {
        let v = splitmix64(&mut st) as usize;
        order.swap(i, v % (i + 1));
    }
    order.iter().map(|&i| (xs[i], zs[i])).unzip()
}

/// Court A (falsifier): N=1M closure allocates fewer than 1000 times
/// on the calling thread. The pre-repair engine measured 1,771 allocs
/// (/tmp/f9-probe, this session) — that mutation fails this assert.
/// Semantics pinned by the deterministic corpus: 25,689,475 pairs in
/// 49 rounds (also asserted; a probing/growth bug in SeenSet changes
/// the derived set and flips this court).
#[test]
fn court_f9a_alloc_falsifier_under_1000() {
    let (ex, ez) = chains_corpus(1_048_576, 50);
    counting_on();
    let (cx, _cz, rounds) = semi_naive_closure(&ex, &ez);
    let (a, b) = counting_off();
    assert_eq!(cx.len(), 25_689_475, "derived-set size drifted");
    assert_eq!(rounds, 49);
    assert!(
        a < 1000,
        "alloc falsifier: {a} allocs / {b} bytes; ceiling 1000"
    );
}

/// Court B: SeenSet flat-table semantics. Insert-only membership must
/// be exact through many growth rehashes; a probing/grow bug flips it.
#[test]
fn court_f9b_seen_set_semantics() {
    let mut s = SeenSet::with_capacity(64);
    assert!(s.is_empty());
    assert!(s.insert(0));
    assert!(!s.insert(0));
    assert!(s.insert(u64::MAX));
    assert!(!s.insert(u64::MAX));
    assert_eq!(s.len(), 2);
    let mut s = SeenSet::with_capacity(64);
    for k in 0..50_000u64 {
        let first = s.insert(k.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        assert!(first, "key {k} reported duplicate on first insert");
    }
    assert_eq!(s.len(), 50_000);
    // Derived-set identity vs the BTreeSet oracle after the swap.
    let (ex, ez) = chains_corpus(10_000, 50);
    let (cx, cz, _) = semi_naive_closure(&ex, &ez);
    let got: std::collections::BTreeSet<u64> =
        cx.iter().zip(&cz).map(|(&x, &z)| pair_key(x, z)).collect();
    assert_eq!(got, naive_closure(&ex, &ez));
}

/// Court C: total churn on the calling thread stays strictly under
/// the MEASURED pre-repair churn (1,675,546,700 bytes, /tmp/f9-probe,
/// this session — bytes here are deterministic: the counter sums
/// requested Layout sizes, not allocator behavior). Post-repair the
/// same 25,689,475-pair derivation churns 1,607,226,664 bytes; the
/// ~4% margin is deterministic (seeded corpus, power-of-two growth).
/// The stronger falsifier is Court A (alloc count, 61x drop).
#[test]
fn court_f9c_churn_bounded() {
    let (ex, ez) = chains_corpus(1_048_576, 50);
    counting_on();
    let (cx, cz, _rounds) = semi_naive_closure(&ex, &ez);
    let (_a, b) = counting_off();
    assert_eq!(cx.len(), cz.len());
    assert_eq!(cx.len(), 25_689_475);
    assert!(
        b < 1_675_546_700,
        "churn {b} bytes exceeded pre-repair envelope 1675546700"
    );
}
