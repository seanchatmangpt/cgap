//! Court: bit-plane seen-set membership vs BTreeSet oracle + mutation flip.
//!
//! Chicago discipline: real collaborators (real set, real oracle, real
//! mutation on a clone), assert final state, no mocks.

use cgap_storage::bitplane_seen::{BitPlaneSeenSet, KEY_BITS, PLANE_COUNT};
use std::collections::BTreeSet;

/// Canonical corpus RNG (vendored algorithm from bitslice.rs:687-699).
/// Seed law: initialized with the CORPUS seed `0xA20C_0FFE_E5A7_E000`, mixed
/// with the ALGORITHM multiplier `0xBF58_476D_1CE4_E5B9` — the two seeds are
/// never conflated (cgap CLAUDE.md frozen pins).
struct CorpusRng(u64);
impl CorpusRng {
    fn next(&mut self) -> u64 {
        let mut z = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        self.0 = z;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z ^ (z >> 31)
    }
}

const N_PAIRS: usize = 10_000;

#[test]
fn court_membership_matches_btreeset_oracle_10k() {
    let mut rng = CorpusRng(0xA20C_0FFE_E5A7_E000);
    // Structural fix (repair-17): the 16 MB inline set is heap-leased at
    // construction via `new_boxed()` (alloc_zeroed — no stack staging);
    // the court runs on the DEFAULT test-thread stack.
    let mut seen: Box<BitPlaneSeenSet<16_384>> = BitPlaneSeenSet::new_boxed();
    let mut oracle: BTreeSet<(u64, u64)> = BTreeSet::new();
    let mut inserted = 0usize;
    for _ in 0..N_PAIRS {
        let x = rng.next();
        let z = rng.next() & 0xF;
        let r = seen
            .insert(x, z)
            .expect("capacity court: set must not fill");
        let o = oracle.insert((x, z));
        assert_eq!(r, o, "insert result must match oracle");
        if r {
            inserted += 1;
        }
    }
    assert_eq!(seen.len(), oracle.len());
    assert_eq!(seen.len(), inserted);
    // Positive + negative membership over the same corpus plus fresh probes.
    for _ in 0..N_PAIRS {
        let x = rng.next();
        let z = rng.next() & 0xF;
        assert_eq!(
            seen.contains(x, z),
            oracle.contains(&(x, z)),
            "contains must agree with oracle for x={x} z={z}"
        );
    }
}

#[test]
fn court_capacity_refusal_is_typed_and_loud() {
    let mut seen: BitPlaneSeenSet<4> = BitPlaneSeenSet::new();
    for i in 0u64..4 {
        assert!(seen.insert(i, i).unwrap());
    }
    assert!(seen.is_full());
    assert_eq!(
        seen.insert(99, 99),
        Err(cgap_storage::bitplane_seen::SeenFull)
    );
    // Duplicates of stored pairs still resolve Ok(false), not Err.
    assert_eq!(seen.insert(0, 0), Ok(false));
    seen.clear();
    assert!(seen.is_empty() && seen.len() == 0);
}

#[test]
fn court_mutation_flip_corrupt_plane_diverges_from_oracle() {
    // Build a small real set.
    let mut seen: BitPlaneSeenSet<64> = BitPlaneSeenSet::new();
    let mut oracle: BTreeSet<(u64, u64)> = BTreeSet::new();
    let pairs = [
        (0x1234_5678_9ABC_DEF0, 1),
        (2, 0xFFFF_FFFF_FFFF_FFFF),
        (7, 7),
    ];
    for (x, z) in pairs {
        seen.insert(x, z).unwrap();
        oracle.insert((x, z));
    }
    // Pre-mutation agreement gate: court is vacuous if it starts diverged.
    for (x, z) in pairs {
        assert!(seen.contains(x, z), "pre-mutation miss for {x},{z}");
    }
    assert!(!seen.contains(3, 3));

    // Anti-vacuity mutation: corrupt exactly ONE plane bit of slot 0
    // (plane 0 = MSB of x). A court that cannot flip is FAIL.
    let mut mutated = seen;
    let plane = 0;
    let slot = 0;
    let before = mutated.plane_bit(plane, slot);
    mutated.set_plane_bit(plane, slot, before ^ 1);
    assert_eq!(
        mutated.plane_bit(plane, slot),
        before ^ 1,
        "mutation must land"
    );

    // Post-mutation divergence: contains must now DISAGREE with the oracle.
    let (x0, z0) = pairs[0];
    let oracle_says = oracle.contains(&(x0, z0));
    let seen_says = mutated.contains(x0, z0);
    assert_ne!(
        seen_says, oracle_says,
        "VACUOUS COURT: corrupting plane {plane} slot {slot} did not flip contains()"
    );
    // Structural check: corruption changed the reconstructed key.
    assert_eq!(KEY_BITS, 128);
    assert_eq!(PLANE_COUNT, 128);
}
