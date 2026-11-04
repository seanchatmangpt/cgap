use cgap_crypto::fips203 as f3;
#[test]
fn verbatim_keygen() {
    let d = [7u8; 32];
    let z = [9u8; 32];
    let mut ek = [0u8; 1184];
    let mut dk = [0u8; 2400];
    // verbatim copy of kem::keygen
    let mut buf = [0u8; 33];
    buf[..32].copy_from_slice(&d);
    buf[32] = 3;
    let mut g = [0u8; 64];
    f3::hash_g(&mut g, &buf);
    let rho: [u8; 32] = g[..32].try_into().unwrap();
    let sigma: [u8; 32] = g[32..].try_into().unwrap();
    let mut a = [[[0i16; 256]; 3]; 3];
    f3::gen_matrix(&mut a, &rho, false);
    let mut s_hat = [[0i16; 256]; 3];
    let mut e_hat = [[0i16; 256]; 3];
    for i in 0..3 {
        s_hat[i] = f3::sample_poly_cbd_prf(&sigma, i as u8);
    }
    for i in 0..3 {
        e_hat[i] = f3::sample_poly_cbd_prf(&sigma, (3 + i) as u8);
    }
    for p in s_hat.iter_mut() { f3::ntt(p); }
    for p in e_hat.iter_mut() { f3::ntt(p); }
    let mut t_hat = [[0i16; 256]; 3];
    for i in 0..3 {
        let mut acc = [0i16; 256];
        let mut prod = [0i16; 256];
        for j in 0..3 {
            f3::poly_basemul_montgomery(&mut prod, &a[i][j], &s_hat[j]);
            f3::poly_add_assign(&mut acc, &prod);
        }
        f3::poly_tomont(&mut acc);
        f3::poly_add_assign(&mut acc, &e_hat[i]);
        f3::poly_reduce(&mut acc);
        t_hat[i] = acc;
    }
    println!("THAT0={:?}", &t_hat[0][..8]);
    let mut pb = [0u8; 384];
    f3::poly_tobytes(&mut pb, &t_hat[0]);
    let mut hx = String::new();
    for b in &pb[..6] { hx.push_str(&format!("{:02x}", b)); }
    println!("PB6={}", hx);
}
