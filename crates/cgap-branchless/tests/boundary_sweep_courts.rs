//! Lane C04 — exhaustive boundary sweep courts for cgap-branchless.
//!
//! Every primitive is checked against an independently-computed scalar
//! reference model (plain Rust loops — real math, not a restatement of
//! the implementation). Boundary patterns: all-ones 0xFFFF..FFFF,
//! all-zeros, checkerboards 0x5555../0xAAAA.., plus a high-entropy
//! deterministic stream from a local SplitMix64 (test-space seed —
//! deliberately NOT the frozen corpus seed 0xA20C_0FFE_E5A7_E000).
//!
//! Anti-vacuity: `reference_model_has_teeth` injects a deliberately
//! wrong expected value and MUST fail — proving the reference models
//! disagree with a wrong claim (a court that cannot flip is FAIL).
//!
//! Divergence note: the lane dispatch names `clamp.rs`; no such module
//! exists in this crate (saturated arithmetic lives in `arithmetic.rs`,
//! not covered by G01/G02's scan/transpose re-lowering). This court
//! covers the scan + swar + transpose surface actually on disk.

use cgap_branchless::scan::{
    count_leading_eq_u8, prefix_and_u64x8, prefix_max_u8x8, prefix_or_u64x8, prefix_sum_u32x8,
    prefix_xor_u64x8, segmented_prefix_sum_u32x8,
};
use cgap_branchless::swar::{swar_eq_u64, swar_has_zero_byte, swar_range_mask, swar_splat};
use cgap_branchless::transpose::{transpose_64x64, transpose_8x8};

// ---------------- boundary patterns ----------------

const ALL_ONES: u64 = u64::MAX;
const ALL_ZEROS: u64 = 0x0;
const CHECKER_55: u64 = 0x5555_5555_5555_5555;
const CHECKER_AA: u64 = 0xAAAA_AAAA_AAAA_AAAA;
const PATTERNS: [u64; 4] = [ALL_ONES, ALL_ZEROS, CHECKER_55, CHECKER_AA];

/// Test-space SplitMix64 PRNG (local; not the frozen corpus generator).
struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z ^ (z >> 31)
    }
}

/// Test-space seed for the high-entropy stream.
const STREAM_SEED: u64 = 0xC04B_0747_1DEA_0001;

// ---------------- reference models (scalar Rust) ----------------

fn ref_prefix_or(a: [u64; 8]) -> [u64; 8] {
    let mut out = [0u64; 8];
    let mut acc = 0u64;
    for i in 0..8 {
        acc |= a[i];
        out[i] = acc;
    }
    out
}

fn ref_prefix_and(a: [u64; 8]) -> [u64; 8] {
    let mut out = [0u64; 8];
    let mut acc = u64::MAX;
    for i in 0..8 {
        acc &= a[i];
        out[i] = acc;
    }
    out
}

fn ref_prefix_xor(a: [u64; 8]) -> [u64; 8] {
    let mut out = [0u64; 8];
    let mut acc = 0u64;
    for i in 0..8 {
        acc ^= a[i];
        out[i] = acc;
    }
    out
}

fn ref_prefix_sum(a: [u32; 8]) -> [u32; 8] {
    let mut out = [0u32; 8];
    let mut acc = 0u32;
    for i in 0..8 {
        acc = acc.wrapping_add(a[i]);
        out[i] = acc;
    }
    out
}

fn ref_segmented_sum(values: [u32; 8], flags: [bool; 8]) -> [u32; 8] {
    let mut out = [0u32; 8];
    let mut acc = 0u32;
    for i in 0..8 {
        if flags[i] {
            acc = 0;
        }
        acc = acc.wrapping_add(values[i]);
        out[i] = acc;
    }
    out
}

fn ref_prefix_max(a: [u8; 8]) -> [u8; 8] {
    let mut out = [0u8; 8];
    let mut acc = 0u8;
    for i in 0..8 {
        acc = acc.max(a[i]);
        out[i] = acc;
    }
    out
}

