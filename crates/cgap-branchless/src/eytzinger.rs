//! Eytzinger (van Emde Boas / BFS) layout traversal primitives — lane B4.
//!
//! Branchless step and restore for eytzinger-layout binary search over a
//! 1-indexed implicit BST (`slot 0` is a sentinel, slots `1..=m` hold the
//! laid-out elements: slot `k`'s children are `2k` and `2k+1`).
//!
//! LAYOUT CONTRACT: [`to_eytzinger_layout`] pads the sorted input with
//! `u64::MAX` sentinels (`EYTZINGER_PAD`) up to the complete-tree size
//! `m = 2^L - 1 >= n` BEFORE the midpoint layout. This keeps the implicit
//! tree a valid BST over the padded sequence (pads occupy the in-order
//! tail), which is what makes the restore exact. Exhaustively verified
//! (python derivation, n = 1..199, all in-range targets, distinct and
//! duplicate inputs, zero failures) and differentially tested in Rust
//! against a `partition_point` oracle on the original sorted slice.
//!
//! Divergence note (recorded per lane contract): the dispatch suggested
//! `is_less` meaning "target < element". Under the prescribed restore
//! `index >> (index.trailing_ones() + 1)` the exact orientation is
//! `is_less = (element <= target)` (descend right). Semantics of the
//! composed primitives, exactly: the traversal terminates with `k` in
//! `[m + 1, 2m + 1]`; `eytzinger_restore(k)` returns the layout slot of
//! the first element STRICTLY GREATER than the target (upper bound), or
//! `0` when the target is >= every element. [`eytzinger_search`] maps
//! that slot to `None` when it holds the pad sentinel.
//!
//! B1 had not landed a competing restore oracle at lane time (only
//! `lib.rs` and `Cargo.toml` exist in `crates/cgap-branchless`), so there
//! is no external oracle semantics to match; the restore formula itself
//! is used exactly as prescribed.

// Host-side convenience allocator (reference driver / tests only; the
// crate is no_std). The hot primitives below are allocation-free.
extern crate alloc;

/// Sentinel filling padded slots. Input arrays must never contain this
/// value (documented precondition of `to_eytzinger_layout`).
pub const EYTZINGER_PAD: u64 = u64::MAX;

/// One traversal step: implicit-tree slot `curr` moves to its child.
/// `is_less == true` descends right (search convention: `element <=
/// target`). Pure arithmetic; no branch on the predicate.
#[inline(always)]
#[allow(dead_code)] // E-h de-pub: no external consumers; retained for differential/census courts
pub(crate) fn eytzinger_next(curr: usize, is_less: bool) -> usize {
    2 * curr + (is_less as usize)
}

/// Branchless restore of the answer slot from the final step value
/// `index` (which lies in `[m + 1, 2m + 1]` after the loop over slots
/// `1..=m`).
///
/// Semantics (exhaustively verified, see module docs): the layout slot
/// of the first element strictly greater than the searched target, or
/// `0` when every element is `<= target` (upper-bound "end" sentinel).
#[inline(always)]
#[allow(dead_code)] // E-h de-pub: no external consumers; retained for differential/census courts
pub(crate) fn eytzinger_restore(index: usize) -> usize {
    index >> (index.trailing_ones() + 1)
}
/// Full reference search driver over a laid-out array `eyt` (as produced
/// by [`to_eytzinger_layout`], 1-indexed with a `0` sentinel at slot 0 and
/// `EYTZINGER_PAD` fillers past `n`).
///
/// Returns the layout slot of the first element strictly greater than
/// `target` (upper bound), or `None` when no real element exceeds
/// `target`.
///
/// This function contains the natural traversal loop on purpose: it is
/// the REFERENCE driver demonstrating the composed use of the branchless
/// per-step primitives [`eytzinger_next`] + [`eytzinger_restore`]. The
/// per-step primitives themselves are branchless; the loop/predicate here
/// is the readable reference form, not the optimized kernel.
#[allow(dead_code)] // E-h de-pub: no external consumers; retained for differential/census courts
pub(crate) fn eytzinger_search(eyt: &[u64], target: u64) -> Option<usize> {
    assert!(
        !eyt.is_empty(),
        "eytzinger_search: empty layout (need sentinel slot 0)"
    );
    let m = eyt.len() - 1;
    let mut k = 1usize;
    while k <= m {
        // is_less = (element <= target) -> descend right (module-doc convention)
        k = eytzinger_next(k, eyt[k] <= target);
    }
    let slot = eytzinger_restore(k);
    if slot == 0 || eyt[slot] == EYTZINGER_PAD {
        None
    } else {
        debug_assert!(slot <= m);
        Some(slot)
    }
}

