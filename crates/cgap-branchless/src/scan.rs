//! Prefix-scan primitives — fully-unrolled, const-friendly, no_std, no-alloc.
//!
//! Harvested from bcinr-logic scan.rs:199-459 (harvest-cite patterns only,
//! never imported). Lane TS2.
//!
//! Census law: every primitive is a 0-branch HOT cell — bodies are fully
//! unrolled (no loop back-edges) and rely on wrapping/cmov lowering.
//! Driver-side loops belong to the caller and are REFERENCE-DRIVER class
//! per house law.
//!
//! INTEGRATION FENCE (DREDP measured -11%): no stream-compaction win is
//! claimed for these primitives. cgap-parquet's `compact_lanes_branchless`
//! per-lane form already autovectorizes and beat a SWAR swap by 11%. The
//! primitives land as library capability; the honest A/B probe result is
//! recorded in docs/jira/v26.11.2/repairs/repair-19-scan-module.md.

/// Inclusive prefix OR over 8 u64 lanes (logarithmic Hillis-Steele,
/// shift-by-1/2/4 network). `out[i] = a[0] | a[1] | ... | a[i]`.
#[inline(always)]
pub const fn prefix_or_u64x8(a: [u64; 8]) -> [u64; 8] {
    let mut v = a;
    v[7] |= v[6];
    v[6] |= v[5];
    v[5] |= v[4];
    v[4] |= v[3];
    v[3] |= v[2];
    v[2] |= v[1];
    v[1] |= v[0];
    v[7] |= v[5];
    v[6] |= v[4];
    v[5] |= v[3];
    v[4] |= v[2];
    v[3] |= v[1];
    v[2] |= v[0];
    v[7] |= v[3];
    v[6] |= v[2];
    v[5] |= v[1];
    v[4] |= v[0];
    v
}

/// Inclusive prefix AND over 8 u64 lanes (shift-by-1/2/4 network).
/// `out[i] = a[0] & a[1] & ... & a[i]`.
#[inline(always)]
pub const fn prefix_and_u64x8(a: [u64; 8]) -> [u64; 8] {
    let mut v = a;
    v[7] &= v[6];
    v[6] &= v[5];
    v[5] &= v[4];
    v[4] &= v[3];
    v[3] &= v[2];
    v[2] &= v[1];
    v[1] &= v[0];
    v[7] &= v[5];
    v[6] &= v[4];
    v[5] &= v[3];
    v[4] &= v[2];
    v[3] &= v[1];
    v[2] &= v[0];
    v[7] &= v[3];
    v[6] &= v[2];
    v[5] &= v[1];
    v[4] &= v[0];
    v
}

/// Inclusive prefix XOR over 8 u64 lanes (shift-by-1/2/4 network).
/// Applied to `[0,1,2,...,7]` this yields the binary-reflected Gray code.
#[inline(always)]
pub const fn prefix_xor_u64x8(a: [u64; 8]) -> [u64; 8] {
    let mut v = a;
    v[7] ^= v[6];
    v[6] ^= v[5];
    v[5] ^= v[4];
    v[4] ^= v[3];
    v[3] ^= v[2];
    v[2] ^= v[1];
    v[1] ^= v[0];
    v[7] ^= v[5];
    v[6] ^= v[4];
    v[5] ^= v[3];
    v[4] ^= v[2];
    v[3] ^= v[1];
    v[2] ^= v[0];
    v[7] ^= v[3];
    v[6] ^= v[2];
    v[5] ^= v[1];
    v[4] ^= v[0];
    v
}

