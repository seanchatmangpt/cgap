//! KEM courts part 2: encode/decode round-trips, determinism, KEM
//! round-trip, and negative corrupted-ct courts.
use cgap_crypto::{fips203, kem};

fn rt<const D: usize>(p: &[i16; 256]) -> bool {
    // encode -> decode -> re-encode must be byte-identical (sound property
    // for all d; encode is lossy below d=12 so direct round-trip is wrong).
    let mut out = [0u8; 384];
    fips203::byte_encode::<D>(p, &mut out[..256 * D / 8]);
    let back = fips203::byte_decode::<D>(&out[..256 * D / 8]);
    let mut out2 = [0u8; 384];
    fips203::byte_encode::<D>(&back, &mut out2[..256 * D / 8]);
    out[..256 * D / 8] == out2[..256 * D / 8]
}

#[test]
fn byte_encode_decode_roundtrip_all_d() {
    let mut p = [0i16; 256];
    let mut st = 12345u32;
    for c in p.iter_mut() {
        st = st.wrapping_mul(1664525).wrapping_add(1013904223);
        *c = ((st >> 17) % 3329) as i16;
    }
    assert!(rt::<1>(&p));
    assert!(rt::<2>(&p));
    assert!(rt::<3>(&p));
    assert!(rt::<4>(&p));
    assert!(rt::<5>(&p));
    assert!(rt::<6>(&p));
    assert!(rt::<7>(&p));
    assert!(rt::<8>(&p));
    assert!(rt::<9>(&p));
    assert!(rt::<10>(&p));
    assert!(rt::<11>(&p));
    assert!(rt::<12>(&p));
}

#[test]
fn kem_determinism() {
    let d = [7u8; 32];
    let z = [9u8; 32];
    let mut ek1 = [0u8; 1184];
    let mut dk1 = [0u8; 2400];
    let mut ek2 = [0u8; 1184];
    let mut dk2 = [0u8; 2400];
    kem::keygen(&d, &z, &mut ek1, &mut dk1);
    kem::keygen(&d, &z, &mut ek2, &mut dk2);
    assert_eq!(ek1, ek2);
    assert_eq!(dk1, dk2);
    let m = [3u8; 32];
    let mut ct1 = [0u8; 1088];
    let mut ss1 = [0u8; 32];
    let mut ct2 = [0u8; 1088];
    let mut ss2 = [0u8; 32];
    kem::encaps(&ek1, &m, &mut ct1, &mut ss1);
    kem::encaps(&ek1, &m, &mut ct2, &mut ss2);
    assert_eq!(ct1, ct2);
    assert_eq!(ss1, ss2);
}

#[test]
fn kem_roundtrip_and_negative() {
    let d = [7u8; 32];
    let z = [9u8; 32];
    let mut ek = [0u8; 1184];
    let mut dk = [0u8; 2400];
    kem::keygen(&d, &z, &mut ek, &mut dk);
    let m = [3u8; 32];
    let mut ct = [0u8; 1088];
    let mut ss = [0u8; 32];
    kem::encaps(&ek, &m, &mut ct, &mut ss);
    let ss2 = kem::decaps(&dk, &ct);
    assert_eq!(ss, ss2, "round-trip");
    for (pos, name) in [(5usize, "u0"), (500, "u1"), (1070, "v")] {
        let mut bad = ct;
        bad[pos] ^= 0x80;
        let rej = kem::decaps(&dk, &bad);
        assert_ne!(rej, ss, "corrupted {} must not decrypt to ss", name);
    }
}