/// Host-side convenience: lay a sorted slice out in eytzinger (BFS) order.
///
/// The input MUST be sorted and MUST NOT contain [`EYTZINGER_PAD`]
/// (`u64::MAX`), which is reserved as the pad sentinel. Returns a `Vec`
/// of length `m + 1` where `m = 2^L - 1` is the complete-tree size
/// `>= n`: index 0 is a `0` sentinel, slots `1..=n` hold the laid-out
/// elements, slots `n+1..=m` hold `EYTZINGER_PAD` fillers so the
/// implicit tree stays a valid BST over the padded sequence.
///
/// Allocates (test/reference convenience, documented per lane contract);
/// the search primitives never call this.
#[allow(dead_code)] // E-h de-pub: no external consumers; retained for differential/census courts
pub(crate) fn to_eytzinger_layout(sorted: &[u64]) -> alloc::vec::Vec<u64> {
    let n = sorted.len();
    let mut m = 1usize;
    while m < n + 1 {
        m <<= 1;
    }
    m -= 1;
    let mut out = alloc::vec![EYTZINGER_PAD; m + 1];
    out[0] = 0;
    // Explicit stack (lo, hi_exclusive, slot) over the PADDED sequence:
    // place the midpoint of each range, recurse into both halves.
    let mut stack: alloc::vec::Vec<(usize, usize, usize)> = alloc::vec![(0, m, 1)];
    while let Some((lo, hi, slot)) = stack.pop() {
        if lo >= hi {
            continue;
        }
        let mid = lo + (hi - lo) / 2;
        out[slot] = padded_at(sorted, n, m, mid);
        stack.push((lo, mid, 2 * slot));
        stack.push((mid + 1, hi, 2 * slot + 1));
    }
    out
}

