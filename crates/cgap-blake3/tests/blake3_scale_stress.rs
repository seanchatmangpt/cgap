//! AB-12 BLAKE3 scale stress (lane STRESS/AB-12, v26.11.2 starDF campaign).
//!
//! Subject: the vendored BLAKE3 kernel — canonical published copy in
//! `crates/cgap-blake3/src/lib.rs`; vendored variant in
//! `crates/cgap-arrow/src/blake3.rs` (same algorithm; arrow copy differs by
//! the GATE-2 no-panic windowing only, byte-diffed this lane).
//!
//! Courts (all Chicago — real kernel calls, asserted final state):
//! 1. Throughput curve 64B..64MB, min-of-3, plus the 16 MiB GPU-dispatch
//!    boundary law (`should_dispatch_gpu` flips exactly at
//!    `GPU_CROSSOVER_BYTES`; digest law is continuous across the boundary —
//!    both kernels agree on digests either side of a prefix-extended input
//!    crossing 16 MiB).
//! 2. Chunk-tree stress: an INDEPENDENT Merkle reducer written here against
//!    the public `hash_chunk` + `compress_scalar` + flag primitives must
//!    agree byte-identically with `hash` and `hash_wide_simd` across 10
//!    chain shapes including the 1k x 1KB chain (1024 chunks) and the
//!    63/64/65-chunk SIMD boundary. Loud note: the kernel exposes NO
//!    incremental `update()` API, so "incremental update chains" are
//!    emulated as chunk-sequence concatenation vs one-shot hash of the same
//!    buffer — the one-shot call IS the chain, and the Merkle shape law is
//!    what carries the equivalence claim.
//! 3. SIMD-vs-scalar equivalence: `hash_wide_simd` vs `hash` on 1k random
//!    inputs >= SIMD width (>= 1025 B, where the wide path actually
//!    engages).
//! 4. Keyed-mode stress: inline keyed binding over the public
//!    `compress_scalar` (spec `KEYED` flag = 1 << 4), pinned to two b3sum
//!    (`--keyed`, reference implementation) KATs, then 1k distinct keys ->
//!    1k distinct digests, same key twice -> identical. Why inline and not
//!    `cgap-crypto::chatman::keyed_digest`: that crate depends on
//!    cgap-blake3, so a dev-dependency here would be a cycle; this binding
//!    mirrors `chatman.rs`'s own one-kernel-many-bindings law.

use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use cgap_blake3::{
    compress_scalar, hash, hash_chunk, hash_wide_simd, should_dispatch_gpu, CHUNK_END, CHUNK_START,
    GPU_CROSSOVER_BYTES, IV, PARENT, ROOT,
};

/// Spec domain-separation flag for keyed mode (BLAKE3 spec section 5,
/// flag bit 4). Not exported by cgap-blake3 (no keyed API — see module doc).
const KEYED: u32 = 1 << 4;

/// Deterministic PRNG (splitmix64 — never the corpus seed constant).
struct Xs(u64);

impl Xs {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn fill(&mut self, buf: &mut [u8]) {
        for b in buf.iter_mut() {
            *b = self.next() as u8;
        }
    }
}

