//! Bit-matrix transpose primitives (lane TS4 / lane G02).
//!
//! Harvest provenance: bcinr `bit_matrix_transpose_8x64` family --
//! `~/bcinr/crates/bcinr-logic/src/algorithms/bit_matrix_transpose_8x8.rs:22`
//! and `bit_matrix_transpose_64x64.rs:24-34`. Harvest-cited, never
//! imported: the three-stage delta-swap decomposition
//! (`0x00AA00AA00AA00AA`/shift 7, `0x0000CCCC0000CCCC`/shift 14,
//! `0x00000000F0F0F0F0`/shift 28) is re-derived here as const-friendly,
//! `#![no_std]`, zero-data-branch Rust under this crate's
//! `forbid(unsafe_code)` house law.
//!
//! # Delta-swap identity
//!
//! Each stage is `x ^= t ^ (t << k)` where `t = (x ^ (x >> k)) & M`.
//! This swaps the bits of `x` selected by `M` with their partners `k`
//! positions away, and is SELF-INVERSE: applying the same stage again
//! restores `x`. Stage masks partition the matrix: every bit of the 8x8
//! matrix belongs to exactly one swapped pair across the three stages,
//! which is why the composition is the complete transpose.
//!
//! # 64x64 stage decomposition (fully unrolled -- lane G02)
//!
//! `transpose_64x64` operates on the full 64x64 bit matrix stored as
//! `[u64; 64]` (word `r` = row `r`, bit `c` = column `c`). The
//! transpose is the quadrant decomposition
//! `T([[A,B],[C,D]]) = [[T(A),T(C)],[T(B),T(D)]]` executed as six
//! delta-swap passes (quadrant sizes 32/16/8/4/2/1): pass `h` swaps,
//! for every word pair `(i, i+h)` in each `2h`-word group, the `h`-bit
//! groups at shift `h` under the `h`-periodic alternating mask, moving
//! each off-quadrant block to its transposed quadrant. Every pass is
//! self-inverse; the six masks partition the 64x64 bit pairs.
//!
//! Lane G02 (branchless-transpose): the former runtime `while`-loop
//! nest left one residual `bne` back-edge guard in the emitted
//! riscv32imc asm (witnessed: verify_disassembly_invariants arm A
//! `[FORBIDDEN] transpose_64x64: bne x1`, exit 1). The six passes are
//! now FULLY UNROLLED at the source level: 192 literal-index
//! `swap_pair!` invocations (32 pairs per pass x 6 passes) with
//! literal shift constants, so the emitted IR contains no loop at
//! all and the delta-swap bodies lower to pure
//! xor/shift/and over the register file -- 0 branches on
//! riscv32imc and aarch64 (receipt:
//! `docs/jira/v26.11.2/lanes/lane-g02-transpose-branchless.md`).
//! The differential court and the single-bit-flip mutant live in
//! `tests/transpose_courts.rs`; the census budget there
//! (`T64_LOOP_RESIDUAL = 2`) is a ceiling, satisfied at 0.
//!
//! Census: both caller cells (`transpose_8x8`, `transpose_64x64`)
//! are HOT (0 branches on host-aarch64 and riscv32imc).

/// Transpose an 8x8 bit matrix stored row-major in one `u64`
/// (byte `r` = row `r`, bit `c` of that byte = column `c`).
///
/// Three delta-swap stages with the harvested masks; each stage is
/// self-inverse and the three masks partition the matrix pairs.
#[inline]
pub const fn transpose_8x8(x: u64) -> u64 {
    let x = dswap(x, 7, 0x00AA_00AA_00AA_00AA);
    let x = dswap(x, 14, 0x0000_CCCC_0000_CCCC);
    dswap(x, 28, 0x0000_0000_F0F0_F0F0)
}

