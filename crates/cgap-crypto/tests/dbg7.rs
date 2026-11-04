use cgap_crypto::fips203 as f3;
#[test]
fn chain() {
    let d = [7u8; 32];
    let mut buf = [0u8; 33];
    buf[..32].copy_from_slice(&d);
    buf[32] = 3;
    let mut g = [0u8; 64];
    f3::hash_g(&mut g, &buf);
    let rho: [u8; 32] = g[..32].try_into().unwrap();
    let sigma: [u8; 32] = g[32..].try_into().unwrap();
    let mut sh = [[0i16; 256]; 3];
    let mut eh = [[0i16; 256]; 3];
    for i in 0..3 {
        sh[i] = f3::sample_poly_cbd_prf(&sigma, i as u8);
        f3::ntt(&mut sh[i]);
        eh[i] = f3::sample_poly_cbd_prf(&sigma, (3 + i) as u8);
        f3::ntt(&mut eh[i]);
    }
    // manual basemul acc for row 0
    let mut acc = [0i16; 256];
    for j in 0..3 {
        let a = f3::sample_ntt(&rho, j as u8, 0); // A[0][j] from (j, i=0)
        let mut prod = [0i16; 256];
        f3::poly_basemul_montgomery(&mut prod, &a, &sh[j]);
        for i in 0..256 { acc[i] += prod[i]; }
    }
    println!("ACC0={:?}", &acc[..8]);
    f3::poly_tomont(&mut acc);
    println!("TOMONT0={:?}", &acc[..8]);
    f3::poly_add_assign(&mut acc, &eh[0]);
    f3::poly_reduce(&mut acc);
    println!("THAT0={:?}", &acc[..8]);
}
