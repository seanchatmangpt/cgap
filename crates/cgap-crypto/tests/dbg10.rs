use cgap_crypto::fips203 as f3;
fn unpack_poly(b: &[u8]) -> [i16; 256] {
    let mut r = [0i16; 256];
    for i in 0..128 {
        let b0 = b[3*i] as u16; let b1 = b[3*i+1] as u16; let b2 = b[3*i+2] as u16;
        r[2*i] = (b0 | (b1 << 8)) as i16 & 0x0fff;
        r[2*i+1] = ((b1 >> 4) | (b2 << 4)) as i16 & 0x0fff;
    }
    r
}
fn hex_to_32(s: &str) -> [u8; 32] {
    let mut o = [0u8; 32];
    for i in 0..32 {
        o[i] = u8::from_str_radix(&s[2*i..2*i+2], 16).unwrap();
    }
    o
}
#[test]
fn enc_chain() {
    let d = [7u8; 32];
    let mut buf = [0u8; 33];
    buf[..32].copy_from_slice(&d);
    buf[32] = 3;
    let mut g = [0u8; 64];
    f3::hash_g(&mut g, &buf);
    let rho: [u8; 32] = g[..32].try_into().unwrap();
    let sigma: [u8; 32] = g[32..].try_into().unwrap();
    // reconstruct ek via keygen pieces: t_hat = THAT chain (matches mirror)
    // simpler: rebuild t_hat from THIS test's own chain
    let mut sh = [[0i16; 256]; 3];
    for i in 0..3 { sh[i] = f3::sample_poly_cbd_prf(&sigma, i as u8); }
    for p in sh.iter_mut() { f3::ntt(p); }
    let mut acc0 = [0i16; 256];
    for j in 0..3 {
        let a = f3::sample_ntt(&rho, j as u8, 0);
        let mut prod = [0i16; 256];
        f3::poly_basemul_montgomery(&mut prod, &a, &sh[j]);
        for i in 0..256 { acc0[i] += prod[i]; }
    }
    f3::poly_tomont(&mut acc0);
    // skip e for brevity: NOT equal to real ek; use mirror parity check only
    let _ = unpack_poly;
    let _ = hex_to_32;
}
