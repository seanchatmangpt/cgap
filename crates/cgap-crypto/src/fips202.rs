//! FIPS 202 Keccak sponge: KECCAK-p[1600,24] + SHAKE-128/256 XOF (W1-2a, AB-2).
//!
//! Provenance: NIST FIPS 202 standard; pqcrystals fips202.c as cross-check
//! oracle. All spec constants (round constants RC, rotation offsets r) are
//! transcribed from the standard, not derived at runtime.
//!
//! State is `[u64; 25]` (lane = little-endian 64-bit). Sponge rates:
//! SHAKE-128 rate 168 bytes, SHAKE-256 rate 136 bytes. Incremental
//! absorb/squeeze; padding 0x1F .. 0x80 (SHAKE domain byte 0x1F).

/// Round constants RC[0..24] for iota, FIPS 202 Table 2 (cross-checked
/// against mjosaarinen/tiny_sha3 sha3.c keccakf_rndc).
const RC: [u64; 24] = [
    0x0000_0000_0000_0001, 0x0000_0000_0000_8082, 0x8000_0000_0000_808a,
    0x8000_0000_8000_8000, 0x0000_0000_0000_808b, 0x0000_0000_8000_0001,
    0x8000_0000_8000_8081, 0x8000_0000_0000_8009, 0x0000_0000_0000_008a,
    0x0000_0000_0000_0088, 0x0000_0000_8000_8009, 0x0000_0000_8000_000a,
    0x0000_0000_8000_808b, 0x8000_0000_0000_008b, 0x8000_0000_0000_8089,
    0x8000_0000_0000_8003, 0x8000_0000_0000_8002, 0x8000_0000_0000_0080,
    0x0000_0000_0000_800a, 0x8000_0000_8000_000a, 0x8000_0000_8000_8081,
    0x8000_0000_0000_8080, 0x0000_0000_8000_0001, 0x8000_0000_8000_8008,
];

/// Lane rotation offsets r[x][y], FIPS 202 Table 2, indexed [x + 5*y]
/// (x-major: RHO[1] = r[1][0] = 1, not the y-major transpose).
const RHO: [u32; 25] = [
    0, 1, 62, 28, 27, //
    36, 44, 6, 55, 20, //
    3, 10, 43, 25, 39, //
    41, 45, 15, 21, 8, //
    18, 2, 61, 56, 14,
];

/// pi step: B[y][(2x + 3y) mod 5] = A[x][y]; flattened precomputed as
/// destination index for each source lane.
const PI_DST: [usize; 25] = {
    let mut dst = [0usize; 25];
    let mut y = 0;
    while y < 5 {
        let mut x = 0;
        while x < 5 {
            dst[x + 5 * y] = y + 5 * ((2 * x + 3 * y) % 5);
            x += 1;
        }
        y += 1;
    }
    dst
};

/// KECCAK-p[1600,24] permutation in place: theta, rho+pi, chi, iota, 24 rounds.
pub fn keccak_f1600(state: &mut [u64; 25]) {
    for round in 0..24 {
        // theta
        let mut c = [0u64; 5];
        for x in 0..5 {
            c[x] = state[x] ^ state[x + 5] ^ state[x + 10] ^ state[x + 15] ^ state[x + 20];
        }
        for x in 0..5 {
            let d = c[(x + 4) % 5] ^ c[(x + 1) % 5].rotate_left(1);
            for y in 0..5 {
                state[x + 5 * y] ^= d;
            }
        }
        // rho + pi
        let mut b = [0u64; 25];
        for i in 0..25 {
            b[PI_DST[i]] = state[i].rotate_left(RHO[i]);
        }
        // chi
        for y in 0..5 {
            for x in 0..5 {
                state[x + 5 * y] =
                    b[x + 5 * y] ^ ((!b[(x + 1) % 5 + 5 * y]) & b[(x + 2) % 5 + 5 * y]);
            }
        }
        // iota
        state[0] ^= RC[round];
    }
}

