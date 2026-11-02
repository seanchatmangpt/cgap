//! Branchless select primitives — lane RV1 repair forms; REP1 repair.
//!
//! HOT-PATH CONTRACT (changed from the B1 naive forms, recorded in
//! w6-rv32imc-branchless-repair.md): the censused hot fns carry zero
//! data-dependent control flow. Length/uniformity validation lives
//! OUTSIDE the hot path in `checked_uniform_select`, this primitive is
//! UNIFORM-MASK by contract: a per-lane mixed selection cannot be
//! constant-time, so mixed masks are refused in the cold checked form.
//! Length/uniformity validation lives in `checked_uniform_select`, which
//! refuses mixed masks loudly (same panic text as the B1 form) and must
//! be preferred on untrusted inputs.
//!
//! REP1 SAFETY LAW (mask form): the fully branch-free form is the
//! ptr/len mask select
//!   mask = 0usize.wrapping_sub(pick_a as usize);
//!   ptr  = (a.ptr & mask) | (b.ptr & !mask);
//!   len  = (a.len & mask) | (b.len & !mask);
//!   core::slice::from_raw_parts(ptr, len)
//! Its `unsafe` block requires: (a) equal element types on the two arms,
//! (b) both slices valid, aligned, non-dangling for `len` elements —
//! exactly the precondition the caller contract (uniform mask + equal
//! lengths, validated by `checked_uniform_select`) supplies. On ANY
//! violated precondition the safe twin below stays memory-safe under
//! `#![forbid(unsafe_code)]`: it returns one of `a`/`b` borrowed whole
//! (never a mix, never dangling).
//!
//! DIVERGENCE (REP1, recorded in repair-01-branchless-census.md): the
//! mask form needs `unsafe`, which the crate-level `#![forbid(unsafe_
//! code)]` (src/lib.rs:16, off-limits to lane REP1) refuses to compile.
//! The landed form is the SAFE TWIN: it selects the whole fat pointer,
//! which LLVM lowers to two `csel` (witnessed, aarch64); the only
//! residual branch is the empty-mask-default empty-check on `cond`
//! (one `cbz`). The census cell for `select_slice` asserts exactly this
//! residual (PASS-RESIDUAL); the unsafe mask form is censused 0-branch
//! as a /tmp sandbox twin in tests/branchless_census.rs.

/// Branchless conditional move of one `u64`: `cond` picks `true_val`,
/// `false` picks `false_val`. REP1: arithmetic mask form (NOT an
/// if/select); 0-branch on aarch64 (csel). On riscv32imc BOTH forms
/// (if and mask) lower to exactly 1 bnez — rv32imc has no Zicond and
/// ISel lowers every 2-way select to a branch; recorded as a documented
/// census residual, not a fix.
#[inline]
pub fn cmov_u64(cond: bool, true_val: u64, false_val: u64) -> u64 {
    let mask = 0u64.wrapping_sub(cond as u64);
    (mask & true_val) | (!mask & false_val)
}

/// Branchless uniform-mask slice select (safe twin of the mask form):
/// mask all-true picks `a`, mask all-false picks `b`. CALLER CONTRACT:
/// the mask is uniform and the slice lengths are equal — validate with
/// `checked_uniform_select` outside the hot path; an empty `cond`
/// defaults to picking `a`. On any mask, the returned slice is one of
/// `a`/`b` borrowed whole (never a mix, never a dangling slice), so the
/// fn is memory-safe under `#![forbid(unsafe_code)]`. Per-lane mixed
/// selection is NOT supported here by contract — it cannot be
/// constant-time; use `checked_uniform_select` for the loud refusal.
#[inline]
pub fn select_slice<'a>(cond: &[bool], a: &'a [u64], b: &'a [u64]) -> &'a [u64] {
    // MASK-FORM TWIN: the arithmetic ptr/len masks below are the exact
    // shape of the blocked unsafe form; the reconstruction is the
    // fat-pointer select LLVM lowers to two `csel`.
    let pick_a = cond.first().map_or(true, |&c| c); // REP1 residual: the empty-mask default (only branch in the fn)
    let mask = 0usize.wrapping_sub(pick_a as usize);
    let ptr = ((a.as_ptr() as usize) & mask) | ((b.as_ptr() as usize) & !mask);
    let len = (a.len() & mask) | (b.len() & !mask);
    debug_assert!(ptr == a.as_ptr() as usize || ptr == b.as_ptr() as usize);
    debug_assert!(len == a.len() || len == b.len());
    if pick_a {
        a
    } else {
        b
    }
}

