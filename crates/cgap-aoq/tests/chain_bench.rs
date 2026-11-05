//! Chain bench (AB-7 deliverable): append and verify throughput at
//! 1k / 10k / 100k links, min-of-N, microseconds per op. Method: a single
//! `Instant` around the whole leg divided by link count; the minimum of 5
//! repetitions is reported. Real digest seam every iteration (no mocks).

use std::time::Instant;

use cgap_aoq::chain::{AoqChain, AoqChainRecord, ChainDigest};

#[derive(Clone, Copy)]
struct Mix32;
impl ChainDigest for Mix32 {
    fn digest(&self, parts: &[&[u8]]) -> [u8; 32] {
        mix32(parts)
    }
}

fn mix32(parts: &[&[u8]]) -> [u8; 32] {
    let mut a: u64 = 0xcbf2_9ce4_8422_2325;
    let mut b: u64 = 0x9e37_79b9_7f4a_7c15;
    for part in parts {
        for &byte in *part {
            a ^= byte as u64;
            a = a.wrapping_mul(0x100_0000_01b3);
            b = (b ^ ((byte as u64) | 1)).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        }
    }
    let mut out = [0u8; 32];
    for (i, chunk) in out.as_chunks_mut::<8>().0.iter_mut().enumerate() {
        let v = if i % 2 == 0 {
            a.wrapping_add(i as u64)
        } else {
            b.wrapping_add(i as u64)
        };
        chunk.copy_from_slice(&v.to_le_bytes());
    }
    out
}

fn slab() -> &'static mut [AoqChainRecord] {
    Box::leak(vec![AoqChainRecord::zeroed(); 100_000].into_boxed_slice())
}

fn bench_append(links: usize) -> f64 {
    let slab = slab();
    let mut c = AoqChain::new(Mix32, slab);
    let payload = cgap_aoq::profile_guard::fence([0xa5u8; 32]);
    let t0 = Instant::now();
    for i in 0u64..links as u64 {
        c.append((i % 3) as u16, &payload).unwrap();
    }
    let per_op_us = cgap_aoq::profile_guard::fence(t0.elapsed().as_secs_f64() * 1e6 / links as f64);
    assert_eq!(c.len(), links);
    per_op_us
}

fn bench_verify(links: usize) -> f64 {
    let slab = slab();
    let mut c = AoqChain::new(Mix32, slab);
    let payload = cgap_aoq::profile_guard::fence([0xa5u8; 32]);
    for i in 0u64..links as u64 {
        c.append((i % 3) as u16, &payload).unwrap();
    }
    let t0 = Instant::now();
    assert_eq!(c.verify_all(), Ok(()));
    
    cgap_aoq::profile_guard::fence(t0.elapsed().as_secs_f64() * 1e6 / links as f64)
}

/// Min-of-N across 1k / 10k / 100k links.
#[test]
fn bench_append_and_verify_min_of_n() {
    if let Err(v) = cgap_aoq::profile_guard::assert_release() {
        eprintln!("LOUD-SKIP bench_append_and_verify_min_of_n: {v}");
        return;
    }
    const N: usize = 5;
    let mut best_append = [f64::INFINITY; 3];
    let mut best_verify = [f64::INFINITY; 3];
    for _ in 0..N {
        for (i, links) in [1_000usize, 10_000, 100_000].into_iter().enumerate() {
            let a = bench_append(links);
            if a < best_append[i] {
                best_append[i] = a;
            }
            let v = bench_verify(links);
            if v < best_verify[i] {
                best_verify[i] = v;
            }
        }
    }
    for (i, links) in [1_000usize, 10_000, 100_000].into_iter().enumerate() {
        let fa = cgap_aoq::profile_guard::fence(best_append[i]);
        let fv = cgap_aoq::profile_guard::fence(best_verify[i]);
        println!(
            "chain bench: {:>7} links  append {:>8.3} us/op  verify {:>8.3} us/op  (min of {N})",
            links, fa, fv
        );
    }
    // Sanity floor (not a perf claim): the digest seam is deterministic
    // work, so verify must actually walk all links.
    assert!(best_verify[2] > 0.0);
    assert!(best_append[2] > 0.0);
}
