//! AF-21 / G7 receipt: per-link receipt-chaining cost on the landed
//! `cgap-aoq::chain::AoqChain` surface (AB-7's chain.rs, landed on disk),
//! bound to the repo's own sovereign BLAKE3 (`cgap-blake3`, std feature,
//! dev-dep). Run in release:
//!
//! ```text
//! cargo test -p cgap-aoq --test receipt_link_bench --release -- --nocapture
//! ```
//!
//! Method:
//! - (a) per-link append cost: min-of-1000 individual `Instant` timings
//!   around one `append` (BLAKE3 over the fixed-width 4-part preimage +
//!   record mint), plus a whole-leg method (min of 5 legs / link count)
//!   for comparability with AB-7's chain_bench.
//! - (b) 64B digest per-call latency: `cgap_blake3::hash` over a 64B
//!   input, n=1000 individually-timed calls, min of 3 repetitions. This
//!   is per-call wall time including call overhead — the number that
//!   replaces the UNSUPPORTED 18.1ns figure (AF-2's correction: never
//!   invent a fence; measured only).
//! - (c) verify cost per link at 1k / 10k / 100k chains (whole-leg min
//!   of 5, divided by link count).

use std::time::Instant;

use cgap_aoq::chain::{AoqChain, AoqChainRecord, ChainDigest};

/// Real BLAKE3 substrate — the repo's own kernel, not a stand-in.
#[derive(Clone, Copy)]
struct Blake3Digest;

impl ChainDigest for Blake3Digest {
    fn digest(&self, parts: &[&[u8]]) -> [u8; 32] {
        // Concatenate the fixed-width parts into the stack preimage. The
        // chain preimage is 32+2+8+32 = 74 bytes; payload path is 32B.
        const MAX: usize = 128;
        let mut buf = [0u8; MAX];
        let mut n = 0;
        for part in parts {
            buf[n..n + part.len()].copy_from_slice(part);
            n += part.len();
        }
        cgap_blake3::hash(&buf[..n])
    }
}

fn slab(cap: usize) -> &'static mut [AoqChainRecord] {
    Box::leak(vec![AoqChainRecord::zeroed(); cap].into_boxed_slice())
}

fn black_box<T>(x: T) -> T {
    std::hint::black_box(x)
}

