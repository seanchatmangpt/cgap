//! AF-3 (AB-12 follow-on): root-cause profile of `hash_wide_simd` vs scalar `hash`.
//!
//! AB-12 measured hash_wide_simd ~430 MiB/s vs scalar 1768.8 MiB/s @64KiB
//! (4x SLOWER). This bin answers, public API only:
//! 1. Root cause: phase attribution (leaf phase vs parent phase) at 64KiB/1MiB,
//!    plus isolated parent-kernel A/B: NEON 4-way reduce_level_simd vs the same
//!    CV set reduced by direct scalar parent compressions.
//! 2. Crossover: is there ANY size in 64KiB..64MiB where wide wins on this host?
//!
//! Plain std, no sleeps, min-of-N Instant timing. Parity asserted before timing.

use std::time::Instant;

use cgap_blake3::{
    compress_scalar, hash, hash_chunk, hash_wide_simd, reduce_level_simd, IV, PARENT,
};

fn time_min<F: FnMut()>(mut f: F, rounds: usize) -> f64 {
    let mut best = f64::INFINITY;
    for _ in 0..rounds {
        let t0 = Instant::now();
        f();
        let dt = t0.elapsed().as_secs_f64();
        if dt < best {
            best = dt;
        }
    }
    best
}

fn fill(buf: &mut [u8]) {
    let mut x: u64 = 0x9E3779B97F4A7C15;
    for b in buf.iter_mut() {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *b = (x >> 33) as u8;
    }
}

fn hex(d: &[u8; 32]) -> String {
    d.iter().map(|b| format!("{:02x}", b)).collect()
}

fn mib_s(bytes: usize, secs: f64) -> f64 {
    (bytes as f64) / secs / (1024.0 * 1024.0)
}

/// The wide path's leaf phase in isolation: hash_chunk per 1KiB chunk.
fn leaf_only(buf: &[u8]) -> u64 {
    let len = buf.len();
    let num_chunks = len.div_ceil(1024);
    let mut acc = 0u64;
    for c in 0..num_chunks {
        let start = c * 1024;
        let end = (start + 1024).min(len);
        let out = hash_chunk(&buf[start..end], c as u64, false);
        acc = acc.wrapping_add(out[0] as u64);
    }
    acc
}

/// Scalar parent reduction of 64 CVs (6 levels, 63 compressions) — the
/// scalar-path parent cost on the same CV set.
fn scalar_reduce64(cvs: &[[u32; 8]; 64]) -> [u32; 8] {
    let mut n = 64usize;
    let mut tree = *cvs;
    while n > 1 {
        let mut out_n = 0usize;
        let mut i = 0usize;
        while i + 1 < n {
            let mut block = [0u32; 16];
            block[..8].copy_from_slice(&tree[i]);
            block[8..].copy_from_slice(&tree[i + 1]);
            let out = compress_scalar(&IV, &block, 0, 64, PARENT);
            let mut cv = [0u32; 8];
            for j in 0..8 {
                cv[j] = out[j];
            }
            tree[out_n] = cv;
            out_n += 1;
            i += 2;
        }
        if n & 1 == 1 {
            tree[out_n] = tree[n - 1];
            out_n += 1;
        }
        n = out_n;
    }
    tree[0]
}

/// NEON 4-way parent reduction of 64 CVs via the public reduce_level_simd,
/// exactly as hash_wide_simd drives it (reduce until 2 remain).
fn wide_reduce64(cvs: &[[u32; 8]; 64]) -> [u32; 8] {
    let mut tree = *cvs;
    let mut count = 64usize;
    while count > 2 {
        reduce_level_simd(&mut tree, &mut count);
    }
    reduce_level_simd(&mut tree, &mut count);
    tree[0]
}

