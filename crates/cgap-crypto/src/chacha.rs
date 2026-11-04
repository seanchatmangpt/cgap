//! ChaCha20 block function + pull-model keystream (RFC 8439), scalar core.
//!
//! Module: AB-3 (AoQ Wave-1 W1-3a), per PG-A3 spec.
//!
//! # Performance note: riscv32imc (no M-ext)
//!
//! RV32IMC lacks the M extension (no hardware mul/mulh). Every ChaCha
//! quarter-round needs one `(a += b) <<= 16`-style rotate and the state
//! update, which compile fine, but any 32x32 multiply emitted here (there
//! is none in the core loop — only add/xor/shift) would be softened to a
//! `__mulsi3` libgcc call. The scalar core below uses **only
//! add/xor/shift/rotate**, so rv32imc runs it at full speed with zero
//! soft-mul calls. Expected throughput on RV32IMC @ typical cores is
//! rotation-bound (~1 rotate = 2 insns via `slli`+`srli`+`or` since there
//! is no `ror`), roughly 3-5x slower than an M-ext or x86/ARM target per
//! byte; that is inherent to ChaCha20 on this ISA, not this
//! implementation. `wrapping_add` and `rotate_left` are the only ops used
//! — no division, no multiplication anywhere in the hot path.
//!
//! # Design
//!
//! - `chacha20_block(key, counter, nonce) -> [u8;64]`: free fn, scalar,
//!   10 double rounds, `#[inline(always)]` quarter-round. Allocation-free.
//! - `chacha20_xor_stream`: pull-model, consumes 64-byte blocks from the
//!   keystream and XORs them in place into the caller's buffer, advancing
//!   the counter per block. `chunks_exact(64)` fast path + stack tail for
//!   the final partial block. No alloc.
//!
//! # Receipt
//!
//! `docs/jira/v26.11.1/lanes/ab-3-chacha.md` (this lane).

// ---------------------------------------------------------------- SIGMA

/// RFC 8439 §2.3 "expand 32-byte k" constants, as u32 words.
const SIGMA: [u32; 4] = [0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574];

#[inline(always)]
fn quarter_round(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    state[a] = state[a].wrapping_add(state[b]);
    state[d] = (state[d] ^ state[a]).rotate_left(16);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] = (state[b] ^ state[c]).rotate_left(12);
    state[a] = state[a].wrapping_add(state[b]);
    state[d] = (state[d] ^ state[a]).rotate_left(8);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] = (state[b] ^ state[c]).rotate_left(7);
}

#[inline(always)]
fn double_round(state: &mut [u32; 16]) {
    // Column rounds.
    quarter_round(state, 0, 4, 8, 12);
    quarter_round(state, 1, 5, 9, 13);
    quarter_round(state, 2, 6, 10, 14);
    quarter_round(state, 3, 7, 11, 15);
    // Diagonal rounds.
    quarter_round(state, 0, 5, 10, 15);
    quarter_round(state, 1, 6, 11, 12);
    quarter_round(state, 2, 7, 8, 13);
    quarter_round(state, 3, 4, 9, 14);
}

