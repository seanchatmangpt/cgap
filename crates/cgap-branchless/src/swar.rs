//! SWAR (SIMD Within A Register) byte-wise kernels — lane B3.
//!
//! Mask conventions (the contract, mirrored by the `#[cfg(test)]` naive
//! oracles below):
//!
//! * `swar_eq_u64` — per-byte equality mask: `0xFF` in every byte position
//!   where `word` and `target` agree, `0x00` elsewhere.
//! * `swar_has_zero_byte` — `true` iff at least one byte of `v` is `0x00`.
//! * `swar_range_mask` — per-byte membership mask: `0xFF` where
//!   `min <= byte <= max` (unsigned, inclusive), `0x00` elsewhere.
//! * Internal `swar_lt` — per-byte unsigned `x < y`: `0x80` in the high bit
//!   of every byte position where `x`'s byte is strictly less than `y`'s.
//!
//! All functions are branch-free: no `cmp`+`br` pairs, no selects. The
//! zero-byte detect is the classic `(v - 0x0101..01) & !v & 0x8080..80`;
//! equality is XOR followed by the exact borrow-free nonzero-per-byte
//! detect `((x & ~HI) + ~HI) | x` (bit 7 set iff byte != 0, no cross-byte
//! borrow — the shorter `(x - LO) & ~x & HI` form false-positives on 0x01
//! bytes that borrow into the lane above), expanded to `0xFF` lanes.
#![allow(clippy::needless_range_loop)]

const HI: u64 = 0x8080_8080_8080_8080;

/// Per-byte unsigned `x < y`: bit 7 of each byte set where `x`'s byte is
/// strictly less than `y`'s byte, all lanes else clear.
///
/// Split each byte `a` into its high bit `h` and low seven bits `l`.
/// `a >= b` iff `h_a > h_b`, or `h_a == h_b` and `l_a >= l_b`. The low-seven
/// comparison is borrow-free: `(x | HI) - (y & !HI)` never crosses a byte
/// boundary (minuend byte in `[0x80, 0xFF]`, subtrahend in `[0x00, 0x7F]`),
/// and its bit 7 is exactly `l_a >= l_b`.
#[inline]
pub const fn swar_lt(x: u64, y: u64) -> u64 {
    let low_ge = (x | HI).wrapping_sub(y & !HI) & HI;
    let high_gt = x & !y & HI;
    let high_eq = !(x ^ y) & HI;
    let ge = high_gt | (high_eq & low_ge);
    !ge & HI
}

/// Per-byte equality mask: `0xFF` in every byte where `word == target`,
/// `0x00` in every byte where they differ.
#[inline]
pub const fn swar_eq_u64(word: u64, target: u64) -> u64 {
    let x = word ^ target;
    let nonzero = (x & !HI).wrapping_add(!HI) | x;
    let nonzero = nonzero & HI;
    ((nonzero ^ HI) >> 7).wrapping_mul(0xFF)
}

/// `true` iff at least one byte of `v` is `0x00`.
#[inline]
pub const fn swar_has_zero_byte(v: u64) -> bool {
    ((v & !HI).wrapping_add(!HI) | v) & HI != HI
}

/// Per-byte inclusive range mask: `0xFF` where `min <= byte <= max`
/// (unsigned), `0x00` elsewhere. Empty when `min > max`.
#[inline]
pub const fn swar_range_mask(val: u64, min: u8, max: u8) -> u64 {
    let lt_min = swar_lt(val, swar_splat(min));
    let gt_max = swar_lt(swar_splat(max), val);
    let inside = !(lt_min | gt_max) & HI;
    (inside >> 7).wrapping_mul(0xFF)
}