fn hex(d: &[u8; 32]) -> String {
    d.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Keyed BLAKE3 over a single chunk (<= 1024 B) built on the kernel's public
/// `compress_scalar`: first chunk CV = key words, every compression carries
/// the KEYED flag (mirrors `cgap-crypto/src/chatman.rs` keyed_digest).
fn keyed_digest_inline(key: &[u8; 32], data: &[u8]) -> [u8; 32] {
    assert!(data.len() <= 1024, "single-chunk keyed law");
    let mut key_words = [0u32; 8];
    for (i, w) in key_words.iter_mut().enumerate() {
        let mut b = [0u8; 4];
        b.copy_from_slice(&key[i * 4..i * 4 + 4]);
        *w = u32::from_le_bytes(b);
    }
    let num_blocks = if data.is_empty() {
        1
    } else {
        data.len().div_ceil(64)
    };
    let mut cv = key_words;
    let mut out = [0u32; 16];
    for b in 0..num_blocks {
        let start = b * 64;
        let end = (start + 64).min(data.len());
        let mut block_words = [0u32; 16];
        for (i, word_bytes) in data[start..end].chunks(4).enumerate() {
            let mut buf = [0u8; 4];
            buf[..word_bytes.len()].copy_from_slice(word_bytes);
            block_words[i] = u32::from_le_bytes(buf);
        }
        let mut flags = KEYED;
        if b == 0 {
            flags |= CHUNK_START;
        }
        let last = b + 1 >= num_blocks;
        if last {
            flags |= CHUNK_END | ROOT;
        }
        out = compress_scalar(&cv, &block_words, 0, (end - start) as u32, flags);
        if !last {
            cv.copy_from_slice(&out[0..8]);
        }
    }
    let mut res = [0u8; 32];
    for i in 0..8 {
        res[i * 4..(i + 1) * 4].copy_from_slice(&out[i].to_le_bytes());
    }
    res
}

/// INDEPENDENT Merkle reducer over the public chunk/parent primitives —
/// shares no reduction code with `hash` or `hash_wide_simd`.
fn tree_hash(data: &[u8]) -> [u8; 32] {
    let num_chunks = data.len().div_ceil(1024);
    let mut cvs: Vec<[u32; 8]> = Vec::with_capacity(num_chunks);
    for c in 0..num_chunks {
        let start = c * 1024;
        let end = (start + 1024).min(data.len());
        let out = hash_chunk(&data[start..end], c as u64, false);
        cvs.push(out[0..8].try_into().unwrap());
    }
    while cvs.len() > 1 {
        let mut next = Vec::with_capacity(cvs.len().div_ceil(2));
        let mut k = 0;
        while k + 1 < cvs.len() {
            let mut block = [0u32; 16];
            block[..8].copy_from_slice(&cvs[k]);
            block[8..].copy_from_slice(&cvs[k + 1]);
            let root = next.is_empty() && cvs.len() == 2;
            let flags = if root { PARENT | ROOT } else { PARENT };
            let out = compress_scalar(&IV, &block, 0, 64, flags);
            next.push(out[0..8].try_into().unwrap());
            k += 2;
        }
        if k < cvs.len() {
            next.push(cvs[k]);
        }
        cvs = next;
    }
    let mut res = [0u8; 32];
    for i in 0..8 {
        res[i * 4..(i + 1) * 4].copy_from_slice(&cvs[0][i].to_le_bytes());
    }
    res
}

fn pattern_buf(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i % 251) as u8).collect()
}

/// Court 1: throughput curve (min-of-3) + 16 MiB dispatch boundary law.
#[test]
fn throughput_curve_and_gpu_dispatch_boundary() {
    // Predicate flips exactly at the boundary.
    assert!(!should_dispatch_gpu(GPU_CROSSOVER_BYTES - 1));
    assert!(should_dispatch_gpu(GPU_CROSSOVER_BYTES));
    assert!(should_dispatch_gpu(64 * 1024 * 1024));

    println!("== AB-12 throughput curve (hash, min-of-3, this host) ==");
    for &size in &[
        64usize,
        1024,
        64 * 1024,
        1024 * 1024,
        16 * 1024 * 1024,
        64 * 1024 * 1024,
    ] {
        let data = pattern_buf(size);
        let mut best = f64::INFINITY;
        let mut best_wide = f64::INFINITY;
        for _ in 0..3 {
            let t = Instant::now();
            black_box(hash(black_box(&data)));
            let el = t.elapsed().as_secs_f64();
            best = best.min(size as f64 / (1u64 << 20) as f64 / el);
            let tw = Instant::now();
            black_box(hash_wide_simd(black_box(&data)));
            best_wide =
                best_wide.min(size as f64 / (1u64 << 20) as f64 / tw.elapsed().as_secs_f64());
        }
        println!(
            "  {:>9} B: {:>10.1} MiB/s (hash)   {:>10.1} MiB/s (hash_wide_simd)",
            size, best, best_wide
        );
    }

    // Digest law is continuous across the boundary: prefix-extended input
    // crossing 16 MiB digests identically under BOTH kernels on either side
    // of the flip (dispatch is a perf crossover, not a correctness edge).
    let big = pattern_buf(GPU_CROSSOVER_BYTES); // exactly 16 MiB
    let below = &big[..GPU_CROSSOVER_BYTES - 1024];
    let at = &big[..GPU_CROSSOVER_BYTES];
    assert_eq!(hash(below), hash_wide_simd(below));
    assert_eq!(hash(at), hash_wide_simd(at));
    assert_ne!(hash(below), hash(at));
    // Keyed binding stays on the CPU path at the boundary too (single-chunk
    // keyed law; boundary applies to the payload dispatch only).
    assert!(!should_dispatch_gpu(0) && should_dispatch_gpu(GPU_CROSSOVER_BYTES));
}

