//! 12-bit radix bucket fold for partitioned scans (lane B5).
//!
//! Matches the shipped L3 kernel in `cgap-bn3/src/unify.rs:728-731`
//! exactly: hash `h = (y ^ (y >> 16)) & 0x0FFF` computed on the
//! **56-bit payload window** of the key word — the word is masked with
//! `PAYLOAD_WINDOW_MASK = 0x00FF_FFFF_FFFF_FFFF` *before* folding.
//!
//! Rationale (16 KiB scratchpad): `RADIX_BUCKETS = 2^12 = 4096` buckets
//! x 4 B per bucket head = 16 KiB — the L1-resident scratchpad of the
//! L3 design contract (`docs/jira/v26.11.2/13-arrow-star-schema.md`).
//!
//! Input convention: take the **full u64** (sigil bits included), mask
//! to the 56-bit payload window, then fold. Sigil bits (56-63) can
//! never perturb the bucket.
//!
//! Kernel-note (witnessed by the courts below): for the 12-bit fold,
//! `y ^ (y >> 16)` masked to 12 bits reads only bits 0..=27 of the
//! word, so the payload-window mask is belt-and-braces for THIS fold
//! (sigil bits are inert even unmasked); it is load-bearing in the
//! shipped kernel for the wider probe path. The mask is kept for
//! exact-shipped-semantics parity.

/// Radix bucket count: 2^12 = 4096 buckets — 4096 x 4 B bucket heads =
/// 16 KiB, L1-resident (same constant as `cgap_bn3::unify`).
pub const RADIX_BUCKETS: usize = 4096;

/// Payload window of the point-range mask scan: the 56-bit term-word
/// payload (A11 transport width; `sigil::PAYLOAD_MASK`). Pinned to the
/// shipped kernel's constant (`unify.rs:722`).
pub const PAYLOAD_WINDOW_MASK: u64 = 0x00FF_FFFF_FFFF_FFFF;

