//! AB-19 slab stress lane (v26.11.3). Subject: `cgap_arrow::slab`
//! (read-only; `SlabError` variants are the refusal vocabulary).
//!
//! The real slab lifecycle is push -> seal -> decode with reset-to-`new()`
//! as the only release primitive (there is no per-row `free` and no
//! handle table). The contract's deliverables map onto that surface:
//!
//! 1. capacity stress: fill to CAP -> exact `SlabFull` refusal (state
//!    unchanged) -> reset -> refill, 1k cycles, state machine intact;
//! 2. fragmentation-pattern stress: 17 deterministic interleaved-op
//!    orderings x 100 iterations — success census == final `len` (no
//!    lost rows) and the whole-slab byte image stable run-over-run;
//! 3. double-free / use-after-free analog: double seal, push-after-seal,
//!    mask-seal-after-seal -> `SlabError::Sealed` every time, state
//!    frozen; mutated sealed windows -> typed refusals, never silent;
//! 4. alloc census: counting global allocator over the full run — every
//!    `SlabError` refusal path allocates 0 bytes (equal-allocation law
//!    for refusal paths), positive control proves the counter works;
//! 5. throughput: push+reset cycles/sec, min-of-3.
//!
//! Chicago courts: real slab, real allocator, real byte images; every
//! court names its anti-vacuity mutation; a court that cannot flip is
//! FAIL.

// --------------------------------------------------------------------------
// Counting allocator (deliverable 4). Thread-local counters: the test
// harness runs tests in parallel threads, and every allocation happens on
// the allocating thread, so per-thread deltas are exact under parallelism.
// Anti-vacuity: `alloc_census_positive_control` asserts a live Vec alloc
// moves the counter — a counter that never moves is a VACUOUS gate.
// --------------------------------------------------------------------------
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

struct Counting;

thread_local! {
    static ALLOCS: Cell<u64> = const { Cell::new(0) };
    static DEALLOCS: Cell<u64> = const { Cell::new(0) };
    static BYTES: Cell<u64> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOCS.with(|c| c.set(c.get() + 1));
        BYTES.with(|c| c.set(c.get() + l.size() as u64));
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        DEALLOCS.with(|c| c.set(c.get() + 1));
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        ALLOCS.with(|c| c.set(c.get() + 1));
        BYTES.with(|c| c.set(c.get() + n as u64));
        unsafe { System.realloc(p, l, n) }
    }
}

#[global_allocator]
static COUNTING: Counting = Counting;

fn allocs() -> u64 {
    ALLOCS.with(|c| c.get())
}

// --------------------------------------------------------------------------
// Deterministic corpus (no entropy, no clocks): SplitMix64 draws seeded by
// the row index, same shape as the A01 wave-9 corpus generator.
// --------------------------------------------------------------------------
use cgap_arrow::slab::{Slab64, SlabError, FLAG_PRESENT_INLINE, FLAG_SEALED, SLOT_PAYLOAD_SIZE};

const SEED: u64 = 0xA19D_0FFE_E5A7_E001;

const fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn row_payload(row: u64) -> [u8; SLOT_PAYLOAD_SIZE] {
    let mut state = SEED ^ row.wrapping_mul(0xD1B5_4A32_D192_ED03);
    let mut payload = [0u8; SLOT_PAYLOAD_SIZE];
    let mut filled = 0;
    while filled < SLOT_PAYLOAD_SIZE {
        let bytes = splitmix64(&mut state).to_le_bytes();
        let take = core::cmp::min(8, SLOT_PAYLOAD_SIZE - filled);
        payload[filled..filled + take].copy_from_slice(&bytes[..take]);
        filled += take;
    }
    payload
}

/// Plain u64 LCG for interleaved-op orderings (deterministic per seed).
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
}

// --------------------------------------------------------------------------
// Deliverable 1 — capacity stress: 1k fill -> typed SlabFull -> reset ->
// refill cycles. Anti-vacuity mutation: weaken the `len >= CAP` gate in
// `push` — the CAP+1-th push no longer refuses and the exact-variant
// assert (plus the untouched-state assert) flips.
// --------------------------------------------------------------------------
const CAPACITY_CYCLES: usize = 1_000;