fn ref_leading_eq(bytes: [u8; 8], target: u8) -> usize {
    let mut n = 0;
    while n < 8 && bytes[n] == target {
        n += 1;
    }
    n
}

fn ref_swar_eq(word: u64, target: u64) -> u64 {
    let mut out = 0u64;
    for i in 0..8 {
        let sh = 8 * i;
        let a = ((word >> sh) & 0xFF) as u8;
        let b = ((target >> sh) & 0xFF) as u8;
        if a == b {
            out |= 0xFFu64 << sh;
        }
    }
    out
}

fn ref_has_zero(v: u64) -> bool {
    (0..8).any(|i| (v >> (8 * i)) & 0xFF == 0)
}

fn ref_range_mask(val: u64, min: u8, max: u8) -> u64 {
    let mut out = 0u64;
    for i in 0..8 {
        let sh = 8 * i;
        let b = ((val >> sh) & 0xFF) as u8;
        if b >= min && b <= max {
            out |= 0xFFu64 << sh;
        }
    }
    out
}

fn ref_transpose8(x: u64) -> u64 {
    let mut out = 0u64;
    for r in 0..8u64 {
        for c in 0..8u64 {
            let bit = (x >> (8 * r + c)) & 1;
            out |= bit << (8 * c + r);
        }
    }
    out
}

fn ref_transpose64(rows: [u64; 64]) -> [u64; 64] {
    let mut out = [0u64; 64];
    for r in 0..64usize {
        for c in 0..64usize {
            let bit = (rows[r] >> c) & 1;
            out[c] |= bit << r;
        }
    }
    out
}


// ---------------- courts: scan ----------------

/// Boundary sweep for every scan primitive over the four corner
/// patterns plus high-entropy lanes. Reference values are computed by
/// the scalar models above.
#[test]
fn scan_boundary_sweep_court() {
    for &p in &PATTERNS {
        let a = [p; 8];
        assert_eq!(prefix_or_u64x8(a), ref_prefix_or(a), "prefix_or p={p:#018x}");
        assert_eq!(prefix_and_u64x8(a), ref_prefix_and(a), "prefix_and p={p:#018x}");
        assert_eq!(prefix_xor_u64x8(a), ref_prefix_xor(a), "prefix_xor p={p:#018x}");

        let a32 = [p as u32; 8];
        assert_eq!(prefix_sum_u32x8(a32), ref_prefix_sum(a32), "prefix_sum p={p:#018x}");

        let a8 = [p as u8; 8];
        assert_eq!(prefix_max_u8x8(a8), ref_prefix_max(a8), "prefix_max p={p:#018x}");

        // count_leading_eq against corner targets.
        for &t in &[0x00u8, 0xFF, 0x55, 0xAA, p as u8] {
            assert_eq!(
                count_leading_eq_u8(a8, t),
                ref_leading_eq(a8, t),
                "cle p={p:#018x} t={t:#04x}"
            );
        }
    }

    // Segmented sum: boundary flag shapes over all-ones / all-zeros
    // values (corner cases: reset at 0, reset at 7, every lane, no lane).
    let flag_shapes: [[bool; 8]; 6] = [
        [false; 8],
        [true; 8],
        [true, false, false, false, false, false, false, false],
        [false, false, false, false, false, false, false, true],
        [true, false, true, false, true, false, true, false],
        [false, true, false, true, false, true, false, true],
    ];
    for &p in &PATTERNS {
        let v = [p as u32; 8];
        for flags in &flag_shapes {
            assert_eq!(
                segmented_prefix_sum_u32x8(v, *flags),
                ref_segmented_sum(v, *flags),
                "segsum p={p:#018x} flags={flags:?}"
            );
        }
    }
}