/// Inclusive wrapping prefix sum over 8 u32 lanes (shift-by-1/2/4 network).
#[inline(always)]
pub const fn prefix_sum_u32x8(a: [u32; 8]) -> [u32; 8] {
    let mut v = a;
    v[7] = v[7].wrapping_add(v[6]);
    v[6] = v[6].wrapping_add(v[5]);
    v[5] = v[5].wrapping_add(v[4]);
    v[4] = v[4].wrapping_add(v[3]);
    v[3] = v[3].wrapping_add(v[2]);
    v[2] = v[2].wrapping_add(v[1]);
    v[1] = v[1].wrapping_add(v[0]);
    v[7] = v[7].wrapping_add(v[5]);
    v[6] = v[6].wrapping_add(v[4]);
    v[5] = v[5].wrapping_add(v[3]);
    v[4] = v[4].wrapping_add(v[2]);
    v[3] = v[3].wrapping_add(v[1]);
    v[2] = v[2].wrapping_add(v[0]);
    v[7] = v[7].wrapping_add(v[3]);
    v[6] = v[6].wrapping_add(v[2]);
    v[5] = v[5].wrapping_add(v[1]);
    v[4] = v[4].wrapping_add(v[0]);
    v
}

/// Segmented inclusive wrapping prefix sum over 8 u32 lanes: a segment
/// restarts wherever the corresponding `flags` lane is `true`. Lanes before
/// the first flag belong to an implicit segment starting at index 0.
#[inline(always)]
pub const fn segmented_prefix_sum_u32x8(values: [u32; 8], flags: [bool; 8]) -> [u32; 8] {
    let mut out = [0u32; 8];
    // Fully unrolled; the reset mask is (flags[i] as u32).wrapping_neg().
    let mut acc: u32 = values[0];
    out[0] = acc;
    let m1 = (flags[1] as u32).wrapping_neg();
    acc &= !m1;
    acc = acc.wrapping_add(values[1]);
    out[1] = acc;
    let m2 = (flags[2] as u32).wrapping_neg();
    acc &= !m2;
    acc = acc.wrapping_add(values[2]);
    out[2] = acc;
    let m3 = (flags[3] as u32).wrapping_neg();
    acc &= !m3;
    acc = acc.wrapping_add(values[3]);
    out[3] = acc;
    let m4 = (flags[4] as u32).wrapping_neg();
    acc &= !m4;
    acc = acc.wrapping_add(values[4]);
    out[4] = acc;
    let m5 = (flags[5] as u32).wrapping_neg();
    acc &= !m5;
    acc = acc.wrapping_add(values[5]);
    out[5] = acc;
    let m6 = (flags[6] as u32).wrapping_neg();
    acc &= !m6;
    acc = acc.wrapping_add(values[6]);
    out[6] = acc;
    let m7 = (flags[7] as u32).wrapping_neg();
    acc &= !m7;
    acc = acc.wrapping_add(values[7]);
    out[7] = acc;
    out
}

/// Per-byte unsigned `x < y`: `0x80` in the high bit of each lane where
/// `x`'s byte is strictly less than `y`'s. Harvest-cite of
/// `swar.rs::swar_lt` (swar.rs:24-39) — scan.rs compiles STANDALONE in the
/// census wrapper (`#[path]` module, scan_courts census), so cross-module
/// paths are forbidden here by design; the formula is pinned to its source.
#[inline(always)]
const fn swar_lt_harvest(x: u64, y: u64) -> u64 {
    let hi = 0x8080_8080_8080_8080u64;
    let low_ge = (x | hi).wrapping_sub(y & !hi) & hi;
    let high_gt = x & !y & hi;
    let high_eq = !(x ^ y) & hi;
    let ge = high_gt | (high_eq & low_ge);
    !ge & hi
}