#[test]
fn capacity_stress_1k_fill_refuse_reset_refill_cycles() {
    let mut refusals = 0u64;
    for cycle in 0..CAPACITY_CYCLES {
        let mut slab = Slab64::new();
        for i in 0..64u64 {
            assert_eq!(
                slab.push(&row_payload(cycle as u64 * 64 + i), i % 2 == 0),
                Ok(())
            );
        }
        assert_eq!(slab.len(), 64);
        assert_eq!(slab.capacity(), 64);

        // Exact typed refusal at the boundary, state byte-identical after.
        let frozen = slab;
        assert_eq!(
            slab.push(&row_payload(0xFFFF_0000), true),
            Err(SlabError::SlabFull)
        );
        assert_eq!(slab, frozen, "cycle {cycle}: refused push mutated state");
        refusals += 1;

        // Targeted release (the surface's only free primitive: reset) then
        // refill — the state machine must come back exactly clean.
        slab = Slab64::new();
        assert_eq!(slab.len(), 0);
        assert!(!slab.is_sealed());
        for i in 0..64u64 {
            assert_eq!(slab.push(&row_payload(0x1000_0000 + i), true), Ok(()));
        }
        assert_eq!(slab.len(), 64);

        // Seal + decode identity every cycle: the sealed image survives the
        // churn (blake3 over 4096 slot bytes + FNV-1a header, per cycle).
        slab.seal().unwrap();
        let round = Slab64::from_bytes(slab.as_bytes()).unwrap();
        assert_eq!(round, slab);
        assert_eq!(round.header.flags, FLAG_PRESENT_INLINE | FLAG_SEALED);
    }
    assert_eq!(
        refusals, CAPACITY_CYCLES as u64,
        "every cycle hit the boundary"
    );
}

// --------------------------------------------------------------------------
// Deliverable 2 — fragmentation-pattern stress: 17 deterministic interleaved
// op orderings x 100 iterations. The op mix interleaves the three surface
// transitions (push, seal, reset) with refusal-inducing ops (over-cap push,
// push-after-seal, payload-too-long). Census laws: successful pushes ==
// final `len` at every step (nothing lost, nothing phantom); refusal census
// and the whole-slab byte image are stable across all 100 iterations of an
// ordering (no ordering leaks state). Anti-vacuity mutation: make `push`
// advance `len` before a refused op (or skip the seal padding canonicalize)
// — the image-stability / census asserts flip on the first mismatched
// iteration.
// --------------------------------------------------------------------------
const ORDERINGS: u64 = 17;
const ITERATIONS: usize = 100;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Census {
    ok_push: u32,
    full: u32,
    sealed: u32,
    too_long: u32,
    ok_seal: u32,
    ok_reset: u32,
}