/// Transpose a 64x64 bit matrix: `rows[r]` is row `r` (bit `c` =
/// column `c`); `out[c]` is row `c` of the transpose, i.e. column `c`
/// of the input.
///
/// Six delta-swap passes (quadrant sizes 32/16/8/4/2/1), fully
/// unrolled to 192 literal-index pair swaps; branch-free bodies,
/// allocation-free, `const`.
#[inline]
pub const fn transpose_64x64(rows: [u64; 64]) -> [u64; 64] {
    macro_rules! swap_pair {
        ($x:ident, $i:literal, $h:literal) => {{
            let t = (($x[$i] >> $h) ^ $x[$i + $h]) & quad_mask($h);
            $x[$i] ^= t << $h;
            $x[$i + $h] ^= t;
        }};
    }
    let mut x = rows;
    // pass h=32 (quad_mask -> 0x00000000FFFFFFFF): 32 pair swaps
    swap_pair!(x, 0, 32);
    swap_pair!(x, 1, 32);
    swap_pair!(x, 2, 32);
    swap_pair!(x, 3, 32);
    swap_pair!(x, 4, 32);
    swap_pair!(x, 5, 32);
    swap_pair!(x, 6, 32);
    swap_pair!(x, 7, 32);
    swap_pair!(x, 8, 32);
    swap_pair!(x, 9, 32);
    swap_pair!(x, 10, 32);
    swap_pair!(x, 11, 32);
    swap_pair!(x, 12, 32);
    swap_pair!(x, 13, 32);
    swap_pair!(x, 14, 32);
    swap_pair!(x, 15, 32);
    swap_pair!(x, 16, 32);
    swap_pair!(x, 17, 32);
    swap_pair!(x, 18, 32);
    swap_pair!(x, 19, 32);
    swap_pair!(x, 20, 32);
    swap_pair!(x, 21, 32);
    swap_pair!(x, 22, 32);
    swap_pair!(x, 23, 32);
    swap_pair!(x, 24, 32);
    swap_pair!(x, 25, 32);
    swap_pair!(x, 26, 32);
    swap_pair!(x, 27, 32);
    swap_pair!(x, 28, 32);
    swap_pair!(x, 29, 32);
    swap_pair!(x, 30, 32);
    swap_pair!(x, 31, 32);

    // pass h=16 (quad_mask -> 0x0000FFFF0000FFFF): 32 pair swaps
    swap_pair!(x, 0, 16);
    swap_pair!(x, 1, 16);
    swap_pair!(x, 2, 16);
    swap_pair!(x, 3, 16);
    swap_pair!(x, 4, 16);
    swap_pair!(x, 5, 16);
    swap_pair!(x, 6, 16);
    swap_pair!(x, 7, 16);
    swap_pair!(x, 8, 16);
    swap_pair!(x, 9, 16);
    swap_pair!(x, 10, 16);
    swap_pair!(x, 11, 16);
    swap_pair!(x, 12, 16);
    swap_pair!(x, 13, 16);
    swap_pair!(x, 14, 16);
    swap_pair!(x, 15, 16);
    swap_pair!(x, 32, 16);
    swap_pair!(x, 33, 16);
    swap_pair!(x, 34, 16);
    swap_pair!(x, 35, 16);
    swap_pair!(x, 36, 16);
    swap_pair!(x, 37, 16);
    swap_pair!(x, 38, 16);
    swap_pair!(x, 39, 16);
    swap_pair!(x, 40, 16);
    swap_pair!(x, 41, 16);
    swap_pair!(x, 42, 16);
    swap_pair!(x, 43, 16);
    swap_pair!(x, 44, 16);
    swap_pair!(x, 45, 16);
    swap_pair!(x, 46, 16);
    swap_pair!(x, 47, 16);

    // pass h=8 (quad_mask -> 0x00FF00FF00FF00FF): 32 pair swaps
    swap_pair!(x, 0, 8);
    swap_pair!(x, 1, 8);
    swap_pair!(x, 2, 8);
    swap_pair!(x, 3, 8);
    swap_pair!(x, 4, 8);
    swap_pair!(x, 5, 8);
    swap_pair!(x, 6, 8);
    swap_pair!(x, 7, 8);
    swap_pair!(x, 16, 8);
    swap_pair!(x, 17, 8);
    swap_pair!(x, 18, 8);
    swap_pair!(x, 19, 8);
    swap_pair!(x, 20, 8);
    swap_pair!(x, 21, 8);
    swap_pair!(x, 22, 8);
    swap_pair!(x, 23, 8);
    swap_pair!(x, 32, 8);
    swap_pair!(x, 33, 8);
    swap_pair!(x, 34, 8);
    swap_pair!(x, 35, 8);
    swap_pair!(x, 36, 8);
    swap_pair!(x, 37, 8);
    swap_pair!(x, 38, 8);
    swap_pair!(x, 39, 8);
    swap_pair!(x, 48, 8);
    swap_pair!(x, 49, 8);
    swap_pair!(x, 50, 8);
    swap_pair!(x, 51, 8);
    swap_pair!(x, 52, 8);
    swap_pair!(x, 53, 8);
    swap_pair!(x, 54, 8);
    swap_pair!(x, 55, 8);

    // pass h=4 (quad_mask -> 0x0F0F0F0F0F0F0F0F): 32 pair swaps
    swap_pair!(x, 0, 4);
    swap_pair!(x, 1, 4);
    swap_pair!(x, 2, 4);
    swap_pair!(x, 3, 4);
    swap_pair!(x, 8, 4);
    swap_pair!(x, 9, 4);
    swap_pair!(x, 10, 4);
    swap_pair!(x, 11, 4);
    swap_pair!(x, 16, 4);
    swap_pair!(x, 17, 4);
    swap_pair!(x, 18, 4);
    swap_pair!(x, 19, 4);
    swap_pair!(x, 24, 4);
    swap_pair!(x, 25, 4);
    swap_pair!(x, 26, 4);
    swap_pair!(x, 27, 4);
    swap_pair!(x, 32, 4);
    swap_pair!(x, 33, 4);
    swap_pair!(x, 34, 4);
    swap_pair!(x, 35, 4);
    swap_pair!(x, 40, 4);
    swap_pair!(x, 41, 4);
    swap_pair!(x, 42, 4);
    swap_pair!(x, 43, 4);
    swap_pair!(x, 48, 4);
    swap_pair!(x, 49, 4);
    swap_pair!(x, 50, 4);
    swap_pair!(x, 51, 4);
    swap_pair!(x, 56, 4);
    swap_pair!(x, 57, 4);
    swap_pair!(x, 58, 4);
    swap_pair!(x, 59, 4);

    // pass h=2 (quad_mask -> 0x3333333333333333): 32 pair swaps
    swap_pair!(x, 0, 2);
    swap_pair!(x, 1, 2);
    swap_pair!(x, 4, 2);
    swap_pair!(x, 5, 2);
    swap_pair!(x, 8, 2);
    swap_pair!(x, 9, 2);
    swap_pair!(x, 12, 2);
    swap_pair!(x, 13, 2);
    swap_pair!(x, 16, 2);
    swap_pair!(x, 17, 2);
    swap_pair!(x, 20, 2);
    swap_pair!(x, 21, 2);
    swap_pair!(x, 24, 2);
    swap_pair!(x, 25, 2);
    swap_pair!(x, 28, 2);
    swap_pair!(x, 29, 2);
    swap_pair!(x, 32, 2);
    swap_pair!(x, 33, 2);
    swap_pair!(x, 36, 2);
    swap_pair!(x, 37, 2);
    swap_pair!(x, 40, 2);
    swap_pair!(x, 41, 2);
    swap_pair!(x, 44, 2);
    swap_pair!(x, 45, 2);
    swap_pair!(x, 48, 2);
    swap_pair!(x, 49, 2);
    swap_pair!(x, 52, 2);
    swap_pair!(x, 53, 2);
    swap_pair!(x, 56, 2);
    swap_pair!(x, 57, 2);
    swap_pair!(x, 60, 2);
    swap_pair!(x, 61, 2);

    // pass h=1 (quad_mask -> 0x5555555555555555): 32 pair swaps
    swap_pair!(x, 0, 1);
    swap_pair!(x, 2, 1);
    swap_pair!(x, 4, 1);
    swap_pair!(x, 6, 1);
    swap_pair!(x, 8, 1);
    swap_pair!(x, 10, 1);
    swap_pair!(x, 12, 1);
    swap_pair!(x, 14, 1);
    swap_pair!(x, 16, 1);
    swap_pair!(x, 18, 1);
    swap_pair!(x, 20, 1);
    swap_pair!(x, 22, 1);
    swap_pair!(x, 24, 1);
    swap_pair!(x, 26, 1);
    swap_pair!(x, 28, 1);
    swap_pair!(x, 30, 1);
    swap_pair!(x, 32, 1);
    swap_pair!(x, 34, 1);
    swap_pair!(x, 36, 1);
    swap_pair!(x, 38, 1);
    swap_pair!(x, 40, 1);
    swap_pair!(x, 42, 1);
    swap_pair!(x, 44, 1);
    swap_pair!(x, 46, 1);
    swap_pair!(x, 48, 1);
    swap_pair!(x, 50, 1);
    swap_pair!(x, 52, 1);
    swap_pair!(x, 54, 1);
    swap_pair!(x, 56, 1);
    swap_pair!(x, 58, 1);
    swap_pair!(x, 60, 1);
    swap_pair!(x, 62, 1);

    x
}

/// One delta-swap stage: swap each bit selected by `mask` with the bit
/// `shift` positions to its left. Self-inverse.
#[inline]
const fn dswap(x: u64, shift: u32, mask: u64) -> u64 {
    let t = (x ^ (x >> shift)) & mask;
    x ^ t ^ (t << shift)
}

/// Alternating quadrant mask for pass `h`: bit groups of width `h`
/// selected iff their `2h`-aligned group index is odd
/// (32->`0x00000000FFFFFFFF`, 16->`0x0000FFFF0000FFFF`, 8->
/// `0x00FF00FF00FF00FF`, 4->`0x0F0F0F0F0F0F0F0F`, 2->`0x3333333333333333`,
/// 1->`0x5555555555555555`). Const-evaluated at each unrolled call site.
#[inline]
const fn quad_mask(h: usize) -> u64 {
    match h {
        32 => 0x0000_0000_FFFF_FFFF,
        16 => 0x0000_FFFF_0000_FFFF,
        8 => 0x00FF_00FF_00FF_00FF,
        4 => 0x0F0F_0F0F_0F0F_0F0F,
        2 => 0x3333_3333_3333_3333,
        _ => 0x5555_5555_5555_5555,
    }
}
