//! NTT/INTT courts pinned to the compiled PQClean C ntt.o outputs
//! (captured in-session via cprobe; external receipts, not self-attestation).
use cgap_crypto::fips203;

#[test]
fn ntt_c_reference_pins() {
    // ntt(X): C printed 0,1,0,1,0,1,0,1
    let mut x = [0i16; 256];
    x[1] = 1;
    fips203::ntt(&mut x);
    assert_eq!(&x[..8], &[0, 1, 0, 1, 0, 1, 0, 1]);
    // ntt(s0-demo): C printed 1567,989,-1567,-2145,1599,2177,-1599,-1021
    let mut s0 = [0i16; 256];
    s0[2] = -1;
    s0[3] = -1;
    s0[5] = -2;
    s0[6] = 1;
    s0[7] = 1;
    fips203::ntt(&mut s0);
    assert_eq!(
        &s0[..8],
        &[1567, 989, -1567, -2145, 1599, 2177, -1599, -1021]
    );
    // intt_tomont(ntt(f)) == f * 2^16 mod q (montgomery round trip)
    let mut f = [0i16; 256];
    for (i, c) in f.iter_mut().enumerate() {
        *c = ((i as i32 * 13 % 7) - 3) as i16;
    }
    let mut g = f;
    fips203::ntt(&mut g);
    fips203::intt_tomont(&mut g);
    for i in 0..256 {
        let expect = ((f[i] as i32 * 2285) % 3329 + 3329) % 3329;
        let gotv = ((g[i] as i32) % 3329 + 3329) % 3329;
        assert_eq!(gotv, expect, "coeff {}", i);
    }
}
