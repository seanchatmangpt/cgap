//! Lane R7-I: shared BLAKE3 vector court over the canonical fixture
//! (vendored byte-identical copy of
//! /Users/sac/smalt/verification/blake3_vectors.json,
//! sha256 dbbcd0c57e177447dd502c24d3b40c841ae70798e906081cb0d7ce35cb617fc6).
//!
//! Laws asserted per vector:
//! 1. `hash` matches the minted digest (digest correctness vs the shared
//!    fixture minted from this very crate at
//!    a216b486fe7d312ad982f66b158c30ae493ec09 -- a drift fence, not a
//!    tautology: if the crate's hash law moves without a re-mint, this fails).
//! 2. `hash_wide_simd` == `hash_many_simd` == `hash` per vector (SIMD
//!    differential; the 2 MiB+1 row spans >= 3 Merkle levels across the
//!    SIMD drivers -- the SIMD witness).
//!
//! Zero-dep law: the fixture sha256 self-pin uses the in-test SHA-256 below
//! (the crate itself is no_std zero-dependency; adding a dev-dep is outside
//! the lane's owned files).

use cgap_blake3::{hash, hash_many_simd, hash_wide_simd};

const FIXTURE: &str = include_str!("blake3_vectors.json");
const SHA256_PIN: &str = "dbbcd0c57e177447dd502c24d3b40c841ae70798e906081cb0d7ce35cb617fc6";

/// Minimal FIPS 180-4 SHA-256 (test-only, zero-dep law).
mod sha256 {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    const H0: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    fn rotr(x: u32, n: u32) -> u32 {
        x.rotate_right(n)
    }

    pub fn sum(b: &[u8]) -> [u8; 32] {
        let mut h = H0;
        let bitlen = (b.len() as u64) * 8;
        let mut msg = b.to_vec();
        msg.push(0x80);
        while msg.len() % 64 != 56 {
            msg.push(0);
        }
        msg.extend_from_slice(&bitlen.to_be_bytes());

        for block in msg.chunks(64) {
            let mut w = [0u32; 64];
            for (i, chunk) in block.chunks(4).enumerate() {
                w[i] = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            }
            for t in 16..64 {
                let s0 = rotr(w[t - 15], 7) ^ rotr(w[t - 15], 18) ^ (w[t - 15] >> 3);
                let s1 = rotr(w[t - 2], 17) ^ rotr(w[t - 2], 19) ^ (w[t - 2] >> 10);
                w[t] = w[t - 16]
                    .wrapping_add(s0)
                    .wrapping_add(w[t - 7])
                    .wrapping_add(s1);
            }
            let mut v = h;
            for t in 0..64 {
                let s1 = rotr(v[4], 6) ^ rotr(v[4], 11) ^ rotr(v[4], 25);
                let ch = (v[4] & v[5]) ^ ((!v[4]) & v[6]);
                let temp1 = v[7]
                    .wrapping_add(s1)
                    .wrapping_add(ch)
                    .wrapping_add(K[t])
                    .wrapping_add(w[t]);
                let s0 = rotr(v[0], 2) ^ rotr(v[0], 13) ^ rotr(v[0], 22);
                let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
                let temp2 = s0.wrapping_add(maj);
                v[7] = v[6];
                v[6] = v[5];
                v[5] = v[4];
                v[4] = v[3].wrapping_add(temp1);
                v[3] = v[2];
                v[2] = v[1];
                v[1] = v[0];
                v[0] = temp1.wrapping_add(temp2);
            }
            for i in 0..8 {
                h[i] = h[i].wrapping_add(v[i]);
            }
        }

        let mut out = [0u8; 32];
        for (i, word) in h.iter().enumerate() {
            out[i * 4..(i + 1) * 4].copy_from_slice(&word.to_be_bytes());
        }
        out
    }
}

/// Parse the fixture's vector rows: (name, size, digest_hex).
/// Hand-rolled scanner (zero-dep law): matches
/// `{"name": "...", "size": N, "input_law": "...", "digest": "<64 hex>"}`.
fn parse_fixture(s: &str) -> Vec<(String, usize, String)> {
    let mut out = Vec::new();
    for line in s.lines() {
        let line = line.trim();
        if !line.starts_with("{\"name\":") {
            continue;
        }
        let get_field = |key: &str| -> Option<String> {
            let start = line.find(&format!("\"{key}\":"))? + key.len() + 4;
            let rest = &line[start..];
            if rest.starts_with('"') {
                let end = rest[1..].find('"')? + 1;
                Some(rest[1..end].to_string())
            } else {
                let end = rest.find([',', '}'])?;
                Some(rest[..end].trim().to_string())
            }
        };
        let (Some(name), Some(size_raw), Some(digest)) =
            (get_field("name"), get_field("size"), get_field("digest"))
        else {
            continue;
        };
        let Ok(size) = size_raw.parse::<usize>() else {
            continue;
        };
        out.push((name, size, digest));
    }
    out
}

#[test]
fn fixture_sha256_self_pin() {
    let digest = sha256::sum(FIXTURE.as_bytes());
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(
        hex, SHA256_PIN,
        "vendored fixture drifted from canonical mint"
    );
}

#[test]
fn shared_vectors_hash_and_simd_differential() {
    let vectors = parse_fixture(FIXTURE);
    assert!(
        vectors.len() >= 11,
        "fixture lost vectors: {} rows",
        vectors.len()
    );
    let mut simd_witness = false;
    for (name, size, digest_hex) in vectors {
        let input: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
        let h = hash(&input);
        let w = hash_wide_simd(&input);
        let m = hash_many_simd(&input);
        let hex: String = h.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex, digest_hex, "hash mismatch on {name}");
        assert_eq!(w, h, "hash_wide_simd mismatch on {name}");
        assert_eq!(m, w, "hash_many_simd mismatch on {name}");
        if size == 2_097_153 {
            simd_witness = true;
        }
    }
    assert!(
        simd_witness,
        "SIMD witness vector (size 2097153) missing from fixture"
    );
}
