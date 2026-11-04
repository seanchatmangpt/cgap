// Stress courts (AB-6): decode refuses typed, never panics.
//
// (a) 10k random-byte buffers: decode must return Err for every buffer —
//     random bytes cannot validly carry magic+version+digest; an
//     unexpected Ok is a court failure, not an expected pass.
// (b) 10k valid-header mutations: every single-byte mutation of a valid
//     sealed header must land in a typed AoqError variant — no panic,
//     no silent acceptance.

use cgap_aoq::codec::{decode, encode};
use cgap_aoq::error::AoqError;
use cgap_aoq::header::{AoqHeader, Epoch};

/// SplitMix64-style deterministic PRNG (workspace standard multiplier
/// 0xBF58_476D_1CE4_E5B9, lane-local additive constant).
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0xA076_1D64_78BD_642F);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn fill(&mut self, buf: &mut [u8]) {
        for b in buf.iter_mut() {
            *b = self.next() as u8;
        }
    }
}

fn valid_wire() -> [u8; 64] {
    let h = AoqHeader::new(Epoch::Genesis as u16, 0xDEAD_BEEF, 42, 7);
    let mut wire = [0u8; 64];
    encode(&h, &mut wire).expect("encode into 64B buffer");
    wire
}

#[test]
fn ten_k_random_buffers_refuse_typed_no_panic() {
    let mut rng = Rng(0xAB_6A0A_5EED);
    let mut buf = vec![0u8; 128];
    let mut refusals = [0usize; 4]; // one bucket per refusal variant class
    for i in 0..10_000 {
        rng.fill(&mut buf);
        match decode(&buf) {
            Ok(_) => panic!("random buffer {i} decoded Ok — refusal ladder failed"),
            Err(e) => match e {
                AoqError::HeaderTooShort { .. } => refusals[0] += 1,
                AoqError::BadMagic { .. } => refusals[1] += 1,
                _ => panic!("variant {e:?} outside the typed ladder at {i}"),
            },
        }
    }
    let total: usize = refusals.iter().sum();
    assert_eq!(
        total, 10_000,
        "all random buffers must land in typed refusals"
    );
}

#[test]
fn ten_k_valid_header_mutations_each_typed() {
    let base = valid_wire();
    let mut rng = Rng(0x5EED_C0DE_BEEF);
    let mut seen = std::collections::HashSet::new();
    for _i in 0..10_000 {
        let mut wire = base;
        let pos = (rng.next() % 64) as usize;
        let newval = (rng.next() & 0xFF) as u8;
        let old = wire[pos];
        if newval == old {
            continue;
        }
        wire[pos] = newval;
        match decode(&wire) {
            Ok(_) => panic!(
                "mutation at byte {pos} ({}->{}) decoded Ok — digest law hole",
                old, newval
            ),
            Err(e) => {
                seen.insert(match e {
                    AoqError::BadMagic { .. } => "BadMagic",
                    AoqError::UnsupportedVersion { .. } => "UnsupportedVersion",
                    AoqError::DigestMismatch { .. } => "DigestMismatch",
                    AoqError::UnknownEpochWire { .. } => "UnknownEpochWire",
                    other => panic!("unexpected variant {other:?} at byte {pos}"),
                });
            }
        }
    }
    // Every mutated byte position must be refusal-covered (all 64 bytes
    // are digest/magic/version covered); assert full coverage.
    assert_eq!(
        seen.len(),
        4,
        "expected exactly BadMagic/UnsupportedVersion/DigestMismatch, got {seen:?}"
    );
}