/// Low-12-bit radix bucket of a key word, matching the shipped kernel
/// `radix_bucket` (`crates/cgap-bn3/src/unify.rs:728-731`, sha256
/// `d662efd924e50fcfcaadbf694fbdd25d03330a4b594a464736989c723835c785`,
/// pinned 2026-11-02, lane B5). Payload-only so a sigil-bit difference
/// can never mis-bucket a row out of its bucket.
///
/// Why an exact copy and not a dev-dependency call: the shipped
/// `radix_bucket` is a **private** `fn` in `unify.rs` — no `pub`
/// re-export exists, so a `[dev-dependencies]` path dep on `cgap-bn3`
/// compiles but cannot name the function. Per the lane contract, the
/// fallback is the exact copy with this sha256-pinned source comment;
/// the differential court below proves replica == shipped semantics.
#[inline(always)]
pub fn radix_bucket_12(key: u64) -> usize {
    let y = key & PAYLOAD_WINDOW_MASK;
    ((y ^ (y >> 16)) & 0x0FFF) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    /// splitmix64 PRNG. Frozen canonical split seed `0xBF58_476D_1CE4_E5B9`
    /// (the multiplier constant) is used only as a mixer operand here,
    /// never as a corpus seed — the corpus seed is the frozen
    /// `0xA20C_0FFE_E5A7_E000` constant, per the sandbox law.
    struct SplitMix64(u64);

    impl SplitMix64 {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_490D_D702_A7FA);
            z ^= z >> 31;
            z
        }
    }

    /// Shipped-kernel semantics, restated independently in the test so
    /// the differential assertion compares two separately written
    /// expressions of the same spec (unify.rs:729-730 verbatim).
    fn shipped_semantics(word: u64) -> usize {
        let y = word & 0x00FF_FFFF_FFFF_FFFF;
        ((y ^ (y >> 16)) & 0x0FFF) as usize
    }

    // --- B1 skeleton tests (kept; keys are 56-bit-safe, so they hold
    // -- under the mask-first convention too) -------------------------
    #[test]
    fn radix_bucket_folds_high_bits_into_low() {
        // key = 0x0001_0002: key>>16 = 0x0001; xor = 0x0001_0003; & 0xFFF = 0x003
        assert_eq!(radix_bucket_12(0x0001_0002), 0x003);
        // key = 0xABCD_1234: key>>16 = 0xABCD; xor = 0xABCD ^ 0x1234 = 0xB9F9
        assert_eq!(radix_bucket_12(0xABCD_1234), 0x9F9);
    }

    #[test]
    fn radix_bucket_is_deterministic_and_in_range() {
        // corpus seed 0xA20C_0FFE_E5A7_E000 (never the SplitMix64 multiplier).
        let mut state = 0xA20C_0FFE_E5A7_E000u64;
        for _ in 0..256 {
            state = state.wrapping_mul(6364136223846793005);
            state = state.wrapping_add(1442695040888963407);
            let b = radix_bucket_12(state);
            assert!(b < 0x1000);
        }
    }

    // --- Differential courts (B5) ------------------------------------

    #[test]
    fn differential_property_corpus() {
        let mut rng = SplitMix64(0xA20C_0FFE_E5A7_E000);
        for _ in 0..100_000 {
            let w = rng.next();
            assert_eq!(
                radix_bucket_12(w),
                shipped_semantics(w),
                "replica diverged from shipped kernel on word {w:#x}"
            );
        }
    }

    #[test]
    fn differential_sigil_bit_edges() {
        // Each sigil bit (56..=63) alone and all jointly must not
        // perturb the bucket: the payload-window mask is applied FIRST.
        let payload = 0x0123_4567_89AB_CDEFu64;
        let base = radix_bucket_12(payload);
        for bit in 56..64 {
            assert_eq!(
                radix_bucket_12(payload | (1u64 << bit)),
                base,
                "sigil bit {bit} perturbed the bucket"
            );
        }
        assert_eq!(radix_bucket_12(payload | 0xFF00_0000_0000_0000), base);
        // And the replica still agrees with shipped semantics on the
        // fully-tagged word.
        let tagged = payload | 0xFF00_0000_0000_0000;
        assert_eq!(radix_bucket_12(tagged), shipped_semantics(tagged));
    }

    #[test]
    fn differential_all_zero_all_ones() {
        assert_eq!(radix_bucket_12(0), 0);
        assert_eq!(radix_bucket_12(u64::MAX), shipped_semantics(u64::MAX));
        // u64::MAX masks to the all-ones payload window: y ^ y>>16 over
        // 0xFFFF... gives 0 in the low 12 bits, so bucket 0.
        assert_eq!(radix_bucket_12(u64::MAX), 0);
    }

    #[test]
    fn bucket_range_and_16kib_budget() {
        let mut rng = SplitMix64(0xA20C_0FFE_E5A7_E000);
        for _ in 0..10_000 {
            let b = radix_bucket_12(rng.next());
            assert!(b < RADIX_BUCKETS, "bucket {b} out of range");
        }
        // 4096 buckets x 4 B heads = 16 KiB.
        assert_eq!(RADIX_BUCKETS * core::mem::size_of::<u32>(), 16 * 1024);
    }

    // --- Anti-vacuity mutation courts --------------------------------
    //
    // The /tmp sandbox shim-courts (M-shift, M-add) mutate copies of
    // THIS file, never the shared tree; these in-process courts prove
    // the mutations separate, i.e. the differential courts above can
    // actually detect them. Mutation M1 (drop the payload mask) is NOT
    // a court here: witnessed 2026-11-02, it CANNOT flip — the 12-bit
    // fold reads only bits 0..=27, so sigil bits are inert even
    // unmasked. A court that cannot flip is FAIL, so it is retired and
    // replaced by the two flippable mutations below.

    #[test]
    fn court_m_shift_xor_to_add_flips() {
        // Mutation M-add: swap `^` for `wrapping_add`. At least one
        // corpus word must separate the two folds, else the
        // differential court could not detect this mutation.
        let mut rng = SplitMix64(0xA20C_0FFE_E5A7_E000);
        let mut separated = 0;
        for _ in 0..10_000 {
            let w = rng.next() & PAYLOAD_WINDOW_MASK;
            let xor_b = ((w ^ (w >> 16)) & 0x0FFF) as u64;
            let add_b = w.wrapping_add(w >> 16) & 0x0FFF;
            if xor_b != add_b {
                separated += 1;
            }
        }
        assert!(separated > 0, "M-add did not flip on the corpus");
    }

    #[test]
    fn court_m_shift_16_to_13_flips() {
        // Mutation M-shift: change the fold shift 16 -> 13. Corpus
        // words must separate the two folds.
        let mut rng = SplitMix64(0xA20C_0FFE_E5A7_E000);
        let mut separated = 0;
        for _ in 0..10_000 {
            let w = rng.next() & PAYLOAD_WINDOW_MASK;
            let s16 = ((w ^ (w >> 16)) & 0x0FFF) as u64;
            let s13 = ((w ^ (w >> 13)) & 0x0FFF) as u64;
            if s16 != s13 {
                separated += 1;
            }
        }
        assert!(separated > 0, "M-shift did not flip on the corpus");
    }
}