fn main() {
    if let Err(v) = cgap_aoq::profile_guard::assert_release() {
        eprintln!("LOUD-SKIP {} : {v}", std::env::args().next().unwrap_or_default());
        return;
    }
    let rounds: usize = std::env::var("AF3_ROUNDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(11);

    // Parity sanity before timing anything.
    {
        let mut buf = vec![0u8; 128 * 1024];
        fill(&mut buf);
        let reference = hash(&buf);
        assert_eq!(
            hex(&reference),
            hex(&hash_wide_simd(&buf)),
            "wide != hash @128KiB"
        );
        // Parent-chain parity: scalar_reduce64 == wide_reduce64 on a real CV set.
        let mut buf64 = vec![0u8; 64 * 1024];
        fill(&mut buf64);
        let mut cvs = [[0u32; 8]; 64];
        for k in 0..64 {
            let out = hash_chunk(&buf64[k * 1024..(k + 1) * 1024], k as u64, false);
            cvs[k].copy_from_slice(&out[0..8]);
        }
        let s = scalar_reduce64(&cvs);
        let w = wide_reduce64(&cvs);
        assert_eq!(s, w, "scalar_reduce64 != wide_reduce64");
    }

    println!("=== AF-3 SIMD profile (arch: {})", std::env::consts::ARCH);

    // ------------------------------------------------------------------
    // 1. Crossover sweep: scalar hash vs hash_wide_simd, 64KiB..64MiB
    // ------------------------------------------------------------------
    println!("\n--- sweep (min of {} rounds) ---", rounds);
    println!(
        "{:>10} {:>14} {:>14} {:>8}   {}",
        "size", "scalar MiB/s", "wide MiB/s", "ratio", "winner"
    );
    let sizes: [usize; 11] = [
        64 * 1024,
        128 * 1024,
        256 * 1024,
        512 * 1024,
        1024 * 1024,
        2 * 1024 * 1024,
        4 * 1024 * 1024,
        8 * 1024 * 1024,
        16 * 1024 * 1024,
        32 * 1024 * 1024,
        64 * 1024 * 1024,
    ];
    for &bytes in sizes.iter() {
        let mut buf = vec![0u8; bytes];
        fill(&mut buf);
        let r = if bytes <= 1024 * 1024 {
            rounds * 4
        } else {
            rounds
        };
        let t_scalar = time_min(
            || {
                let d = hash(&buf);
                std::hint::black_box(&d);
            },
            r,
        );
        let t_wide = time_min(
            || {
                let d = hash_wide_simd(&buf);
                std::hint::black_box(&d);
            },
            r,
        );
        let ratio = t_wide / t_scalar;
        let winner = if ratio < 1.0 { "wide" } else { "scalar" };
        println!(
            "{:>10} {:>14.1} {:>14.1} {:>7.2}x   {}",
            bytes,
            mib_s(bytes, t_scalar),
            mib_s(bytes, t_wide),
            ratio,
            winner
        );
    }

    // ------------------------------------------------------------------
    // 2. Phase attribution at 64KiB and 1MiB
    // ------------------------------------------------------------------
    for &bytes in [64 * 1024usize, 1024 * 1024].iter() {
        let mut buf = vec![0u8; bytes];
        fill(&mut buf);
        let r = if bytes <= 64 * 1024 {
            rounds * 4
        } else {
            rounds
        };

        let t_full = time_min(
            || {
                let d = hash_wide_simd(&buf);
                std::hint::black_box(&d);
            },
            r,
        );
        let t_leaf = time_min(
            || {
                let a = leaf_only(&buf);
                std::hint::black_box(a);
            },
            r,
        );
        let t_scalar = time_min(
            || {
                let d = hash(&buf);
                std::hint::black_box(&d);
            },
            r,
        );
        let t_leaf_scalar = time_min(
            || {
                let a = leaf_only(&buf);
                std::hint::black_box(a);
            },
            r,
        );

        println!("\n--- attribution @ {} ---", bytes);
        println!(
            "wide full:        {:>9.1} MiB/s  ({:.2} us)",
            mib_s(bytes, t_full),
            t_full * 1e6
        );
        println!(
            "leaf-only:        {:>9.1} MiB/s  ({:.2} us)",
            mib_s(bytes, t_leaf),
            t_leaf * 1e6
        );
        println!(
            "scalar hash full: {:>9.1} MiB/s  ({:.2} us)",
            mib_s(bytes, t_scalar),
            t_scalar * 1e6
        );
        println!(
            "parent cost (wide)   = full - leaf = {:.2} us ({:.1}% of full)",
            (t_full - t_leaf) * 1e6,
            100.0 * (t_full - t_leaf) / t_full
        );
        println!(
            "parent cost (scalar) = scalar - leaf = {:.2} us ({:.1}% of scalar)",
            (t_scalar - t_leaf_scalar) * 1e6,
            100.0 * (t_scalar - t_leaf_scalar) / t_scalar
        );
    }

    // ------------------------------------------------------------------
    // 3. Isolated parent-kernel A/B: 64-CV reduction, scalar vs NEON 4-way
    // ------------------------------------------------------------------
    {
        let mut buf64 = vec![0u8; 64 * 1024];
        fill(&mut buf64);
        let mut cvs = [[0u32; 8]; 64];
        for k in 0..64 {
            let out = hash_chunk(&buf64[k * 1024..(k + 1) * 1024], k as u64, false);
            cvs[k].copy_from_slice(&out[0..8]);
        }
        let r = rounds * 8;
        let t_scalar_p = time_min(
            || {
                let c = scalar_reduce64(&cvs);
                std::hint::black_box(c);
            },
            r,
        );
        let t_wide_p = time_min(
            || {
                let c = wide_reduce64(&cvs);
                std::hint::black_box(c);
            },
            r,
        );
        println!("\n--- isolated 64-CV parent reduction ---");
        println!(
            "scalar chain : {:>8.2} us  ({:.0} ns/parent)",
            t_scalar_p * 1e6,
            t_scalar_p * 1e9 / 63.0
        );
        println!(
            "NEON 4-way   : {:>8.2} us  ({:.0} ns/parent)",
            t_wide_p * 1e6,
            t_wide_p * 1e9 / 63.0
        );
        println!("ratio: {:.2}x", t_wide_p / t_scalar_p);
    }

    println!("\nAF3_ROUNDS={} (env-overridable)", rounds);
}