#[inline(always)]
fn u32_from_le(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

/// One ChaCha20 block: 64 bytes of keystream for `(key, counter, nonce)`.
///
/// RFC 8439 §2.3. Serialized little-endian.
#[must_use]
pub fn chacha20_block(key: &[u8; 32], counter: u32, nonce: &[u8; 12]) -> [u8; 64] {
    let state: [u32; 16] = [
        SIGMA[0],
        SIGMA[1],
        SIGMA[2],
        SIGMA[3],
        u32_from_le(&key[0..4]),
        u32_from_le(&key[4..8]),
        u32_from_le(&key[8..12]),
        u32_from_le(&key[12..16]),
        u32_from_le(&key[16..20]),
        u32_from_le(&key[20..24]),
        u32_from_le(&key[24..28]),
        u32_from_le(&key[28..32]),
        counter,
        u32_from_le(&nonce[0..4]),
        u32_from_le(&nonce[4..8]),
        u32_from_le(&nonce[8..12]),
    ];

    let mut working = state;
    for _ in 0..10 {
        double_round(&mut working);
    }

    let mut out = [0u8; 64];
    for (i, w) in working.iter().enumerate() {
        let mixed = w.wrapping_add(state[i]);
        out[i * 4..i * 4 + 4].copy_from_slice(&mixed.to_le_bytes());
    }
    out
}

/// Pull-model XOR stream: encrypt/decrypt `buf` in place with the ChaCha20
/// keystream starting at `counter`, advancing one block per 64 bytes.
///
/// The final partial block is handled on the stack (no allocation). The
/// counter wraps mod 2^32 (`wrapping_add`), matching the stress walk gate.
pub fn chacha20_xor_stream(key: &[u8; 32], nonce: &[u8; 12], mut counter: u32, buf: &mut [u8]) {
    for chunk in buf.chunks_exact_mut(64) {
        let ks = chacha20_block(key, counter, nonce);
        for (b, k) in chunk.iter_mut().zip(ks.iter()) {
            *b ^= k;
        }
        counter = counter.wrapping_add(1);
    }
    let tail = buf.chunks_exact_mut(64).into_remainder();
    if !tail.is_empty() {
        let ks = chacha20_block(key, counter, nonce);
        for (b, k) in tail.iter_mut().zip(ks.iter()) {
            *b ^= k;
        }
    }
}

// ------------------------------------------------------------ RFC 8439

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    fn hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    /// RFC 8439 §2.3.2 — ChaCha20 block function test vector.
    #[test]
    fn rfc8439_2_3_2_block() {
        let key: [u8; 32] = hex(
            "000102030405060708090a0b0c0d0e0f\
             101112131415161718191a1b1c1d1e1f",
        )
        .try_into()
        .unwrap();
        let nonce: [u8; 12] = hex("000000090000004a00000000").try_into().unwrap();
        let out = chacha20_block(&key, 1, &nonce);
        let want = hex(
            "10f1e7e4d13b5915500fdd1fa32071c4\
             c7d1f4c733c068030422aa9ac3d46c4e\
             d2826446079faa0914c2d705d98b02a2\
             b5129cd1de164eb9cbd083e8a2503c4e",
        );
        assert_eq!(out.as_slice(), &want[..]);
    }

    /// RFC 8439 §2.4.2 — keystream (first 64 bytes = block 1), key
    /// 00..1f, nonce 000000000000004a00000000, initial counter 1.
    #[test]
    fn rfc8439_2_4_2_keystream() {
        let key: [u8; 32] = hex(
            "000102030405060708090a0b0c0d0e0f\
             101112131415161718191a1b1c1d1e1f",
        )
        .try_into()
        .unwrap();
        let nonce: [u8; 12] = hex("000000000000004a00000000").try_into().unwrap();
        let out = chacha20_block(&key, 1, &nonce);
        let want = hex(
            "224f51f3401bd9e12fde276fb8631ded8c131f823d2c06\
             e27e4fcaec9ef3cf788a3b0aa372600a92b57974cded2b\
             9334794cba40c63e34cdea212c4cf07d41b769a6749f3f",
        );
        // want is 69 bytes (23+23+23); the first 64 are block 1.
        assert_eq!(want.len(), 69);
        assert_eq!(out.as_slice(), &want[..64]);
    }

    /// RFC 8439 §2.4.2 (the spec's "§2.4.4"; RFC 8439 has no §2.4.4 —
    /// the Sunscreen encryption vector lives inside §2.4.2) — full
    /// 114-byte encryption, exact ciphertext. Key 00..1f, nonce
    /// 000000000000004a00000000, counter 1.
    #[test]
    fn rfc8439_sunscreen_encryption() {
        let key: [u8; 32] = hex(
            "000102030405060708090a0b0c0d0e0f\
             101112131415161718191a1b1c1d1e1f",
        )
        .try_into()
        .unwrap();
        let nonce: [u8; 12] = hex("000000000000004a00000000").try_into().unwrap();
        let counter = 1u32;
        let plaintext = b"Ladies and Gentlemen of the class of '99: If I could offer you \
                         only one tip for the future, sunscreen would be it.";
        let mut buf = plaintext.to_vec();
        chacha20_xor_stream(&key, &nonce, counter, &mut buf);
        let want = hex(
            "6e2e359a2568f98041ba0728dd0d6981\
             e97e7aec1d4360c20a27afccfd9fae0b\
             f91b65c5524733ab8f593dabcd62b357\
             1639d624e65152ab8f530c359f0861d8\
             07ca0dbf500d6a6156a38e088a22b65e\
             52bc514d16ccf806818ce91ab7793736\
             5af90bbf74a35be6b40b8eedf2785e42\
             874d",
        );
        assert_eq!(want.len(), 114);
        assert_eq!(buf.len(), 114);
        assert_eq!(&buf[..], &want[..]);
        // Chicago: real round-trip through the same ciphertext.
        chacha20_xor_stream(&key, &nonce, counter, &mut buf);
        assert_eq!(&buf[..], &plaintext[..]);
    }

    /// Same vector, verified via independent block calls (kept separate so
    /// a transcription error in the hex fixture cannot mask a keystream
    /// defect): XORing the ciphertext with fresh block calls must yield
    /// the plaintext back, byte-for-byte.
    #[test]
    fn rfc8439_sunscreen_independent_blocks() {
        let key: [u8; 32] = hex(
            "000102030405060708090a0b0c0d0e0f\
             101112131415161718191a1b1c1d1e1f",
        )
        .try_into()
        .unwrap();
        let nonce: [u8; 12] = hex("000000000000004a00000000").try_into().unwrap();
        let plaintext = *b"Ladies and Gentlemen of the class of '99: If I could offer you \
                          only one tip for the future, sunscreen would be it.";
        let mut buf = plaintext;
        chacha20_xor_stream(&key, &nonce, 1, &mut buf);
        // Independent verification: ct ^ ks must reproduce the plaintext.
        for (blk, chunk) in buf.chunks_exact(64).enumerate() {
            let ks = chacha20_block(&key, 1 + blk as u32, &nonce);
            for ((b, k), p) in chunk
                .iter()
                .zip(ks.iter())
                .zip(plaintext.chunks(64).nth(blk).expect("block index")) {
                assert_eq!(b ^ k, *p, "block {blk} mismatch");
            }
        }
    }

    // ------------------------------------------------------ BENCH (min-of-N)

    /// Throughput baseline: MB/s keystream at 64B / 1KB / 64KB / 1MB,
    /// min-of-N runs (min is the least-noise estimate). No external deps;
    /// `std::time::Instant` under `#[cfg(test)]` only.
    #[test]
    fn bench_keystream_throughput_baseline() {
        let key = [0x42u8; 32];
        let nonce = [0x99u8; 12];
        let sizes: [usize; 4] = [64, 1024, 64 * 1024, 1024 * 1024];
        let runs = 8usize;
        for &size in &sizes {
            let mut buf = vec![0u8; size];
            let mut min_ns = u128::MAX;
            for _ in 0..runs {
                buf.iter_mut().for_each(|b| *b = 0xA5);
                let t0 = std::time::Instant::now();
                chacha20_xor_stream(&key, &nonce, 0, &mut buf);
                let dt = t0.elapsed().as_nanos();
                if dt < min_ns {
                    min_ns = dt;
                }
            }
            let mbs = (size as f64 / (1024.0 * 1024.0)) / (min_ns as f64 / 1e9);
            eprintln!(
                "BENCH chacha20_xor_stream size={size:>7} min={min_ns:>10} ns  {mbs:>8.1} MB/s"
            );
            // Keep the buffer alive/useful so the optimizer can't elide.
            assert!(buf.iter().any(|&b| b != 0xA5));
        }
    }

    // ------------------------------------------------------------- STRESS

    /// Deterministic xorshift PRNG (no external dep; Chicago: real bytes).
    struct Xs(u64);
    impl Xs {
        fn next_u64(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
        fn fill(&mut self, buf: &mut [u8]) {
            for chunk in buf.chunks_mut(8) {
                let v = self.next_u64().to_le_bytes();
                chunk.copy_from_slice(&v[..chunk.len()]);
            }
        }
    }

    /// Counter-wrap walk: xor_stream over counters 0xFFFFFFF0..=0xFFFFFFFF
    /// (+32 blocks of material) must be byte-identical to 32 independent
    /// `chacha20_block` calls with wrapping counters.
    #[test]
    fn stress_counter_wrap_walk() {
        let key = [0x11u8; 32];
        let nonce = [0x22u8; 12];
        let blocks = 32u32;
        let start = 0xFFFF_FFF0u32;
        let len = blocks as usize * 64;
        let mut buf = vec![0u8; len];
        chacha20_xor_stream(&key, &nonce, start, &mut buf);
        for blk in 0..blocks {
            let ks = chacha20_block(&key, start.wrapping_add(blk), &nonce);
            let lo = blk as usize * 64;
            assert_eq!(
                &buf[lo..lo + 64],
                &ks[..],
                "wrap block {blk} (counter 0x{:08X}) mismatch",
                start.wrapping_add(blk)
            );
        }
    }

    /// 100k random-position round-trips: xor_stream applied twice from the
    /// same counter is the identity, on buffers whose length and content
    /// vary randomly (exercises the stack-tail path heavily).
    #[test]
    fn stress_random_position_roundtrips() {
        let mut rng = Xs(0x9E37_79B9_7F4A_7C15);
        let mut key = [0u8; 32];
        rng.fill(&mut key);
        let mut nonce = [0u8; 12];
        for i in 0..100_000u32 {
            let len = (rng.next_u64() % 600) as usize + 1; // 1..=600, crosses 64-boundary
            let mut buf = vec![0u8; len];
            rng.fill(&mut buf);
            let orig = buf.clone();
            let ctr = if i % 4 == 0 {
                (rng.next_u64() & 0xFFFF_FFFF) as u32
            } else {
                0
            };
            if i % 8 == 0 {
                rng.fill(&mut nonce);
            }
            chacha20_xor_stream(&key, &nonce, ctr, &mut buf);
            assert_ne!(buf, orig, "iteration {i}: xor changed nothing?");
            chacha20_xor_stream(&key, &nonce, ctr, &mut buf);
            assert_eq!(buf, orig, "iteration {i}: round-trip failed (len {len})");
        }
    }

    /// Tail-path exactness: buffer lengths 1..=63 XORed against an
    /// independent block call must round-trip exactly.
    #[test]
    fn tail_path_exact() {
        let key = [0x5Au8; 32];
        let nonce = [0xC3u8; 12];
        for len in 1..=63usize {
            let mut buf = vec![0u8; len];
            chacha20_xor_stream(&key, &nonce, 7, &mut buf);
            let ks = chacha20_block(&key, 7, &nonce);
            assert_eq!(&buf[..], &ks[..len]);
            chacha20_xor_stream(&key, &nonce, 7, &mut buf);
            assert!(buf.iter().all(|&b| b == 0));
        }
    }

    /// Counter=0 zero-nonce keystream is deterministic across repeated
    /// calls (conservation: same inputs, same bytes).
    #[test]
    fn determinism() {
        let key = [7u8; 32];
        let nonce = [9u8; 12];
        let a = chacha20_block(&key, 0, &nonce);
        let b = chacha20_block(&key, 0, &nonce);
        assert_eq!(a, b);
    }
}