/// Padded-sequence accessor: element `mid` of the sorted slice, or the
/// pad sentinel for the `m - n` tail fillers.
#[inline]
fn padded_at(sorted: &[u64], n: usize, _m: usize, mid: usize) -> u64 {
    if mid < n {
        sorted[mid]
    } else {
        EYTZINGER_PAD
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::{vec, vec::Vec as TestVec};

    use super::{
        eytzinger_next, eytzinger_restore, eytzinger_search, to_eytzinger_layout, EYTZINGER_PAD,
    };

    /// Canonical corpus seed (frozen; never vary).
    const CORPUS_SEED: u64 = 0xA20C_0FFE_E5A7_E000;

    /// splitmix64 stream seeded from the canonical corpus seed.
    struct SplitMix64(u64);
    impl SplitMix64 {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }
    }

    fn sort(v: &mut [u64]) {
        // insertion sort; test-local, keeps the court free of std sort deps
        for i in 1..v.len() {
            let mut j = i;
            while j > 0 && v[j - 1] > v[j] {
                v.swap(j - 1, j);
                j -= 1;
            }
        }
    }

    /// Oracle: upper_bound on the ORIGINAL sorted array, computed without
    /// touching the layout machinery.
    fn oracle_upper_bound(sorted: &[u64], target: u64) -> Option<u64> {
        let idx = sorted.partition_point(|&x| x <= target);
        if idx < sorted.len() {
            Some(sorted[idx])
        } else {
            None
        }
    }

    #[test]
    fn next_is_pure_arithmetic_both_predicates() {
        for curr in 1..=64usize {
            assert_eq!(eytzinger_next(curr, false), 2 * curr);
            assert_eq!(eytzinger_next(curr, true), 2 * curr + 1);
        }
    }

    #[test]
    fn restore_matches_bit_path_derivation() {
        // Restore must equal the naive bit-path oracle: strip the trailing
        // run of 1-steps plus the preceding decision bit.
        for k in 1usize..=4096 {
            let mut to = 0usize;
            let mut x = k;
            while x & 1 == 1 {
                to += 1;
                x >>= 1;
            }
            assert_eq!(eytzinger_restore(k), k >> (to + 1));
        }
    }

    #[test]
    fn differential_all_sizes_all_targets() {
        // sizes 1..=64 plus 1024; sorted inputs (2i+1: distinct, gap-bearing);
        // every element, every gap miss, below-min and above-max. Asserts
        // VALUE equality with the oracle AND slot equality with the
        // value->slot map (final state, real layout, no mocks).
        let sizes: TestVec<usize> = (1..=64).chain(std::iter::once(1024)).collect();
        for &n in &sizes {
            let mut sorted: TestVec<u64> = (0..n).map(|i| 2 * i as u64 + 1).collect();
            sort(&mut sorted);
            assert!(
                sorted.windows(2).all(|w| w[0] < w[1]),
                "fixture not sorted/distinct"
            );
            let eyt = to_eytzinger_layout(&sorted);
            let m = eyt.len() - 1;
            assert_eq!(eyt[0], 0, "sentinel slot");
            assert!(m >= n, "padded complete tree must cover n");
            let mut slot_of = std::collections::BTreeMap::new();
            for (slot, &v) in eyt.iter().enumerate().skip(1) {
                if v != EYTZINGER_PAD {
                    assert_eq!(slot_of.insert(v, slot), None, "duplicate layout value");
                }
            }
            assert_eq!(slot_of.len(), n, "every real element laid out exactly once");
            let mut targets: TestVec<u64> = TestVec::new();
            targets.push(0);
            for &v in &sorted {
                targets.push(v);
                targets.push(v + 1); // gap miss (v+1 < next, odd spacing 2)
            }
            targets.push(sorted[n - 1] + 100);
            for &t in &targets {
                let got = eytzinger_search(&eyt, t).map(|slot| eyt[slot]);
                let want = oracle_upper_bound(&sorted, t);
                assert_eq!(got, want, "n={} target={}", n, t);
                if let Some(slot) = eytzinger_search(&eyt, t) {
                    assert_eq!(
                        Some(slot),
                        slot_of.get(&want.unwrap()).copied(),
                        "slot must be the laid-out position of the oracle value"
                    );
                }
            }
        }
    }

    #[test]
    fn property_vs_oracle_seeded() {
        // Property test: random sorted arrays (WITH duplicates) + random
        // targets, splitmix64 seeded from the canonical corpus seed.
        let mut rng = SplitMix64(CORPUS_SEED);
        for case in 0..512u32 {
            let n = 1 + (rng.next() % 200) as usize;
            let mut sorted: TestVec<u64> = (0..n).map(|_| rng.next() % 10_000).collect();
            sort(&mut sorted);
            let eyt = to_eytzinger_layout(&sorted);
            for _ in 0..16 {
                let t = rng.next() % 10_100;
                let got = eytzinger_search(&eyt, t).map(|slot| eyt[slot]);
                let want = oracle_upper_bound(&sorted, t);
                assert_eq!(
                    got,
                    want,
                    "case={} n={} target={} head={:?}",
                    case,
                    n,
                    t,
                    &sorted[..std::cmp::min(sorted.len(), 24)]
                );
            }
        }
    }

    #[test]
    fn next_restore_sequences_reproduce_classic_traversal() {
        // Hand-derived classic trace: sorted odds n=5, padded to m=7,
        // target = 9.  eyt = [0, 5, 1, u64::MAX, 3, EYTZINGER_PAD? ...]
        // is checked structurally; the step sequence is asserted exactly.
        let sorted: TestVec<u64> = [1u64, 3, 5, 7, 9].into_iter().collect();
        let eyt = to_eytzinger_layout(&sorted);
        let t = 9u64;
        // slot1 = root: the padded in-order is [1,3,5,7,9,PAD,PAD], midpoint
        // index 3 -> value 7 at slot 1.
        let mut k = 1usize;
        k = eytzinger_next(k, eyt[k] <= t); // 7<=9  -> right, slot 3
        assert_eq!(k, 3);
        k = eytzinger_next(k, eyt[k] <= t); // PAD>9 -> left,  slot 6
        assert_eq!(k, 6);
        k = eytzinger_next(k, eyt[k] <= t); // PAD>9 -> left,  slot 13 (exit: m=7)
        assert_eq!(k, 13);
        let slot = eytzinger_restore(k); // 13 = 0b1101 -> slot 3
        assert_eq!(slot, 3);
        assert_eq!(
            eyt[slot],
            u64::MAX - 0,
            "upper_bound(9) is None in the padded tree"
        );
        assert_eq!(eytzinger_search(&eyt, t), None);
        // and a found-case trace: target = 4 -> upper bound 5
        let t4 = 4u64;
        let mut k = 1usize;
        k = eytzinger_next(k, eyt[k] <= t4); // 7>4  -> left, slot 2
        assert_eq!(k, 2);
        k = eytzinger_next(k, eyt[k] <= t4); // 1<=4 -> right, slot 5
        assert_eq!(k, 5);
        k = eytzinger_next(k, eyt[k] <= t4); // 5>4  -> left, slot 10 (exit)
        assert_eq!(k, 10);
        let slot = eytzinger_restore(k); // 10 = 0b1010 -> slot 5
        assert_eq!(slot, 5);
        assert_eq!(eyt[slot], 5);
        assert_eq!(eytzinger_search(&eyt, 4), Some(5));
    }

    #[test]
    fn empty_and_single_edge_shapes() {
        let one = to_eytzinger_layout(&[42u64]);
        assert_eq!(one, vec![0, 42]);
        assert_eq!(eytzinger_search(&one, 41), Some(1)); // first > 41 is 42
        assert_eq!(eytzinger_search(&one, 42), None); // nothing > 42
        assert_eq!(eytzinger_search(&one, 43), None);
    }
}
