use cgap_crypto::kem;
#[test]
fn decaps_probe() {
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
    println!("SS_MATCH={}", ss == ss2);
    println!("SS1={:?}", &ss[..8]);
    println!("SS2={:?}", &ss2[..8]);
    // was the pke layer fine?
    let m2 = pke_public_probe(&dk, &ct);
    println!("M_MATCH={}", m == m2);
}
fn pke_public_probe(_dk: &[u8; 2400], _ct: &[u8; 1088]) -> [u8; 32] {
    [0u8; 32]
}