/// High-entropy differential: 4_096 deterministic SplitMix64 rounds
/// against every scan primitive reference model.
#[test]
fn scan_high_entropy_differential_court() {
    let mut rng = SplitMix64(STREAM_SEED);
    for round in 0..4_096u32 {
        let mut a = [0u64; 8];
        for lane in a.iter_mut() {
            *lane = rng.next();
        }
        assert_eq!(prefix_or_u64x8(a), ref_prefix_or(a), "round {round} or");
        assert_eq!(prefix_and_u64x8(a), ref_prefix_and(a), "round {round} and");
        assert_eq!(prefix_xor_u64x8(a), ref_prefix_xor(a), "round {round} xor");

        let mut a32 = [0u32; 8];
        for (i, lane) in a.iter().enumerate() {
            a32[i] = if i % 2 == 0 { *lane as u32 } else { (*lane >> 32) as u32 };
        }
        assert_eq!(prefix_sum_u32x8(a32), ref_prefix_sum(a32), "round {round} sum");

        let mut flags = [false; 8];
        for (_i, f) in flags.iter_mut().enumerate() {
            *f = rng.next() & 1 == 1;
        }
        assert_eq!(
            segmented_prefix_sum_u32x8(a32, flags),
            ref_segmented_sum(a32, flags),
            "round {round} segsum"
        );

        let mut a8 = [0u8; 8];
        for (i, b) in a8.iter_mut().enumerate() {
            *b = (a[i] & 0xFF) as u8;
        }
        assert_eq!(prefix_max_u8x8(a8), ref_prefix_max(a8), "round {round} max");
        let t = (rng.next() & 0xFF) as u8;
        assert_eq!(count_leading_eq_u8(a8, t), ref_leading_eq(a8, t), "round {round} cle");
    }
}


// ---------------- courts: swar ----------------

/// Boundary sweep for the public swar kernels: corner words, corner
/// splats, empty/inclusive range edges (min > max, min == max,
/// min == 0x00, max == 0xFF).
#[test]
fn swar_boundary_sweep_court() {
    let edges = [0x00u8, 0x01, 0x7F, 0x80, 0xFE, 0xFF];
    for &p in &PATTERNS {
        // eq / has_zero on the corner words themselves.
        assert_eq!(swar_eq_u64(p, p), ref_swar_eq(p, p), "eq self p={p:#018x}");
        assert_eq!(swar_eq_u64(p, !p), ref_swar_eq(p, !p), "eq not p={p:#018x}");
        assert_eq!(swar_eq_u64(p, ALL_ZEROS), ref_swar_eq(p, 0), "eq zero p={p:#018x}");
        assert_eq!(swar_has_zero_byte(p), ref_has_zero(p), "zero p={p:#018x}");

        for &e in &edges {
            let s = swar_splat(e);
            assert_eq!(swar_eq_u64(p, s), ref_swar_eq(p, s), "eq splat p={p:#018x} e={e:#04x}");
            assert_eq!(swar_eq_u64(s, p), ref_swar_eq(s, p), "eq rsplat p={p:#018x} e={e:#04x}");
            assert_eq!(swar_has_zero_byte(s), ref_has_zero(s), "zero splat e={e:#04x}");
            // Full edge-pair range sweep: every (min, max) edge pair.
            for &m in &edges {
                for &x in &edges {
                    let val = s;
                    let got = swar_range_mask(val, m.min(x), m.max(x));
                    let want = ref_range_mask(val, m.min(x), m.max(x));
                    assert_eq!(got, want, "range e={e:#04x} [{:#04x},{:#04x}]", m.min(x), m.max(x));
                }
            }
            // Empty range must be all-zero on every lane.
            assert_eq!(swar_range_mask(s, 0x81, 0x7F), 0, "empty range e={e:#04x}");
        }
    }

    // High-entropy differential: 20_000 rounds.
    let mut rng = SplitMix64(STREAM_SEED ^ 0xC04);
    for round in 0..20_000u32 {
        let x = rng.next();
        let y = rng.next();
        assert_eq!(swar_eq_u64(x, y), ref_swar_eq(x, y), "round {round} eq");
        assert_eq!(swar_has_zero_byte(x), ref_has_zero(x), "round {round} zero");
        let lo = (rng.next() & 0xFF) as u8;
        let hi = (rng.next() & 0xFF) as u8;
        let (mn, mx) = (lo.min(hi), lo.max(hi));
        assert_eq!(swar_range_mask(x, mn, mx), ref_range_mask(x, mn, mx), "round {round} range");
    }
}

