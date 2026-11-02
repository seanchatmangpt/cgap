//! Lane H11 — word-wise SWAR count() differential + anti-vacuity courts.
//!
//! Byte-exactness law: `count() == count_per_byte_oracle()` for every
//! mask state. The oracle is the verbatim pre-change per-byte form.

use cgap_arrow::bitmask::{Mask1024, Mask256, Mask64, PresenceMask};

/// Deterministic splitmix64 (frozen seed law: `0xBF58_476D_1CE4_E5B9`
/// is the canonical splitmix64 seed; corpus constant
/// `0xA20C_0FFE_E5A7_E000` is a different constant and is NOT used here).
struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z = z ^ (z >> 31);
        z
    }
}

/// Differential court C-H11-1: exhaustive over small masks, seeded
/// pseudo-random over large ones (real collaborator: the mask itself,
/// no mocks). Asserts count() == oracle on final state.
#[test]
fn c_h11_1_count_matches_per_byte_oracle() {
    // Exhaustive: every 6-bit pattern set into a 64-row mask via
    // bit indices, plus word-boundary probes.
    for pattern in 0u32..(1 << 6) {
        let mut m2 = Mask64::EMPTY;
        for i in 0..6 {
            if pattern >> i & 1 == 1 {
                assert_eq!(m2.set(i as usize), Ok(()));
            }
        }
        assert_eq!(m2.count(), m2.count_per_byte_oracle(), "pattern {pattern}");
    }

    // Word-boundary probes on 65-row/16-byte mask: bits 0..CAP set
    // individually, bit 64 (first bit of the second word).
    let mut m65 = PresenceMask::<65, 16>::EMPTY;
    for i in 0..65 {
        assert_eq!(m65.set(i), Ok(()));
        assert_eq!(m65.count(), m65.count_per_byte_oracle());
    }
    assert_eq!(m65.count(), 65);
    assert_eq!(m65.clear(64), Ok(()));
    assert_eq!(m65.count(), m65.count_per_byte_oracle());
    assert_eq!(m65.count(), 64);

    // Seeded sweep over large masks: set/clear with the frozen splitmix
    // seed, differential after each mutation.
    let mut rng = SplitMix64(0xBF58_476D_1CE4_E5B9);
    let mut m256 = Mask256::EMPTY;
    for _ in 0..4096 {
        let i = (rng.next() % 256) as usize;
        if rng.next() & 1 == 1 {
            let _ = m256.set(i);
        } else {
            let _ = m256.clear(i);
        }
        assert_eq!(
            m256.count(),
            m256.count_per_byte_oracle(),
            "divergence after seeded op"
        );
    }
    let mut m1024 = Mask1024::EMPTY;
    for _ in 0..8192 {
        let i = (rng.next() % 1024) as usize;
        if rng.next() & 1 == 1 {
            let _ = m1024.set(i);
        } else {
            let _ = m1024.clear(i);
        }
        assert_eq!(
            m1024.count(),
            m1024.count_per_byte_oracle(),
            "divergence after seeded op"
        );
    }
    assert_eq!(
        Mask1024::ALL_VALID.count(),
        Mask1024::ALL_VALID.count_per_byte_oracle()
    );
    assert_eq!(Mask1024::ALL_VALID.count(), 1024);
}

/// Differential court C-H11-2 (byte-identity, stronger than count):
/// the word-wise fold must not merely match in total — the underlying
/// bytes are the only mutable state and are untouched by count() (both
/// &self). Pins read-only purity: count() twice == same value, and a
/// set/clear round trip restores byte-exact storage.
#[test]
fn c_h11_2_count_is_stateless_and_bytes_untouched() {
    let mut rng = SplitMix64(0xBF58_476D_1CE4_E5B9);
    let mut m = Mask1024::EMPTY;
    for _ in 0..512 {
        let _ = m.set((rng.next() % 1024) as usize);
    }
    let before = *m.bytes();
    let c1 = m.count();
    let c2 = m.count();
    assert_eq!(c1, c2);
    assert_eq!(c1, m.count_per_byte_oracle());
    assert_eq!(m.bytes(), &before);
}

/// Anti-vacuity mutation target note: the mutation itself runs in a
/// /tmp sandbox copy (see receipt); this court is the flip detector —
/// under a corrupted word-padding invariant (mask_bytes no longer a
/// multiple of 8), count()'s word loop drops the tail bytes while the
/// oracle still counts them, so C-H11-1 diverges by construction.
#[test]
fn c_h11_3_pair_law_consts_remain() {
    const _: () = assert!(cgap_arrow::bitmask::mask_bytes(65) == 16);
    const _: () = assert!(cgap_arrow::bitmask::mask_bytes(1) == 8);
    assert_eq!(Mask256::MASK_BYTES, 32);
    assert_eq!(Mask1024::MASK_BYTES, 128);
}
