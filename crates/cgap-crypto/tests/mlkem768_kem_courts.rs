//! KEM courts part 1: basemul vs schoolbook.
use cgap_crypto::fips203;

#[test]
fn basemul_matches_schoolbook() {
    let mut a = [0i16; 256];
    let mut b = [0i16; 256];
    for i in 0..256 {
        a[i] = ((i as i32 * 37 % 300) - 150) as i16;
        b[i] = ((i as i32 * 91 % 300) - 150) as i16;
    }
    let mut ah = a;
    let mut bh = b;
    fips203::ntt(&mut ah);
    fips203::ntt(&mut bh);
    let mut c = [0i16; 256];
    fips203::poly_basemul_montgomery(&mut c, &ah, &bh);
    fips203::intt_tomont(&mut c);
    let mut ref_c = [0i64; 256];
    for i in 0..256 {
        for j in 0..256 {
            let t = a[i] as i64 * b[j] as i64;
            if i + j < 256 {
                ref_c[i + j] += t;
            } else {
                ref_c[i + j - 256] -= t;
            }
        }
    }
    for i in 0..256 {
        let expect = ((ref_c[i] % 3329 + 3329) % 3329) as i32;
        let gotv = ((c[i] as i32) % 3329 + 3329) % 3329;
        assert_eq!(gotv, expect, "coeff {}", i);
    }
}
