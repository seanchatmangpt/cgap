//! mlkem768_courts - AB-5 court battery for the ML-KEM-768 kernel.
//!
//! External ground truth:
//! - liboqs 0.13.1-dev ML-KEM-768 (FIPS 203 final) via a compiled C oracle
//!   (tests/vectors/mlkem768/oracle.c) for interop courts.
//! - PQClean MLKEM512 zetas literal (ntt.c), extracted from ntt.o (offset
//!   4500) and cross-checked against the spec-generated table.
//! - FIPS 202 known answers (SHAKE-256("") / SHA3-512("")).
use cgap_crypto::fips203;

// ---------------------------------------------------------------------------
// zetas table pin
// ---------------------------------------------------------------------------

/// External receipt: PQClean MLKEM512 zetas literal (ntt.c; also extracted
/// from the compiled ntt.o at offset 4500 and cross-checked). Any value edit
/// to fips203::ZETAS flips this court. Anti-vacuity: mutating any ZETAS
/// entry changes the blake3 digest (zetas_digest_pin) and the entry check.
#[rustfmt::skip]
const PQCLEAN_ZETAS_128: [i16; 128] = [
    -1044, -758, -359, -1517, 1493, 1422, 287, 202,
    -171, 622, 1577, 182, 962, -1202, -1474, 1468,
    573, -1325, 264, 383, -829, 1458, -1602, -130,
    -681, 1017, 732, 608, -1542, 411, -205, -1571,
    1223, 652, -552, 1015, -1293, 1491, -282, -1544,
    516, -8, -320, -666, -1618, -1162, 126, 1469,
    -853, -90, -271, 830, 107, -1421, -247, -951,
    -398, 961, -1508, -725, 448, -1065, 677, -1275,
    -1103, 430, 555, 843, -1251, 871, 1550, 105,
    422, 587, 177, -235, -291, -460, 1574, 1653,
    -246, 778, 1159, -147, -777, 1483, -602, 1119,
    -1590, 644, -872, 349, 418, 329, -156, -75,
    817, 1097, 603, 610, 1322, -1285, -1465, 384,
    -1215, -136, 1218, -1335, -874, 220, -1187, -1659,
    -1185, -1530, -1278, 794, -1510, -854, -870, 478,
    -108, -308, 996, 991, 958, -1460, 1522, 1628,
];

#[test]
fn zetas_table_pqclean_entry_check() {
    assert_eq!(
        &fips203::ZETAS[0..128],
        &PQCLEAN_ZETAS_128,
        "ZETAS[0..128] must equal the PQClean literal"
    );
    // tail layout: 128..=255 mirrored, 256..=258 = [1441, -1044, 0]
    let rev: Vec<i16> = fips203::ZETAS[0..128].iter().copied().rev().collect();
    assert_eq!(&fips203::ZETAS[128..256], &rev[..]);
    assert_eq!(fips203::ZETAS[256], 1441);
    assert_eq!(fips203::ZETAS[257], -1044);
    assert_eq!(fips203::ZETAS[258], 0);
}

/// blake3 digest pin over the whole 259-entry table (LE i16 bytes).
#[test]
fn zetas_digest_pin() {
    let mut bytes = Vec::new();
    for v in fips203::ZETAS.iter() {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    let d = cgap_blake3::hash(&bytes);
    let mut hexs = String::new();
    for b in d.iter() {
        hexs.push_str(&format!("{:02x}", b));
    }
    assert_eq!(
        hexs,
        "1225892f302b3678d778c4ff059ca318bd2cd2c25ff98b99f3cada566c4cc4f6"
    );
}
