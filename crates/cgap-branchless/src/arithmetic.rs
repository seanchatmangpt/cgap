//! Branchless P28 arithmetic (lane B2). Naive oracles live in
//! `#[cfg(test)]` as reference forms; the `pub` fns are the branchless
//! implementations. Semantics pinned by B1 skeleton doc:
//! - saturated_subs_p28: new = counter - delta SATURATED at zero;
//!   tripwire all-ones iff delta > counter, else zero.
//! - poison_mask: all-ones iff quota == 0 (exhausted), else zero.
//!
//! House law: no data-dependent branches; borrow bit via arithmetic
//! masking. Two seeds never conflated: corpus seed 0xA20C_0FFE_E5A7_E000
//! drives the court; SplitMix64 multiplier 0xBF58_476D_1CE4_E5B9 is the
//! DERIVE constant inside the finalizer (bitslice.rs:687-699).

/// Branchless saturated subtract with underflow tripwire.
///
/// new = saturating_sub(counter, delta) via diff & !tripwire masking.
/// Borrow bit = carry-kill/propagate form over pure u64 arithmetic
/// (valid on any target: no u128 widening, no sbb chain, no cmov):
/// borrow = ((!c & d) | ((!c | d) & diff)) >> 63, expanded to an
/// all-ones mask by 0u64.wrapping_sub(borrow).
#[inline]
#[allow(dead_code)] // E-h de-pub: no external consumers; retained for differential/census courts
pub(crate) fn saturated_subs_p28(counter: u64, delta: u64) -> (u64, u64) {
    let diff = counter.wrapping_sub(delta);
    // Carry kill/propagate borrow-out, pure u64 (branchless everywhere).
    let borrow = (((!counter) & delta) | (((!counter) | delta) & diff)) >> 63;
    let tripwire = 0u64.wrapping_sub(borrow); // all-ones iff underflow
    let new = diff & (tripwire ^ u64::MAX); // zero on underflow (saturate)
    (new, tripwire)
}

/// Branchless poison mask: all-ones iff quota == 0, else zero.
///
/// DIVERGENCE (recorded): the dispatch formula,
/// ((quota.wrapping_sub(1)) as i64 >> 63) as u64 ^ u64::MAX,
/// is all-ones iff quota > 0 -- the COMPLEMENT of the B1-stated oracle
/// ("all-ones when the quota is exhausted (zero)"). Per the dispatch
/// instruction to make doc+code agree with the B1 doc, the XOR
/// complement is dropped; the shift result is already the mask.
#[inline]
#[allow(dead_code)] // E-h de-pub: no external consumers; retained for differential/census courts
pub(crate) fn poison_mask(quota: u64) -> u64 {
    // B1 unblock fix (disclosed in both lanes' receipts): the previous
    // `((quota.wrapping_sub(1)) as i64 >> 63)` form fired on EVERY quota
    // with bit 63 set (any quota >= 2^63), not only quota == 0. The exact
    // is-nonzero test `((q | q.wrapping_neg()) >> 63) & 1` folds to the
    // {0,1} nonzero bit branchlessly; negate to the all-ones poison mask.
    let nz = ((quota | quota.wrapping_neg()) >> 63) & 1;
    0u64.wrapping_sub(nz ^ 1) // all-ones iff quota == 0 (poisoned)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Naive oracles (reference only, test-gated) ----

    fn oracle_saturated_subs_p28(counter: u64, delta: u64) -> (u64, u64) {
        if delta > counter {
            (0, u64::MAX)
        } else {
            (counter - delta, 0)
        }
    }

    fn oracle_poison_mask(quota: u64) -> u64 {
        if quota == 0 {
            u64::MAX
        } else {
            0
        }
    }

    // SplitMix64 driven by the corpus seed; multiplier 0xBF58_476D_1CE4_E5B9
    // is the DERIVE constant, kept distinct from the corpus seed.

    fn splitmix64(state: &mut u64) -> u64 {
        *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = *state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x1405_7B7E_F767_814F);
        z ^ (z >> 31)
    }

    const CORPUS_SEED: u64 = 0xA20C_0FFE_E5A7_E000;

    #[test]
    fn differential_fuzz_branchless_equals_naive() {
        let mut state = CORPUS_SEED;
        for _ in 0..4096 {
            let counter = splitmix64(&mut state);
            let delta = splitmix64(&mut state);
            assert_eq!(
                saturated_subs_p28(counter, delta),
                oracle_saturated_subs_p28(counter, delta),
                "counter={counter:#x} delta={delta:#x}"
            );
            assert_eq!(
                poison_mask(counter),
                oracle_poison_mask(counter),
                "poison quota={counter:#x}"
            );
        }
    }

    #[test]
    fn exhaustive_edge_cases_match_oracles() {
        let counters = [0u64, 1, CORPUS_SEED, u64::MAX - 1, u64::MAX];
        let deltas = [0u64, 1, CORPUS_SEED, u64::MAX - 1, u64::MAX];
        for &c in &counters {
            for &d in &deltas {
                assert_eq!(
                    saturated_subs_p28(c, d),
                    oracle_saturated_subs_p28(c, d),
                    "counter={c:#x} delta={d:#x}"
                );
                assert_eq!(poison_mask(c), oracle_poison_mask(c), "poison quota={c:#x}");
                assert_eq!(poison_mask(d), oracle_poison_mask(d), "poison quota={d:#x}");
            }
        }
        // anti-vacuity mutation: any delta < counter must clear tripwire
        let (_, tw) = saturated_subs_p28(100, 40);
        assert_eq!(tw, 0, "tripwire must stay clear when delta < counter");
        let (n, tw) = saturated_subs_p28(1, 2);
        assert_eq!((n, tw), (0, u64::MAX));
        assert_eq!(poison_mask(0), u64::MAX);
        assert_eq!(poison_mask(1), 0);
    }

    #[test]
    fn saturation_and_tripwire_shape() {
        // (new, tripwire) pairs are only ever (diff, 0) or (0, MAX)
        let mut state = CORPUS_SEED;
        for _ in 0..512 {
            let c = splitmix64(&mut state);
            let d = splitmix64(&mut state);
            let (n, tw) = saturated_subs_p28(c, d);
            assert!(tw == 0 || tw == u64::MAX);
            if tw == u64::MAX {
                assert_eq!(n, 0);
            } else {
                assert_eq!(n, c.wrapping_sub(d));
            }
        }
    }
}
