//! fips203_cbd_court - CBD sampling courts for fips203::poly_cbd.
//!
//! Scalar reference = the spec law (FIPS 203/204 SamplePolyCBD, consecutive
//! per-coefficient bit fields): coefficient i consumes bits 2·eta·i ..
//! 2·eta·i + 2·eta − 1 of the byte stream LSB-first; first eta bits are the
//! a-field,
//! last eta bits the b-field; r[i] = popcount(a) − popcount(b). Real
//! collaborator: the crate function under test; no mocks.
use cgap_crypto::fips203::{self, CbdError};

/// Scalar spec-law CBD reference (bit-exact law, one branch per bit).
fn cbd_ref(buf: &[u8], eta: usize) -> Vec<i16> {
    let bit = |n: usize| -> u32 {
        let byte = buf[n / 8];
        ((byte >> (n % 8)) & 1) as u32
    };
    let mut out = Vec::new();
    for i in 0..256 {
        let base = i * 2 * eta;
        let mut a = 0u32;
        let mut b = 0u32;
        for k in 0..eta {
            a += bit(base + k);
        }
        for k in 0..eta {
            b += bit(base + eta + k);
        }
        out.push(a as i16 - b as i16);
    }
    out
}

/// Every single byte of the CBD input buffer takes every value 0..=255,
/// exhaustively over all byte positions and both etas; every coefficient of
/// every output must equal the scalar reference (all other coefficients must
/// be zero, which the full-vector compare enforces).
#[test]
fn cbd_exhaustive_per_byte_vs_scalar() {
    for eta in [2u8, 4u8] {
        let buf_len = eta as usize * 64;
        for byte_pos in 0..buf_len {
            for v in 0..=255u8 {
                let mut buf = [0u8; 256];
                buf[byte_pos] = v;
                let mut got = [0i16; 256];
                fips203::poly_cbd(&mut got, &buf[..buf_len], eta).unwrap();
                assert_eq!(
                    got.to_vec(),
                    cbd_ref(&buf, eta as usize),
                    "eta {} byte_pos {} v {}",
                    eta,
                    byte_pos,
                    v
                );
            }
        }
    }
}

/// Deterministic random full-buffer sweep (seeded LCG, replayable).
#[test]
fn cbd_random_polys_vs_scalar() {
    let mut s: u64 = 0xA20C_0FFE_E5A7_E005;
    let mut next = move || -> u64 {
        s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        s
    };
    for eta in [2u8, 4u8] {
        let buf_len = eta as usize * 64;
        for _ in 0..16 {
            let mut buf = [0u8; 256];
            for b in buf[..buf_len].iter_mut() {
                *b = next() as u8;
            }
            let mut got = [0i16; 256];
            fips203::poly_cbd(&mut got, &buf[..buf_len], eta).unwrap();
            assert_eq!(got.to_vec(), cbd_ref(&buf, eta as usize));
        }
    }
}

/// CBD value range law: eta=4 coefficients stay in [-4, 4]; eta=2 in [-2, 2].
#[test]
fn cbd_coefficient_range_law() {
    let mut s: u64 = 0xA20C_0FFE_E5A7_E006;
    let mut next = move || -> u64 {
        s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        s
    };
    for eta in [2u8, 4u8] {
        let buf_len = eta as usize * 64;
        let bound = eta as i16;
        for _ in 0..16 {
            let mut buf = [0u8; 256];
            for b in buf[..buf_len].iter_mut() {
                *b = next() as u8;
            }
            let mut got = [0i16; 256];
            fips203::poly_cbd(&mut got, &buf[..buf_len], eta).unwrap();
            for c in got.iter() {
                assert!(*c >= -bound && *c <= bound);
            }
        }
    }
}

/// Loud refusal: the former `_ => ()` silent zero-fill is now a typed Err for
/// any eta outside {2, 4}.
#[test]
fn cbd_unsupported_eta_refused() {
    let buf = [0u8; 512];
    let mut r = [0i16; 256];
    for bad_eta in [0u8, 1, 3, 5, 255] {
        let len = (bad_eta as usize * 64).min(512);
        assert_eq!(
            fips203::poly_cbd(&mut r, &buf[..len], bad_eta),
            Err(CbdError::UnsupportedEta(bad_eta))
        );
    }
    assert!(fips203::poly_cbd(&mut r, &buf[..128], 2).is_ok());
    assert!(fips203::poly_cbd(&mut r, &buf[..256], 4).is_ok());
}
