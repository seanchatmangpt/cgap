//! Quick round-trip + determinism sanity before full courts.
use cgap_crypto::{fips203, kem};

#[test]
fn roundtrip_sanity() {
    let d = [7u8; 32];
    let z = [9u8; 32];
    let mut ek = [0u8; kem::EK_LEN];
    let mut dk = [0u8; kem::DK_LEN];
    kem::keygen(&d, &z, &mut ek, &mut dk);
    let m = [3u8; 32];
    let mut ct = [0u8; kem::CT_LEN];
    let mut ss1 = [0u8; 32];
    kem::encaps(&ek, &m, &mut ct, &mut ss1);
    let ss2 = kem::decaps(&dk, &ct);
    assert_eq!(ss1, ss2, "round-trip failed");
    // determinism
    let mut ek2 = [0u8; 1184];
    let mut dk2 = [0u8; 2400];
    kem::keygen(&d, &z, &mut ek2, &mut dk2);
    assert_eq!(ek, ek2);
}