/// (a) min-of-1000 individually-timed appends, microseconds per link.
fn bench_append_per_link() -> (f64, f64) {
    let s = slab(2_048);
    let mut c = AoqChain::new(Blake3Digest, s);
    let payload = black_box([0xa5u8; 32]);
    // Warm-up: 64 appends so first-touch effects don't set the floor.
    for i in 0u64..64 {
        c.append((i % 3) as u16, &payload).unwrap();
    }
    let mut samples: Vec<f64> = Vec::with_capacity(1_000);
    let mut idx = c.len();
    for _ in 0..1_000 {
        let e = (idx % 3) as u16;
        let t0 = Instant::now();
        let r = c.append(e, &payload);
        let dt = t0.elapsed();
        black_box(r.unwrap());
        idx += 1;
        samples.push(dt.as_secs_f64() * 1e6);
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let min = samples[0];
    let mean = samples.iter().sum::<f64>() / samples.len() as f64;
    (min, mean)
}

/// (a') whole-leg append: min of 5 legs over `links` appends / count.
fn bench_append_leg(links: usize) -> f64 {
    const LEGS: usize = 5;
    let s = slab(links * LEGS);
    let mut c = AoqChain::new(Blake3Digest, s);
    let payload = [0xa5u8; 32];
    let mut best = f64::INFINITY;
    for _ in 0..LEGS {
        let t0 = Instant::now();
        for i in 0u64..links as u64 {
            black_box(c.append((i % 3) as u16, &payload).unwrap());
        }
        let per = t0.elapsed().as_secs_f64() * 1e6 / links as f64;
        if per < best {
            best = per;
        }
    }
    best
}

/// (b) 64B digest per-call latency: n=1000 individually-timed calls,
/// min of 3 reps. Returns (min_ns, mean_ns).
fn bench_hash64_per_call() -> (f64, f64) {
    let input = black_box([0x5au8; 64]);
    let mut global_min = f64::INFINITY;
    let mut last_mean = 0.0;
    for _ in 0..3 {
        let mut samples = [0f64; 1_000];
        for s in samples.iter_mut() {
            let t0 = Instant::now();
            black_box(cgap_blake3::hash(&input));
            *s = t0.elapsed().as_nanos() as f64;
        }
        global_min = global_min.min(samples.iter().cloned().fold(f64::INFINITY, f64::min));
        last_mean = samples.iter().sum::<f64>() / samples.len() as f64;
    }
    (global_min, last_mean)
}

/// (c) verify cost per link: whole-leg min of 5 over verify_all, µs/link.
fn bench_verify(links: usize) -> f64 {
    let s = slab(links);
    let mut c = AoqChain::new(Blake3Digest, s);
    let payload = [0xa5u8; 32];
    for i in 0u64..links as u64 {
        c.append((i % 3) as u16, &payload).unwrap();
    }
    let mut best = f64::INFINITY;
    for _ in 0..5 {
        let t0 = Instant::now();
        black_box(c.verify_all().unwrap());
        let per = t0.elapsed().as_secs_f64() * 1e6 / links as f64;
        if per < best {
            best = per;
        }
    }
    best
}

#[test]
fn g7_receipt_link_bench() {
    // Non-vacuity: prove the BLAKE3 substrate is real before timing it.
    let mut slab0 = [AoqChainRecord::zeroed(); 2];
    let mut c0 = AoqChain::new(Blake3Digest, &mut slab0);
    c0.append(7, b"nonvacuity").unwrap();
    assert_eq!(c0.verify_all(), Ok(()));
    let recs = c0.records();
    assert_ne!(recs[0].digest, [0u8; 32], "BLAKE3 substrate folded to zero");
    let expected = {
        let mut pre = [0u8; 74];
        pre[32..34].copy_from_slice(&7u16.to_le_bytes());
        pre[42..74].copy_from_slice(&cgap_blake3::hash(b"nonvacuity"));
        cgap_blake3::hash(&pre)
    };
    assert_eq!(
        recs[0].digest, expected,
        "record digest must equal H(prev ‖ epoch ‖ index ‖ H(payload))"
    );

    let (app_min, app_mean) = bench_append_per_link();
    let app_leg = bench_append_leg(1_000);
    let (h64_min, h64_mean) = bench_hash64_per_call();
    let v1k = bench_verify(1_000);
    let v10k = bench_verify(10_000);
    let v100k = bench_verify(100_000);

    println!("\n=== AF-21 G7 per-link receipt-chaining cost (release) ===");
    println!("(a) append min-of-1000 per-link : {:8.3} us/link", app_min);
    println!("(a) append mean-of-1000         : {:8.3} us/link", app_mean);
    println!("(a') append whole-leg (1k)      : {:8.3} us/link", app_leg);
    println!("(b) BLAKE3 64B per-call min     : {:8.1} ns/call", h64_min);
    println!("(b) BLAKE3 64B per-call mean    : {:8.1} ns/call", h64_mean);
    println!("(c) verify 1k links             : {:8.3} us/link", v1k);
    println!("(c) verify 10k links            : {:8.3} us/link", v10k);
    println!("(c) verify 100k links           : {:8.3} us/link", v100k);
    // Machine-parseable receipt block.
    println!(
        "G7RESULT append_min_us_per_link={:.3} append_mean_us_per_link={:.3} append_leg1k_us_per_link={:.3} hash64_min_ns={:.1} hash64_mean_ns={:.1} verify_us_per_link_1k={:.3} verify_us_per_link_10k={:.3} verify_us_per_link_100k={:.3}",
        app_min, app_mean, app_leg, h64_min, h64_mean, v1k, v10k, v100k
    );

    // Sanity bounds only (not fences): a real BLAKE3 call on this hardware
    // is far above zero and far below a microsecond-scale path is not
    // expected for a 2-hash link; these fail only on structural breakage
    // (optimizer folding / pathological machine), never gate a number.
    assert!(app_min > 0.0 && app_min < 100.0, "append per-link out of structural range: {app_min}");
    assert!(h64_min > 5.0, "64B hash per-call implausibly fast (folded?): {h64_min} ns");
}