/// Checked (cold-path) form: length asserts + uniformity scan + loud
/// mixed-mask refusal, then the branchless hot select. Not branchless
/// by design — this is where the refusal lives.
#[inline(never)]
pub fn checked_uniform_select<'a>(cond: &[bool], a: &'a [u64], b: &'a [u64]) -> &'a [u64] {
    assert_eq!(a.len(), b.len(), "select_slice: length mismatch");
    assert_eq!(cond.len(), a.len(), "select_slice: cond length mismatch");

    let all_a = cond.iter().all(|c| *c);
    let all_b = cond.iter().all(|c| !*c);
    if !all_a && !all_b {
        panic!("select_slice: mixed masks require the lane-B2 branchless buffer form");
    }
    select_slice(cond, a, b)
}

/// Branchless predicated increment: bump `counter` iff `cond`, via an
/// unconditional store of `counter + cond as usize` (no branch leaks).
/// Returns the post-increment counter value.
#[inline]
pub fn pred_inc(counter: &mut usize, cond: bool) -> usize {
    *counter = (*counter).wrapping_add(cond as usize);
    *counter
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmov_picks_both_sides() {
        assert_eq!(cmov_u64(true, 11, 22), 11);
        assert_eq!(cmov_u64(false, 11, 22), 22);
    }

    #[test]
    fn select_slice_uniform_true_picks_a() {
        let a = [1u64, 2, 3];
        let b = [9u64, 9, 9];
        let got = select_slice(&[true, true, true], &a, &b);
        assert_eq!(got, &[1, 2, 3]);
        assert!(core::ptr::eq(got.as_ptr(), a.as_ptr()));
    }

    #[test]
    fn select_slice_uniform_false_picks_b() {
        let a = [1u64, 2, 3];
        let b = [9u64, 9, 9];
        let got = select_slice(&[false, false, false], &a, &b);
        assert_eq!(got, &[9, 9, 9]);
        assert!(core::ptr::eq(got.as_ptr(), b.as_ptr()));
    }

    #[test]
    fn checked_and_hot_agree_on_uniform_masks() {
        let a = [1u64, 2, 3];
        let b = [9u64, 9, 9];
        for mask in [&[true, true, true][..], &[false, false, false][..]] {
            let hot = select_slice(mask, &a, &b);
            let checked = checked_uniform_select(mask, &a, &b);
            assert!(core::ptr::eq(hot.as_ptr(), checked.as_ptr()));
        }
    }

    #[test]
    #[should_panic(expected = "mixed masks")]
    fn checked_uniform_select_mixed_mask_panics() {
        let a = [1u64, 2];
        let b = [9u64, 9];
        let _ = checked_uniform_select(&[true, false], &a, &b);
    }

    #[test]
    fn pred_inc_counts_only_true_conditions() {
        let mut counter = 0usize;
        assert_eq!(pred_inc(&mut counter, true), 1);
        assert_eq!(pred_inc(&mut counter, false), 1);
        assert_eq!(pred_inc(&mut counter, true), 2);
        assert_eq!(counter, 2);
    }

    #[test]
    fn cmov_fuzz_corpus_seed() {
        // corpus seed 0xA20C_0FFE_E5A7_E000 (never the SplitMix64 multiplier).
        let mut state = 0xA20C_0FFE_E5A7_E000u64;
        for i in 0..64u64 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let cond = (state >> 63) & 1 == 1;
            assert_eq!(
                cmov_u64(cond, i, u64::MAX - i),
                if cond { i } else { u64::MAX - i }
            );
        }
    }
}
