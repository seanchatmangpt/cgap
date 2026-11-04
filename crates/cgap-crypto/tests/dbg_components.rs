use cgap_crypto::fips203 as f3;
fn rand_poly(seed: u8) -> [i16; 256] {
    let mut p = [0i16; 256];
    let mut st = seed as u32 + 1;
    for c in p.iter_mut() {
        st = st.wrapping_mul(1664525).wrapping_add(1013904223);
        *c = (st >> 16) as i16 % 3329;
    }
    p
}
#[test]
fn ntt_roundtrip() {
    for seed in 0..4u8 {
        let x = rand_poly(seed);
        let mut y = x;
        f3::ntt(&mut y);
        f3::intt_tomont(&mut y);
        for i in 0..256 {
            let expect = ((x[i] as i32 * 2285) % 3329 + 3329) % 3329;
            let got = ((y[i] as i32) % 3329 + 3329) % 3329;
            assert_eq!(got, expect, "i={} seed={}", i, seed);
        }
    }
}
#[test]
fn basemul_matches_schoolbook() {
    // basemul+intt_tomont on canonical reps == plain negacyclic schoolbook
    // product mod q (empirical law proven by the R/R^-1 probe in dbg2).
    let a = rand_poly(1);
    let b = rand_poly(2);
    let mut ahn = a;
    let mut bhn = b;
    f3::ntt(&mut ahn);
    f3::ntt(&mut bhn);
    let mut c = [0i16; 256];
    f3::poly_basemul_montgomery(&mut c, &ahn, &bhn);
    f3::intt_tomont(&mut c);
    let mut ref_c = [0i64; 256];
    for i in 0..256 {
        for j in 0..256 {
            let t = a[i] as i64 * b[j] as i64;
            if i + j < 256 { ref_c[i + j] += t; } else { ref_c[i + j - 256] -= t; }
        }
    }
    for i in 0..256 {
        let expect = ((ref_c[i] % 3329 + 3329) % 3329) as i32;
        let got = ((c[i] as i32) % 3329 + 3329) % 3329;
        assert_eq!(got, expect, "i={}", i);
    }
}