// ---------------- courts: transpose ----------------

/// Boundary sweep for both transpose cells: corner words, identity
/// matrix, single-bit matrices, and 256 high-entropy 64x64 rounds.
#[test]
fn transpose_boundary_sweep_court() {
    for &p in &PATTERNS {
        assert_eq!(transpose_8x8(p), ref_transpose8(p), "8x8 p={p:#018x}");
        assert_eq!(transpose_8x8(transpose_8x8(p)), p, "8x8 involution p={p:#018x}");
    }
    // Identity: transpose is identity.
    let mut id = [0u64; 64];
    for (i, r) in id.iter_mut().enumerate() {
        *r = 1u64 << i;
    }
    assert_eq!(transpose_64x64(id), id, "64x64 identity");

    // Single-bit matrices: rows[r] = 1<<c  <=>  out[c] = 1<<r.
    for r in (0..64usize).step_by(9) {
        for c in (0..64usize).step_by(7) {
            let mut m = [0u64; 64];
            m[r] = 1u64 << c;
            let out = transpose_64x64(m);
            let mut want = [0u64; 64];
            want[c] = 1u64 << r;
            assert_eq!(out, want, "64x64 single bit r={r} c={c}");
        }
    }

    // High-entropy rounds + corner-word row fills.
    let mut rng = SplitMix64(STREAM_SEED ^ 0xC04_C04);
    for round in 0..256u32 {
        let mut rows = [0u64; 64];
        for lane in rows.iter_mut() {
            *lane = rng.next();
        }
        let out = transpose_64x64(rows);
        assert_eq!(out, ref_transpose64(rows), "64x64 round {round}");
        // Involution: transposing the transpose restores the input.
        assert_eq!(transpose_64x64(out), rows, "64x64 involution round {round}");
    }
    for &p in &PATTERNS {
        let rows = [p; 64];
        assert_eq!(transpose_64x64(rows), ref_transpose64(rows), "64x64 fill p={p:#018x}");
    }
}


// ---------------- anti-vacuity twins ----------------

/// ANTI-VACUITY (court that must flip): a deliberately WRONG expected
/// value for `prefix_xor_u64x8` must fail the court. If this twin
/// passes, the reference model has no teeth and every sweep above is
/// vacuous.
#[test]
#[should_panic(expected = "reference-model-teeth")]
fn scan_reference_model_has_teeth() {
    let a = [CHECKER_55; 8];
    let mut wrong = ref_prefix_xor(a);
    wrong[3] ^= 0x1; // injected error: flip one bit of the expected lane
    assert_eq!(
        prefix_xor_u64x8(a),
        wrong,
        "reference-model-teeth: injected wrong expectation must fail"
    );
}

/// ANTI-VACUITY twin for the transpose reference model: a wrong
/// single-bit expectation must fail.
#[test]
#[should_panic(expected = "reference-model-teeth")]
fn transpose_reference_model_has_teeth() {
    let mut m = [0u64; 64];
    m[3] = 1u64 << 5;
    let mut want = ref_transpose64(m);
    want[5] ^= 1u64 << 4; // wrong row bit injected
    assert_eq!(
        transpose_64x64(m),
        want,
        "reference-model-teeth: injected wrong transpose expectation must fail"
    );
}

/// ANTI-VACUITY twin for the swar reference model: a wrong equality
/// mask on a corner pattern must fail.
#[test]
#[should_panic(expected = "reference-model-teeth")]
fn swar_reference_model_has_teeth() {
    let wrong = ref_swar_eq(ALL_ZEROS, ALL_ZEROS) ^ 0xFF; // drop one lane
    assert_eq!(
        swar_eq_u64(ALL_ZEROS, ALL_ZEROS),
        wrong,
        "reference-model-teeth: injected wrong swar expectation must fail"
    );
}