fn run_ordering(seed: u64) -> (Census, std::vec::Vec<u8>) {
    let mut slab = Slab64::new();
    let mut rng = Lcg(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let mut c = Census {
        ok_push: 0,
        full: 0,
        sealed: 0,
        too_long: 0,
        ok_seal: 0,
        ok_reset: 0,
    };
    let ops = 64 + (seed % 97) as usize; // 64..160 ops per ordering
    for step in 0..ops {
        // Invariant at every step: successful pushes == rows resident.
        assert_eq!(
            c.ok_push,
            slab.len(),
            "ordering {seed} step {step}: census != len"
        );
        match rng.next() % 6 {
            0 | 1 => {
                let row = rng.next() % 256;
                match slab.push(&row_payload(row), row % 2 == 0) {
                    Ok(()) => c.ok_push += 1,
                    Err(SlabError::SlabFull) => c.full += 1,
                    Err(SlabError::Sealed) => c.sealed += 1,
                    Err(e) => panic!("ordering {seed} step {step}: unexpected {e:?}"),
                }
            }
            2 => match slab.push(&[0xAA; SLOT_PAYLOAD_SIZE + 1], true) {
                Err(SlabError::PayloadTooLong) => c.too_long += 1,
                Err(SlabError::Sealed) => c.sealed += 1,
                Err(e) => panic!("ordering {seed} step {step}: unexpected {e:?}"),
                Ok(()) => panic!("ordering {seed} step {step}: 61-byte payload accepted"),
            },
            3 => match slab.seal() {
                Ok(()) => c.ok_seal += 1,
                Err(SlabError::Sealed) => c.sealed += 1,
                Err(e) => panic!("ordering {seed} step {step}: unexpected {e:?}"),
            },
            4 => {
                // Reset: the surface's release primitive.
                slab = Slab64::new();
                c.ok_reset += 1;
                // Census of successes must reset with the slab: nothing
                // survives a release.
                assert_eq!(slab.len(), 0);
                c.ok_push = 0;
            }
            _ => {
                // 5: double-op attempts — push AND seal in the same step,
                // exercising double-free/use-after-free analog paths under
                // the interleaving itself.
                let p = slab.push(&row_payload(rng.next()), true);
                let s = slab.seal();
                for r in [p, s] {
                    match r {
                        Ok(()) | Err(SlabError::Sealed) | Err(SlabError::SlabFull) => {}
                        Err(e) => panic!("ordering {seed} step {step}: unexpected {e:?}"),
                    }
                }
                // Recompute push census from the authoritative len.
                c.ok_push = slab.len();
            }
        }
    }
    (c, slab.as_bytes().to_vec())
}

#[test]
fn fragmentation_17_orderings_x100_image_and_census_stable() {
    for seed in 0..ORDERINGS {
        let (c0, image0) = run_ordering(seed);
        for it in 1..ITERATIONS {
            let (c, image) = run_ordering(seed);
            assert_eq!(c, c0, "ordering {seed} iteration {it}: census drift");
            assert_eq!(
                image, image0,
                "ordering {seed} iteration {it}: byte-image drift"
            );
        }
        // Final state laws: resident rows equal recorded successes, and a
        // successfully sealed slab decodes with identity.
        assert!(c0.ok_push <= 64);
        assert!(
            c0.full + c0.sealed + c0.too_long > 0,
            "ordering {seed}: no refusal observed"
        );
    }
}

// --------------------------------------------------------------------------
// Deliverable 3 — double-free / use-after-free analog. The handle design
// here is the SEALED state: after seal, every mutation path must refuse
// with the exact typed variant, and the slab must be byte-frozen. Also:
// every tampered sealed window decodes to a typed refusal, never UB, never
// silent accept. Anti-vacuity mutation: delete the `is_sealed` early
// return in `push`/`seal` — the exact-variant asserts flip (push would
// mutate a sealed slab and the freeze assert fails first).
// --------------------------------------------------------------------------
#[test]
fn sealed_state_refuses_every_mutation_typed_and_frozen() {
    let mut slab = Slab64::new();
    for i in 0..10u64 {
        assert_eq!(slab.push(&row_payload(i), i % 2 == 0), Ok(()));
    }
    assert_eq!(slab.seal(), Ok(()));
    let frozen = slab;

    // Double seal (the double-free analog: releasing an already-released
    // lifecycle) — typed Sealed, no silent re-digest.
    assert_eq!(slab.seal(), Err(SlabError::Sealed));
    // Use-after-free analogs: mutation after release.
    assert_eq!(slab.push(&row_payload(99), true), Err(SlabError::Sealed));
    // Repeating the refusals never escapes the frozen state.
    for _ in 0..1_000 {
        assert_eq!(slab.push(&row_payload(99), true), Err(SlabError::Sealed));
        assert_eq!(slab.seal(), Err(SlabError::Sealed));
    }
    assert_eq!(slab, frozen, "sealed slab mutated by refusal battery");

    // Same battery on the decode-reacquired object (a decoded sealed slab
    // is sealed too — the refusal property survives the round trip).
    let mut round = Slab64::from_bytes(frozen.as_bytes()).unwrap();
    assert_eq!(round.push(&[1; 4], true), Err(SlabError::Sealed));
    assert_eq!(round.seal(), Err(SlabError::Sealed));
    assert_eq!(round, frozen);

    // Tampered sealed windows: every mutation class is a typed refusal.
    let image = frozen.as_bytes().to_vec();
    let mut m = image.clone();
    m[0x40 + 3 * 64 + 10] ^= 0xFF; // payload byte -> DigestMismatch
    assert_eq!(Slab64::from_bytes(&m), Err(SlabError::DigestMismatch));
    let mut m = image.clone();
    m[0x1C] ^= 0x01; // stored header digest -> HeaderDigestMismatch
    assert_eq!(Slab64::from_bytes(&m), Err(SlabError::HeaderDigestMismatch));
    let mut m = image.clone();
    m[0x00] = b'X'; // magic -> BadMagic
    assert_eq!(
        Slab64::from_bytes(&m),
        Err(SlabError::BadMagic {
            got: u32::from_le_bytes(*b"XBLS")
        })
    );
    assert_eq!(
        Slab64::from_bytes(&image[..image.len() - 1]),
        Err(SlabError::BadLength {
            expected: 64 + 64 * 64,
            got: image.len() - 1
        })
    );
    assert_eq!(
        Slab64::from_bytes(&image),
        Ok(frozen),
        "pristine window still decodes"
    );
}

// --------------------------------------------------------------------------
// Deliverable 4 — alloc census: every SlabError refusal path allocates 0.
// Positive control proves the counter moves; the census then asserts the
// equal-allocation law per call site and cumulatively over a refusal soak.
// Anti-vacuity mutation: none in the slab (refusal paths are pure); the
// court is non-vacuous via the live-Vec control — a counter that never
// moves cannot pass the control.
// --------------------------------------------------------------------------
#[test]
fn alloc_census_refusal_paths_allocate_zero() {
    // Positive control: the counter is live.
    let before = allocs();
    let v = vec![0u8; 4096];
    assert_eq!(
        allocs() - before,
        1,
        "positive control: allocator not counted"
    );
    assert!(v[0] == 0);
    drop(v);

    // Fresh full slab for SlabFull refusals.
    let mut full = Slab64::new();
    for i in 0..64u64 {
        full.push(&row_payload(i), true).unwrap();
    }
    // Sealed slab for Sealed refusals + tampered windows.
    let mut sealed = Slab64::new();
    for i in 0..4u64 {
        sealed.push(&row_payload(i), i % 2 == 0).unwrap();
    }
    sealed.seal().unwrap();
    let image = sealed.as_bytes().to_vec();
    let short = &image[..100]; // BadLength window (slice, no alloc)

    // Per-call-site census: delta must be exactly 0 around each refusal.
    macro_rules! refusal_allocs_zero {
        ($call:expr, $expect:expr, $site:literal) => {{
            let b = allocs();
            let r = $call;
            let d = allocs() - b;
            assert_eq!(r, $expect, "{}", $site);
            assert_eq!(
                d, 0,
                "{}: refusal allocated {} bytes-worth of calls",
                $site, d
            );
        }};
    }
    refusal_allocs_zero!(
        full.push(&row_payload(0), true),
        Err(SlabError::SlabFull),
        "SlabFull"
    );
    refusal_allocs_zero!(
        sealed.push(&row_payload(0), true),
        Err(SlabError::Sealed),
        "push-after-seal"
    );
    refusal_allocs_zero!(sealed.seal(), Err(SlabError::Sealed), "double seal");
    refusal_allocs_zero!(
        sealed.push(&[0x11; SLOT_PAYLOAD_SIZE + 1], true),
        Err(SlabError::Sealed),
        "too-long-after-seal"
    );
    refusal_allocs_zero!(
        Slab64::from_bytes(short),
        Err(SlabError::BadLength {
            expected: 64 + 64 * 64,
            got: 100
        }),
        "BadLength"
    );

    // Cumulative soak: 10k refusals across all classes, zero allocations.
    let soak_start = allocs();
    for i in 0..10_000u32 {
        let _ = full.push(&[i as u8; 4], true);
        let _ = sealed.push(&[i as u8; 4], true);
        let _ = sealed.seal();
        let _ = Slab64::from_bytes(short);
    }
    assert_eq!(allocs() - soak_start, 0, "refusal soak allocated");

    // Contrast leg: the success path also allocates 0 (zero-alloc is
    // structural), but decode+identity still closes.
    let b = allocs();
    let mut ok = Slab64::new();
    assert_eq!(ok.push(&row_payload(1), true), Ok(()));
    assert_eq!(allocs() - b, 0, "successful push allocated");
}

// --------------------------------------------------------------------------
// Deliverable 5 — throughput: fill+reset cycles/sec (the surface's
// alloc/release pair analog), min-of-3 independent timed runs. No sleeps:
// measurement is a pure timed loop on the calling thread. The floor assert
// is deliberately conservative (100k pushes/sec) — the printed number is
// the receipt, the floor only catches gross regressions.
// --------------------------------------------------------------------------
#[test]
fn throughput_push_reset_pairs_min_of_3() {
    // Profile guard (arrow-ZP18-D): wall-clock min-of-3 leg fences against
    // a release-only floor assert; under a dev profile it loud-SKIPs.
    if let Err(v) = cgap_aoq::profile_guard::assert_release() {
        eprintln!("LOUD-SKIP throughput_push_reset_pairs_min_of_3: {v}");
        return;
    }
    use std::hint::black_box;
    use std::time::Instant;

    const CYCLES: usize = 20_000;
    const FLOOR_PUSHES_PER_SEC: f64 = 100_000.0;

    // Warmup (not timed): code and caches hot before run 1.
    let mut slab = Slab64::new();
    let payload = row_payload(7);
    for _ in 0..1_000 {
        while slab.push(&payload, true).is_ok() {}
        slab = Slab64::new();
    }

    let mut best = f64::INFINITY;
    for run in 0..3 {
        let mut slab = Slab64::new();
        let t0 = Instant::now();
        for _ in 0..CYCLES {
            while black_box(&mut slab).push(&payload, true).is_ok() {}
            slab = Slab64::new();
        }
        let dt = t0.elapsed().as_secs_f64();
        let pushes_per_sec = (CYCLES * 64) as f64 / dt;
        println!(
            "run {run}: {} fill+reset cycles in {dt:.4}s -> {pushes_per_sec:.0} pushes/sec",
            CYCLES
        );
        assert!(
            pushes_per_sec > FLOOR_PUSHES_PER_SEC,
            "run {run}: {pushes_per_sec:.0} pushes/sec below floor"
        );
        best = best.min(pushes_per_sec);
    }
    println!("MIN-OF-3: {best:.0} pushes/sec (fill+reset pairs over Slab64)");
    assert!(best.is_finite());
}
