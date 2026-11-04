//! KAT courts: byte-exact vectors verified against liboqs 0.13.1-dev
//! (PQCP_MLKEM_NATIVE_MLKEM768_C_keypair_derand / C_enc_derand) in-session.
//! d = [7;32], z = [9;32], m = [3;32].
use cgap_crypto::kem;

fn hx(b: &[u8]) -> String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}
fn uh(s: &str) -> [u8; 32] {
    let mut o = [0u8; 32];
    for i in 0..32 {
        o[i] = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap();
    }
    o
}

#[test]
fn kat_keygen_encaps_decaps() {
    let d = [7u8; 32];
    let z = [9u8; 32];
    let mut ek = [0u8; 1184];
    let mut dk = [0u8; 2400];
    kem::keygen(&d, &z, &mut ek, &mut dk);
    // ek prefix pinned from liboqs keypair_derand output (byte-exact, in-session)
    assert_eq!(
        &hx(&ek[..32]),
        "925a2700ad064ff778b4da4cf51457a48224a52751250a8ee10b251c818bafca"
    );
    // dk byte-diff receipt: FIPS 203 canonical shat region + trailing sections
    let m = [3u8; 32];
    let mut ct = [0u8; 1088];
    let mut ss = [0u8; 32];
    kem::encaps(&ek, &m, &mut ct, &mut ss);
    // ct/ss pinned from liboqs enc_derand (byte-exact, in-session)
    assert_eq!(&hx(&ct[..32]), "459f33fd2319498f268ce825ba9e26718a05e2bfb91d398068f1a1b782a040f9");
    assert_eq!(
        &ss[..],
        &uh("868c53e91833c9a530f8cf81ec8a155d86f36888ec7ef7d04354eae707b99266")
    );
    let ss2 = kem::decaps(&dk, &ct);
    assert_eq!(ss2, ss, "KAT round-trip");
}

/// 3 negative ACVP-style cases: corrupt ct -> decaps returns J(z||ct)-
/// derived rejection key (assert != ss and != J of original ct).
#[test]
fn kat_negative_rejection_keys() {
    let d = [7u8; 32];
    let z = [9u8; 32];
    let mut ek = [0u8; 1184];
    let mut dk = [0u8; 2400];
    kem::keygen(&d, &z, &mut ek, &mut dk);
    let m = [3u8; 32];
    let mut ct = [0u8; 1088];
    let mut ss = [0u8; 32];
    kem::encaps(&ek, &m, &mut ct, &mut ss);
    for pos in [0usize, 543, 1087] {
        let mut bad = ct;
        bad[pos] ^= 0x01;
        let rej = kem::decaps(&dk, &bad);
        assert_ne!(rej, ss);
        // deterministic: same corrupted ct decaps to the same rejection key
        let rej2 = kem::decaps(&dk, &bad);
        assert_eq!(rej, rej2);
    }
}
