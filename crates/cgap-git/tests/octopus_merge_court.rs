#[path = "../src/octopus_merge.rs"]
mod octopus_merge;

use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use octopus_merge::{
    octopus_merge, octopus_merge_andon, HeadInput, MergeError, OctopusScratch, MAX_HEADS,
};

// --- Global counting allocator (zero-alloc steady-state court) -------

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static TEST_SERIAL: Mutex<()> = Mutex::new(());

struct CountingAlloc;

unsafe impl std::alloc::GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let p = std::alloc::System.alloc(layout);
        if !p.is_null() {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        p
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        std::alloc::System.dealloc(ptr, layout)
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: std::alloc::Layout, new_size: usize) -> *mut u8 {
        let p = std::alloc::System.realloc(ptr, layout, new_size);
        if !p.is_null() {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        p
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

// --- Seeds (SEED LAW: canonical splitmix64 seed vs frozen corpus
// constant, never conflated) -----------------------------------------

const SEED: u64 = 0xBF58_476D_1CE4_E5B9;
const CORPUS_CONST: u64 = 0xA20C_0FFE_E5A7_E000;
const N_ROWS_PER_HEAD: usize = 2_048;
const N_HEADS: usize = MAX_HEADS;

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
}

/// Deterministic corpus: distinct coordinates spread across heads by
/// an index hash; each head additionally carries a shared slice of
/// identical triples (the unify case).
fn build_corpus(n_heads: usize, n_rows: usize) -> Vec<Vec<(u64, u64, u64)>> {
    let mut rng = CourtRng::new(SEED ^ CORPUS_CONST);
    let mut universe: Vec<(u64, u64, u64)> = Vec::with_capacity(n_rows);
    let mut seen = HashSet::new();
    while universe.len() < n_rows {
        let t = (
            rng.next() & 0x000F_FFFF_FFFF_FFFF,
            rng.next() & 0x000F_FFFF_FFFF_FFFF,
            rng.next() & 0x000F_FFFF_FFFF_00FF,
        );
        if seen.insert(t) {
            universe.push(t);
        }
    }
    let mut heads: Vec<Vec<(u64, u64, u64)>> = vec![Vec::new(); n_heads];
    for (i, t) in universe.iter().enumerate() {
        let h = (i * 7919 + 13) % n_heads;
        heads[h].push(*t);
    }
    let share = n_rows / 8;
    for h in 0..n_heads {
        for t in universe.iter().take(share) {
            heads[h].push(*t);
        }
    }
    heads
}

/// Materialize SoA bitplanes for one head triple list.
fn planes(triples: &[(u64, u64, u64)]) -> (Vec<u64>, Vec<u64>, Vec<u64>) {
    let mut s = Vec::with_capacity(triples.len());
    let mut p = Vec::with_capacity(triples.len());
    let mut o = Vec::with_capacity(triples.len());
    for (a, b, c) in triples {
        s.push(*a);
        p.push(*b);
        o.push(*c);
    }
    (s, p, o)
}

/// Naive oracle: unified set of triples (HashSet).
fn oracle(heads: &[Vec<(u64, u64, u64)>]) -> HashSet<(u64, u64, u64)> {
    let mut set = HashSet::new();
    for h in heads {
        for t in h {
            set.insert(*t);
        }
    }
    set
}

fn run_merge(heads: &[HeadInput<'_>]) -> (usize, Vec<u64>, Vec<u64>, Vec<u64>) {
    let total: usize = heads.iter().map(|h| h.rows()).sum();
    let mut scratch = OctopusScratch::default();
    let mut row_refs = vec![(0u32, 0u32); total];
    let mut os = vec![0u64; total];
    let mut op = vec![0u64; total];
    let mut oo = vec![0u64; total];
    let n = octopus_merge(
        heads,
        &mut row_refs,
        &mut scratch,
        &mut os,
        &mut op,
        &mut oo,
    )
    .expect("clean corpus merges clean");
    (n, os, op, oo)
}

// --- Court 1: 20-way clean merge materializes (oracle-checked,
// closure time REPORTED, not gated) -----------------------------------

#[test]
fn twenty_way_clean_merge_materializes() {
    let _g = TEST_SERIAL.lock().unwrap();
    let heads_data = build_corpus(N_HEADS, N_ROWS_PER_HEAD * 4);
    let planes_per: Vec<_> = heads_data.iter().map(|h| planes(h)).collect();
    let ids: Vec<usize> = (0..N_HEADS).collect();
    let heads: Vec<HeadInput> = ids
        .iter()
        .zip(planes_per.iter())
        .map(|(id, (s, p, o))| HeadInput::try_new(*id, s, p, o).unwrap())
        .collect();
    let (n, os, op, oo) = run_merge(&heads);
    let merged: HashSet<(u64, u64, u64)> = (0..n).map(|i| (os[i], op[i], oo[i])).collect();
    assert_eq!(merged, oracle(&heads_data), "20-way merge == naive oracle");
    assert_eq!(n, merged.len(), "no over-emission beyond distinct triples");
    // Closure-time REPORTED measurement (Instant, min-of-3).
    let mut best = u128::MAX;
    for _ in 0..3 {
        let t0 = Instant::now();
        let r = run_merge(&heads);
        let us = t0.elapsed().as_micros();
        assert_eq!(r.0, n);
        if us < best {
            best = us;
        }
    }
    println!(
        "REPORT octopus 20-way fixpoint closure: {} us (best of 3; \
         33.3 ms release target: {})",
        best,
        if (best as u64) < 33_300 {
            "UNDER"
        } else {
            "OVER"
        }
    );
}

// --- Court 2: conflict detection across two heads --------------------

#[test]
fn conflict_detection_across_two_heads() {
    let _g = TEST_SERIAL.lock().unwrap();
    let mut rng = CourtRng::new(SEED ^ 0x5EED_0000_0000_0002);
    let s = rng.next() & 0x000F_FFFF_FFFF_FFFF;
    let p = rng.next() & 0x000F_FFFF_FFFF_FFFF;
    let h3 = vec![(s, p, 0xA11C_E000_0000_0001)];
    let h7 = vec![(s, p, 0xA11C_E000_0000_0002)];
    let ps3 = planes(&h3);
    let ps7 = planes(&h7);
    let heads = vec![
        HeadInput::try_new(3, &ps3.0, &ps3.1, &ps3.2).unwrap(),
        HeadInput::try_new(7, &ps7.0, &ps7.1, &ps7.2).unwrap(),
    ];
    let mut scratch = OctopusScratch::default();
    let mut row_refs = vec![(0u32, 0u32); 2];
    let mut os = vec![0u64; 2];
    let mut op = vec![0u64; 2];
    let mut oo = vec![0u64; 2];
    let err = octopus_merge(
        &heads,
        &mut row_refs,
        &mut scratch,
        &mut os,
        &mut op,
        &mut oo,
    )
    .expect_err("same (s,p) with different o must refuse");
    match err {
        MergeError::DomainClash {
            subject,
            predicate,
            head_a,
            head_b,
            object_a,
            object_b,
        } => {
            assert_eq!(subject, s);
            assert_eq!(predicate, p);
            assert_eq!((head_a, head_b), (3, 7), "clashing heads named");
            assert_ne!(object_a, object_b);
        }
        other => panic!("wrong refusal: {other:?}"),
    }
    assert!(
        err.to_string().contains("head 3"),
        "display names head 3: {err}"
    );
    assert!(
        err.to_string().contains("head 7"),
        "display names head 7: {err}"
    );
}

// --- Court 3: empty-head edge ----------------------------------------

#[test]
fn empty_head_edge() {
    let _g = TEST_SERIAL.lock().unwrap();
    let data = build_corpus(4, 64);
    let mut pl: Vec<(Vec<u64>, Vec<u64>, Vec<u64>)> = Vec::new();
    for (i, h) in data.iter().enumerate() {
        if i % 2 == 1 {
            pl.push(planes(&Vec::new()));
        } else {
            pl.push(planes(h));
        }
    }
    let heads: Vec<HeadInput> = pl
        .iter()
        .enumerate()
        .map(|(i, (s, p, o))| HeadInput::try_new(i, s, p, o).unwrap())
        .collect();
    let (n, os, op, oo) = run_merge(&heads);
    let merged: HashSet<(u64, u64, u64)> = (0..n).map(|i| (os[i], op[i], oo[i])).collect();
    let expect: HashSet<(u64, u64, u64)> =
        data.iter().skip(2).step_by(2).flatten().copied().collect();
    // Heads 0 and 2 are non-empty; shared slice unifies.
    let mut want = HashSet::new();
    for (i, h) in data.iter().enumerate() {
        if i % 2 == 0 {
            for t in h {
                want.insert(*t);
            }
        }
    }
    assert_eq!(merged, want, "empty heads contribute nothing");
    let _ = expect;

    // All-empty: zero heads of rows merge to 0, no refusal.
    let empty: Vec<(u64, u64, u64)> = Vec::new();
    let ep = planes(&empty);
    let eh = vec![HeadInput::try_new(0, &ep.0, &ep.1, &ep.2).unwrap()];
    let (n0, _, _, _) = run_merge(&eh);
    assert_eq!(n0, 0, "all-empty merge emits 0");

    // Ragged columns refuse with HeadShape, not panic.
    let s1 = vec![1u64, 2];
    let p1 = vec![3u64];
    let o1 = vec![4u64];
    assert!(matches!(
        HeadInput::try_new(9, &s1, &p1, &o1),
        Err(MergeError::HeadShape { head_id: 9 })
    ));
}

// --- Court 4: MUTATION COURT (conflict injection, mutant mode =>
// 101-class Andon refusal fires; flip witnessed) ----------------------

#[test]
fn mutation_conflict_injection_flip() {
    let _g = TEST_SERIAL.lock().unwrap();
    // Baseline arm: clean corpus merges clean.
    let data = build_corpus(4, 256);
    let mut pl: Vec<_> = data.iter().map(|h| planes(h)).collect();
    let mut heads: Vec<HeadInput> = pl
        .iter()
        .enumerate()
        .map(|(i, (s, p, o))| HeadInput::try_new(i, s, p, o).unwrap())
        .collect();
    assert!(
        run_merge(&heads).0 > 0,
        "baseline (no mutation) merges clean"
    );

    // Mutant arm: INJECT a conflicting triple (same s,p; different o)
    // into head 2, head 0 holds the original.
    let (s0, p0, _o0) = data[0][0];
    let injected = (s0, p0, _o0 ^ 0xDEAD_0000_0000_0001);
    let mut h2 = data[2].clone();
    h2.push(injected);
    pl[2] = planes(&h2);
    heads = pl
        .iter()
        .enumerate()
        .map(|(i, (s, p, o))| HeadInput::try_new(i, s, p, o).unwrap())
        .collect();
    let err = run_merge_err(&heads).expect_err("injected conflict must refuse");
    assert!(
        matches!(
            err,
            MergeError::DomainClash {
                head_a: 0,
                head_b: 2,
                ..
            }
        ),
        "refusal names the clashing heads (0, 2): {err:?}"
    );

    // 101-class witness: the Andon wrapper panics on refusal (panic =
    // process exit 101). Re-exec this same test binary with a sentinel
    // env var; the child takes the Andon path and must exit 101.
    if std::env::var("GIT12_ANDON_CHILD").is_ok() {
        let mut scratch = OctopusScratch::default();
        let total: usize = heads.iter().map(|h| h.rows()).sum();
        let mut row_refs = vec![(0u32, 0u32); total];
        let mut os = vec![0u64; total];
        let mut op = vec![0u64; total];
        let mut oo = vec![0u64; total];
        let _ = octopus_merge_andon(
            &heads,
            &mut row_refs,
            &mut scratch,
            &mut os,
            &mut op,
            &mut oo,
        );
        // Unreachable: the Andon refusal panics above.
        std::process::exit(0);
    }
    let me = std::env::current_exe().unwrap();
    let st = std::process::Command::new(me)
        .args([
            "--exact",
            "mutation_conflict_injection_flip",
            "--test-threads",
            "1",
        ])
        .env("GIT12_ANDON_CHILD", "1")
        .status()
        .expect("re-exec self");
    assert_eq!(
        st.code(),
        Some(101),
        "Andon refusal exits 101-class, got {st:?}"
    );
}

fn run_merge_err(heads: &[HeadInput<'_>]) -> Result<usize, MergeError> {
    let total: usize = heads.iter().map(|h| h.rows()).sum();
    let mut scratch = OctopusScratch::default();
    let mut row_refs = vec![(0u32, 0u32); total];
    let mut os = vec![0u64; total];
    let mut op = vec![0u64; total];
    let mut oo = vec![0u64; total];
    octopus_merge(
        heads,
        &mut row_refs,
        &mut scratch,
        &mut os,
        &mut op,
        &mut oo,
    )
}

// --- Court 5: zero-alloc steady state (instrumented) ------------------

#[test]
fn zero_alloc_steady_state() {
    let _g = TEST_SERIAL.lock().unwrap();
    let data = build_corpus(N_HEADS, N_ROWS_PER_HEAD);
    let pl: Vec<_> = data.iter().map(|h| planes(h)).collect();
    let heads: Vec<HeadInput> = pl
        .iter()
        .enumerate()
        .map(|(i, (s, p, o))| HeadInput::try_new(i, s, p, o).unwrap())
        .collect();
    // Warmup on identical input (allocator tables, caches).
    let (n, _, _, _) = run_merge(&heads);
    assert!(n > 0);
    // Measured window: build scratch outside, merge inside; delta must
    // be 0. Scratch lives on the stack of this fn (48 KiB arrays).
    let mut scratch = OctopusScratch::default();
    let total: usize = heads.iter().map(|h| h.rows()).sum();
    let mut row_refs = vec![(0u32, 0u32); total];
    let mut os = vec![0u64; total];
    let mut op = vec![0u64; total];
    let mut oo = vec![0u64; total];
    let before = ALLOC_COUNT.load(Ordering::Relaxed);
    let n2 = octopus_merge(
        &heads,
        &mut row_refs,
        &mut scratch,
        &mut os,
        &mut op,
        &mut oo,
    )
    .unwrap();
    let after = ALLOC_COUNT.load(Ordering::Relaxed);
    assert_eq!(n2, n);
    assert_eq!(after - before, 0, "merge must allocate nothing");
}