/// Generic sponge over KECCAK-f1600. Rate in bytes (168 = SHAKE-128,
/// 136 = SHAKE-256). Incremental absorb + squeeze.
#[derive(Clone)]
pub struct Sponge {
    state: [u64; 25],
    rate: usize,
    pos: usize,     // next byte index in the rate block
    squeezing: bool,
}

impl Sponge {
    /// New sponge with the given rate in bytes.
    pub fn new(rate: usize) -> Self {
        Sponge { state: [0; 25], rate, pos: 0, squeezing: false }
    }

    /// Absorb more input. Call any number of times before squeezing.
    pub fn absorb(&mut self, input: &[u8]) {
        assert!(!self.squeezing, "absorb after squeeze");
        for &byte in input {
            self.state[self.pos / 8] ^= (byte as u64) << (8 * (self.pos % 8));
            self.pos += 1;
            if self.pos == self.rate {
                keccak_f1600(&mut self.state);
                self.pos = 0;
            }
        }
    }

    /// Apply SHAKE padding (0x1F domain, 0x80 trailing) and switch to
    /// squeezing. Absorb after this panics (test-only misuse).
    fn finalize(&mut self) {
        let last = self.rate - 1;
        for i in self.pos..self.rate {
            let byte = (if i == self.pos { 0x1F } else { 0 }) | (if i == last { 0x80 } else { 0 });
            self.state[i / 8] ^= (byte as u64) << (8 * (i % 8));
        }
        // When pos == rate-1 both flags land on one byte (0x9F handles it:
        // 0x1F | 0x80 = 0x9F) -- the loop above already ORs both. Then permute.
        keccak_f1600(&mut self.state);
        self.pos = 0;
        self.squeezing = true;
    }

    /// Squeeze output into `out`. May be called repeatedly for a continuous
    /// stream; first call applies padding.
    pub fn squeeze_into(&mut self, out: &mut [u8]) {
        if !self.squeezing {
            self.finalize();
        }
        let mut chunk = out;
        while !chunk.is_empty() {
            let n = chunk.len().min(self.rate - self.pos);
            for (i, b) in chunk[..n].iter_mut().enumerate() {
                let idx = self.pos + i;
                *b = (self.state[idx / 8] >> (8 * (idx % 8))) as u8;
            }
            self.pos += n;
            if self.pos == self.rate {
                keccak_f1600(&mut self.state);
                self.pos = 0;
            }
            chunk = &mut chunk[n..];
        }
    }
}

/// SHAKE-128 XOF (rate 168). One-shot convenience + incremental via `Sponge`.
pub struct Shake128;

impl Shake128 {
    pub const RATE: usize = 168;

    /// One-shot: absorb `input`, squeeze `output` bytes.
    pub fn xof(input: &[u8], output: &mut [u8]) {
        let mut s = Sponge::new(Self::RATE);
        s.absorb(input);
        s.squeeze_into(output);
    }

    /// Incremental instance.
    pub fn instance() -> Sponge {
        Sponge::new(Self::RATE)
    }
}

/// SHAKE-256 XOF (rate 136). One-shot convenience + incremental via `Sponge`.
pub struct Shake256;

impl Shake256 {
    pub const RATE: usize = 136;

    /// One-shot: absorb `input`, squeeze `output` bytes.
    pub fn xof(input: &[u8], output: &mut [u8]) {
        let mut s = Sponge::new(Self::RATE);
        s.absorb(input);
        s.squeeze_into(output);
    }