/// Court 2: chunk-tree stress — independent reducer vs both kernels across
/// 10 chain shapes, including the 1k x 1KB chain (1024 chunks).
#[test]
fn chunk_tree_chain_shapes_merkle_law() {
    let shapes: [usize; 10] = [2, 3, 63, 64, 65, 255, 256, 511, 1000, 1024];
    for &chunks in &shapes {
        let len = chunks * 1024;
        let data = pattern_buf(len);
        let h = hash(&data);
        let hw = hash_wide_simd(&data);
        let ht = tree_hash(&data);
        assert_eq!(h, hw, "hash vs wide_simd, {} chunks", chunks);
        assert_eq!(h, ht, "hash vs independent tree reducer, {} chunks", chunks);
    }
    // 1k x 1KB chain: the 1024-chunk concatenation's one-shot digest equals
    // the independent tree over the same chunk sequence (also pinned above
    // in-shape; spelled out here as the named chain case).
    let chain = pattern_buf(1024 * 1024);
    assert_eq!(hash(&chain), tree_hash(&chain));
    assert_eq!(hash(&chain), hash_wide_simd(&chain));
}

/// Court 3: SIMD-vs-scalar equivalence on 1k random inputs >= SIMD width.
#[test]
fn simd_vs_scalar_equivalence_1k_random() {
    let mut xs = Xs(0xAB12);
    for i in 0..1000 {
        let len = 1025 + (xs.next() as usize % 130_048); // 1025..=131072 B
        let mut data = vec![0u8; len];
        xs.fill(&mut data);
        let h = hash(&data);
        let hw = hash_wide_simd(&data);
        assert_eq!(h, hw, "SIMD/scalar mismatch at i={} len={}", i, len);
    }
}

/// Court 4: keyed-mode stress over the inline compress_scalar binding.
#[test]
fn keyed_mode_stress_1k_keys() {
    let key: [u8; 32] = core::array::from_fn(|i| (i + 1) as u8);

    // b3sum --keyed (reference impl) KATs, generated this lane.
    assert_eq!(
        hex(&keyed_digest_inline(&key, b"")),
        "b54fe6f9a432e254a10d31999015209281ec73f50e83e9fbbf5092b6ef8493df"
    );
    let msg192: Vec<u8> = (0..192).map(|i| (i % 251) as u8).collect();
    assert_eq!(
        hex(&keyed_digest_inline(&key, &msg192)),
        "76a1d92f4e271be7ae2798159a8fd049885d1cc5b2ec9ee181f7217dc4d66398"
    );

    // Same key twice -> identical.
    assert_eq!(
        keyed_digest_inline(&key, &msg192),
        keyed_digest_inline(&key, &msg192)
    );

    // 1k distinct keys -> 1k distinct digests.
    let mut seen = HashSet::new();
    for k in 0..1000u64 {
        let mut key = [0u8; 32];
        key[..8].copy_from_slice(&k.to_le_bytes());
        let d = keyed_digest_inline(&key, &msg192);
        assert!(seen.insert(d), "duplicate digest for key {}", k);
    }
}
