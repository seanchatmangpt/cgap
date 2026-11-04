use cgap_crypto::fips203 as f3;
#[test]
fn components() {
    let d = [7u8; 32];
    let mut buf = [0u8; 33];
    buf[..32].copy_from_slice(&d);
    buf[32] = 3;
    let mut g = [0u8; 64];
    f3::hash_g(&mut g, &buf);
    let mut hx = String::new();
    for b in &g[..16] { hx.push_str(&format!("{:02x}", b)); }
    println!("G={}", hx);
    let sigma: [u8; 32] = g[32..].try_into().unwrap();
    let mut prfbuf = [0u8; 128];
    f3::prf(&mut prfbuf, &sigma, 0);
    let mut px = String::new();
    for b in &prfbuf[..8] { px.push_str(&format!("{:02x}", b)); }
    println!("PRF0={}", px);
    let mut p = [0i16; 256];
    f3::poly_cbd(&mut p, &prfbuf, 2);
    println!("S0={:?}", &p[..8]);
    let rho: [u8; 32] = g[..32].try_into().unwrap();
    let a00 = f3::sample_ntt(&rho, 0, 0);
    println!("A00={:?}", &a00[..8]);
}