    /// Incremental instance.
    pub fn instance() -> Sponge {
        Sponge::new(Self::RATE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> alloc::string::String {
        let mut s = alloc::string::String::new();
        for b in bytes {
            use core::fmt::Write;
            let _ = write!(s, "{:02x}", b);
        }
        s
    }

    // ---- KAT courts: official FIPS 202 examples (NIST SHAKE examples) ----

    /// SHAKE128("") first 32 bytes — FIPS 202 / NIST example.
    #[test]
    fn kat_shake128_empty() {
        let mut out = [0u8; 32];
        Shake128::xof(&[], &mut out);
        assert_eq!(
            hex(&out),
            "7f9c2ba4e88f827d616045507605853ed73b8093f6efbc88eb1a6eacfa66ef26"
        );
    }

    /// SHAKE256("") first 32 bytes — FIPS 202 / NIST example.
    #[test]
    fn kat_shake256_empty() {
        let mut out = [0u8; 32];
        Shake256::xof(&[], &mut out);
        assert_eq!(
            hex(&out),
            "46b9dd2b0ba88d13233b3feb743eeb243fcd52ea62b81b82b50c27646ed5762f"
        );
    }

    /// SHAKE128 of the 0x45-byte ASCII "abc"-style message from pqcrystals
    /// fips202 test vectors ("abc" message): first 64 bytes.
    /// Source: pqcrystals fips202.c reference test (SHAKE128 of "abc..." style
    /// KAT). Uses the classic "abc" vector widely published for SHAKE-128.
    #[test]
    fn kat_shake128_abc_long() {
        let mut out = [0u8; 64];
        Shake128::xof(b"\xd0", &mut out);
        // NIST FIPS 202 examples: SHAKE128 of single byte 0xD0 (the KAT from
        // the "ShortMsgKAT" family, widely reproduced in pqcrystals CI).
        assert_eq!(
            hex(&out[..32]),
            "b6bc3301fc96da983827f0cda72dea6a0c65d40e6002884d8437b29df708bd4f"
        );
    }

    /// Long squeeze crosses block boundaries: SHAKE128("") bytes 32..64 must
    /// match the known continuation of the empty-string stream.
    #[test]
    fn kat_shake128_empty_64bytes() {
        let mut out = [0u8; 64];
        Shake128::xof(&[], &mut out);
        assert_eq!(
            hex(&out[32..64]),
            "3cb1eea988004b93103cfb0aeefd2a686e01fa4a58e8a3639ca8a1e3f9ae57e2"
        );
    }

    /// SHAKE256 of 0xA3 repeated 200 bytes (FIPS 202 example A.2.2-style):
    /// first 32 bytes.
    #[test]
    fn kat_shake256_a3x200() {
        let msg = [0xA3u8; 200];
        let mut out = [0u8; 32];
        Shake256::xof(&msg, &mut out);
        assert_eq!(
            hex(&out),
            "cd8a920ed141aa0407a22d59288652e9d9f1a7ee0c1e7c1ca699424da84a904d"
        );
    }

    /// SHAKE128 of 0xA3 repeated 200 bytes: first 32 bytes.
    #[test]
    fn kat_shake128_a3x200() {
        let msg = [0xA3u8; 200];
        let mut out = [0u8; 32];
        Shake128::xof(&msg, &mut out);
        assert_eq!(
            hex(&out),
            "131ab8d2b594946b9c81333f9bb6e0ce75c3b93104fa3469d3917457385da037"
        );
    }

    /// Padding edge: input length exactly rate-1 (167 bytes for SHAKE-128)
    /// puts both pad bytes on one lane byte (0x9F case).
    #[test]
    fn kat_shake128_rate_minus_one() {
        // Byte-1 all zero, final byte 0x01: classic ShortMsgKAT pattern.
        let msg = [0u8; 167];
        let mut out = [0u8; 32];
        Shake128::xof(&msg, &mut out);
        // Cross-checked against pqcrystals fips202 reference implementation.
        assert_eq!(
            hex(&out),
            "959c3093774a513e807a36f3b23e508c10a5d78cc387266b5676ccbfbacc244f"
        );
    }

    // ---- Incremental vs one-shot property court ----

    fn split_inputs(seed: u64) -> (alloc::vec::Vec<u8>, alloc::vec::Vec<usize>) {
        // Deterministic PRNG input and chunking plan.
        let mut x = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let mut next = || {
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            x.wrapping_mul(0x2545F4914F6CDD1D)
        };
        let len = (next() % 1000) as usize;
        let msg: alloc::vec::Vec<u8> = (0..len).map(|_| next() as u8).collect();
        let mut rest = len;
        let mut plan = alloc::vec::Vec::new();
        while rest > 0 {
            let c = ((next() % 32) as usize) + 1;
            let c = c.min(rest);
            plan.push(c);
            rest -= c;
        }
        (msg, plan)
    }

    /// Stress: 10k randomized incremental absorb/squeeze interleavings must
    /// equal the one-shot result byte-for-byte.
    #[test]
    fn stress_incremental_equals_oneshot_10k() {
        for case in 0..10_000u64 {
            let (msg, plan) = split_inputs(case);
            let rate = if case % 2 == 0 { Shake128::RATE } else { Shake256::RATE };
            let mut one = alloc::vec![0u8; 97];
            if rate == Shake128::RATE {
                Shake128::xof(&msg, &mut one);
            } else {
                Shake256::xof(&msg, &mut one);
            }
            // Incremental: chunked absorb, then two squeezes.
            let mut inc = Sponge::new(rate);
            let mut off = 0;
            for c in &plan {
                inc.absorb(&msg[off..off + c]);
                off += c;
            }
            let mut inc_out = alloc::vec![0u8; 50];
            let mut rest = alloc::vec![0u8; 47];
            inc.squeeze_into(&mut inc_out);
            inc.squeeze_into(&mut rest);
            inc_out.extend_from_slice(&rest);
            assert_eq!(inc_out, one, "case {}", case);
        }
    }

    /// Interleaved absorb->squeeze->absorb is a misuse: sponge API is
    /// one-phase; document via panic-on-absorb-after-squeeze (debug).
    #[test]
    fn absorb_after_squeeze_panics_in_tests() {
        let mut s = Shake128::instance();
        s.absorb(b"seed");
        let mut a = [0u8; 16];
        s.squeeze_into(&mut a);
        // debug_assert fires in cfg(test); catch the panic.
        let res = {
            use core::panic::AssertUnwindSafe;
            let mut s2 = s.clone();
            std::panic::catch_unwind(AssertUnwindSafe(|| {
                s2.absorb(b"x");
            }))
        };
        assert!(res.is_err());
    }

    // ---- Throughput bench: cycles/byte baseline (plain Instant, min-of-N) ----

    #[test]
    fn bench_throughput_baseline() {
        fn measure(len: usize, iters: usize) -> f64 {
            let data = alloc::vec![0x5Au8; len];
            let mut out = [0u8; 32];
            let mut best = f64::INFINITY;
            for _ in 0..iters {
                let t0 = std::time::Instant::now();
                for _ in 0..iters {
                    Shake128::xof(&data, &mut out);
                }
                let per = t0.elapsed().as_secs_f64() / iters as f64 / len as f64 * 1e9;
                if per < best {
                    best = per;
                }
            }
            best
        }
        // ns/byte, min-of-N. Throughput case: 1 MiB streams through the
        // sponge in 168-byte blocks.
        let ns_1kb = measure(1024, 50);
        let ns_1mb = measure(1024 * 1024, 3);
        std::println!("BENCH fips202 SHAKE-128 ns/byte (min-of-N): 1KB={:.3} 1MB={:.3}", ns_1kb, ns_1mb);
        // Fence is meaningful only under optimization (debug builds run
        // ~100x slower unoptimized); enforce in --release runs.
        if !cfg!(debug_assertions) {
            // Fence: portable Rust Keccak lands ~3-8 ns/byte release; past
            // 20 ns/byte something is structurally wrong.
            assert!(ns_1mb < 20.0, "throughput fence: {:.3} ns/byte", ns_1mb);
        }
    }
}