/// Broadcast `b` to all eight byte lanes.
#[inline]
pub const fn swar_splat(b: u8) -> u64 {
    u64::from_ne_bytes([b; 8])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Naive oracle: per-byte 0xFF/0x00 equality mask.
    fn naive_eq(word: u64, target: u64) -> u64 {
        let mut r = 0u64;
        for i in 0..8 {
            let sh = 8 * (7 - i);
            let a = (word >> sh) & 0xFF;
            let b = (target >> sh) & 0xFF;
            if a == b {
                r |= 0xFF << sh;
            }
        }
        r
    }

    /// Naive oracle: any byte exactly zero.
    fn naive_has_zero(v: u64) -> bool {
        (0..8).any(|i| (v >> (8 * i)) & 0xFF == 0)
    }

    /// Naive oracle: per-byte 0xFF where min <= byte <= max.
    fn naive_range(val: u64, min: u8, max: u8) -> u64 {
        let mut r = 0u64;
        for i in 0..8 {
            let b = ((val >> (8 * (7 - i))) & 0xFF) as u8;
            if b >= min && b <= max {
                r |= 0xFFu64 << (8 * (7 - i));
            }
        }
        r
    }

    fn corpus_splitmix64(mut state: u64) -> u64 {
        state = state.wrapping_add(0xBF58_476D_1CE4_E5B9);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z ^ (z >> 31)
    }

    const CORPUS_SEED: u64 = 0xA20C_0FFE_E5A7_E000;

    /// 50k differential pairs per kernel against the naive oracles.
    #[test]
    fn swar_differential_property() {
        let mut s = CORPUS_SEED;
        for _ in 0..50_000 {
            let x = corpus_splitmix64(s);
            let y = corpus_splitmix64(x);
            s = y;
            assert_eq!(
                swar_eq_u64(x, y),
                naive_eq(x, y),
                "eq x={x:#018x} y={y:#018x}"
            );
            assert_eq!(swar_has_zero_byte(x), naive_has_zero(x), "zero x={x:#018x}");
            assert_eq!(swar_eq_u64(x, y), naive_eq(x, y));
            for _ in 0..4 {
                let mn = (y >> 8 * (s % 8)) as u8;
                let mx = (y >> 8 * ((s >> 3) % 8)) as u8;
                let (lo, hi) = (mn.min(mx), mx.max(mn));
                assert_eq!(
                    swar_range_mask(x, lo, hi),
                    naive_range(x, lo, hi),
                    "range x={x:#018x} [{lo:#04x},{hi:#04x}]"
                );
            }
        }
    }

    /// Exhaustive per-byte edge sweep: both operands uniform over
    /// {0x00, 0x7F, 0x80, 0xFF} across all 8 lanes.
    #[test]
    fn swar_byte_edge_exhaustive() {
        let edges = [0x00u8, 0x7F, 0x80, 0xFF];
        for &a in &edges {
            for &b in &edges {
                let x = swar_splat(a);
                let y = swar_splat(b);
                assert_eq!(swar_eq_u64(x, y), naive_eq(x, y));
                assert_eq!(swar_has_zero_byte(x), naive_has_zero(x));
                assert_eq!(swar_lt(x, y), naive_lt_mask(x, y));
                assert_eq!(swar_range_mask(x, b, 0xFF), naive_range(x, b, 0xFF));
                assert_eq!(swar_range_mask(x, 0x00, a), naive_range(x, 0x00, a));
            }
        }
    }

    /// EXHAUSTIVE 8-bit-correctness court for `swar_lt` (repair-18, BH2
    /// lead): every one of the 65,536 (a, b) byte pairs is expanded into
    /// ALL 8 lanes, so each of the 8 * 65,536 = 524,288 lane instances is
    /// checked against the naive byte-loop oracle — not sampled. Mutation
    /// witness: any top-bit-naive form ((x|HI)-(y&!HI) & HI without the
    /// `high_gt | (high_eq & ...)` fold) returns a wrong mask on at least
    /// one lane with a >= 0x80 byte, flipping this court. A pass with
    /// this court present is non-vacuous by construction (full domain).
    #[test]
    fn swar_lt_exhaustive_full_byte_domain() {
        for a in 0u16..=255 {
            for b in 0u16..=255 {
                let (a, b) = (a as u8, b as u8);
                let x = swar_splat(a);
                let y = swar_splat(b);
                assert_eq!(
                    swar_lt(x, y),
                    naive_lt_mask(x, y),
                    "swar_lt lane-incorrect a={a:#04x} b={b:#04x}"
                );
            }
        }
    }

    /// Naive oracle for the internal less-than: 0x80 per byte where x < y.
    fn naive_lt_mask(x: u64, y: u64) -> u64 {
        let mut r = 0u64;
        for i in 0..8 {
            let sh = 8 * (7 - i);
            if ((x >> sh) & 0xFF) < ((y >> sh) & 0xFF) {
                r |= 0x80u64 << sh;
            }
        }
        r
    }

    /// Known-answer edges: all-equal, all-differ, embedded zero byte,
    /// range boundary inclusivity at 0x7F/0x80 and min>max emptiness.
    #[test]
    fn swar_known_answer_edges() {
        assert_eq!(
            swar_eq_u64(0x1212_1212_1212_1212, 0x1212_1212_1212_1212),
            0xFFFF_FFFF_FFFF_FFFF
        );
        assert_eq!(swar_eq_u64(0x1212_1212_1212_1212, 0x1313_1313_1313_1313), 0);
        assert!(swar_has_zero_byte(0x0100_2030_4050_6070));
        assert!(!swar_has_zero_byte(0x0101_0101_0101_0101));
        assert_eq!(
            swar_range_mask(swar_splat(0x7F), 0x7F, 0x80),
            0xFFFF_FFFF_FFFF_FFFF
        );
        assert_eq!(
            swar_range_mask(swar_splat(0x80), 0x7F, 0x80),
            0xFFFF_FFFF_FFFF_FFFF
        );
        assert_eq!(swar_range_mask(swar_splat(0x35), 0x40, 0x20), 0);
    }
}