/// Inclusive prefix maximum over 8 u8 lanes. SWAR parallel-prefix form:
/// the 8 lanes are packed into one u64 word and each Hillis-Steele step
/// (d = 1, 2, 4) selects `max(v[i], v[i-d])` through per-byte `lt` masks
/// plus mask arithmetic — every step is pure shift/and/or/mul, no
/// compare-select (RV32IMC has no cmov; the former `u8::max` chain lowered
/// to a bgeu/bltu branch cascade).
///
/// Zero-padding at the word boundaries is harmless for unsigned max: lane 0
/// has no predecessor (compares against 0) and lane 7's step-d predecessor
/// is 0 whenever `i - d < 0`.
#[inline(always)]
pub const fn prefix_max_u8x8(a: [u8; 8]) -> [u8; 8] {
    let mut w = u64::from_le_bytes(a);
    // 0xFF per-lane expansion: (0x01 lane) * 0xFF = 0xFF lane, no cross-byte
    // carry (lane products are 0x00 or 0xFF, both carry-free into byte i+1).
    // Step d=1: lane i takes max(v[i], v[i-1]) — fully unrolled (census
    // law: no loop back-edges in HOT cells).
    let pred1 = w << 8;
    let m1 = (swar_lt_harvest(w, pred1) >> 7).wrapping_mul(0xFF);
    w = (w & !m1) | (pred1 & m1);
    // Step d=2.
    let pred2 = w << 16;
    let m2 = (swar_lt_harvest(w, pred2) >> 7).wrapping_mul(0xFF);
    w = (w & !m2) | (pred2 & m2);
    // Step d=4.
    let pred4 = w << 32;
    let m4 = (swar_lt_harvest(w, pred4) >> 7).wrapping_mul(0xFF);
    w = (w & !m4) | (pred4 & m4);
    w.to_le_bytes()
    // post-condition: out[i] == max(a[0..=i]) — asserted in scan_courts
}

/// Count leading bytes of an 8-lane word equal to `target`, SWAR-style,
/// fully unrolled. Returns the length of the longest prefix of equal bytes.
///
/// The former `mask.trailing_ones()` lowered to a `beqz`-guarded ctz libcall
/// on riscv32imc (no Zbb); this form computes the trailing-ones count of the
/// 8-bit match mask with pure arithmetic: isolate the lowest zero bit
/// `z = ~mask & (mask + 1) & 0xFF` (0 when all 8 match), then the run
/// length is `popcount8(z - 1)` (0xFFFF..FF & 0xFF = 0xFF when z == 0, so
/// the full-match case folds to popcount(0xFF) = 8). No ctz intrinsic, no
/// zero-guard branch.
#[inline(always)]
pub const fn count_leading_eq_u8(bytes: [u8; 8], target: u8) -> usize {
    // Lane-match bitmask: bit i set iff bytes[i] == target (each `(a == t)`
    // lowers to xor + seqz — compare, not branch — on riscv32imc).
    let t = target;
    let m0 = ((bytes[0] == t) as u8) << 0;
    let m1 = ((bytes[1] == t) as u8) << 1;
    let m2 = ((bytes[2] == t) as u8) << 2;
    let m3 = ((bytes[3] == t) as u8) << 3;
    let m4 = ((bytes[4] == t) as u8) << 4;
    let m5 = ((bytes[5] == t) as u8) << 5;
    let m6 = ((bytes[6] == t) as u8) << 6;
    let m7 = ((bytes[7] == t) as u8) << 7;
    let mask = m0 as u64
        | m1 as u64
        | m2 as u64
        | m3 as u64
        | m4 as u64
        | m5 as u64
        | m6 as u64
        | m7 as u64;
    // Trailing-ones run of the 8-bit mask, pure arithmetic (the former
    // `trailing_ones()` lowered to a beqz-guarded ctz libcall on RV32IMC).
    let z = (!mask & mask.wrapping_add(1)) & 0xFF; // lowest zero bit (0 if all 8 set)
    let mut p = z.wrapping_sub(1) & 0xFF; // bits strictly below it (0xFF when z==0)
    p = p.wrapping_sub((p >> 1) & 0x55);
    p = (p & 0x33).wrapping_add((p >> 2) & 0x33);
    p = p.wrapping_add(p >> 4) & 0x0F;
    p as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_examples() {
        assert_eq!(
            prefix_sum_u32x8([1, 1, 1, 1, 1, 1, 1, 1]),
            [1, 2, 3, 4, 5, 6, 7, 8]
        );
        assert_eq!(prefix_xor_u64x8([0, 1, 2, 3, 4, 5, 6, 7])[2], 3);
        assert_eq!(count_leading_eq_u8(*b"aaabcd  ", b'a'), 3);
        assert_eq!(count_leading_eq_u8([0u8; 8], 0), 8);
        let out = segmented_prefix_sum_u32x8(
            [1, 2, 3, 4, 5, 6, 7, 8],
            [false, false, false, true, false, false, true, false],
        );
        assert_eq!(out, [1, 3, 6, 4, 9, 15, 7, 15]);
    }
}
