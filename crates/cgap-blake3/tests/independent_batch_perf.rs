//! PL15 perf non-vacuity leg: the batch independent path at N=64 (1 KiB
//! payloads) must beat the per-frame path by a stated margin, or the court
//! names the miss honestly. Method: cgap-aoq profile_guard release gate
//! (loud SKIP on a dev profile, repo AF-25 caller pattern), Instant min-of-5 rounds, black_box fences
//! on inputs and digests (anti-DCE), parity asserted BEFORE timing is
//! trusted.

use std::time::Instant;

use cgap_aoq::profile_guard;
use cgap_blake3::{hash, hash_many_independent_into};

const N: usize = 64;
const PAYLOAD: usize = 1024;
const ROUNDS: usize = 5;

fn fill(buf: &mut [u8]) {
    let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
    for b in buf.iter_mut() {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *b = (x >> 33) as u8;
    }
}

#[test]
fn court_batch_beats_per_frame_at_n64() {
    if let Err(v) = profile_guard::assert_release() {
        eprintln!("LOUD-SKIP court_batch_beats_per_frame_at_n64: {v}");
        return;
    }
    let mut data = vec![0u8; N * PAYLOAD];
    fill(&mut data);
    let payloads: Vec<&[u8]> = data.chunks(PAYLOAD).collect();
    let mut got = vec![[0u8; 32]; N];
    let mut want = vec![[0u8; 32]; N];

    // Parity BEFORE timing (trust gate).
    hash_many_independent_into(&payloads, &mut got);
    for i in 0..N {
        want[i] = hash(payloads[i]);
    }
    assert_eq!(got, want, "parity gate: batch != sequential, timing void");

    // Timed legs, min-of-ROUNDS.
    let batch_ns = (0..ROUNDS)
        .map(|_| {
            let t0 = Instant::now();
            hash_many_independent_into(profile_guard::fence(&payloads), &mut got);
            profile_guard::fence(&got);
            t0.elapsed().as_nanos() as f64
        })
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let per_frame_ns = (0..ROUNDS)
        .map(|_| {
            let t0 = Instant::now();
            for i in 0..N {
                want[i] = hash(profile_guard::fence(payloads[i]));
            }
            profile_guard::fence(&want);
            t0.elapsed().as_nanos() as f64
        })
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    let speedup = per_frame_ns / batch_ns;
    println!(
        "PL15 perf leg (N=64 x 1 KiB, min-of-{ROUNDS}): batch={batch_ns:.0} ns per_frame={per_frame_ns:.0} ns speedup={speedup:.3}x (PL15 predicted 1.4-1.6x)"
    );
    // Stated margin: batch must win outright (>1.0x). A miss is NAMED in the
    // assert message, not hidden.
    assert!(
        speedup > 1.0,
        "PL15 MARGIN MISS (named honestly): batch path {speedup:.3}x did not beat per-frame at N=64; prediction was 1.4-1.6x"
    );
}
